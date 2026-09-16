import { defineStore } from 'pinia'
import { ref } from 'vue'
import { sendIpcToMain } from '@renderer/utils/ipc'

export interface AccountInfo {
  id?: number
  username: string
  email: string
  hwid: string
  credits: number
  status: string
  location: string
  device_model: string
  token?: string
}

interface UserPayload {
  id?: number
  username: string
  email?: string | null
  hwid: string
  credits?: number
  status?: string
  location?: string | null
  device_model?: string | null
  session_token?: string | null
}

const STORAGE_KEY = 'v1per_account'
const HWID_CACHE_KEY = 'v1per_hwid_cache'
const PCINFO_CACHE_KEY = 'v1per_pcinfo_cache'
const API_BASE = 'https://firmwaresss-devices.vercel.app'

interface PcInfo {
  machine_guid: string
  disk_serial: string
  board_serial: string
  mac: string
  cpu: string
  os_version: string
  ram: string
  device_model: string
}

// Cache HWID and PC info in localStorage so login/register don't
// block on PowerShell WMI queries every time.
function getCachedHwid(): string | null {
  try { return localStorage.getItem(HWID_CACHE_KEY) } catch { return null }
}
function setCachedHwid(hwid: string) {
  try { localStorage.setItem(HWID_CACHE_KEY, hwid) } catch { /* ignore */ }
}
function getCachedPcInfo(): PcInfo | null {
  try {
    const raw = localStorage.getItem(PCINFO_CACHE_KEY)
    return raw ? JSON.parse(raw) : null
  } catch { return null }
}
function setCachedPcInfo(info: PcInfo) {
  try { localStorage.setItem(PCINFO_CACHE_KEY, JSON.stringify(info)) } catch { /* ignore */ }
}

async function getHwidCached(): Promise<string> {
  const cached = getCachedHwid()
  if (cached) return cached
  const hwid = await sendIpcToMain<string>('get_hwid')
  setCachedHwid(hwid)
  return hwid
}

async function getPcInfoCached(): Promise<PcInfo> {
  const cached = getCachedPcInfo()
  if (cached) return cached
  const info = await sendIpcToMain<PcInfo>('get_pc_info')
  setCachedPcInfo(info)
  return info
}

function loadSaved(): AccountInfo | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? (JSON.parse(raw) as AccountInfo) : null
  } catch {
    return null
  }
}

function apiError(data: unknown, fallback: string): Error {
  const err = (data as { error?: unknown })?.error
  let msg = fallback
  if (typeof err === 'string') msg = err
  else if (err && typeof err === 'object') msg = (err as { message?: string }).message || JSON.stringify(err)
  return new Error(msg)
}

async function parseJson(res: Response): Promise<Record<string, unknown> | null> {
  try {
    return await res.json()
  } catch {
    return null
  }
}

