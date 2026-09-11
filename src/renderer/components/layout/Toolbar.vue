<template>
  <div class="toolbar">
    <SearchInput />
    <div class="controls">
      <button type="button" aria-label="Minimize" title="Minimize" @click="minimizeWindow">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
      <button type="button" class="close" aria-label="Close" title="Close" @click="closeWindow">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="6" y1="6" x2="18" y2="18" />
          <line x1="18" y1="6" x2="6" y2="18" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SearchInput from './SearchInput.vue'

const getWindow = async () => {
  if (!window.__TAURI_INTERNALS__) return null
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  return getCurrentWindow()
}

const minimizeWindow = async () => {
  const win = await getWindow()
  if (win) await win.minimize()
}

const closeWindow = async () => {
  const win = await getWindow()
  if (win) await win.close()
}
</script>

<style scoped>
.toolbar {
  display: flex;
  height: 54px;
  align-items: center;
  justify-content: space-between;
  padding-left: 15px;
  -webkit-app-region: drag;
  z-index: 2;
}

.controls {
  display: flex;
  align-self: flex-start;
  -webkit-app-region: no-drag;
  height: 30px;
}

.controls button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 30px;
  padding: 1px;
  background: none;
  border: none;
  outline: none;
  cursor: pointer;
  color: var(--color-font-label);
  transition: background-color 0.2s ease-in-out;
}

.controls button:hover {
  background-color: var(--color-button-background-hover);
}

.controls button.close:hover {
  background-color: var(--color-btn-close);
}

.controls svg {
  height: 14px;
}
</style>