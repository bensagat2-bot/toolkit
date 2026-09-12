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

async function sha256(text: string): Promise<string> {
  const data = new TextEncoder().encode(text)
  const digest = await crypto.subtle.digest('SHA-256', data)
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
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
    if (account.value) {
      initialized.value = true
      return
    }
    try {
      const hwid = await sendIpcToMain<string>('get_hwid')
      const res = await fetch(`${API_BASE}/api/users?hwid=${encodeURIComponent(hwid)}`)
      const data = await res.json()
      const existing = (data.users || []).find((u: any) => u.hwid === hwid)
      if (existing) {
        account.value = {
          id: existing.id,
          username: existing.username,
          email: existing.email || '',
          hwid: existing.hwid,
          credits: existing.credits || 0,
          status: existing.status || 'active',
          location: existing.location || '',
          device_model: existing.device_model || '',
        }
        persist()
        isAuthed.value = true
      }
    } catch {
      // offline: stay unauthed, user can still browse free pages
    }
    initialized.value = true
  }

  async function register(username: string, email: string, password: string) {
    const hwid = await sendIpcToMain<string>('get_hwid')
    const deviceModel = await sendIpcToMain<string>('get_device_model')
    const passwordHash = await sha256(password)
    const res = await fetch(`${API_BASE}/api/users`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username,
        email,
        password_hash: passwordHash,
        hwid,
        credits: 0,
        status: 'active',
        location: '',
        device_model: deviceModel,
      }),
    })
    const data = await res.json()
    if (!res.ok) throw new Error(data.error || 'Registration failed')
    const user = data.user
    account.value = {
      id: user.id,
      username: user.username,
      email: user.email || '',
      hwid: user.hwid,
      credits: user.credits || 0,
      status: user.status || 'active',
      location: user.location || '',
      device_model: user.device_model || '',
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

  function updateCredits(credits: number) {
    if (!account.value) return
    account.value.credits = credits
    persist()
    syncCreditsToServer(credits)
  }

  async function syncCreditsToServer(credits: number) {
    if (!account.value?.id) return
    try {
      await fetch(`${API_BASE}/api/users`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: account.value.id,
          credits,
          reason: 'Toolkit firmware download',
        }),
      })
    } catch {
      // offline: local count stays; server syncs later
    }
  }

  async function refreshCredits() {
    if (!account.value) return
    try {
      const res = await fetch(`${API_BASE}/api/users?hwid=${encodeURIComponent(account.value.hwid)}`)
      const data = await res.json()
      const fresh = (data.users || []).find((u: any) => u.hwid === account.value.hwid)
      if (fresh && fresh.credits !== account.value.credits) {
        account.value.credits = fresh.credits || 0
        persist()
      }
    } catch {
      // offline: ignore
    }
  }

  return { account, isAuthed, initialized, init, register, logout, updateCredits, refreshCredits }
})