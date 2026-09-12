<template>
  <div class="banned-page">
    <div class="banned-card">
      <div class="icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <path d="M4.9 4.9l14.2 14.2" />
        </svg>
      </div>
      <h2 class="title">Access Revoked</h2>
      <p class="sub">{{ message }}</p>
      <button class="btn" @click="signOut">Sign out</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAccountStore } from '@/store/accountStore'

defineProps<{ message: string }>()
const store = useAccountStore()

function signOut() {
  store.logout()
}
</script>

<style scoped>
.banned-page {
  position: relative;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
}
.banned-card {
  max-width: 360px;
  width: 90%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
  padding: 28px 22px;
  border: 1px solid rgba(244, 67, 54, 0.35);
  border-radius: 12px;
  background: rgba(244, 67, 54, 0.06);
}
.icon {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #f44336;
  background: rgba(244, 67, 54, 0.14);
}
.icon svg {
  width: 26px;
  height: 26px;
}
.title {
  font-size: 20px;
  font-weight: 700;
  margin: 0;
  color: #f44336;
}
.sub {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}
.btn {
  margin-top: 8px;
  padding: 9px 18px;
  font-size: 13px;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
  border: none;
  background: rgba(244, 67, 54, 0.18);
  color: #f44336;
  transition: background 0.2s ease;
}
.btn:hover {
  background: rgba(244, 67, 54, 0.28);
}
</style>