<template>
  <div class="settings-page">
    <div class="settings-header">
      <h2>Settings</h2>
      <span class="tagline">Appearance and preferences</span>
    </div>

    <div class="settings-section">
      <h3>Theme</h3>
      <p class="section-hint">Choose how the toolkit looks.</p>
      <div class="theme-options">
        <div
          v-for="theme in themes"
          :key="theme.id"
          :class="['theme-card', { active: current === theme.id }]"
          @click="applyTheme(theme.id)"
        >
          <div class="theme-preview" :class="`preview-${theme.id}`">
            <div class="preview-nav"></div>
            <div class="preview-body">
              <div class="preview-line"></div>
              <div class="preview-line short"></div>
            </div>
          </div>
          <span class="theme-name">{{ theme.label }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const themes = [
  { id: 'light', label: 'Default' },
  { id: 'dark', label: 'Dark' },
]

const current = ref('light')

function applyTheme(id) {
  current.value = id
  document.documentElement.setAttribute('data-theme', id)
  localStorage.setItem('v1per-theme', id)
}

onMounted(() => {
  const saved = localStorage.getItem('v1per-theme')
  if (saved) {
    current.value = saved
    document.documentElement.setAttribute('data-theme', saved)
  }
})
</script>

<style scoped>
.settings-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.settings-header { display: flex; align-items: center; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } .tagline { font-size: 12px; color: var(--text-secondary); } }
.settings-section { display: flex; flex-direction: column; gap: 8px; h3 { margin: 0; font-size: 14px; } .section-hint { margin: 0; font-size: 12px; color: var(--text-secondary); } }
.theme-options { display: flex; gap: 16px; flex-wrap: wrap; }
.theme-card { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 12px; border: 2px solid var(--border-primary); border-radius: 10px; background: var(--bg-tertiary); cursor: pointer; transition: border-color 0.2s ease, transform 0.2s ease; &.active { border-color: var(--accent-primary); transform: scale(1.02); } &:hover { border-color: var(--accent-primary); } }
.theme-name { font-size: 13px; font-weight: 600; color: var(--text-primary); }
.theme-preview { width: 120px; height: 80px; border-radius: 6px; overflow: hidden; display: flex; }
.preview-light { background: #f4f6f8; .preview-nav { background: #dfe7ee; } .preview-body { background: #ffffff; } .preview-line { background: #c6d4e0; } }
.preview-dark { background: #232b33; .preview-nav { background: #1a2127; } .preview-body { background: #2c353f; } .preview-line { background: #4b5a68; } }
.preview-nav { width: 22%; height: 100%; }
.preview-body { flex: 1; padding: 10px; display: flex; flex-direction: column; gap: 6px; }
.preview-line { height: 8px; border-radius: 2px; &.short { width: 60%; } }
</style>