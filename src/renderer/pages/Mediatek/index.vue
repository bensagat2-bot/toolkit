<template>
  <div class="mtk-page">
    <div class="mtk-header">
      <h2>MediaTek Tools</h2>
      <div class="header-controls">
        <div class="status-badge" :class="state">{{ statusLabel }}</div>
      </div>
    </div>

    <div class="mtk-content">
      <div class="flash-panel">
        <div class="action-group">
          <h3>Download Agent</h3>
          <div class="row">
            <input :value="daName" class="text-input" placeholder="No DA selected" readonly />
            <button class="btn btn-sm" @click="browseDa" :disabled="busy">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Scatter Firmware</h3>
          <div class="row">
            <input :value="scatterName" class="text-input" placeholder="No scatter file" readonly />
            <button class="btn btn-sm" @click="browseScatter" :disabled="busy">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Auth File (Optional)</h3>
          <div class="row">
            <input :value="authName" class="text-input" placeholder="No auth file" readonly />
            <button class="btn btn-sm" @click="browseAuth" :disabled="busy">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Connection Options</h3>
          <label class="check"><input type="checkbox" v-model="forceBrom" :disabled="busy" /> Force BROM</label>
          <label class="check"><input type="checkbox" v-model="usePreloaderFw" :disabled="busy" /> Use preloader from FW</label>
          <label class="check"><input type="checkbox" v-model="erasePreloader" :disabled="busy" /> Force BROM Erase preloader</label>
        </div>

        <button class="btn btn-primary start-btn" @click="startFlash" :disabled="busy || !scatterPath">
          Start Flash
        </button>
      </div>

      <div class="output-card">
        <div class="output-card-header">
          <span>Firmware Partitions</span>
          <span v-if="busy" class="elapsed">Elapsed: {{ elapsed }}s</span>
        </div>
        <div class="partition-table-wrap">
          <table class="partition-table">
            <thead>
              <tr>
                <th>Partition name</th>
                <th>Start Address</th>
                <th>Size</th>
                <th>Filename</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="partitions.length === 0" class="empty-row">
                <td colspan="4">No scatter file selected.</td>
              </tr>
              <tr v-for="(p, i) in partitions" :key="i">
                <td>{{ p.name }}</td>
                <td>{{ p.address }}</td>
                <td>{{ formatSize(p.size) }}</td>
                <td>{{ p.filename }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="terminal-header">
          <span>Output</span>
          <button class="btn-link" @click="clearLog">Clear</button>
        </div>
        <div class="terminal-body" ref="terminalRef">
          <div v-for="(line, i) in logLines" :key="i" :class="['log-line', line.type]">{{ line.text }}</div>
          <div v-if="logLines.length === 0" class="log-empty">Ready. Select a scatter file and connect your device.</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { sendIpcToMain, showSelectDialog, rendererOn } from '@renderer/utils/ipc'

const busy = ref(false)
const state = ref('none')
const daPath = ref('')
const scatterPath = ref('')
const authPath = ref('')
const forceBrom = ref(false)
const usePreloaderFw = ref(false)
const erasePreloader = ref(false)
const partitions = ref([])
const logLines = ref([])
const elapsed = ref(0)
const terminalRef = ref(null)
let elapsedTimer = null

const statusLabel = computed(() => (busy.value ? 'Flashing' : 'Idle'))

const daName = computed(() => daPath.value ? daPath.value.split(/[/\\]/).pop() : '')
const scatterName = computed(() => scatterPath.value ? scatterPath.value.split(/[/\\]/).pop() : '')
const authName = computed(() => authPath.value ? authPath.value.split(/[/\\]/).pop() : '')

function addLog(text, type = 'info') {
  logLines.value.push({ text, type })
}

watch(() => logLines.value.length, () => {
  nextTick(() => { if (terminalRef.value) terminalRef.value.scrollTop = terminalRef.value.scrollHeight })
})

const clearLog = () => { logLines.value = [] }

const pickFile = async (opts) => (await showSelectDialog(opts)).filePaths[0] || ''

const browseDa = async () => {
  const p = await pickFile({ title: 'Select Download Agent', filters: [{ name: 'DA Files', extensions: ['bin'] }] })
  if (p) { daPath.value = p; addLog(`DA selected: ${daName.value}`, 'system') }
}

const browseScatter = async () => {
  const p = await pickFile({ title: 'Select Scatter File', filters: [{ name: 'Scatter Files', extensions: ['txt', 'xml'] }] })
  if (p) { scatterPath.value = p; addLog(`Scatter selected: ${scatterName.value}`, 'system'); await loadScatter() }
}

const browseAuth = async () => {
  const p = await pickFile({ title: 'Select Auth File', filters: [{ name: 'Auth Files', extensions: ['bin', 'auth'] }] })
  if (p) { authPath.value = p; addLog(`Auth selected: ${authName.value}`, 'system') }
}

async function loadScatter() {
  partitions.value = []
  try {
    const data = await sendIpcToMain('mtk_load_scatter', { path: scatterPath.value })
    partitions.value = data.partitions || []
    addLog(`Loaded ${partitions.value.length} partitions from scatter.`, 'success')
  } catch (e) {
    addLog(`Failed to load scatter: ${e}`, 'error')
  }
}

function formatSize(bytes) {
  if (!bytes) return '0'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let n = bytes
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++ }
  return `${n.toFixed(i ? 2 : 0)} ${units[i]}`
}

