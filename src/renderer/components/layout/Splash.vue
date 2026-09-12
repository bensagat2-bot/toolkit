<template>
  <div class="splash">
    <div class="splash-logo">
      <img src="@/assets/images/vxper.png" alt="V1Per" />
    </div>
    <div class="splash-spinner"></div>
    <p>Initializing V1Per Toolkit</p>
  </div>
</template>

<style scoped>
.splash { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; background: var(--color-main-background); }
.splash-logo { width: 80px; }
.splash-logo img { width: 100%; height: auto; opacity: 0.9; }
.splash-spinner { width: 28px; height: 28px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
.splash p { font-size: 13px; color: var(--text-secondary); margin: 0; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>