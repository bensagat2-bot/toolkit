<template>
  <div class="debloat-page">
    <div class="debloat-header">
      <h2>Debloat Apps</h2>
      <div class="search-bar">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
        <input v-model="search" placeholder="Search apps..." />
      </div>
    </div>

    <div class="debloat-toolbar">
      <button class="btn btn-primary" @click="loadApps" :disabled="loading">
        {{ loading ? 'Loading...' : 'Load Apps' }}
      </button>
      <div class="toolbar-actions" v-if="apps.length">
        <span class="selection-count">{{ selected.length }} / {{ apps.length }}</span>
        <button class="btn btn-sm" :disabled="!selected.length" @click="uninstallSelected">Uninstall</button>
        <button class="btn btn-sm" :disabled="!selected.length" @click="enableSelected">Restore</button>
        <button class="btn-link" @click="selectAll">All</button>
        <button class="btn-link" @click="deselectAll">None</button>
        <div class="filter-group">
          <button :class="['btn-tab', { active: filter === 'all' }]" @click="filter = 'all'">All</button>
          <button :class="['btn-tab', { active: filter === 'system' }]" @click="filter = 'system'">System</button>
          <button :class="['btn-tab', { active: filter === 'user' }]" @click="filter = 'user'">User</button>
          <button :class="['btn-tab', { active: filter === 'disabled' }]" @click="filter = 'disabled'">Disabled</button>
        </div>
      </div>
    </div>

    <div class="app-list-wrapper">
      <transition name="fade">
        <div class="loading-overlay" v-if="loading">
          <div class="loading-card">
            <div class="loading-spinner"></div>
            <p class="loading-title">Please wait while loading all the apps...</p>
            <p class="loading-sub">Fetching installed packages from device</p>
          </div>
        </div>
      </transition>

      <div class="app-list scroll" v-if="!loading && filteredApps.length">
        <div class="app-grid">
          <div
            v-for="(app, i) in filteredApps"
            :key="app.package_name"
            :class="['app-card', { selected: isSelected(app) }]"
            :style="{ '--i': i }"
          >
            <div class="app-card-inner">
              <div class="app-icon" :style="{ background: icons[app.package_name] ? 'transparent' : iconColor(app.package_name) }">
                <img v-if="icons[app.package_name]" :src="'data:image/png;base64,' + icons[app.package_name]" class="app-icon-img" />
                <span v-else class="app-icon-text">{{ app.app_name.charAt(0).toUpperCase() }}</span>
              </div>
              <div class="app-info">
                <div class="app-name" :title="app.app_name">{{ app.app_name }}</div>
                <div class="app-package" :title="app.package_name">{{ app.package_name }}</div>
                <div class="app-tags">
                  <span v-if="app.is_system" class="tag-system">System</span>
                  <span v-if="!app.is_system" class="tag-user">User</span>
                  <span v-if="app.is_disabled" class="tag-disabled">Disabled</span>
                </div>
              </div>
              <div class="app-actions">
                <input type="checkbox" :checked="isSelected(app)" @change="toggle(app)" />
              </div>
            </div>
          </div>
        </div>

        <div class="list-padding">
          <div v-for="i in 6" :key="i" style="height: 0; margin-bottom: 0;" />
        </div>
      </div>

      <div class="empty-state" v-if="!loading && !filteredApps.length && !apps.length">
        <div class="empty-icon">D</div>
        <h3>Debloat Apps</h3>
        <p>Click "Load Apps" to list all installed packages from your connected device.</p>
      </div>

      <div class="empty-state" v-if="!loading && !filteredApps.length && apps.length && search">
        <p>No apps match "{{ search }}".</p>
      </div>
    </div>

    <transition name="toast">
      <div class="toast-msg" v-if="toast" :class="toastType">{{ toast }}</div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, nextTick } from 'vue'
import { sendIpcToMain } from '@renderer/utils/ipc'

const apps = ref([])
const icons = ref({})
const selected = ref([])
const search = ref('')
const filter = ref('all')
const loading = ref(false)
const toast = ref('')
const toastType = ref('')

const filteredApps = computed(() => {
  let list = apps.value
  const q = search.value.trim().toLowerCase()
  if (q) list = list.filter(a => a.app_name.toLowerCase().includes(q) || a.package_name.toLowerCase().includes(q))
  if (filter.value === 'system') list = list.filter(a => a.is_system)
  if (filter.value === 'user') list = list.filter(a => !a.is_system)
  if (filter.value === 'disabled') list = list.filter(a => a.is_disabled)
  return list
})

