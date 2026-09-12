<template>
  <div class="xiaomi-page">
    <aside class="sidebar">
      <div class="sidebar-brand">
        <div class="brand-icon">X</div>
        <span class="brand-text">Xiaomi Tools</span>
      </div>

      <div class="sidebar-section">
        <div class="section-title">Device Status</div>
        <div class="device-status" :class="'status-' + deviceMode">
          <span class="status-dot"></span>
          <span>{{ deviceStatusText }}</span>
        </div>
        <button class="op-btn" @click="detectDevice">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
          Refresh
        </button>
      </div>

      <div class="sidebar-section" v-if="deviceInfo">
        <div class="section-title">Info</div>
        <div class="info-row"><span class="info-label">Model</span><span>{{ deviceInfo.model }}</span></div>
        <div class="info-row"><span class="info-label">Brand</span><span>{{ deviceInfo.brand }}</span></div>
        <div class="info-row"><span class="info-label">Android</span><span>{{ deviceInfo.android }}</span></div>
        <div class="info-row"><span class="info-label">Bootloader</span><span :class="bootloaderClass">{{ deviceInfo.bootloader }}</span></div>
        <div class="info-row"><span class="info-label">Root</span><span>{{ deviceInfo.root_status }}</span></div>
        <div class="info-row" v-if="deviceInfo.product"><span class="info-label">Product</span><span>{{ deviceInfo.product }}</span></div>
      </div>

      <div class="sidebar-section">
        <div class="section-title">Bootloader</div>
        <button class="op-btn op-btn--primary" @click="showMiUnlock = true" :disabled="deviceMode !== 'fastboot'">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          Mi Unlock Wizard
        </button>
        <button class="op-btn" @click="rebootBootloader" :disabled="deviceMode === 'none'">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
          Reboot to Bootloader
        </button>
      </div>

      <div class="sidebar-section">
        <div class="section-title">ROM Flash</div>
        <button class="op-btn op-btn--accent" @click="browseRom">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
          Load ROM Folder
        </button>
        <button class="op-btn" @click="openFlashTool">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
          Flash Tool
        </button>
      </div>

      <div class="sidebar-section">
        <div class="section-title">ADB / Fastboot</div>
        <div class="cmd-row">
          <input v-model="customArgs" class="field-input" placeholder="e.g. shell getprop ro.product.model" />
        </div>
        <div class="cmd-btns">
          <button class="op-btn op-btn--sm" @click="runAdbCmd" :disabled="deviceMode === 'none' || !customArgs">ADB</button>
          <button class="op-btn op-btn--sm" @click="runFastbootCmd" :disabled="deviceMode !== 'fastboot' || !customArgs">Fastboot</button>
        </div>
      </div>
    </aside>

    <MiUnlockWizard :visible="showMiUnlock" @close="showMiUnlock = false" />

    <main class="main-content">
      <div class="main-header">
        <h2>Xiaomi Service Toolkit</h2>
      </div>

      <div class="content-area" v-if="!romFolder">
        <div class="welcome-card">
          <div class="welcome-icon">X</div>
          <h3>Xiaomi Service Toolkit</h3>
          <p>Connect a device via ADB or Fastboot to get started.</p>
          <div class="feature-grid">
            <div class="feature-item">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="5" y="2" width="14" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
              <span>Device Detection</span>
            </div>
            <div class="feature-item">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
              <span>Bootloader Unlock</span>
            </div>
            <div class="feature-item">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
              <span>ROM Flashing</span>
            </div>
            <div class="feature-item">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
              <span>HyperOS Tools</span>
            </div>
          </div>
        </div>
      </div>

      <div class="content-area" v-else>
        <div class="rom-bar">
          <div class="rom-info">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
            <span>{{ romFolder }}</span>
          </div>
          <button class="hdr-btn" @click="romFolder = ''">Clear</button>
        </div>
        <div class="rom-type" v-if="romType">{{ romType.toUpperCase() }} ROM detected</div>
        <div class="partition-card" v-if="romCommands.length">
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
      </div>
    </main>
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
let pollTimer = null

