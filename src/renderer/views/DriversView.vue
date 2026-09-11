<template>
  <div class="container">
    <div class="header">
      <div class="left">
        <div :class="['select', { active: popupVisible }]">
          <div class="select-label" @click.stop="popupVisible = !popupVisible">
            <span>{{ currentCategory }}</span>
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </div>
          <div class="popup">
            <ul class="scroll">
              <li
                v-for="cat in categories"
                :key="cat.id"
                :class="{ active: store.category === cat.id }"
                @click="handleCategory(cat.id)"
              >
                {{ cat.name }}
              </li>
            </ul>
          </div>
        </div>
        <div class="tabs">
          <span
            v-for="s in sorts"
            :key="s.id"
            :class="['tab', { active: store.sort === s.id }]"
            @click="store.setSort(s.id)"
          >
            {{ s.name }}
          </span>
        </div>
      </div>
      <button type="button" class="refresh" aria-label="Refresh" title="Refresh" @click="refresh">
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
      </button>
    </div>

    <div class="list">
      <div ref="listRef" class="list-content scroll">
        <ul v-if="store.total">
          <li v-for="item in store.pageList" :key="item.id" class="item">
            <div class="thumb" :style="{ background: item.gradient }">
              <span class="initials">{{ item.initials }}</span>
              <span class="source">{{ item.source }}</span>
            </div>
            <div class="desc">
              <h4>{{ item.name }}</h4>
              <p class="description">{{ item.description }}</p>
              <div class="meta">
                <span class="badge">{{ item.version }}</span>
                <span>{{ item.size }}</span>
                <span>{{ item.platform }}</span>
              </div>
              <div class="footer">
                <span class="stats">{{ item.downloads }} downloads</span>
                <a class="download" :href="item.downloadUrl" :download="`${item.id}-${item.version}`">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                    <polyline points="7 10 12 15 17 10" />
                    <line x1="12" y1="15" x2="12" y2="3" />
                  </svg>
                  <span>Direct Download</span>
                </a>
              </div>
            </div>
          </li>
          <li v-for="i in 6" :key="'pad' + i" class="pad" />
        </ul>
        <div v-else class="noitem">
          <p>No drivers found</p>
        </div>
        <div v-if="store.maxPage > 1" class="pagination">
          <ul>
            <li>
              <button
                type="button"
                aria-label="Previous page"
                :disabled="store.page === 1"
                @click="store.setPage(store.page - 1)"
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <polyline points="15 18 9 12 15 6" />
                </svg>
              </button>
            </li>
            <li v-for="p in store.maxPage" :key="p" :class="{ active: p === store.page }">
              <button type="button" @click="store.setPage(p)">{{ p }}</button>
            </li>
            <li>
              <button
                type="button"
                aria-label="Next page"
                :disabled="store.page === store.maxPage"
                @click="store.setPage(store.page + 1)"
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <polyline points="9 18 15 12 9 6" />
                </svg>
              </button>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useDriversStore } from '@/stores/drivers'
import { categories, sorts } from '@/data/drivers'
import type { DriverCategory } from '@/data/drivers'

const store = useDriversStore()
const popupVisible = ref(false)
const listRef = ref<HTMLElement | null>(null)

const currentCategory = computed(
  () => categories.find(cat => cat.id === store.category)?.name ?? 'All',
)

const handleCategory = (id: DriverCategory | 'all') => {
  store.setCategory(id)
  popupVisible.value = false
}

const refresh = () => {
  if (listRef.value) listRef.value.scrollTop = 0
  store.setSort('recommended')
  store.setQuery('')
  store.setCategory('all')
}

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (popupVisible.value && !target.closest('.select')) popupVisible.value = false
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.container {
  height: 100%;
  display: flex;
  flex-flow: column nowrap;
  position: relative;
  overflow: hidden;
}

.header {
  flex: none;
  width: 100%;
  display: flex;
  flex-flow: row nowrap;
  align-items: center;
  padding: 10px 15px 5px;
}

.left {
  flex: auto;
  display: flex;
  flex-flow: row nowrap;
  align-items: center;
  gap: 10px;
}

.select {
  position: relative;
  font-size: 12px;
  color: var(--color-font);
}

.select-label {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 12px;
  border-radius: 3px;
  cursor: pointer;
  transition: color 0.3s ease, background-color 0.3s ease;
}

.select-label:hover {
  color: var(--color-primary-font-hover);
  background-color: var(--color-button-background-hover);
}

.select-label svg {
  width: 0.9em;
  transition: transform 0.2s ease;
}

.select.active .select-label svg {
  transform: rotate(180deg);
}

.popup {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 8px;
  min-width: 140px;
  border-radius: 4px;
  background-color: var(--color-content-background);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
  opacity: 0;
  transform: scale(0.95, 0.8);
  transform-origin: 0 0;
  pointer-events: none;
  transition: 0.25s ease;
  transition-property: transform, opacity;
  z-index: 10;
}

