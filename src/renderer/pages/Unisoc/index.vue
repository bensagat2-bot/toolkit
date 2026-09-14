<template>
  <div class="mtk-page">
    <div class="mtk-header">
      <h2>Unisoc Tools</h2>
    </div>

    <div class="mtk-content">
      <div class="flash-panel">
        <div class="action-group">
          <h3>Chipset</h3>
          <select v-model="selectedPkg" class="text-input">
            <option v-for="pkg in packageList" :key="pkg.id" :value="pkg.id">
              {{ pkg.name }} {{ !packageInstalled[pkg.id] ? '(missing)' : '' }}
            </option>
          </select>
        </div>

        <div class="action-group">
          <h3>Device</h3>
          <select v-model="selectedDevice" class="text-input">
            <option value="">Auto-detect</option>
            <option v-for="alias in filteredAliases" :key="alias" :value="alias">{{ alias }}</option>
          </select>
        </div>

        <div class="action-group">
          <h3>Firmware Folder</h3>
          <div class="row">
            <input :value="folderName" class="text-input" placeholder="No firmware folder" readonly />
            <button class="btn btn-sm" @click="browseFolder">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Operations</h3>
          <button class="btn btn-primary" @click="runUnlock" :disabled="busy">Unlock Bootloader</button>
          <button class="btn btn-primary" @click="runFlash" :disabled="busy || !folderPath || selected.length === 0">Flash Firmware</button>
          <button class="btn" @click="runEraseFrp" :disabled="busy">Erase FRP</button>
          <button class="btn" @click="runDump" :disabled="busy">Dump Partitions</button>
          <button class="btn btn-danger" @click="stop" :disabled="busy">Stop</button>
          <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>
        </div>

        <div class="action-group">
          <h3>Connection</h3>
          <div class="row">
            <label class="field-label">Wait (s)</label>
            <input v-model.number="connWait" type="number" min="0" class="text-input" />
          </div>
          <div class="row">
            <label class="field-label">Baud</label>
            <input v-model="connBaud" class="text-input" placeholder="auto" />
          </div>
        </div>
      </div>

      <div class="output-card">
        <div class="output-card-header" v-if="folderPath">
          <span>Firmware Partitions</span>
          <div class="header-actions">
            <span v-if="partitions.length" class="selection-count">{{ selected.length }} / {{ partitions.length }} selected</span>
            <button class="btn-link" :disabled="!partitions.length" @click="selectAll">Select All</button>
            <button class="btn-link" :disabled="!partitions.length" @click="deselectAll">Deselect All</button>
          </div>
        </div>
        <div class="output-card-header" v-else>
          <span>Unisoc Service Toolkit</span>
        </div>

        <div class="partition-table-wrap" v-if="folderPath && partitions.length">
          <table class="partition-table">
            <thead>
              <tr>
                <th class="col-check"><input type="checkbox" :checked="allChecked" @change="toggleAll" /></th>
                <th>Partition name</th>
                <th>Filename</th>
                <th>Size</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(p, i) in partitions" :key="i" :class="{ selected: isSelected(p) }" @click="toggle(p)">
                <td class="col-check" @click.stop><input type="checkbox" :checked="isSelected(p)" @change="toggle(p)" /></td>
                <td>{{ p.name }}</td>
                <td>{{ p.file }}</td>
                <td>{{ formatSize(p.size) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="output-body welcome-body" v-else>
          <div class="welcome-icon">S</div>
          <h3>Unisoc Service Toolkit</h3>
          <p>Select a chipset and firmware folder to get started.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, showSelectFolder, showSelectDialog } from '@renderer/utils/ipc'
import { setPending } from './runStore'
import { PACKAGES, DEVICE_ALIASES } from './unisoc-data'
import { UNISOC_OPS, OPS_GROUPS } from './unisoc-ops'

const router = useRouter()
const folderPath = ref('')
const partitions = ref([])
const selected = ref([])
const packageList = Object.values(PACKAGES)
const filteredAliases = computed(() =>
  Object.entries(DEVICE_ALIASES).filter(([_, pkgId]) => pkgId === selectedPkg.value).map(([alias]) => alias)
)
const selectedPkg = ref('ums9230')
const selectedDevice = ref('')
const packageInstalled = ref({})
const connWait = ref(300)
const connBaud = ref('')
const connBlk = ref('')
const form = reactive({})
const errorMsg = ref('')
const busy = ref(false)
UNISOC_OPS.forEach((op) => { form[opIndex(op)] = reactive({}) })
const opsGroups = OPS_GROUPS
const folderName = computed(() => folderPath.value ? folderPath.value.split(/[/\\]/).pop() : '')
const allChecked = computed(() => partitions.value.length > 0 && selected.value.length === partitions.value.length)

function opIndex(op) { return `${op.group}:${op.name}:${op.label}` }
function opsByGroup(groupId) { return UNISOC_OPS.filter((op) => op.group === groupId) }
function isSelected(p) { return selected.value.some((s) => s.name === p.name) }
function toggle(p) {
  const idx = selected.value.findIndex((s) => s.name === p.name)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push({ name: p.name, file: p.file })
}
function toggleAll() { selected.value = allChecked.value ? [] : partitions.value.map((p) => ({ name: p.name, file: p.file })) }
function selectAll() { selected.value = partitions.value.map((p) => ({ name: p.name, file: p.file })) }
function deselectAll() { selected.value = [] }
function formatSize(bytes) {
  if (!bytes) return '0'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0; let n = bytes
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++ }
  return `${n.toFixed(i ? 2 : 0)} ${units[i]}`
}

