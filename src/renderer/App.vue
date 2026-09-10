<template>
  <div id="container" class="view-container">
    <div id="sidebar">
      <div class="logo">
        <svg viewBox="0 0 24 24" fill="currentColor" class="logo-icon">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </svg>
        <span class="logo-text">V1Per</span>
      </div>
      <nav class="nav-list">
        <router-link to="/mediatek" class="nav-item" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="4" y="4" width="16" height="16" rx="2" />
            <path d="M9 9h6M9 12h6M9 15h4" />
          </svg>
          <span>Mediatek</span>
        </router-link>
        <router-link to="/unisoc" class="nav-item" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z" />
          </svg>
          <span>Unisoc</span>
        </router-link>
        <router-link to="/utilities" class="nav-item" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
          </svg>
          <span>Utilities</span>
        </router-link>
        <router-link to="/drivers" class="nav-item" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2" />
            <path d="M3 9h18M9 3v18" />
          </svg>
          <span>Drivers</span>
        </router-link>
      </nav>
      <div class="nav-bottom">
        <router-link to="/setting" class="nav-item" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4" />
          </svg>
          <span>Settings</span>
        </router-link>
      </div>
    </div>
    <div id="main">
      <router-view />
    </div>
  </div>
</template>

<script setup>
</script>

<style lang="less">
@import './assets/styles/index.less';

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
  background-color: var(--color-app-background);
}

#container {
  display: flex;
  height: 100%;
  background-color: var(--color-app-background);
}

#sidebar {
  width: 180px;
  height: 100%;
  background-color: var(--color-sidebar-background);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.logo {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.logo-icon {
  width: 28px;
  height: 28px;
  color: var(--color-primary);
}

.logo-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-font);
}

.nav-list {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-bottom {
  padding: 8px;
  border-top: 1px solid var(--color-border);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  color: var(--color-font-secondary);
  text-decoration: none;
  font-size: 13px;
  transition: all 0.2s ease;

  svg {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  &:hover {
    background-color: var(--color-hover);
    color: var(--color-font);
  }

  &.active {
    background-color: var(--color-primary);
    color: #fff;
  }
}

#main {
  flex: 1;
  overflow: hidden;
  background-color: var(--color-main-background);
}
</style>