const deviceStatusText = computed(() => {
  const map = {
    none: 'No Device',
    adb: 'Connected (ADB)',
    fastboot: 'Connected (Fastboot)',
    unauthorized: 'Unauthorized',
  }
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
  try {
    const info = await sendIpcToMain('xiaomi_detect_device')
    deviceInfo.value = info
    deviceMode.value = info.mode || 'none'
  } catch {
    deviceInfo.value = null
    deviceMode.value = 'none'
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
  } catch {
    romType.value = 'unknown'
    romCommands.value = []
  }
}

function openFlashTool() {
  router.push({ path: '/xiaomi/flash' })
}

function runAdbCmd() {
  if (!customArgs.value) return
  const args = customArgs.value.split(/\s+/)
  setPending({ type: 'adb_cmd', args })
  router.push({ path: '/xiaomi/run' })
}

function runFastbootCmd() {
  if (!customArgs.value) return
  const args = customArgs.value.split(/\s+/)
  setPending({ type: 'fastboot_cmd', args })
  router.push({ path: '/xiaomi/run' })
}

onMounted(() => {
  detectDevice()
  pollTimer = setInterval(detectDevice, 5000)
})

onBeforeUnmount(() => {
  if (pollTimer) clearInterval(pollTimer)
})
</script>

<style scoped>
.xiaomi-page { height: 100%; display: flex; overflow: hidden; }

.sidebar {
  width: 260px; flex-shrink: 0; display: flex; flex-direction: column;
  background: var(--bg-secondary); border-right: 1px solid var(--border-primary);
  overflow-y: auto;
}
.sidebar-brand { display: flex; align-items: center; gap: 10px; padding: 14px 16px; border-bottom: 1px solid var(--border-primary); }
.brand-icon { width: 28px; height: 28px; border-radius: 6px; background: linear-gradient(135deg, #ff6b00, #ff9800); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700; }
.brand-text { font-size: 13px; font-weight: 600; color: var(--text-primary); }
.sidebar-section { padding: 12px 14px; border-bottom: 1px solid var(--border-primary); display: flex; flex-direction: column; gap: 6px; }
.section-title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-secondary); font-weight: 600; margin-bottom: 2px; }

.device-status { display: flex; align-items: center; gap: 6px; font-size: 11px; padding: 5px 8px; border-radius: 4px; background: var(--bg-tertiary); }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: #666; }
.status-adb .status-dot { background: #4caf50; }
.status-fastboot .status-dot { background: #2196f3; }
.status-unauthorized .status-dot { background: #ff9800; }

.info-row { display: flex; justify-content: space-between; font-size: 11px; padding: 2px 0; }
.info-label { color: var(--text-secondary); }
.text-ok { color: #4caf50; }
.text-warn { color: #ff9800; }

.cmd-row { display: flex; gap: 4px; }
.cmd-btns { display: flex; gap: 4px; }
.field-input { flex: 1; min-width: 0; background: var(--bg-tertiary); border: 1px solid var(--border-primary); border-radius: 3px; padding: 4px 6px; font-size: 11px; color: var(--text-primary); outline: none; }
.field-input:focus { border-color: var(--accent-primary); }

.op-btn {
  display: flex; align-items: center; gap: 8px; width: 100%; padding: 7px 10px;
  border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-tertiary);
  color: var(--text-primary); font-size: 11px; cursor: pointer; transition: all 0.15s;
}
.op-btn svg { width: 14px; height: 14px; flex-shrink: 0; }
.op-btn:hover:not(:disabled) { border-color: var(--accent-primary); }
.op-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.op-btn--sm { padding: 5px 8px; font-size: 10px; width: auto; }
.op-btn--primary { border-color: #ff6b0033; background: #ff6b0015; color: #ffab40; }
.op-btn--primary:hover { background: #ff6b0025; border-color: #ff6b00; }
.op-btn--accent { border-color: #22c55e33; background: #22c55e15; color: #4ade80; }
.op-btn--accent:hover { background: #22c55e25; border-color: #22c55e; }

.main-content { flex: 1; display: flex; flex-direction: column; min-width: 0; padding: 16px; gap: 12px; overflow: hidden; }
.main-header { display: flex; align-items: center; flex-shrink: 0; }
.main-header h2 { font-size: 15px; margin: 0; color: var(--text-primary); }

.content-area { flex: 1; display: flex; flex-direction: column; min-height: 0; }

.welcome-card { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; text-align: center; }
.welcome-icon { width: 64px; height: 64px; border-radius: 16px; background: linear-gradient(135deg, #ff6b00, #ff9800); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 28px; font-weight: 700; }
.welcome-card h3 { font-size: 18px; margin: 0; color: var(--text-primary); }
.welcome-card p { font-size: 13px; color: var(--text-secondary); }
.feature-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-top: 16px; }
.feature-item { display: flex; align-items: center; gap: 8px; padding: 12px 16px; border: 1px solid var(--border-primary); border-radius: 8px; background: var(--bg-secondary); font-size: 12px; color: var(--text-primary); }
.feature-item svg { width: 20px; height: 20px; color: var(--accent-primary); flex-shrink: 0; }

.rom-bar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-secondary); flex-shrink: 0; }
.rom-info { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-secondary); min-width: 0; }
.rom-info svg { width: 16px; height: 16px; flex-shrink: 0; color: var(--accent-primary); }
.rom-info span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.rom-type { font-size: 11px; color: var(--accent-primary); padding: 4px 0; }
.hdr-btn { padding: 4px 10px; border: 1px solid var(--border-primary); border-radius: 3px; background: var(--bg-secondary); color: var(--text-secondary); font-size: 11px; cursor: pointer; }
.hdr-btn:hover { color: var(--accent-primary); border-color: var(--accent-primary); }

.partition-card { flex: 1; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; display: flex; flex-direction: column; min-height: 0; }
.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.partition-table th, .partition-table td { text-align: left; padding: 7px 12px; white-space: nowrap; }
.partition-table thead th { background: var(--color-content-background); border-bottom: 1px solid var(--border-primary); color: var(--text-secondary); font-weight: 500; font-size: 11px; text-transform: uppercase; }
.partition-table tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); }
.col-type { font-weight: 500; text-transform: uppercase; font-size: 10px; color: var(--accent-primary); }
.col-cmd { color: var(--text-secondary); font-family: monospace; font-size: 11px; }
</style>
