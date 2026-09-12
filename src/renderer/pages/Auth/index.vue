<template>
  <div class="auth-page">
    <transition name="auth">
      <div class="auth-wrap" v-if="!loading">
        <h2 class="auth-title">Create Your Account</h2>
        <p class="auth-sub">Join V1Per Toolkit and start servicing devices.</p>

        <form class="auth-form" @submit.prevent="submit">
          <div class="field" v-for="(f, i) in fields" :key="f.key" :style="{ '--i': i }">
            <input
              v-model="f.value"
              :type="f.type"
              class="text-input"
              :placeholder="f.placeholder"
              :required="f.required"
              :autocomplete="f.autocomplete"
            />
          </div>

          <transition name="fade">
            <div class="error-msg" v-if="error">{{ error }}</div>
          </transition>

          <button class="btn btn-primary" type="submit">
            Create Account
          </button>
        </form>

        <p class="auth-foot">By creating an account you agree to the toolkit terms.</p>
      </div>
    </transition>

    <transition name="auth">
      <div class="loading-wrap" v-if="loading">
        <div class="loading-ring"></div>
        <p class="loading-text">Creating your account...</p>
        <p class="loading-sub">Please wait a moment</p>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useAccountStore } from '@renderer/store/accountStore'

const store = useAccountStore()
const loading = ref(false)
const error = ref('')

const fields = reactive([
  { key: 'username', value: '', type: 'text', placeholder: 'Username', required: true, autocomplete: 'username' },
  { key: 'email', value: '', type: 'email', placeholder: 'Email (optional)', required: false, autocomplete: 'email' },
  { key: 'password', value: '', type: 'password', placeholder: 'Password', required: true, autocomplete: 'new-password' },
  { key: 'confirm', value: '', type: 'password', placeholder: 'Confirm Password', required: true, autocomplete: 'new-password' },
])

async function submit() {
  error.value = ''
  const username = fields[0].value.trim()
  const email = fields[1].value.trim()
  const password = fields[2].value
  const confirm = fields[3].value

  if (password.length < 6) {
    error.value = 'Password must be at least 6 characters.'
    return
  }
  if (password !== confirm) {
    error.value = 'Passwords do not match.'
    return
  }

  loading.value = true
  try {
    await store.register(username, email, password)
  } catch (e: any) {
    error.value = e?.message || 'Failed to create account. Check your connection.'
    loading.value = false
  }
}
</script>

<style scoped>
.auth-page { position: relative; height: 100%; display: flex; align-items: center; justify-content: center; background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%); overflow: hidden; }
.auth-page::before, .auth-page::after { content: ''; position: absolute; border-radius: 50%; filter: blur(60px); opacity: 0.5; pointer-events: none; }
.auth-page::before { width: 360px; height: 360px; background: radial-gradient(circle, rgba(77,131,175,0.35), transparent 70%); top: -80px; left: -100px; animation: float 9s ease-in-out infinite; }
.auth-page::after { width: 420px; height: 420px; background: radial-gradient(circle, rgba(96,165,250,0.3), transparent 70%); bottom: -120px; right: -120px; animation: float 12s ease-in-out infinite reverse; }
@keyframes float { 0%, 100% { transform: translateY(0) } 50% { transform: translateY(-24px) } }

.auth-wrap { position: relative; z-index: 1; width: 100%; max-width: 340px; display: flex; flex-direction: column; align-items: center; gap: 10px; text-align: center; }
.auth-title { font-size: 24px; font-weight: 700; margin: 0; background: linear-gradient(135deg, var(--text-primary), var(--accent-primary)); -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent; }
.auth-sub { font-size: 13px; color: var(--text-secondary); margin: 0 0 14px; }
.auth-form { width: 100%; display: flex; flex-direction: column; gap: 12px; }
.field { opacity: 0; animation: fadeUp 0.5s ease forwards; animation-delay: calc(var(--i) * 0.08s); }
@keyframes fadeUp { from { opacity: 0; transform: translateY(14px); } to { opacity: 1; transform: translateY(0); } }
.text-input { width: 100%; background: rgba(255,255,255,0.06); border: 1px solid var(--border-primary); border-radius: 8px; padding: 12px 14px; font-size: 14px; color: var(--text-primary); outline: none; transition: border-color 0.25s ease, box-shadow 0.25s ease, background 0.25s ease; }
.text-input:focus { border-color: var(--accent-primary); box-shadow: 0 0 0 3px rgba(77,131,175,0.15); background: rgba(255,255,255,0.1); }
.btn { width: 100%; padding: 12px; font-size: 14px; font-weight: 600; border-radius: 8px; cursor: pointer; border: none; background: linear-gradient(135deg, var(--accent-primary), #60a5fa); color: #fff; transition: transform 0.2s ease, box-shadow 0.2s ease, opacity 0.2s ease; }
.btn:hover { transform: translateY(-1px); box-shadow: 0 6px 18px rgba(77,131,175,0.35); }
.btn:active { transform: translateY(0); opacity: 0.9; }
.error-msg { font-size: 12px; color: #f44336; }
.auth-foot { font-size: 11px; color: var(--text-secondary); opacity: 0.8; margin: 6px 0 0; }

.loading-wrap { position: absolute; inset: 0; z-index: 2; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%); }
.loading-ring { width: 46px; height: 46px; border: 3px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.9s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.loading-text { font-size: 15px; font-weight: 600; margin: 0; color: var(--text-primary); }
.loading-sub { font-size: 12px; color: var(--text-secondary); margin: 0; }

.auth-enter-active, .auth-leave-active { transition: opacity 0.35s ease, transform 0.35s ease; }
.auth-enter-from, .auth-leave-to { opacity: 0; transform: scale(0.96); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.25s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>