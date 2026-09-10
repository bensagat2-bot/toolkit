<template>
  <div :class="['toolbar', { fullscreen: isFullscreen }]">
    <div :class="pageTitleClass">{{ pageTitle }}</div>
    <div :class="'control'">
      <button type="button" :class="['btn', 'min']" title="Minimize" @click="setWindowMinimize">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M5 12h14" />
        </svg>
      </button>
      <button type="button" :class="['btn', 'close']" title="Close" @click="setWindowClose">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { setWindowMinimize, setWindowClose } from '@renderer/utils/ipc'
import { isFullscreen } from '@renderer/store'

const route = useRoute()

const pageTitle = computed(() => route.meta.name ?? 'V1Per')
const pageTitleClass = 'page-title'
</script>

<style lang="less" scoped>
@height-toolbar: 54px;

.toolbar {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: @height-toolbar;
  padding-left: 15px;
  -webkit-app-region: drag;
  z-index: 2;
  user-select: none;
}

.page-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--color-font);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.control {
  display: flex;
  align-self: stretch;
  -webkit-app-region: no-drag;

  .btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    background: none;
    border: none;
    outline: none;
    padding: 1px;
    cursor: pointer;
    color: var(--color-font-label);
    transition: background-color 0.2s ease-in-out;

    svg {
      width: 14px;
      height: 14px;
    }

    &:hover {
      &.min {
        background-color: var(--color-button-background-hover);
        color: var(--color-font);
      }

      &.close {
        background-color: var(--color-btn-close);
        color: #fff;
      }
    }
  }
}

.fullscreen {
  -webkit-app-region: no-drag;
}
</style>