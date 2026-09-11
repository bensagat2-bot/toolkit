<template>
  <div id="container">
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
              <svg viewBox="0 0 24 24" fill="currentColor" v-html="item.icon"></svg>
            </router-link>
          </li>
        </ul>
      </nav>
    </aside>
    <div id="main">
      <div class="main-content">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup>
const menus = [
  { to: '/unisoc', name: 'Unisoc', icon: '<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>' },
  { to: '/mediatek', name: 'MediaTek', icon: '<path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>' },
  { to: '/utilities', name: 'Utilities', icon: '<path d="M22.7 19l-9.1-9.1c.9-2.3.4-5-1.5-6.9-2-2-5-2.4-7.4-1.3L9 6 6 9 1.6 4.7C.4 7.1.9 10.1 2.9 12.1c1.9 1.9 4.6 2.4 6.9 1.5l9.1 9.1c.4.4 1 .4 1.4 0l2.3-2.3c.5-.4.5-1.1.1-1.4z"/>' },
  { to: '/drivers', name: 'Drivers', icon: '<path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>' },
  { to: '/settings', name: 'Settings', icon: '<path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>' },
]
</script>

<style>
:root {
  --color-font: #333;
  --color-nav-font: #888;
  --color-content-background: #f5f5f5;
  --color-app-background: #e8e8e8;
  --color-main-background: #fff;
  --color-primary: #4caf50;
  --color-primary-light-300: #81c784;
  --color-primary-light-400: #66bb6a;
  --color-primary-light-500: #4caf50;
  --color-primary-dark-200: #388e3c;
  --color-primary-alpha-700: rgba(76, 175, 80, 0.7);
  --color-primary-alpha-900: rgba(76, 175, 80, 0.9);
}

* { margin: 0; padding: 0; box-sizing: border-box; }
html, body, #root { height: 100%; }

body {
  user-select: none;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Microsoft YaHei', sans-serif;
  color: var(--color-font);
  background-color: var(--color-content-background);
  transition: background-color 0.4s ease;
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
  background-color: var(--color-app-background);
  transition: background-color 0.4s ease;
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
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  position: relative;
  width: 100%;
}

.nav-item::before {
  content: '';
  display: block;
  width: 100%;
  padding-bottom: 84%;
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
  transition: background-color 0.3s ease, opacity 0.3s ease;
}

.link svg {
  width: 32%;
  height: 32%;
  flex: none;
}

.link::before {
  content: '';
  display: block;
  position: absolute;
  left: 0;
  top: 0;
  width: 3px;
  height: 100%;
  background-color: var(--color-primary-dark-200);
  border-radius: 4px;
  transform: translateX(-100%);
  transition: transform 0.3s ease;
}

.link:hover:not(.active) {
  background-color: var(--color-primary-light-400);
  opacity: 0.8;
}

.link:active:not(.active) {
  opacity: 0.6;
  background-color: var(--color-primary-light-300);
}

.link.active {
  background-color: var(--color-primary-light-300);
  color: var(--color-font);
}

.link.active::before {
  transform: translateX(0);
}

.link.active:hover {
  background-color: var(--color-primary-light-400);
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
}

.main-content {
  position: relative;
  flex: auto;
  min-height: 0;
  overflow: hidden;
}
</style>
