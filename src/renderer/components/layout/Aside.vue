<template>
  <div class="aside">
    <div class="logo">V1</div>
    <NavBar />
  </div>
</template>

<script setup lang="ts">
import NavBar from './NavBar.vue'
</script>

<style scoped>
.aside {
  display: flex;
  flex-flow: column nowrap;
  height: 100%;
  -webkit-user-select: none;
}

.logo {
  box-sizing: border-box;
  padding: 0 13%;
  height: 50px;
  color: var(--color-nav-font);
  opacity: 0.8;
  flex: none;
  text-align: center;
  line-height: 50px;
  font-weight: bold;
}
</style>