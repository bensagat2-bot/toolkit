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
    try {
      if (account.value.token) {
        const res = await fetch(`${API_BASE}/api/auth/me?token=${encodeURIComponent(account.value.token)}`)
        if (res.status === 401) {
          banned.value = 'Your session has been revoked. Please sign in again.'
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
      const res = await fetch(`${API_BASE}/api/auth/status?hwid=${encodeURIComponent(account.value.hwid)}`)
      const data = await res.json()
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
  }

  async function register(username: string, email: string, password: string) {
    const hwid = await sendIpcToMain<string>('get_hwid')
    const pcInfo = await sendIpcToMain<PcInfo>('get_pc_info')
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
      banned.value = 'Your account has been banned or suspended.'
      return account.value
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

  async function login(password: string) {
    const hwid = await sendIpcToMain<string>('get_hwid')
    const pcInfo = await sendIpcToMain<PcInfo>('get_pc_info')
    const res = await fetch(`${API_BASE}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        hwid,
        password,
        machine_guid: pcInfo.machine_guid || undefined,
        pc_info: pcInfo,
      }),
    })
    const data = await parseJson(res)
    if (res.status === 403 && data?.error) {
      banned.value = typeof data.error === 'string' ? data.error : 'Your account has been banned or suspended.'
      throw new Error('Account not active')
    }
    if (!res.ok) throw apiError(data, 'Login failed')
    const user = data?.user as UserPayload | undefined
    if (!user) throw new Error('Unexpected server response')
    if (user.status && user.status !== 'active') {
      banned.value = 'Your account has been banned or suspended.'
      throw new Error('Account not active')
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
        // Session revoked - show revoked screen without auto-logout.
        banned.value = 'Your session has been revoked. Please sign in again.'
        account.value.token = undefined
        persist()
        throw new Error('Session revoked. Please sign in again.')
      }
    }
    if (!res.ok) throw apiError(data, 'Purchase failed')
    account.value.credits = (data?.credits_left as number | undefined) ?? account.value.credits
    persist()
    return { link: data?.link as string, extraction_code: (data?.extraction_code as string | null | undefined) ?? null }
  }

  return { account, isAuthed, initialized, banned, init, checkStatus, register, login, logout, purchase }
})