const startFlash = async () => {
  busy.value = true
  state.value = 'flashing'
  logLines.value = []
  elapsed.value = 0
  elapsedTimer = setInterval(() => { elapsed.value += 1 }, 1000)
  try {
    const result = await sendIpcToMain('mtk_flash', {
      da_path: daPath.value || null,
      scatter_path: scatterPath.value,
      auth_path: authPath.value || null,
      force_brom: forceBrom.value,
      use_preloader_from_fw: usePreloaderFw.value,
      force_brom_erase_preloader: erasePreloader.value,
      timeout_secs: 120,
    })
    state.value = 'done'
    addLog(`Flash complete: ${result.flashed.length} partitions, ${formatSize(result.bytes)}.`, 'success')
  } catch (e) {
    state.value = 'none'
    addLog(`Flash failed: ${e}`, 'error')
  } finally {
    clearInterval(elapsedTimer)
    elapsedTimer = null
    busy.value = false
    if (state.value !== 'done') state.value = 'none'
  }
}

const onProgress = (_event, data) => {
  if (!data || !data.message) return
  addLog(data.message, String(data.message).endsWith('OK') ? 'success' : 'info')
}

onMounted(() => {
  rendererOn('mtk:progress', onProgress)
})

onBeforeUnmount(() => {
  if (elapsedTimer) clearInterval(elapsedTimer)
})
</script>

<style scoped>
.mtk-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.mtk-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }
.header-controls { display: flex; align-items: center; gap: 12px; }
.status-badge { padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 500; &.flashing { background: #e65100; color: #ffcc80; } &.done { background: #1b5e20; color: #a5d6a7; } &.none { background: #424242; color: #9e9e9e; } }
.mtk-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.flash-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.check { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-primary); cursor: pointer; input { accent-color: var(--accent-primary); } }
.start-btn { width: 100%; padding: 10px; font-size: 13px; font-weight: 600; margin-top: auto; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 11px; cursor: pointer; padding: 0; &:hover { text-decoration: underline; } }
.output-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.output-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-primary); span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); } .elapsed { color: var(--accent-primary); } }
.partition-table-wrap { max-height: 40%; overflow-y: auto; border-bottom: 1px solid var(--border-primary); }
.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; th, td { text-align: left; padding: 4px 10px; white-space: nowrap; } thead th { position: sticky; top: 0; background: var(--bg-tertiary); color: var(--text-secondary); font-weight: 500; text-transform: uppercase; letter-spacing: 0.5px; font-size: 10px; } tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); &:hover { background: var(--bg-tertiary); } } td:first-child { font-weight: 500; } .empty-row td { color: var(--text-secondary); font-style: italic; text-align: center; } }
.terminal-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-primary); span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); } }
.terminal-body { flex: 1; overflow-y: auto; padding: 8px 12px; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.5; }
.log-line { white-space: pre-wrap; word-break: break-all; &.info { color: var(--text-primary); } &.success { color: #4caf50; } &.warn { color: #ff9800; } &.error { color: #f44336; } &.system { color: var(--accent-primary); font-weight: 500; } }
.log-empty { color: var(--text-secondary); font-style: italic; }
</style>