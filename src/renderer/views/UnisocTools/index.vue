<template>
  <div class="unisoc-page">
    <div class="unisoc-header">
      <h2>Unisoc Tools</h2>
      <div class="header-controls">
        <div class="device-picker">
          <label>Chipset</label>
          <select v-model="selectedPkg" class="select-input">
            <option v-for="pkg in packageList" :key="pkg.id" :value="pkg.id">
              {{ pkg.name }} {{ !packageInstalled[pkg.id] ? '(missing)' : '' }}
            </option>
          </select>
        </div>
        <div class="device-picker">
          <label>Device</label>
          <select v-model="selectedDevice" class="select-input">
            <option value="">Auto-detect</option>
            <option v-for="alias in filteredAliases" :key="alias" :value="alias">{{ alias }}</option>
          </select>
        </div>
        <div class="status-badge" :class="deviceMode">{{ deviceModeLabel }}</div>
        <button class="btn btn-sm" @click="detectDevice" :disabled="isRunning">Detect</button>
      </div>
    </div>

    <div class="unisoc-content">
      <div class="actions-panel">
        <div class="action-group">
          <h3>Unlock</h3>
          <button class="btn btn-primary" @click="runUnlock" :disabled="!canRun">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>
            Unlock Bootloader
          </button>
          <button class="btn" @click="runEraseFrp" :disabled="!canRun">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/></svg>
            Erase FRP
          </button>
        </div>
        <div class="action-group">
          <h3>Partitions</h3>
          <button class="btn" @click="runParts" :disabled="!canRun">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 10h16M4 14h16M4 18h16"/></svg>
            List Partitions
          </button>
          <button class="btn" @click="runDump" :disabled="!canRun">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>
            Dump All
          </button>
        </div>
        <div class="action-group">
          <h3>Flash / Erase</h3>
          <input v-model="flashPartition" placeholder="Partition name" class="text-input" />
          <button class="btn btn-sm" @click="selectImage">Browse Image</button>
          <div v-if="flashImage" class="file-label">{{ flashImage.split(/[/\\]/).pop() }}</div>
          <button class="btn btn-primary" @click="runFlash" :disabled="!canRun || !flashPartition || !flashImage">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
            Flash
          </button>
          <button class="btn btn-danger" @click="runErase" :disabled="!canRun || !flashPartition">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
            Erase
          </button>
        </div>
        <div class="action-group">
          <button class="btn btn-danger" @click="stopOperation" :disabled="!isRunning">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
            Stop
          </button>
        </div>
      </div>

      <div class="terminal-panel">
        <div class="terminal-header">
          <span>Output</span>
          <button class="btn-link" @click="clearLog">Clear</button>
        </div>
        <div class="terminal-body" ref="terminalRef">
          <div v-for="(line, i) in logLines" :key="i" :class="['log-line', line.type]">{{ line.text }}</div>
          <div v-if="logLines.length === 0" class="log-empty">Ready.</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { sendIpcToMain, rendererOn, rendererOff, showSelectDialog } from '@renderer/utils/ipc'
import { PACKAGES, DEVICE_ALIASES } from './unisoc-data'

const selectedPkg = ref('ums9230')
const selectedDevice = ref('')
const deviceMode = ref('none')
const deviceModeLabel = ref('No Device')
const isRunning = ref(false)
const logLines = ref([])
const flashPartition = ref('')
const flashImage = ref('')
const terminalRef = ref(null)
const packageInstalled = ref({})
const packageList = Object.values(PACKAGES)

const filteredAliases = computed(() => {
  return Object.entries(DEVICE_ALIASES)
    .filter(([_, pkgId]) => pkgId === selectedPkg.value)
    .map(([alias]) => alias)
})

const canRun = computed(() => !isRunning.value && deviceMode.value !== 'none')

function appendLog(text, type = 'info') {
  logLines.value.push({ text, type })
  nextTick(() => { if (terminalRef.value) terminalRef.value.scrollTop = terminalRef.value.scrollHeight })
}

