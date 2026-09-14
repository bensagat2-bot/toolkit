<template>
  <div class="mtk-page">
    <div class="mtk-header">
      <h2>Xiaomi Tools</h2>
    </div>

    <div class="mtk-content">
      <div class="flash-panel">
        <div class="action-group">
          <h3>Device Status</h3>
          <div class="device-status" :class="'status-' + deviceMode">
            <span class="status-dot"></span>
            <span>{{ deviceStatusText }}</span>
          </div>
          <button class="btn btn-sm" @click="detectDevice" :disabled="busy">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
            Refresh
          </button>
          <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>
        </div>

        <div class="action-group" v-if="deviceInfo">
          <h3>Device Info</h3>
          <div class="info-row"><span>Model</span><span>{{ deviceInfo.model }}</span></div>
          <div class="info-row"><span>Brand</span><span>{{ deviceInfo.brand }}</span></div>
          <div class="info-row"><span>Android</span><span>{{ deviceInfo.android }}</span></div>
          <div class="info-row"><span>Bootloader</span><span :class="bootloaderClass">{{ deviceInfo.bootloader }}</span></div>
          <div class="info-row"><span>Root</span><span>{{ deviceInfo.root_status }}</span></div>
          <div class="info-row" v-if="deviceInfo.product"><span>Product</span><span>{{ deviceInfo.product }}</span></div>
        </div>

        <div class="action-group">
          <h3>Bootloader</h3>
          <button class="btn btn-primary" @click="showMiUnlock = true" :disabled="deviceMode !== 'fastboot'">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
            Mi Unlock Wizard
          </button>
          <button class="btn" @click="rebootBootloader" :disabled="deviceMode === 'none'">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
            Reboot to Bootloader
          </button>
        </div>

        <div class="action-group">
          <h3>ROM Flash</h3>
          <div class="row">
            <input :value="romFolder" class="text-input" placeholder="No ROM folder" readonly />
            <button class="btn btn-sm" @click="browseRom">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>ADB / Fastboot</h3>
          <div class="row">
            <input v-model="customArgs" class="text-input" placeholder="e.g. shell getprop ro.product.model" />
          </div>
          <div class="row">
            <button class="btn btn-sm" @click="runAdbCmd" :disabled="deviceMode === 'none' || !customArgs">ADB</button>
            <button class="btn btn-sm" @click="runFastbootCmd" :disabled="deviceMode !== 'fastboot' || !customArgs">Fastboot</button>
          </div>
        </div>
      </div>

      <div class="output-card">
        <div class="output-card-header" v-if="romFolder">
          <span>Firmware Partitions</span>
          <div class="header-actions">
            <span class="rom-type-badge">{{ romType.toUpperCase() }}</span>
          </div>
        </div>
        <div class="output-card-header" v-else>
          <span>Xiaomi Service Toolkit</span>
        </div>

        <div class="output-body" v-if="romFolder && romCommands.length">
          <table class="partition-table">
            <thead>
              <tr>
                <th>Type</th>
                <th>Partition</th>
                <th>Command</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(cmd, i) in romCommands" :key="i">
                <td class="col-type">{{ cmd.type }}</td>
                <td>{{ cmd.partition || '-' }}</td>
                <td class="col-cmd">{{ (cmd.args || []).join(' ') }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="output-body welcome-body" v-else>
          <div class="welcome-icon">X</div>
          <h3>Xiaomi Service Toolkit</h3>
          <p>Connect a device via ADB or Fastboot to get started.</p>
        </div>
      </div>
    </div>

    <MiUnlockWizard :visible="showMiUnlock" @close="showMiUnlock = false" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, showSelectFolder } from '@renderer/utils/ipc'
import { setPending } from './runStore'
import MiUnlockWizard from './MiUnlockWizard.vue'

const router = useRouter()
const deviceInfo = ref(null)
const deviceMode = ref('none')
const customArgs = ref('')
const romFolder = ref('')
const romType = ref('')
const romCommands = ref([])
const showMiUnlock = ref(false)
const errorMsg = ref('')
const busy = ref(false)
let pollTimer = null

