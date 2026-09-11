<template>
  <div class="aside">
    <div class="logo">
      <img src="@/assets/images/vxper.png" alt="V1Per" class="logo-img" />
    </div>
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
  padding: 8px 13%;
  height: 50px;
  color: var(--color-nav-font);
  opacity: 0.8;
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
</style>