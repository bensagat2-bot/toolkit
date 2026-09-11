<template>
  <div id="app">
    <aside id="sidebar">
      <div class="logo">
        <img src="./assets/images/vxper.png" alt="V1Per" class="logo-img" />
      </div>
      <nav class="menu">
        <router-link v-for="item in menus" :key="item.to" :to="item.to" class="nav-item" active-class="active">
          <svg viewBox="0 0 24 24" fill="currentColor" v-html="item.icon"></svg>
          <span class="nav-label">{{ item.name }}</span>
        </router-link>
      </nav>
    </aside>
    <main id="main">
      <router-view />
    </main>
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
* { margin: 0; padding: 0; box-sizing: border-box; }
html, body, #root { height: 100%; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }

#app {
  display: flex;
  height: 100%;
  background: #1a1a2e;
  color: #e0e0e0;
}

#sidebar {
  width: 64px;
  background: #16213e;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  gap: 4px;
  border-right: 1px solid #0f3460;
}

.logo {
  width: 40px;
  height: 40px;
  margin-bottom: 12px;
}

.logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 8px;
}

.menu {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-item {
  width: 48px;
  height: 48px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  text-decoration: none;
  color: #8892b0;
  transition: all 0.2s;
  font-size: 9px;
  gap: 2px;
}

.nav-item:hover {
  background: #1a1a2e;
  color: #ccd6f6;
}

.nav-item.active {
  background: #e94560;
  color: white;
}

.nav-item svg {
  width: 20px;
  height: 20px;
}

.nav-label {
  line-height: 1;
}

#main {
  flex: 1;
  overflow: auto;
  background: #1a1a2e;
}
</style>
