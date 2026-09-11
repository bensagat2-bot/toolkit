<template>
  <div id="container">
    <Aside id="left" />
    <div id="right">
      <Toolbar id="toolbar" />
      <main id="view">
        <router-view v-slot="{ Component }">
          <transition name="page" mode="out-in">
            <component :is="Component" class="view-container" />
          </transition>
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
  box-shadow: -1px 0 0 rgba(255, 255, 255, 0.07);
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

.page-enter-active,
.page-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>