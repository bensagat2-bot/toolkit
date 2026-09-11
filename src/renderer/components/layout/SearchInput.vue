<template>
  <div class="container">
    <div :class="['search', { active: focus }]">
      <div class="form">
        <input
          ref="inputRef"
          v-model="text"
          type="text"
          :placeholder="placeholder"
          aria-label="Search"
          @focus="focus = true"
          @blur="handleBlur"
          @input="handleInput"
          @keyup.enter="handleSubmit"
        >
        <button v-if="text" type="button" aria-label="Clear" @click="handleClear">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="6" y1="6" x2="18" y2="18" />
            <line x1="18" y1="6" x2="6" y2="18" />
          </svg>
        </button>
        <button type="button" aria-label="Search" @click="handleSubmit">
          <svg
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
        </button>
      </div>
      <div v-if="focus && suggestions.length" class="list">
        <ul>
          <li v-for="item in suggestions" :key="item" @mousedown.prevent="handlePick(item)">
            <span>{{ item }}</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDriversStore } from '@/stores/drivers'

const store = useDriversStore()
const focus = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const text = ref(store.query)

const placeholder = 'Search for drivers...'

const suggestions = computed(() => {
  const q = text.value.trim().toLowerCase()
  if (!q) return []
  return store.allNames.filter(name => name.toLowerCase().includes(q)).slice(0, 8)
})

const handleInput = () => {
  store.setQuery(text.value)
}

const handleClear = () => {
  text.value = ''
  store.setQuery('')
  inputRef.value?.focus()
}

const handleSubmit = () => {
  store.setQuery(text.value)
  focus.value = false
  inputRef.value?.blur()
}

const handlePick = (item: string) => {
  text.value = item
  store.setQuery(item)
  focus.value = false
  inputRef.value?.blur()
}

const handleBlur = () => {
  setTimeout(() => {
    focus.value = false
  }, 80)
}
</script>

<style scoped>
.container {
  position: relative;
  width: 35%;
  height: 28px;
  -webkit-app-region: no-drag;
}

.search {
  position: absolute;
  width: 100%;
  border-radius: 3px;
  display: flex;
  flex-flow: column nowrap;
  background-color: var(--color-primary-light-300-alpha-700);
  transition: box-shadow 0.4s ease, background-color 0.4s ease;
}

.search.active {
  background-color: var(--color-primary-light-600-alpha-100);
  box-shadow: 0 1px 5px rgba(0, 0, 0, 0.2);
}

.form {
  display: flex;
  height: 28px;
  position: relative;
}

.form input {
  flex: auto;
  min-width: 0;
  border: none;
  outline: none;
  background-color: transparent;
  padding: 0 5px;
  font-size: 13.5px;
  color: var(--color-font);
}

.form input::placeholder {
  color: var(--color-button-font);
  font-size: 0.98em;
}

.form button {
  flex: none;
  border: none;
  background-color: transparent;
  outline: none;
  cursor: pointer;
  height: 100%;
  padding: 6px 7px;
  color: var(--color-button-font);
  transition: background-color 0.2s ease;
}

.form button:hover {
  background-color: var(--color-button-background-hover);
}

.form button:active {
  background-color: var(--color-button-background-active);
}

.form button:last-child {
  border-top-right-radius: 3px;
  border-bottom-right-radius: 3px;
}

.form svg {
  width: 13px;
  height: 13px;
}

.list {
  font-size: 13px;
  overflow: hidden;
}

.list li {
  cursor: pointer;
  padding: 8px 5px;
  line-height: 1.3;
  transition: background-color 0.2s ease;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.list li:hover {
  background-color: var(--color-primary-dark-100-alpha-200);
}

.list li:last-child {
  border-bottom-left-radius: 3px;
  border-bottom-right-radius: 3px;
}
</style>