<template>
  <div class="search">
    <svg
      class="icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
    <input
      v-model="query"
      class="scroll"
      type="text"
      placeholder="Search drivers"
      aria-label="Search drivers"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useDriversStore } from '@/stores/drivers'

const store = useDriversStore()
const query = computed({
  get: () => store.query,
  set: (value: string) => store.setQuery(value),
})
</script>

<style scoped>
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 260px;
  padding: 6px 12px;
  border-radius: 4px;
  background-color: var(--color-button-background);
  color: var(--color-font-label);
  -webkit-app-region: no-drag;
}

.icon {
  flex: none;
  width: 14px;
  height: 14px;
}

input {
  flex: auto;
  border: none;
  outline: none;
  background: transparent;
  font-size: 13px;
  color: var(--color-font);
}
</style>