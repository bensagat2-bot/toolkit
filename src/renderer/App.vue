<template>
  <div id="container">
    <Aside id="left" />
    <div id="right">
      <Toolbar id="toolbar" />
      <main id="view">
        <router-view v-slot="{ Component }">
          <component :is="Component" class="view-container" />
        </router-view>
      </main>
      <PlayBar id="player" />
    </div>
  </div>
</template>

<script setup lang="ts">
import Aside from '@/components/layout/Aside.vue'
import Toolbar from '@/components/layout/Toolbar.vue'
import PlayBar from '@/components/layout/PlayBar.vue'
</script>

<style scoped>
#container {
  position: relative;
  display: flex;
  height: 100%;
  background-color: var(--color-app-background);
}

#left {
  flex: none;
  width: 6.6%;
  min-width: 44px;
  display: flex;
  flex-flow: column nowrap;
  -webkit-app-region: drag;
}

#right {
  flex: auto;
  display: flex;
  flex-flow: column nowrap;
  background-color: var(--color-main-background);
  border-top-left-radius: 4px;
  border-bottom-left-radius: 4px;
  overflow: hidden;
  box-shadow: 0 0 4px rgba(0, 0, 0, 0.1);
}

#toolbar,
#player {
  flex: none;
}

#view {
  position: relative;
  flex: auto;
  min-height: 0;
}

.view-container {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
}
</style>