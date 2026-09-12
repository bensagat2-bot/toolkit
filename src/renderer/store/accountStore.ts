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

const STORAGE_KEY = 'v1per_account'
const API_BASE = 'https://firmwaresss-devices.vercel.app'

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

  const isAuthed = ref(!!account.value)

  function persist() {
    if (account.value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(account.value))
    } else {
      localStorage.removeItem(STORAGE_KEY)
    }
  }

  async function init() {
    if (account.value?.token) {
      initialized.value = true
      return
    }
    initialized.value = true
  }

  async function register(username: string, email: string, password: string) {
    const hwid = await sendIpcToMain<string>('get_hwid')
    const deviceModel = await sendIpcToMain<string>('get_device_model')
    const res = await fetch(`${API_BASE}/api/users`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username,
        email,
        password,
        hwid,
        device_model: deviceModel,
      }),
    })
    const data = await parseJson(res)
    if (!res.ok) throw apiError(data, 'Registration failed')
    const user = data?.user as Record<string, unknown> | undefined
    if (!user) throw new Error('Unexpected server response')
    account.value = {
      id: user.id,
      username: user.username,
      email: user.email || '',
      hwid: user.hwid,
      credits: user.credits || 0,
      status: user.status || 'active',
      location: user.location || '',
      device_model: user.device_model || '',
      token: user.session_token,
    }
    persist()
    isAuthed.value = true
    return account.value
  }

  async function login(password: string) {
    const hwid = await sendIpcToMain<string>('get_hwid')
    const res = await fetch(`${API_BASE}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ hwid, password }),
    })
    const data = await parseJson(res)
    if (!res.ok) throw apiError(data, 'Login failed')
    const user = data?.user as Record<string, unknown> | undefined
    if (!user) throw new Error('Unexpected server response')
    account.value = {
      id: user.id,
      username: user.username,
      email: user.email || '',
      hwid: user.hwid,
      credits: user.credits || 0,
      status: user.status || 'active',
      location: user.location || '',
      device_model: user.device_model || '',
      token: user.session_token,
    }
    persist()
    isAuthed.value = true
    return account.value
  }

  function logout() {
    account.value = null
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
      }),
    })
    const data = await parseJson(res)
    if (!res.ok) throw apiError(data, 'Purchase failed')
    account.value.credits = (data?.credits_left as number | undefined) ?? account.value.credits
    persist()
    return { link: data?.link as string, extraction_code: (data?.extraction_code as string | null | undefined) ?? null }
  }

  return { account, isAuthed, initialized, init, register, login, logout, purchase }
})