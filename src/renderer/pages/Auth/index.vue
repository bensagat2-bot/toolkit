<template>
  <div class="auth-page">
    <div class="auth-card">
      <div class="auth-logo">
        <img src="@/assets/images/vxper.png" alt="V1Per" />
      </div>
      <h2>Create Your V1Per Account</h2>
      <p class="auth-sub">Your account is linked to this device (HWID) so credits stay with you.</p>

      <form class="auth-form" @submit.prevent="submit">
        <div class="field">
          <label>Username</label>
          <input v-model="username" class="text-input" placeholder="Choose a username" required autocomplete="username" />
        </div>
        <div class="field">
          <label>Email</label>
          <input v-model="email" type="email" class="text-input" placeholder="you@email.com" autocomplete="email" />
        </div>
        <div class="field">
          <label>Password</label>
          <input v-model="password" type="password" class="text-input" placeholder="Create a password" required autocomplete="new-password" />
        </div>
        <div class="field">
          <label>Confirm Password</label>
          <input v-model="confirm" type="password" class="text-input" placeholder="Repeat password" required autocomplete="new-password" />
        </div>

        <div class="error-msg" v-if="error">{{ error }}</div>

        <button class="btn btn-primary" type="submit" :disabled="loading">
          {{ loading ? 'Creating account...' : 'Create Account' }}
        </button>
      </form>

      <div class="auth-foot">
        <span class="hwid-line" v-if="hwid">HWID: <strong>{{ hwid }}</strong></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAccountStore } from '@renderer/store/accountStore'
import { sendIpcToMain } from '@renderer/utils/ipc'

const store = useAccountStore()
const username = ref('')
const email = ref('')
const password = ref('')
const confirm = ref('')
const error = ref('')
const loading = ref(false)
const hwid = ref('')

onMounted(async () => {
  try {
    hwid.value = await sendIpcToMain<string>('get_hwid')
  } catch {
    hwid.value = ''
  }
})

async function submit() {
  error.value = ''
  if (password.value.length < 6) {
    error.value = 'Password must be at least 6 characters.'
    return
  }
  if (password.value !== confirm.value) {
    error.value = 'Passwords do not match.'
    return
  }
  loading.value = true
  try {
    await store.register(username.value, email.value, password.value)
  } catch (e: any) {
    error.value = e?.message || 'Failed to create account. Check your connection.'
  }
  loading.value = false
}
</script>

<style scoped>
.auth-page { height: 100%; display: flex; align-items: center; justify-content: center; background: var(--bg-primary); padding: 24px; }
.auth-card { width: 100%; max-width: 380px; display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 32px; border: 1px solid var(--border-primary); border-radius: 12px; background: var(--bg-secondary); box-shadow: 0 4px 16px rgba(0,0,0,0.08); }
.auth-logo { width: 64px; margin-bottom: 8px; }
.auth-logo img { width: 100%; height: auto; }
.auth-card h2 { font-size: 18px; margin: 0; }
.auth-sub { font-size: 12px; color: var(--text-secondary); text-align: center; margin: 0 0 12px; }
.auth-form { width: 100%; display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 4px; label { font-size: 11px; color: var(--text-secondary); } }
.text-input { width: 100%; background: var(--bg-primary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 8px 10px; font-size: 13px; color: var(--text-primary); outline: none; }
.text-input:focus { border-color: var(--accent-primary); }
.btn { width: 100%; padding: 10px; font-size: 13px; }
.error-msg { font-size: 12px; color: #f44336; text-align: center; }
.auth-foot { margin-top: 8px; }
.hwid-line { font-size: 10px; color: var(--text-secondary); strong { font-family: monospace; color: var(--text-primary); } }
</style>