function isSelected(app) {
  return selected.value.includes(app.package_name)
}

function toggle(app) {
  const idx = selected.value.indexOf(app.package_name)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(app.package_name)
}

function selectAll() {
  selected.value = filteredApps.value.map(a => a.package_name)
}

function deselectAll() {
  selected.value = []
}

function showToast(msg, type = 'info') {
  toast.value = msg
  toastType.value = type
  setTimeout(() => { toast.value = '' }, 4000)
}

async function loadApps() {
  loading.value = true
  try {
    await sendIpcToMain('ensure_su_access')
    const result = await sendIpcToMain('debloat_list_packages')
    apps.value = result
    selected.value = []
    showToast(`Loaded ${result.length} apps`, 'success')
    fetchIcons(result.map((a) => a.package_name))
  } catch (e) {
    showToast(e?.message || 'Failed to load apps', 'error')
  }
  loading.value = false
}

async function fetchIcons(packages) {
  try {
    const batch = await sendIpcToMain('debloat_get_icons', { packageNames: packages })
    if (batch) icons.value = batch
  } catch {
    // Icons are optional, fallback to letter+color
  }
}

async function uninstallSelected() {
  if (!selected.value.length) return
  loading.value = true
  let successCount = 0
  let failCount = 0
  for (const pkg of [...selected.value]) {
    try {
      const result = await sendIpcToMain('debloat_uninstall', { package_name: pkg })
      apps.value = apps.value.filter(a => a.package_name !== pkg)
      successCount++
    } catch (e) {
      failCount++
      showToast(`Failed: ${pkg} - ${e?.message || ''}`, 'error')
    }
  }
  selected.value = []
  loading.value = false
  if (successCount > 0) {
    showToast(`Uninstalled ${successCount} app(s)${failCount > 0 ? `, ${failCount} failed` : ''}`, failCount > 0 ? 'warn' : 'success')
  }
}

async function enableSelected() {
  if (!selected.value.length) return
  loading.value = true
  let successCount = 0
  let failCount = 0
  for (const pkg of [...selected.value]) {
    try {
      const result = await sendIpcToMain('debloat_enable', { package_name: pkg })
      const app = apps.value.find(a => a.package_name === pkg)
      if (app) app.is_disabled = false
      successCount++
    } catch (e) {
      failCount++
      showToast(`Failed: ${pkg} - ${e?.message || ''}`, 'error')
    }
  }
  selected.value = []
  loading.value = false
  if (successCount > 0) {
    showToast(`Restored ${successCount} app(s)${failCount > 0 ? `, ${failCount} failed` : ''}`, failCount > 0 ? 'warn' : 'success')
  }
}

function iconColor(pkg) {
  let hash = 0
  for (let i = 0; i < pkg.length; i++) hash = pkg.charCodeAt(i) + ((hash << 5) - hash)
  const colors = ['#4daf7c', '#3ba7a0', '#5b8def', '#e0a03d', '#7d5be0', '#e05b7a', '#2f8f5b', '#1f7f77', '#345fbe', '#b87a1f', '#5a34be', '#be3450']
  return colors[Math.abs(hash) % colors.length]
}
</script>

<style scoped>
.debloat-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 12px; animation: pageIn 0.35s ease; }

