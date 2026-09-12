<template>
  <Splash v-if="!store.initialized" />
  <AuthPage v-else-if="!store.isAuthed" />
  <div v-else id="container">
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
import { onMounted } from 'vue'
import Splash from '@/components/layout/Splash.vue'
import Aside from '@/components/layout/Aside.vue'
import Toolbar from '@/components/layout/Toolbar.vue'
import PlayBar from '@/components/layout/PlayBar.vue'
import AuthPage from '@/pages/Auth/index.vue'
import { useAccountStore } from '@/store/accountStore'

const store = useAccountStore()

onMounted(() => {
  store.init()
})
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