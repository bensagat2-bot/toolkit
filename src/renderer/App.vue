<template>
  <div id="container" class="view-container">
    <aside id="sidebar">
      <div class="logo">V1Per</div>
      <nav class="menu">
        <ul class="list" role="toolbar">
          <li v-for="item in menus" :key="item.to" class="nav-item" role="presentation">
            <router-link
              :to="item.to"
              class="link"
              exact-active-class="active"
              :aria-label="item.name"
              :title="item.name"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <component :is="item.icon" />
              </svg>
            </router-link>
          </li>
        </ul>
      </nav>
    </aside>
    <div id="main">
      <router-view v-slot="{ Component }">
        <component :is="Component" class="view-container" />
      </router-view>
    </div>
  </div>
</template>

<script setup>
import { onMounted, h } from 'vue'

const menus = [
  {
    to: '/mediatek',
    name: 'Mediatek Tools',
    icon: () => h('g', null, [
      h('rect', { x: '4', y: '4', width: '16', height: '16', rx: '2' }),
      h('path', { d: 'M9 9h6M9 12h6M9 15h4' }),
    ]),
  },
  {
    to: '/unisoc',
    name: 'Unisoc Tools',
    icon: () => h('path', { d: 'M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z' }),
  },
  {
    to: '/utilities',
    name: 'Utilities Tools',
    icon: () => h('g', null, [
      h('circle', { cx: '12', cy: '12', r: '3' }),
      h('path', { d: 'M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z' }),
    ]),
  },
  {
    to: '/drivers',
    name: 'Drivers',
    icon: () => h('g', null, [
      h('rect', { x: '3', y: '3', width: '18', height: '18', rx: '2' }),
      h('path', { d: 'M3 9h18M9 3v18' }),
    ]),
  },
  {
    to: '/setting',
    name: 'Settings',
    icon: () => h('g', null, [
      h('circle', { cx: '12', cy: '12', r: '3' }),
      h('path', { d: 'M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z' }),
    ]),
  },
]

onMounted(() => {
  document.getElementById('root').style.display = 'block'
})
</script>

<style lang="less">
@import './assets/styles/index.less';

html, body {
  height: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
  box-sizing: border-box;
  font-size: 16px;
}

#root {
  height: 100%;
  color: var(--color-font);
  background-color: var(--color-app-background);
  position: relative;
  overflow: hidden;
  box-sizing: border-box;
}

#container {
  display: flex;
  height: 100%;
  background-color: var(--color-app-background);
}

#sidebar {
  flex: none;
  width: 6.6%;
  min-width: 56px;
  max-width: 84px;
  height: 100%;
  background-color: var(--color-sidebar-background);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  transition: background-color .4s ease;
}

.logo {
  box-sizing: border-box;
  padding: 0 13%;
  height: 50px;
  color: var(--color-nav-font);
  opacity: .8;
  flex: none;
  text-align: center;
  line-height: 50px;
  font-weight: bold;
  white-space: nowrap;
  overflow: hidden;
}

.menu {
  flex: auto;
}

.list {
  width: 100%;
  -webkit-user-select: none;
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
    width: 32%;
    height: 32%;
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
}

.view-container {
  position: absolute !important;
  left: 0;
  top: 0;
  height: 100%;
  width: 100%;
  overflow: hidden;
}
</style>