const browseFolder = async () => {
  errorMsg.value = ''
  const p = await showSelectFolder('Select Firmware Folder')
  if (!p) return
  folderPath.value = p
  try {
    const data = await sendIpcToMain('unisoc_scan_folder', { path: folderPath.value })
    partitions.value = data.partitions || []
    selected.value = partitions.value.map((p) => ({ name: p.name, file: p.file }))
  } catch (e) {
    console.error('Scan folder failed:', e)
    errorMsg.value = 'Failed to scan firmware folder.'
    partitions.value = []
  }
}

const pickFile = async (op, field) => {
  const filters = op.name === 'repartition' ? [{ name: 'XML', extensions: ['xml'] }] : undefined
  const res = await showSelectDialog({ title: 'Select File', filters })
  if (res.filePaths && res.filePaths.length) { form[opIndex(op)][field.key] = res.filePaths[0] }
}

function runUnlock() {
  setPending({ type: 'unlock', pkg_id: selectedPkg.value, device: selectedDevice.value || null })
  router.push({ path: '/unisoc/run' })
}
function runFlash() {
  setPending({ type: 'flash', pkg_id: selectedPkg.value, device: selectedDevice.value || null, folder: folderPath.value, partitions: selected.value })
  router.push({ path: '/unisoc/run' })
}
function runEraseFrp() {
  setPending({ type: 'erase_frp', pkg_id: selectedPkg.value, device: selectedDevice.value || null })
  router.push({ path: '/unisoc/run' })
}
function runDump() {
  setPending({ type: 'dump', pkg_id: selectedPkg.value, device: selectedDevice.value || null })
  router.push({ path: '/unisoc/run' })
}
async function runOp(op) {
  errorMsg.value = ''
  if (op.danger) {
    const ok = await sendIpcToMain('confirm_action', { message: `Run "${op.label}"? This may erase data.` })
    if (!ok) return
  }
  const values = form[opIndex(op)] || {}
  const cliOp = { name: op.name }
  for (const f of op.fields || []) {
    if (f.type === 'file' && !values[f.key]) {
      errorMsg.value = `${f.label} is required for ${op.label}`
      return
    }
    cliOp[f.key] = values[f.key] || undefined
  }
  busy.value = true
  setPending({ type: 'cli', pkg_id: selectedPkg.value, device: selectedDevice.value || null, cli: { wait_secs: connWait.value, baudrate: connBaud.value || undefined, blk_size: connBlk.value || undefined, ops: [cliOp] } })
  router.push({ path: '/unisoc/run' })
}
async function stop() { await sendIpcToMain('stop_process') }

onMounted(async () => { packageInstalled.value = await sendIpcToMain('get_packages') })
</script>

<style scoped>
.mtk-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.mtk-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }

.mtk-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.flash-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; align-items: center; }
.field-label { font-size: 11px; color: var(--text-secondary); white-space: nowrap; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; }
.text-input:focus { border-color: var(--accent-primary); }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } &.btn-danger { background: var(--color-status-error); color: var(--color-white); border-color: var(--color-status-error); } }

.output-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.output-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } .header-actions { display: flex; align-items: center; gap: 10px; } .selection-count { color: var(--accent-primary); } .btn-link { background: none; border: none; color: var(--accent-primary); font-size: 12px; cursor: pointer; padding: 0; &:hover:not(:disabled) { text-decoration: underline; } &:disabled { opacity: 0.4; cursor: not-allowed; } } }
.partition-table-wrap { flex: 1; overflow-y: auto; }
.welcome-body { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; text-align: center; padding: 40px; flex: 1; h3 { font-size: 16px; margin: 0; } p { font-size: 12px; color: var(--text-secondary); margin: 0; } }
.welcome-icon { width: 48px; height: 48px; border-radius: 12px; background: linear-gradient(135deg, #7c3aed, #a855f7); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; }

.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.partition-table th, .partition-table td { text-align: left; padding: 6px 10px; white-space: nowrap; }
.partition-table thead th { position: sticky; top: 0; background: var(--color-content-background); border-bottom: var(--color-list-header-border-bottom); color: var(--color-font); font-weight: 500; font-size: 12px; }
.partition-table tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); cursor: pointer; &:hover { background: var(--bg-tertiary); } &.selected { background: color-mix(in srgb, var(--accent-primary) 8%, transparent); } }
.col-check { width: 28px; text-align: center; input { accent-color: var(--accent-primary); cursor: pointer; } }
</style>
