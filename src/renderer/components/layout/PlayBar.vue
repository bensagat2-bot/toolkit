<template>
  <div class="player">
    <div class="picContent">
      <div class="emptyPic">V<span>1</span></div>
    </div>
    <div class="infoContent">
      <div class="title">V1Per Servicing Toolkit</div>
      <div class="status">{{ statusText }}</div>
    </div>
    <div class="btnContent">
      <div class="btn" aria-label="Drivers" title="Drivers" @click="goTo('/drivers')">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
          <polyline points="9 22 9 12 15 12 15 22" />
        </svg>
      </div>
      <div class="btn" aria-label="Refresh" title="Refresh" @click="refresh">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="23 4 23 10 17 10" />
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
        </svg>
      </div>
      <div class="btn" aria-label="Settings" title="Settings" @click="goTo('/settings')">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const statusText = computed(() => {
  return route.name ? `${route.name} · Ready` : 'Ready'
})

const goTo = (path: string) => {
  void router.push(path)
}

const refresh = () => {
  window.location.reload()
}
</script>

<style scoped>
.player {
  position: relative;
  height: 66px;
  border-top: 1px solid var(--color-primary-alpha-900);
  box-sizing: border-box;
  display: flex;
  flex-flow: row nowrap;
  align-items: center;
  padding: 6px;
  z-index: 2;
}

.player::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  background-color: var(--color-main-background);
  opacity: 0.9;
  z-index: -1;
}

.picContent {
  height: 100%;
  aspect-ratio: 1 / 1;
  flex: none;
}

.emptyPic {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-primary-light-900-alpha-200);
  border-radius: 4px;
  color: var(--color-primary-light-400-alpha-800);
  user-select: none;
  font-size: 20px;
  font-family: Consolas, "Courier New", monospace;
}

.emptyPic span {
  padding-left: 3px;
}

.infoContent {
  padding: 0 10px;
  flex: auto;
  display: flex;
  flex-flow: column nowrap;
  justify-content: center;
  align-items: flex-start;
  font-size: 13px;
  color: var(--color-font);
  min-width: 0;
  line-height: 1.5;
}

.title {
  max-width: 100%;
  font-size: 12px;
  color: var(--color-font-label);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.status {
  padding-top: 3px;
  height: 23px;
  max-width: 100%;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.btnContent {
  height: 100%;
  flex: none;
  display: flex;
  flex-flow: row nowrap;
  align-items: center;
  padding-left: 10px;
  padding-right: 15px;
  gap: 18px;
}

.btn {
  flex: none;
  height: 52%;
  transition: color 0.3s ease, opacity 0.3s ease;
  color: var(--color-button-font);
  cursor: pointer;
}

.btn svg {
  height: 100%;
  filter: drop-shadow(0 0 1px rgba(0, 0, 0, 0.2));
}

.btn:hover {
  opacity: 0.8;
}

.btn:active {
  opacity: 0.6;
}
</style>