const deviceStatusText = computed(() => {
  const map = { none: 'No Device', adb: 'Connected (ADB)', fastboot: 'Connected (Fastboot)', unauthorized: 'Unauthorized' }
  return map[deviceMode.value] || 'Unknown'
})

const bootloaderClass = computed(() => {
  if (!deviceInfo.value) return ''
  const b = deviceInfo.value.bootloader.toLowerCase()
  if (b.includes('unlock')) return 'text-ok'
  if (b.includes('lock')) return 'text-warn'
  return ''
})

async function detectDevice() {
  busy.value = true
  errorMsg.value = ''
  try {
    const info = await sendIpcToMain('xiaomi_detect_device')
    deviceInfo.value = info
    deviceMode.value = info.mode || 'none'
  } catch (e) {
    console.error('Device detection failed:', e)
    errorMsg.value = 'Device detection failed. Ensure USB debugging is enabled.'
    deviceInfo.value = null
    deviceMode.value = 'none'
  } finally {
    busy.value = false
  }
}

function rebootBootloader() {
  setPending({ type: 'reboot_bootloader' })
  router.push({ path: '/xiaomi/run' })
}

async function browseRom() {
  const p = await showSelectFolder('Select ROM Folder')
  if (!p) return
  romFolder.value = p
  try {
    const data = await sendIpcToMain('xiaomi_scan_rom', { folder: p })
    romType.value = data.rom_type || 'unknown'
    romCommands.value = data.commands || []
  } catch (e) {
    console.error('ROM scan failed:', e)
    errorMsg.value = 'Failed to scan ROM folder.'
    romType.value = 'unknown'
    romCommands.value = []
  }
}

function runAdbCmd() {
  if (!customArgs.value) return
  setPending({ type: 'adb_cmd', args: customArgs.value.split(/\s+/) })
  router.push({ path: '/xiaomi/run' })
}

function runFastbootCmd() {
  if (!customArgs.value) return
  setPending({ type: 'fastboot_cmd', args: customArgs.value.split(/\s+/) })
  router.push({ path: '/xiaomi/run' })
}

onMounted(() => { detectDevice(); pollTimer = setInterval(detectDevice, 5000) })
onBeforeUnmount(() => { if (pollTimer) clearInterval(pollTimer) })
</script>

<style scoped>
.mtk-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.mtk-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }

.mtk-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.flash-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; }
.text-input:focus { border-color: var(--accent-primary); }

.device-status { display: flex; align-items: center; gap: 6px; font-size: 11px; padding: 5px 8px; border-radius: 4px; background: var(--bg-tertiary); }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: #666; }
.status-adb .status-dot { background: var(--color-status-success); }
.status-fastboot .status-dot { background: var(--color-status-info); }
.status-unauthorized .status-dot { background: var(--color-status-warn); }

.info-row { display: flex; justify-content: space-between; font-size: 11px; padding: 2px 0; span:first-child { color: var(--text-secondary); } }
.text-ok { color: var(--color-status-success); }
.text-warn { color: var(--color-status-warn); }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }

.output-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.output-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } .header-actions { display: flex; align-items: center; gap: 10px; } .rom-type-badge { color: var(--accent-primary); font-weight: 600; font-size: 11px; } }
.output-body { flex: 1; overflow-y: auto; }
.welcome-body { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; text-align: center; padding: 40px; h3 { font-size: 16px; margin: 0; } p { font-size: 12px; color: var(--text-secondary); margin: 0; } }
.welcome-icon { width: 48px; height: 48px; border-radius: 12px; background: linear-gradient(135deg, #ff6b00, #ff9800); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; }

.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.partition-table th, .partition-table td { text-align: left; padding: 6px 10px; white-space: nowrap; }
.partition-table thead th { position: sticky; top: 0; background: var(--color-content-background); border-bottom: var(--color-list-header-border-bottom); color: var(--color-font); font-weight: 500; font-size: 12px; }
.partition-table tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); }
.col-type { font-weight: 500; text-transform: uppercase; font-size: 10px; color: var(--accent-primary); }
.col-cmd { color: var(--text-secondary); font-family: monospace; font-size: 11px; }
</style>
