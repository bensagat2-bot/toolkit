<template>
  <div class="mtk-page">
    <div class="mtk-header">
      <h2>MediaTek Tools</h2>
      <div class="header-controls">
        <div class="status-badge" :class="state">{{ statusLabel }}</div>
        <button class="btn btn-sm" @click="findDevice" :disabled="busy">Find Device</button>
      </div>
    </div>

    <div class="mtk-content">
      <div class="actions-panel">
        <div class="action-group">
          <h3>Download Agent</h3>
          <div class="da-picker">
            <input :value="daName" class="text-input" placeholder="No DA selected" readonly />
            <button class="btn btn-sm" @click="browseDa" :disabled="busy">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6M16 13H8M16 17H8"/></svg>
              Browse DA
            </button>
          </div>
          <button class="btn btn-primary" @click="connect" :disabled="busy || !daPath">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z"/></svg>
            Connect Device
          </button>
        </div>
        <div class="action-group">
          <h3>Auth (optional)</h3>
          <div class="da-picker">
            <input :value="authName" class="text-input" placeholder="No auth file" readonly />
            <button class="btn btn-sm" @click="browseAuth" :disabled="busy">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6"/></svg>
              Browse
            </button>
          </div>
        </div>
        <div class="action-group">
          <h3>Operations</h3>
          <button class="btn" @click="deviceInfo" :disabled="busy || !connected">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
            Device Info
          </button>
          <button class="btn" @click="listPartitions" :disabled="busy || !connected">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 10h16M4 14h16M4 18h16"/></svg>
            List Partitions
          </button>
          <button class="btn btn-danger" @click="disconnect" :disabled="busy || !connected">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 12H9M12 3a9 9 0 100 18 9 9 0 000-18z"/></svg>
            Disconnect
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
          <div v-if="logLines.length === 0" class="log-empty">Ready. Select a DA file and connect your device.</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch } from 'vue'
import { sendIpcToMain, showSelectDialog } from '@renderer/utils/ipc'

const busy = ref(false)
const connected = ref(false)
const state = ref('none')
const daPath = ref('')
const authPath = ref('')
const logLines = ref([])
const terminalRef = ref(null)

const statusLabel = computed(() => {
  if (state.value === 'da') return 'DA Mode'
  if (state.value === 'brom') return 'BROM'
  if (state.value === 'preloader') return 'Preloader'
  if (connected.value) return 'Connected'
  return 'Disconnected'
})

const daName = computed(() => daPath.value ? daPath.value.split(/[/\\]/).pop() : '')
const authName = computed(() => authPath.value ? authPath.value.split(/[/\\]/).pop() : '')

function addLog(text, type = 'info') {
  logLines.value.push({ text, type })
}

watch(() => logLines.value.length, () => {
  nextTick(() => { if (terminalRef.value) terminalRef.value.scrollTop = terminalRef.value.scrollHeight })
})

const clearLog = () => {
  logLines.value = []
}

const browseDa = async () => {
  const result = await showSelectDialog({ title: 'Select Download Agent', filters: [{ name: 'DA Files', extensions: ['bin'] }] })
  if (result.filePaths[0]) {
    daPath.value = result.filePaths[0]
    addLog(`DA selected: ${daName.value}`, 'system')
  }
}

const browseAuth = async () => {
  const result = await showSelectDialog({ title: 'Select Auth File', filters: [{ name: 'Auth Files', extensions: ['bin', 'auth'] }] })
  if (result.filePaths[0]) {
    authPath.value = result.filePaths[0]
    addLog(`Auth selected: ${authName.value}`, 'system')
  }
}

const findDevice = async () => {
  busy.value = true
  addLog('Scanning for MediaTek device...', 'info')
  try {
    const found = await sendIpcToMain('mtk_find_port')
    addLog(found ? 'MediaTek device detected.' : 'No MediaTek device found.', found ? 'success' : 'warn')
  } catch (e) {
    addLog(`Scan failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const connect = async () => {
  busy.value = true
  addLog('Connecting...', 'info')
  try {
    const info = await sendIpcToMain('mtk_connect', { da_path: daPath.value, auth_path: authPath.value || null })
    connected.value = info.connected
    state.value = (info.connection || '').toLowerCase()
    addLog(`Connected in ${info.connection} mode.`, 'success')
    addLog(`Chip: ${info.chip} (hw 0x${info.hw_code.toString(16).toUpperCase().padStart(4, '0')})`, 'system')
    if (info.da_loaded) {
      addLog(`DA loaded. Found ${info.partitions.length} partitions.`, 'success')
    } else {
      addLog('No DA loaded (preloader commands only).', 'warn')
    }
  } catch (e) {
    connected.value = false
    state.value = 'none'
    addLog(`Connect failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const deviceInfo = async () => {
  busy.value = true
  try {
    const info = await sendIpcToMain('mtk_device_info')
    addLog(`Connection: ${info.connection} | Chip: ${info.chip} | HW: 0x${info.hw_code.toString(16).toUpperCase().padStart(4, '0')}`, 'system')
  } catch (e) {
    addLog(`Failed to read device info: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const listPartitions = async () => {
  busy.value = true
  addLog('Reading partition table...', 'info')
  try {
    const data = await sendIpcToMain('mtk_list_partitions')
    addLog(`Partition table (${data.partitions.length} entries):`, 'system')
    for (const part of data.partitions) {
      addLog(`  ${part.name}  size=${part.size}  addr=0x${part.address.toString(16).toUpperCase()}`, 'info')
    }
  } catch (e) {
    addLog(`Failed to list partitions: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const disconnect = async () => {
  busy.value = true
  try {
    await sendIpcToMain('mtk_disconnect')
    connected.value = false
    state.value = 'none'
    addLog('Device disconnected.', 'success')
  } catch (e) {
    addLog(`Disconnect failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}
</script>

<style scoped>
.mtk-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.mtk-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }
.header-controls { display: flex; align-items: center; gap: 12px; }
.status-badge { padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 500; &.da { background: #1b5e20; color: #a5d6a7; } &.brom, &.preloader { background: #0d47a1; color: #90caf9; } &.none { background: #424242; color: #9e9e9e; } }
.mtk-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.actions-panel { width: 260px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.da-picker { display: flex; gap: 6px; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.btn { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } &.btn-danger { background: #c62828; color: #fff; border-color: #c62828; } }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 11px; cursor: pointer; padding: 0; &:hover { text-decoration: underline; } }
.terminal-panel { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.terminal-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-primary); span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); } }
.terminal-body { flex: 1; overflow-y: auto; padding: 8px 12px; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.5; }
.log-line { white-space: pre-wrap; word-break: break-all; &.info { color: var(--text-primary); } &.success { color: #4caf50; } &.warn { color: #ff9800; } &.error { color: #f44336; } &.system { color: var(--accent-primary); font-weight: 500; } }
.log-empty { color: var(--text-secondary); font-style: italic; }
</style>