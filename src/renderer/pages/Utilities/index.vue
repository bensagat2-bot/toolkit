<template>
  <div class="utils-page">
    <div class="utils-header">
      <h2>Utilities Tool</h2>
      <span class="tagline">Utilities Tool is now open!!!</span>
    </div>

    <div class="utils-grid">
      <div class="op-card">
        <div class="op-icon">R</div>
        <div class="op-body">
          <h3>Root</h3>
          <p>Pick a root manager, patch the boot image, unlock bootloader and flash.</p>
          <label class="row select-row">
            <span class="select-label">Root Manager</span>
            <select v-model="rootManager" class="text-input select-input">
              <option value="ksu-next">KernelSU-Next</option>
              <option value="ksu">KernelSU</option>
              <option value="sukisu">SukiSU-Ultra</option>
              <option value="folkpatch">FolkPatch</option>
            </select>
          </label>
          <div class="row">
            <input :value="rootBootImg" class="text-input" placeholder="boot.img / init_boot.img" readonly />
            <button class="btn btn-sm" @click="pickRootImage">Browse</button>
          </div>
          <button class="btn btn-primary op-run" @click="runRoot" :disabled="!rootBootImg">
            Run Root
          </button>
        </div>
      </div>

      <div class="op-card">
        <div class="op-icon">F</div>
        <div class="op-body">
          <h3>Force Fastboot</h3>
          <p>Flood the MTK preloader serial port with FASTBOOT to drop the device into fastboot.</p>
          <button class="btn btn-primary op-run" @click="runForceFastboot">
            Force Fastboot
          </button>
        </div>
      </div>

      <div class="op-card">
        <div class="op-icon">S</div>
        <div class="op-body">
          <h3>Scrcpy</h3>
          <p>Mirror and control the connected Android device over USB or Wi-Fi.</p>
          <button class="btn btn-primary op-run" @click="runScrcpy">
            Launch Scrcpy
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { showSelectDialog } from '@renderer/utils/ipc'
import { setPending } from './runStore'

const router = useRouter()
const rootBootImg = ref('')
const rootManager = ref('ksu-next')

const pickRootImage = async () => {
  const res = await showSelectDialog({ title: 'Select boot.img', filters: [{ name: 'Images', extensions: ['img', 'bin'] }] })
  if (res.filePaths && res.filePaths.length) rootBootImg.value = res.filePaths[0]
}

function runRoot() {
  setPending({ type: 'root', boot_img: rootBootImg.value, manager: rootManager.value })
  router.push({ path: '/utilities/run' })
}

function runForceFastboot() {
  setPending({ type: 'force_fastboot' })
  router.push({ path: '/utilities/run' })
}

function runScrcpy() {
  setPending({ type: 'scrcpy' })
  router.push({ path: '/utilities/run' })
}
</script>

<style scoped>
.utils-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.utils-header { display: flex; align-items: center; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } .tagline { font-size: 12px; color: var(--text-secondary); } }
.utils-grid { flex: 1; display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; align-content: start; overflow-y: auto; }
.op-card { display: flex; gap: 12px; padding: 14px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-tertiary); }
.op-icon { flex-shrink: 0; width: 44px; height: 44px; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; border-radius: 6px; background: var(--accent-primary); color: #fff; }
.op-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 8px; h3 { margin: 0; font-size: 14px; } p { margin: 0; font-size: 12px; color: var(--text-secondary); } }
.row { display: flex; gap: 6px; }
.select-row { align-items: center; }
.select-label { font-size: 12px; color: var(--text-secondary); flex-shrink: 0; }
.select-input { flex: 1; min-width: 0; cursor: pointer; appearance: auto; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }
.op-run { margin-top: auto; width: 100%; }
</style>