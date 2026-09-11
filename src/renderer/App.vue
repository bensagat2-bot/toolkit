<template>
  <div id="container" class="view-container">
    <aside id="sidebar">
      <div class="logo">
        <img src="./assets/images/vxper.png" alt="V1Per" class="logo-img" />
      </div>
      <nav ref="dom_menu" class="menu">
        <ul class="list" role="toolbar">
          <li v-for="item in menus" :key="item.to" class="nav-item" role="presentation">
            <router-link
              :to="item.to"
              class="link"
              exact-active-class="active"
              :aria-label="item.name"
              :title="item.name"
            >
              <svg viewBox="0 0 24 24" fill="currentColor" :height="item.size" :width="item.size">
                <component :is="item.icon" />
              </svg>
            </router-link>
          </li>
        </ul>
      </nav>
    </aside>
    <div id="main">
      <Toolbar />
      <div class="main-content">
        <router-view v-slot="{ Component }">
          <transition name="page-fade" mode="out-in">
            <component :is="Component" class="view-container" />
          </transition>
        </router-view>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, h } from 'vue'
import Toolbar from './components/layout/Toolbar.vue'
import { useIconSize } from './utils/useIconSize'

const dom_menu = ref<HTMLElement>()
const iconSize = useIconSize(dom_menu, 0.32)

const menus = computed(() => {
  const size = iconSize.value
  return [
    {
      to: '/mediatek',
      name: 'Mediatek Tools',
      icon: () => h('path', { d: 'M15 9H9v6h6V9zm-2 4h-2v-2h2v2zm8-2V9h-2V7c0-1.1-.9-2-2-2h-2V3h-2v2h-2V3H9v2H7c-1.1 0-2 .9-2 2v2H3v2h2v2H3v2h2v2c0 1.1.9 2 2 2h2v2h2v-2h2v2h2v-2h2c1.1 0 2-.9 2-2v-2h2v-2h-2v-2h2zm-4 6H7V7h10v10z' }),
      size,
    },
    {
      to: '/unisoc',
      name: 'Unisoc Tools',
      icon: () => h('path', { d: 'M17 16l-4-4V8.82C14.16 8.4 15 7.3 15 6c0-1.66-1.34-3-3-3S9 4.34 9 6c0 1.3.84 2.4 2 2.82V12l-4 4H3v5h5v-3.05l4-4.2 4 4.2V21h5v-5h-4z' }),
      size,
    },
    {
      to: '/utilities',
      name: 'Utilities Tools',
      icon: () => h('path', { d: 'M22.7 19l-9.1-9.1c.9-2.3.4-5-1.5-6.9-2-2-5-2.4-7.4-1.3L9 6 6 9 1.6 4.7C.4 7.1.9 10.1 2.9 12.1c1.9 1.9 4.6 2.4 6.9 1.5l9.1 9.1c.4.4 1 .4 1.4 0l2.3-2.3c.5-.4.5-1.1.1-1.4z' }),
      size,
    },
    {
      to: '/drivers',
      name: 'Drivers',
      icon: () => h('path', { d: 'M15 7v4h1v2h-3V5h2l-3-4-3 4h2v8H8v-2.07c.7-.37 1.2-1.08 1.2-1.93 0-1.21-.99-2.2-2.2-2.2-1.21 0-2.2.99-2.2 2.2 0 .85.5 1.56 1.2 1.93V13c0 1.11.89 2 2 2h3v3h-1c-.26 0-.5.21-.5.5s.24.5.5.5h2c.26 0 .5-.21.5-.5s-.24-.5-.5-.5h-1v-3h3c1.11 0 2-.89 2-2V9h1V7h-4z' }),
      size,
    },
    {
      to: '/setting',
      name: 'Settings',
      icon: () => h('path', { d: 'M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z' }),
      size,
    },
  ]
})

onMounted(() => {
  document.getElementById('root').style.display = 'block'
})
</script>

<style lang="less">
@import './assets/styles/index.less';
@import './assets/styles/layout.less';

html, body {
  height: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
  box-sizing: border-box;
}

#root {
  height: 100%;
  color: var(--color-font);
  background-color: var(--color-content-background);
  position: relative;
  overflow: hidden;
  box-sizing: border-box;
  box-shadow: 0 0 8px rgba(0, 0, 0, 0.5);
  border-radius: 4px;
}

#container {
  position: relative;
  display: flex;
  height: 100%;
  background-color: var(--color-app-background);
}

#sidebar {
  flex: none;
  width: 6.6%;
  height: 100%;
  display: flex;
  flex-direction: column;
  transition: background-color .4s ease;
  -webkit-app-region: drag;
  -webkit-user-select: none;
}

.logo {
  box-sizing: border-box;
  padding: 8px 13%;
  height: 50px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
}

.logo-img {
  width: 100%;
  height: auto;
  max-height: 34px;
  object-fit: contain;
}

.menu {
  flex: auto;
}

.list {
  width: 100%;
  -webkit-user-select: none;
  -webkit-app-region: no-drag;
}

.nav-item {
  position: relative;
  width: 100%;

  &::before {
    content: '';
    display: block;
    width: 100%;
    padding-bottom: 84%;
  }
}

.link {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-nav-font);
  text-decoration: none;
  text-align: center;
  outline: none;
  cursor: pointer;
  transition: background-color .3s ease, opacity .3s ease;

  svg {
    flex: none;
  }

  &::before {
    content: '';
    display: block;
    position: absolute;
    left: 0;
    top: 0;
    width: 3px;
    height: 100%;
    background-color: var(--color-primary-dark-200-alpha-700);
    border-radius: 4px;
    transform: translateX(-100%);
    transition: transform .3s ease;
  }

  &:hover:not(.active) {
    background-color: var(--color-primary-light-400-alpha-700);
    opacity: .8;
  }

  &:active:not(.active) {
    opacity: .6;
    background-color: var(--color-primary-light-300-alpha-600);
  }

  &.active {
    background-color: var(--color-primary-light-300-alpha-700);

    &::before {
      transform: translateX(0);
    }

    &:hover {
      background-color: var(--color-primary-light-300-alpha-800);
    }
  }
}

#main {
  flex: auto;
  position: relative;
  min-width: 0;
  overflow: hidden;
  background-color: var(--color-main-background);
  border-top-left-radius: 4px;
  border-bottom-left-radius: 4px;
  box-shadow: 0 0 4px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-flow: column nowrap;
}

.main-content {
  position: relative;
  flex: auto;
  min-height: 0;
  overflow: hidden;
}

.view-container {
  position: absolute !important;
  left: 0;
  top: 0;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity .2s ease;
}

.page-fade-enter-from,
.page-fade-leave-to {
  opacity: 0;
}
</style>