function clearLog() { logLines.value = [] }

function listenOutput(channel, callback) {
  const handler = (_, msg) => callback(msg)
  rendererOn(channel, handler)
  return () => rendererOff(channel, handler)
}

async function detectDevice() {
  appendLog('Detecting device...', 'system')
  try {
    const result = await sendIpcToMain('unisoc:detect')
    deviceMode.value = result.mode
    const labels = { adb: 'ADB', fastboot: 'Fastboot', download: 'Download', none: 'No Device' }
    deviceModeLabel.value = labels[result.mode] || 'No Device'
    if (result.serial) appendLog(`${result.mode.toUpperCase()}: ${result.serial}`, 'success')
    else if (result.mode === 'download') appendLog('Spreadtrum download mode.', 'success')
    else appendLog('No Unisoc device detected.', 'warn')
  } catch (e) { appendLog(`Error: ${e}`, 'error') }
}

async function runUnlock() {
  const confirmed = await sendIpcToMain('unisoc:confirm', 'Unlock bootloader? This will modify boot partitions.')
  if (!confirmed) return
  isRunning.value = true; clearLog(); appendLog('Starting unlock...', 'system')
  const cleanup = listenOutput('unisoc:unlock:output', msg => {
    if (msg.type === 'output') appendLog(msg.data)
    else if (msg.type === 'error') appendLog(msg.data, 'error')
    else if (msg.type === 'done') { isRunning.value = false; appendLog(msg.code === 0 ? 'Done.' : `Exit ${msg.code}`, msg.code === 0 ? 'success' : 'error') }
  })
  await sendIpcToMain('unisoc:unlock', { pkgId: selectedPkg.value, device: selectedDevice.value || undefined }).catch(e => appendLog(`${e}`, 'error'))
  cleanup()
}

async function runEraseFrp() {
  const confirmed = await sendIpcToMain('unisoc:confirm', 'Erase FRP partition?')
  if (!confirmed) return
  isRunning.value = true; clearLog(); appendLog('Erasing FRP...', 'system')
  const cleanup = listenOutput('unisoc:erasefrp:output', msg => {
    if (msg.type === 'output') appendLog(msg.data)
    else if (msg.type === 'error') appendLog(msg.data, 'error')
    else if (msg.type === 'done') { isRunning.value = false; appendLog(msg.code === 0 ? 'Done.' : `Exit ${msg.code}`, msg.code === 0 ? 'success' : 'error') }
  })
  await sendIpcToMain('unisoc:erasefrp', { pkgId: selectedPkg.value, device: selectedDevice.value || undefined }).catch(e => appendLog(`${e}`, 'error'))
  cleanup()
}

async function runParts() {
  isRunning.value = true; clearLog(); appendLog('Reading partitions...', 'system')
  const cleanup = listenOutput('unisoc:parts:output', msg => {
    if (msg.type === 'output') appendLog(msg.data)
    else if (msg.type === 'partitions') { for (const line of msg.data.split('\n')) if (line.trim()) appendLog(line.trim()) }
    else if (msg.type === 'error') appendLog(msg.data, 'error')
    else if (msg.type === 'done') { isRunning.value = false; appendLog(msg.code === 0 ? 'Done.' : `Exit ${msg.code}`, msg.code === 0 ? 'success' : 'error') }
  })
  await sendIpcToMain('unisoc:parts', { pkgId: selectedPkg.value, device: selectedDevice.value || undefined }).catch(e => appendLog(`${e}`, 'error'))
  cleanup()
}

