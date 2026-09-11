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
          <div class="row">
            <input :value="daName" class="text-input" placeholder="No DA selected" readonly />
            <button class="btn btn-sm" @click="browseDa" :disabled="busy">Browse DA</button>
          </div>
          <button class="btn btn-primary" @click="connect" :disabled="busy || !daPath">
            Connect Device
          </button>
        </div>
        <div class="action-group">
          <h3>Auth (optional)</h3>
          <div class="row">
            <input :value="authName" class="text-input" placeholder="No auth file" readonly />
            <button class="btn btn-sm" @click="browseAuth" :disabled="busy">Browse</button>
          </div>
        </div>
        <div class="action-group">
          <h3>Flash / Read</h3>
          <input v-model="partition" class="text-input" placeholder="Partition name" />
          <div class="row">
            <button class="btn btn-sm" @click="browseImage" :disabled="busy || !connected">Image</button>
            <button class="btn btn-sm" @click="browseDest" :disabled="busy || !connected">Dest</button>
            <button class="btn btn-sm" @click="listPartitions" :disabled="busy || !connected">Parts</button>
          </div>
          <button class="btn btn-primary" @click="writePartition" :disabled="busy || !connected || !partition || !imagePath">
            Write Partition
          </button>
          <button class="btn" @click="readPartition" :disabled="busy || !connected || !partition || !destPath">
            Read Partition
          </button>
          <button class="btn btn-danger" @click="erasePartition" :disabled="busy || !connected || !partition">
            Erase Partition
          </button>
        </div>
        <div class="action-group">
          <h3>Boot</h3>
          <div class="row">
            <button class="btn" @click="reboot('normal')" :disabled="busy || !connected">Normal</button>
            <button class="btn" @click="reboot('fastboot')" :disabled="busy || !connected">Fastboot</button>
            <button class="btn" @click="reboot('meta')" :disabled="busy || !connected">Meta</button>
          </div>
          <div class="row">
            <button class="btn" @click="getBootctrl" :disabled="busy || !connected">Active Slot</button>
            <button class="btn" @click="getStorage" :disabled="busy || !connected">Storage</button>
          </div>
          <button class="btn btn-danger" @click="unlockBootloader" :disabled="busy || !connected">Unlock Bootloader</button>
        </div>
        <div class="action-group">
          <h3>Registers / Memory</h3>
          <div class="row">
            <input v-model="regAddr" class="text-input" placeholder="Address (hex)" />
            <input v-model="regValue" class="text-input" placeholder="Value (hex)" />
          </div>
          <div class="row">
            <button class="btn btn-sm" @click="readRegister" :disabled="busy || !connected">Read Reg</button>
            <button class="btn btn-sm" @click="writeRegister" :disabled="busy || !connected">Write Reg</button>
          </div>
          <input v-model="memAddr" class="text-input" placeholder="Memory addr (hex)" />
          <div class="row">
            <button class="btn btn-sm" @click="peek" :disabled="busy || !connected || !destPath">Peek</button>
            <button class="btn btn-sm" @click="poke" :disabled="busy || !connected || !imagePath">Poke</button>
          </div>
        </div>
        <div class="action-group">
          <button class="btn" @click="readEfuses" :disabled="busy || !connected || !destPath">Read Efuses</button>
          <button class="btn btn-danger" @click="writeEfuses" :disabled="busy || !connected || !imagePath">Write Efuses</button>
          <button class="btn btn-danger" @click="disconnect" :disabled="busy || !connected">Disconnect</button>
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
import { sendIpcToMain, showSelectDialog, showSaveDialog } from '@renderer/utils/ipc'

const busy = ref(false)
const connected = ref(false)
const state = ref('none')
const daPath = ref('')
const authPath = ref('')
const partition = ref('')
const imagePath = ref('')
const destPath = ref('')
const regAddr = ref('')
const regValue = ref('')
const memAddr = ref('')
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

const clearLog = () => { logLines.value = [] }

async function confirmAction(message) {
  return sendIpcToMain('confirm_action', { message })
}

const pickFile = async (opts) => (await showSelectDialog(opts)).filePaths[0] || ''
const pickDest = async (opts) => (await showSaveDialog(opts)).filePath || ''

const browseDa = async () => {
  const p = await pickFile({ title: 'Select Download Agent', filters: [{ name: 'DA Files', extensions: ['bin'] }] })
  if (p) { daPath.value = p; addLog(`DA selected: ${daName.value}`, 'system') }
}

const browseAuth = async () => {
  const p = await pickFile({ title: 'Select Auth File', filters: [{ name: 'Auth Files', extensions: ['bin', 'auth'] }] })
  if (p) { authPath.value = p; addLog(`Auth selected: ${authName.value}`, 'system') }
}

const browseImage = async () => {
  const p = await pickFile({ title: 'Select Image', filters: [{ name: 'Images', extensions: ['img', 'bin'] }] })
  if (p) { imagePath.value = p; addLog(`Image: ${p.split(/[/\\]/).pop()}`, 'system') }
}