export const useAccountStore = defineStore('account', () => {
  const account = ref<AccountInfo | null>(loadSaved())
  const initialized = ref(false)
  const banned = ref<string | null>(null)
  const deleted = ref<string | null>(null)

  const isAuthed = ref(!!account.value)

  function persist() {
    if (account.value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(account.value))
    } else {
      localStorage.removeItem(STORAGE_KEY)
    }
  }

  // Server-side account status (banned/suspended/revoked) + live credit sync.
  // Called on startup and on a timer so admin-added credits show up without
  // requiring a re-login. Never force-logs-out, it only shows the banned screen.
  async function checkStatus() {
    if (!account.value?.hwid) return
    let nonce = 0
    try {
      if (account.value.token) {
        const res = await fetch(`${API_BASE}/api/auth/me?token=${encodeURIComponent(account.value.token)}&_=${++nonce}`)
        if (res.status === 401) {
          // Token invalid - check if account was deleted or just session revoked
          try {
            const statusRes = await fetch(`${API_BASE}/api/auth/status?hwid=${encodeURIComponent(account.value.hwid)}&_=${++nonce}`)
            const statusData = await statusRes.json()
            if (statusData?.registered === false) {
              deleted.value = 'Your account has been deleted by an administrator.'
              return
            }
          } catch { /* offline fall through to normal logout */ }
          account.value.token = undefined
          persist()
          isAuthed.value = false
          return
        }
        const me = await res.json()
        if (me?.status === 'banned') banned.value = 'Your account has been banned.'
        else if (me?.status === 'suspended') banned.value = 'Your account has been suspended.'
        else banned.value = null
        if (typeof me?.credits === 'number' && account.value) {
          account.value.credits = me.credits
          persist()
        }
        return
      }
      const res = await fetch(`${API_BASE}/api/auth/status?hwid=${encodeURIComponent(account.value.hwid)}&_=${++nonce}`)
      if (!res.ok) return
      const data = await res.json()
      if (data?.registered === false && account.value) {
        deleted.value = 'Your account has been deleted by an administrator.'
        return
      }
      if (data?.status === 'banned') banned.value = 'Your account has been banned.'
      else if (data?.status === 'suspended') banned.value = 'Your account has been suspended.'
      else banned.value = null
    } catch {
      // offline: keep current state
    }
  }

  async function init() {
    if (account.value?.token) {
      initialized.value = true
      checkStatus()
      return
    }
    initialized.value = true
    // Pre-cache HWID and PC info so login/register don't block on PowerShell
    getHwidCached().catch(() => {})
    getPcInfoCached().catch(() => {})
  }

  async function register(username: string, email: string, password: string) {
    const hwid = await getHwidCached()
    const pcInfo = await getPcInfoCached()
    const res = await fetch(`${API_BASE}/api/users`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username,
        email,
        password,
        hwid,
        device_model: pcInfo.device_model || 'Unknown device',
        machine_guid: pcInfo.machine_guid || undefined,
        pc_info: pcInfo,
      }),
    })
    const data = await parseJson(res)
    if (!res.ok) throw apiError(data, 'Registration failed')
    const user = data?.user as UserPayload | undefined
    if (!user) throw new Error('Unexpected server response')
    if (user.status && user.status !== 'active') {
      throw new Error('Your account has been banned or suspended.')
    }
    account.value = {
      id: user.id,
      username: user.username,
      email: user.email || '',
      hwid: user.hwid,
      credits: user.credits || 0,
      status: user.status || 'active',
      location: user.location || '',
      device_model: user.device_model || '',
      token: user.session_token ?? undefined,
    }
    persist()
    isAuthed.value = true
    return account.value
  }

  async function login(username: string, password: string) {
    const hwid = await getHwidCached()
    const pcInfo = await getPcInfoCached()
    const res = await fetch(`${API_BASE}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username,
        hwid,
        password,
        machine_guid: pcInfo.machine_guid || undefined,
        pc_info: pcInfo,
      }),
    })
    const data = await parseJson(res)
    if (res.status === 403 && data?.error) {
      throw new Error(typeof data.error === 'string' ? data.error : 'Account not active')
    }
    if (!res.ok) throw apiError(data, 'Login failed')
    const user = data?.user as UserPayload | undefined
    if (!user) throw new Error('Unexpected server response')
    if (user.status && user.status !== 'active') {
      throw new Error('Your account has been banned or suspended.')
    }
    account.value = {
      id: user.id,
      username: user.username,
      email: user.email || '',
      hwid: user.hwid,
      credits: user.credits || 0,
      status: user.status || 'active',
      location: user.location || '',
      device_model: user.device_model || '',
      token: user.session_token ?? undefined,
    }
    persist()
    isAuthed.value = true
    return account.value
  }

  function logout() {
    account.value = null
    banned.value = null
    deleted.value = null
    isAuthed.value = false
    persist()
  }

  async function purchase(source: string, firmwareId: number): Promise<{ link: string; extraction_code?: string | null }> {
    if (!account.value?.token) throw new Error('Not signed in')
    const res = await fetch(`${API_BASE}/api/firmware/purchase`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        source,
        firmware_id: firmwareId,
        token: account.value.token,
        hwid: account.value.hwid,
      }),
    })
    const data = await parseJson(res)
    if (res.status === 401 || res.status === 403) {
      const msg = apiError(data, 'Invalid session').message
      if (String(msg).toLowerCase().includes('not active') || String(msg).toLowerCase().includes('banned')) {
        banned.value = 'Your account has been banned or suspended.'
      } else if (account.value) {
        account.value.token = undefined
        persist()
        isAuthed.value = false
        throw new Error('Session expired. Please sign in again.')
      }
    }
    if (!res.ok) throw apiError(data, 'Purchase failed')
    account.value.credits = (data?.credits_left as number | undefined) ?? account.value.credits
    persist()
    return { link: data?.link as string, extraction_code: (data?.extraction_code as string | null | undefined) ?? null }
  }

  return { account, isAuthed, initialized, banned, deleted, init, checkStatus, register, login, logout, purchase }
})