async function runDump() {
  const confirmed = await sendIpcToMain('unisoc:confirm', 'Dump ALL partitions? This may take a long time.')
  if (!confirmed) return
  isRunning.value = true; clearLog(); appendLog('Starting dump...', 'system')
  const cleanup = listenOutput('unisoc:dump:output', msg => {
    if (msg.type === 'output') appendLog(msg.data)
    else if (msg.type === 'error') appendLog(msg.data, 'error')
    else if (msg.type === 'done') { isRunning.value = false; appendLog(msg.code === 0 ? 'Done.' : `Exit ${msg.code}`, msg.code === 0 ? 'success' : 'error') }
  })
  await sendIpcToMain('unisoc:dump', { pkgId: selectedPkg.value, device: selectedDevice.value || undefined }).catch(e => appendLog(`${e}`, 'error'))
  cleanup()
}

async function runFlash() {
  const confirmed = await sendIpcToMain('unisoc:confirm', `Flash ${flashPartition.value}?`)
  if (!confirmed) return
  isRunning.value = true; clearLog(); appendLog('Flashing...', 'system')
  const cleanup = listenOutput('unisoc:flash:output', msg => {
    if (msg.type === 'output') appendLog(msg.data)
    else if (msg.type === 'error') appendLog(msg.data, 'error')
    else if (msg.type === 'done') { isRunning.value = false; appendLog(msg.code === 0 ? 'Done.' : `Exit ${msg.code}`, msg.code === 0 ? 'success' : 'error') }
  })
  await sendIpcToMain('unisoc:flash', { pkgId: selectedPkg.value, device: selectedDevice.value || undefined, partition: flashPartition.value, image: flashImage.value }).catch(e => appendLog(`${e}`, 'error'))
  cleanup()
}

async function runErase() {
  const confirmed = await sendIpcToMain('unisoc:confirm', `ERASE ${flashPartition.value}? This is destructive!`)
  if (!confirmed) return
  isRunning.value = true; clearLog(); appendLog('Erasing...', 'system')
  const cleanup = listenOutput('unisoc:erase:output', msg => {
    if (msg.type === 'output') appendLog(msg.data)
    else if (msg.type === 'error') appendLog(msg.data, 'error')
    else if (msg.type === 'done') { isRunning.value = false; appendLog(msg.code === 0 ? 'Done.' : `Exit ${msg.code}`, msg.code === 0 ? 'success' : 'error') }
  })
  await sendIpcToMain('unisoc:erase', { pkgId: selectedPkg.value, device: selectedDevice.value || undefined, partition: flashPartition.value }).catch(e => appendLog(`${e}`, 'error'))
  cleanup()
}

async function stopOperation() {
  await sendIpcToMain('unisoc:stop')
  appendLog('Stopped.', 'warn')
}

async function selectImage() {
  const file = await sendIpcToMain('unisoc:select-file')
  if (file) flashImage.value = file
}

onMounted(async () => {
  packageInstalled.value = await sendIpcToMain('unisoc:get-packages')
  detectDevice()
})
</script>

<style lang="less" scoped>
.unisoc-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.unisoc-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }
.header-controls { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.device-picker { display: flex; align-items: center; gap: 6px; font-size: 12px; label { color: var(--text-secondary); } }
.select-input { background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.status-badge { padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 500; &.adb { background: #1b5e20; color: #a5d6a7; } &.fastboot { background: #0d47a1; color: #90caf9; } &.download { background: #e65100; color: #ffcc80; } &.none { background: #424242; color: #9e9e9e; } }
.unisoc-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.actions-panel { width: 220px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.text-input { background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.file-label { font-size: 11px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.btn { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } &.btn-danger { background: #c62828; color: #fff; border-color: #c62828; } }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 11px; cursor: pointer; padding: 0; &:hover { text-decoration: underline; } }
.terminal-panel { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.terminal-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-primary); span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); } }
.terminal-body { flex: 1; overflow-y: auto; padding: 8px 12px; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.5; }
.log-line { white-space: pre-wrap; word-break: break-all; &.info { color: var(--text-primary); } &.success { color: #4caf50; } &.warn { color: #ff9800; } &.error { color: #f44336; } &.system { color: var(--accent-primary); font-weight: 500; } }
.log-empty { color: var(--text-secondary); font-style: italic; }
</style>