const browseDest = async () => {
  const p = await pickDest({ title: 'Select Destination' })
  if (p) { destPath.value = p; addLog(`Destination: ${p.split(/[/\\]/).pop()}`, 'system') }
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

const writePartition = async () => {
  if (!await confirmAction(`Write ${partition.value} from ${imagePath.value.split(/[/\\]/).pop()}?`)) return
  busy.value = true
  addLog(`Writing ${partition.value}...`, 'info')
  try {
    await sendIpcToMain('mtk_write_partition', { partition: partition.value, path: imagePath.value })
    addLog(`Wrote ${partition.value} successfully.`, 'success')
  } catch (e) {
    addLog(`Write failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const readPartition = async () => {
  busy.value = true
  addLog(`Reading ${partition.value}...`, 'info')
  try {
    await sendIpcToMain('mtk_read_partition', { partition: partition.value, path: destPath.value })
    addLog(`Read ${partition.value} to ${destPath.value.split(/[/\\]/).pop()}.`, 'success')
  } catch (e) {
    addLog(`Read failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const erasePartition = async () => {
  if (!await confirmAction(`ERASE ${partition.value}? This cannot be undone.`)) return
  busy.value = true
  addLog(`Erasing ${partition.value}...`, 'info')
  try {
    await sendIpcToMain('mtk_erase_partition', { partition: partition.value })
    addLog(`Erased ${partition.value}.`, 'success')
  } catch (e) {
    addLog(`Erase failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const reboot = async (mode) => {
  busy.value = true
  addLog(`Rebooting to ${mode}...`, 'info')
  try {
    await sendIpcToMain('mtk_reboot', { mode })
    addLog(`Reboot to ${mode} sent.`, 'success')
  } catch (e) {
    addLog(`Reboot failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const getBootctrl = async () => {
  busy.value = true
  try {
    const data = await sendIpcToMain('mtk_bootctrl')
    addLog(`Active slot: ${data.active_slot}`, 'system')
  } catch (e) {
    addLog(`Failed to read boot control: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const getStorage = async () => {
  busy.value = true
  try {
    const data = await sendIpcToMain('mtk_storage')
    addLog(`Storage: ${data.storage}`, 'system')
  } catch (e) {
    addLog(`Failed to read storage: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const unlockBootloader = async () => {
  if (!await confirmAction('Unlock bootloader (seccfg)? Device will reboot after this.')) return
  busy.value = true
  addLog('Unlocking bootloader...', 'info')
  try {
    await sendIpcToMain('mtk_set_seccfg_lock_state', { unlock: true })
    addLog('Bootloader unlock applied.', 'success')
  } catch (e) {
    addLog(`Unlock failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const readRegister = async () => {
  const addr = parseInt(regAddr.value, 16)
  if (Number.isNaN(addr)) { addLog('Invalid address.', 'error'); return }
  busy.value = true
  try {
    const data = await sendIpcToMain('mtk_read_register', { addr })
    addLog(`reg[0x${addr.toString(16).toUpperCase()}] = 0x${data.value.toString(16).toUpperCase().padStart(8, '0')}`, 'system')
  } catch (e) {
    addLog(`Read register failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const writeRegister = async () => {
  const addr = parseInt(regAddr.value, 16)
  const value = parseInt(regValue.value, 16)
  if (Number.isNaN(addr) || Number.isNaN(value)) { addLog('Invalid address/value.', 'error'); return }
  if (!await confirmAction(`Write register 0x${addr.toString(16).toUpperCase()} = 0x${value.toString(16).toUpperCase()}?`)) return
  busy.value = true
  try {
    await sendIpcToMain('mtk_write_register', { addr, value })
    addLog(`Wrote register 0x${addr.toString(16).toUpperCase()}.`, 'success')
  } catch (e) {
    addLog(`Write register failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const peek = async () => {
  const addr = parseInt(memAddr.value, 16)
  if (Number.isNaN(addr)) { addLog('Invalid address.', 'error'); return }
  busy.value = true
  addLog(`Peeking memory at 0x${addr.toString(16).toUpperCase()}...`, 'info')
  try {
    await sendIpcToMain('mtk_peek', { addr, size: 4096, path: destPath.value })
    addLog(`Peeked 4096 bytes to ${destPath.value.split(/[/\\]/).pop()}.`, 'success')
  } catch (e) {
    addLog(`Peek failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const poke = async () => {
  const addr = parseInt(memAddr.value, 16)
  if (Number.isNaN(addr)) { addLog('Invalid address.', 'error'); return }
  if (!await confirmAction(`Poke memory at 0x${addr.toString(16).toUpperCase()} from ${imagePath.value.split(/[/\\]/).pop()}?`)) return
  busy.value = true
  addLog(`Poking memory at 0x${addr.toString(16).toUpperCase()}...`, 'info')
  try {
    await sendIpcToMain('mtk_poke', { addr, path: imagePath.value })
    addLog('Poke complete.', 'success')
  } catch (e) {
    addLog(`Poke failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const readEfuses = async () => {
  busy.value = true
  addLog('Reading efuses...', 'info')
  try {
    await sendIpcToMain('mtk_read_efuses', { path: destPath.value })
    addLog(`Efuses saved to ${destPath.value.split(/[/\\]/).pop()}.`, 'success')
  } catch (e) {
    addLog(`Read efuses failed: ${e}`, 'error')
  } finally {
    busy.value = false
  }
}

const writeEfuses = async () => {
  if (!await confirmAction('WRITE EFUSES from selected image? IRREVERSIBLE.')) return
  busy.value = true
  addLog('Writing efuses...', 'info')
  try {
    await sendIpcToMain('mtk_write_efuses', { path: imagePath.value })
    addLog('Efuses written.', 'success')
  } catch (e) {
    addLog(`Write efuses failed: ${e}`, 'error')
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
.actions-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } &.btn-danger { background: #c62828; color: #fff; border-color: #c62828; } }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 11px; cursor: pointer; padding: 0; &:hover { text-decoration: underline; } }
.terminal-panel { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.terminal-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-primary); span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); } }
.terminal-body { flex: 1; overflow-y: auto; padding: 8px 12px; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.5; }
.log-line { white-space: pre-wrap; word-break: break-all; &.info { color: var(--text-primary); } &.success { color: #4caf50; } &.warn { color: #ff9800; } &.error { color: #f44336; } &.system { color: var(--accent-primary); font-weight: 500; } }
.log-empty { color: var(--text-secondary); font-style: italic; }
</style>