.debloat-header { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.debloat-header h2 { font-size: 16px; margin: 0; }

.search-bar { flex: 1; display: flex; align-items: center; gap: 8px; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 6px 10px; min-width: 0; }
.search-bar svg { width: 14px; height: 14px; color: var(--text-secondary); flex-shrink: 0; }
.search-bar input { flex: 1; min-width: 0; background: none; border: none; outline: none; font-size: 13px; color: var(--text-primary); }

.debloat-toolbar { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; padding-bottom: 8px; border-bottom: 1px solid var(--border-primary); }
.toolbar-actions { display: flex; align-items: center; gap: 10px; flex: 1; flex-wrap: wrap; }
.selection-count { font-size: 11px; color: var(--accent-primary); font-weight: 600; white-space: nowrap; }
.filter-group { display: flex; gap: 2px; margin-left: auto; border: 1px solid var(--border-primary); border-radius: 4px; overflow: hidden; }
.btn-tab { padding: 3px 10px; border: none; background: var(--bg-secondary); color: var(--text-secondary); font-size: 11px; cursor: pointer; transition: all 0.15s; }
.btn-tab.active { background: var(--accent-primary); color: #fff; }
.btn-tab:hover:not(.active) { background: var(--bg-tertiary); }

.app-list-wrapper { flex: 1; min-height: 0; position: relative; overflow: hidden; }
.app-list { height: 100%; overflow-y: auto; padding: 8px 0; }
.app-grid { display: flex; flex-flow: row wrap; gap: 12px; }

.app-card {
  width: calc(33.33% - 8px); max-width: 360px; box-sizing: border-box;
  border: 1px solid var(--border-primary); border-radius: 8px; background: var(--bg-primary);
  cursor: pointer; transition: all 0.2s ease; animation: cardFade 0.35s ease both;
  animation-delay: calc(var(--i) * 0.025s);
}
.app-card:hover { border-color: var(--accent-primary); box-shadow: 0 2px 8px rgba(77,131,175,0.12); }
.app-card.selected { border-color: var(--accent-primary); background: color-mix(in srgb, var(--accent-primary) 6%, transparent); }
@keyframes cardFade { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }

.app-card-inner { display: flex; align-items: center; gap: 10px; padding: 10px; }

.app-icon { flex: none; width: 38px; height: 38px; border-radius: 8px; display: flex; align-items: center; justify-content: center; overflow: hidden; }
.app-icon-img { width: 38px; height: 38px; object-fit: cover; border-radius: 8px; }
.app-icon-text { font-size: 18px; font-weight: 700; color: #fff; text-shadow: 0 1px 2px rgba(0,0,0,0.2); }

.app-info { flex: 1; min-width: 0; }
.app-name { font-size: 12px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.app-package { font-size: 10px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-top: 1px; }
.app-tags { display: flex; gap: 4px; margin-top: 3px; flex-wrap: wrap; }
.app-tags span { font-size: 9px; padding: 1px 6px; border-radius: 8px; font-weight: 500; }
.tag-system { background: rgba(77,131,175,0.12); color: var(--accent-primary); }
.tag-user { background: rgba(76,175,80,0.12); color: #4caf50; }
.tag-disabled { background: rgba(244,67,54,0.12); color: #f44336; }

.app-actions { flex: none; display: flex; align-items: center; }
.app-actions input { accent-color: var(--accent-primary); cursor: pointer; width: 16px; height: 16px; }

.list-padding { height: 1px; }

.loading-overlay { position: absolute; inset: 0; z-index: 10; display: flex; align-items: center; justify-content: center; backdrop-filter: blur(3px); }
.loading-card { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 40px 50px; border-radius: 12px; background: var(--bg-secondary); border: 1px solid var(--border-primary); box-shadow: 0 4px 24px rgba(0,0,0,0.15); }
.loading-spinner { width: 40px; height: 40px; border: 3px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.9s linear infinite; }
.loading-title { font-size: 15px; font-weight: 600; margin: 0; color: var(--text-primary); }
.loading-sub { font-size: 12px; margin: 0; color: var(--text-secondary); animation: pulse 1.4s ease-in-out infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }

.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; text-align: center; padding: 60px 20px; height: 100%; }
.empty-icon { width: 52px; height: 52px; border-radius: 12px; background: linear-gradient(135deg, var(--accent-primary), #60a5fa); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 22px; font-weight: 700; }
.empty-state h3 { font-size: 16px; margin: 0; }
.empty-state p { font-size: 12px; color: var(--text-secondary); margin: 0; }

.toast-msg { position: fixed; bottom: 80px; left: 50%; transform: translateX(-50%); z-index: 9999; padding: 10px 20px; border-radius: 8px; background: var(--bg-secondary); border: 1px solid var(--accent-primary); font-size: 12px; color: var(--text-primary); box-shadow: 0 4px 16px rgba(0,0,0,0.2); }
.toast-msg.error { border-color: #f44336; color: #f44336; }
.toast-msg.success { border-color: #4caf50; color: #4caf50; }
.toast-msg.warn { border-color: #ff9800; color: #ff9800; }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 12px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; white-space: nowrap; transition: all 0.15s; }
.btn:hover:not(:disabled) { border-color: var(--accent-primary); }
.btn:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-sm { padding: 4px 10px; font-size: 11px; }
.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 11px; cursor: pointer; padding: 0; }
.btn-link:hover { text-decoration: underline; }

.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.toast-enter-active, .toast-leave-active { transition: all 0.25s ease; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }

@keyframes pageIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
</style>