.select.active .popup {
  opacity: 1;
  transform: scale(1);
  pointer-events: initial;
}

.popup ul {
  max-height: 250px;
  padding: 6px;
}

.popup li {
  padding: 8px 12px;
  border-radius: 3px;
  font-size: 12px;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.popup li:hover {
  background-color: var(--color-button-background-hover);
}

.popup li.active {
  color: var(--color-primary);
  background-color: var(--color-button-background);
}

.tabs {
  display: flex;
  flex-flow: row nowrap;
  gap: 25px;
  padding: 0 15px;
  font-size: 12px;
}

.tab {
  position: relative;
  display: block;
  padding: 8px 0;
  cursor: pointer;
  transition: color 0.4s ease;
}

.tab:hover {
  color: var(--color-primary);
}

.tab.active {
  color: var(--color-primary);
  cursor: default;
}

.tab.active::after {
  content: '';
  position: absolute;
  left: 0;
  bottom: 0;
  width: 100%;
  height: 2px;
  border-radius: 20px;
  background-color: var(--color-primary-alpha-300);
}

.refresh {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 30px;
  border: none;
  border-radius: 3px;
  background: none;
  color: var(--color-button-font);
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.refresh:hover {
  background-color: var(--color-button-background-hover);
}

.refresh svg {
  width: 15px;
}

.list {
  flex: auto;
  min-height: 0;
  position: relative;
}

.list-content {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  padding: 10px 15px 15px;
  font-size: 14px;
}

.list-content ul {
  display: flex;
  flex-flow: row wrap;
  justify-content: space-between;
}

.item {
  max-width: 360px;
  width: 32%;
  box-sizing: border-box;
  display: flex;
  margin-bottom: 20px;
  cursor: pointer;
  transition: opacity 0.4s ease;
}

.item:hover {
  opacity: 0.7;
}

.pad {
  width: 32%;
  max-width: 360px;
  margin-bottom: 0;
  height: 0;
}

.thumb {
  flex: none;
  width: 40%;
  aspect-ratio: 1 / 1;
  display: flex;
  flex-flow: column nowrap;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: 4px;
  overflow: hidden;
  box-shadow: 0 0 2px rgba(0, 0, 0, 0.2);
}

.initials {
  color: rgba(255, 255, 255, 0.92);
  font-size: 24px;
  font-weight: bold;
}

.source {
  color: rgba(255, 255, 255, 0.75);
  font-size: 11px;
}

.desc {
  flex: auto;
  padding: 2px 15px 2px 7px;
  overflow: hidden;
}

.desc h4 {
  font-size: 14px;
  line-height: 1.3;
  display: -webkit-box;
  overflow: hidden;
  text-overflow: ellipsis;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.description {
  margin-top: 6px;
  font-size: 12px;
  line-height: 1.3;
  color: var(--color-font-label);
  display: -webkit-box;
  overflow: hidden;
  text-overflow: ellipsis;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.meta {
  display: flex;
  flex-flow: row wrap;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  font-size: 12px;
  line-height: 1.2;
  color: var(--color-font-label);
}

.badge {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 11px;
  color: var(--color-primary);
  background-color: var(--color-button-background);
}

.footer {
  margin-top: 10px;
  overflow: hidden;
}

.stats {
  float: left;
  line-height: 24px;
  font-size: 11px;
  color: var(--color-font-label);
}

.download {
  float: right;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 3px;
  font-size: 12px;
  color: var(--color-primary);
  background-color: var(--color-button-background);
  text-decoration: none;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
  transition: background-color 0.3s ease;
}

.download:hover {
  color: var(--color-primary);
  background-color: var(--color-button-background-hover);
}

.download svg {
  width: 13px;
}

.pagination {
  text-align: center;
  padding: 15px 0;
}

.pagination ul {
  display: inline-flex;
  flex-flow: row nowrap;
  border-radius: 4px;
  background-color: var(--color-button-background);
}

.pagination li {
  display: flex;
}

.pagination button {
  display: block;
  padding: 7px 12px;
  border: none;
  background: transparent;
  font-size: 13px;
  line-height: 1.2;
  color: var(--color-button-font);
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.pagination button:disabled {
  opacity: 0.3;
  cursor: default;
}

.pagination button:hover:not(:disabled) {
  background-color: var(--color-button-background-hover);
}

.pagination li.active button {
  background-color: var(--color-button-background-selected);
}

.pagination svg {
  width: 13px;
}

.noitem {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: 100%;
  display: flex;
  flex-flow: column nowrap;
  justify-content: center;
  align-items: center;
}

.noitem p {
  font-size: 24px;
  color: var(--color-font-label);
}
</style>