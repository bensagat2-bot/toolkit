<template>
  <div class="unisoc-page">
    <div class="unisoc-header">
      <h2>Unisoc Tools</h2>
    </div>

    <div class="unisoc-content">
      <div class="flash-panel">
        <div class="action-group">
          <h3>Chipset</h3>
          <select v-model="selectedPkg" class="select-input">
            <option v-for="pkg in packageList" :key="pkg.id" :value="pkg.id">
              {{ pkg.name }} {{ !packageInstalled[pkg.id] ? '(missing)' : '' }}
            </option>
          </select>
        </div>

        <div class="action-group">
          <h3>Device</h3>
          <select v-model="selectedDevice" class="select-input">
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
          <button class="btn btn-primary" @click="runUnlock">
            Unlock Bootloader
          </button>
          <button class="btn btn-primary" @click="runFlash" :disabled="!folderPath || selected.length === 0">
            Flash Firmware
          </button>
          <button class="btn" @click="runEraseFrp">
            Erase FRP
          </button>
          <button class="btn" @click="runDump">
            Dump Partitions
          </button>
          <button class="btn btn-danger" @click="stop">
            Stop
          </button>
        </div>

        <div class="action-group">
          <h3>Connection</h3>
          <div class="row">
            <label class="field-label">Wait (s)</label>
            <input v-model.number="connWait" type="number" min="0" class="text-input" />
          </div>
          <div class="row">
            <label class="field-label">Baudrate</label>
            <input v-model="connBaud" class="text-input" placeholder="auto" />
          </div>
          <div class="row">
            <label class="field-label">Blk size</label>
            <input v-model="connBlk" class="text-input" placeholder="auto" />
          </div>
          <div class="row">
            <label class="field-label">Kick</label>
            <input v-model="connKick" type="checkbox" class="checkbox" />
            <label class="field-label">Kick to</label>
            <input v-model="connKickto" class="text-input" placeholder="dl_diag" />
          </div>
        </div>
      </div>

      <div class="right-col">
        <div class="output-card">
          <div class="output-card-header">
            <span>Firmware Partitions</span>
            <div class="header-actions">
              <span v-if="partitions.length" class="selection-count">{{ selected.length }} / {{ partitions.length }} selected</span>
              <button class="btn-link" :disabled="!partitions.length" @click="selectAll">Select All</button>
              <button class="btn-link" :disabled="!partitions.length" @click="deselectAll">Deselect All</button>
            </div>
          </div>
          <div class="partition-table-wrap">
            <table class="partition-table">
              <thead>
                <tr>
                  <th class="col-check">
                    <input type="checkbox" :checked="allChecked" @change="toggleAll" />
                  </th>
                  <th>Partition name</th>
                  <th>Filename</th>
                  <th>Size</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="partitions.length === 0" class="empty-row">
                  <td colspan="4">Select a firmware folder to see its partitions.</td>
                </tr>
                <tr v-for="(p, i) in partitions" :key="i" :class="{ selected: isSelected(p) }" @click="toggle(p)">
                  <td class="col-check" @click.stop><input type="checkbox" :checked="isSelected(p)" @change="toggle(p)" /></td>
                  <td>{{ p.name }}</td>
                  <td>{{ p.file }}</td>
                  <td>{{ formatSize(p.size) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="ops-card">
          <div class="ops-card-header">
            <span>SPD CLI Tools</span>
          </div>
          <div class="ops-body">
            <div v-for="grp in opsGroups" :key="grp.id" class="ops-group">
              <div class="ops-group-title">{{ grp.label }}</div>
              <div class="ops-grid">
                <div v-for="op in opsByGroup(grp.id)" :key="op.name + op.label" class="op-item">
                  <div v-for="f in op.fields || []" :key="f.key" class="row">
                    <label class="field-label">{{ f.label }}</label>
                    <input v-if="f.type === 'text'" v-model="form[opIndex(op)][f.key]" class="text-input" :placeholder="f.placeholder || ''" />
                    <select v-else-if="f.type === 'select'" v-model="form[opIndex(op)][f.key]" class="select-input">
                      <option v-for="o in f.options || []" :key="o" :value="o">{{ o }}</option>
                    </select>
                    <div v-else class="row file-row">
                      <input :value="form[opIndex(op)][f.key] || ''" class="text-input" readonly />
                      <button class="btn btn-sm" @click="pickFile(op, f)">...</button>
                    </div>
                  </div>
                  <button class="btn" :class="{ 'btn-danger': op.danger }" @click="runOp(op)">
                    {{ op.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>
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
  Object.entries(DEVICE_ALIASES)
    .filter(([_, pkgId]) => pkgId === selectedPkg.value)
    .map(([alias]) => alias)
)

const selectedPkg = ref('ums9230')
const selectedDevice = ref('')
const packageInstalled = ref({})

const connWait = ref(300)
const connKick = ref(false)
const connKickto = ref('')
const connBaud = ref('')
const connBlk = ref('')

const form = reactive({})
UNISOC_OPS.forEach((op) => { form[opIndex(op)] = reactive({}) })

const opsGroups = OPS_GROUPS
const folderName = computed(() => folderPath.value ? folderPath.value.split(/[/\\]/).pop() : '')
const allChecked = computed(() => partitions.value.length > 0 && selected.value.length === partitions.value.length)

function opIndex(op) {
  return `${op.group}:${op.name}:${op.label}`
}

function opsByGroup(groupId) {
  return UNISOC_OPS.filter((op) => op.group === groupId)
}

function isSelected(p) {
  return selected.value.some((s) => s.name === p.name)
}

function toggle(p) {
  const idx = selected.value.findIndex((s) => s.name === p.name)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push({ name: p.name, file: p.file })
}

function toggleAll() {
  selected.value = allChecked.value ? [] : partitions.value.map((p) => ({ name: p.name, file: p.file }))
}

function selectAll() {
  selected.value = partitions.value.map((p) => ({ name: p.name, file: p.file }))
}

function deselectAll() {
  selected.value = []
}

function formatSize(bytes) {
  if (!bytes) return '0'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let n = bytes
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++ }
  return `${n.toFixed(i ? 2 : 0)} ${units[i]}`
}

const browseFolder = async () => {
  const p = await showSelectFolder('Select Firmware Folder')
  if (!p) return
  folderPath.value = p
  await loadFolder()
}

async function loadFolder() {
  partitions.value = []
  selected.value = []
  try {
    const data = await sendIpcToMain('unisoc_scan_folder', { path: folderPath.value })
    partitions.value = data.partitions || []
    selected.value = partitions.value.map((p) => ({ name: p.name, file: p.file }))
  } catch (e) {
    partitions.value = []
  }
}

const pickFile = async (op, field) => {
  const filters = op.name === 'repartition' ? [{ name: 'XML', extensions: ['xml'] }] : undefined
  const res = await showSelectDialog({ title: 'Select File', filters })
  if (res.filePaths && res.filePaths.length) {
    form[opIndex(op)][field.key] = res.filePaths[0]
  }
}

function runUnlock() {
  setPending({ type: 'unlock', pkg_id: selectedPkg.value, device: selectedDevice.value || null })
  router.push({ path: '/unisoc/run' })
}

function runFlash() {
  setPending({
    type: 'flash',
    pkg_id: selectedPkg.value,
    device: selectedDevice.value || null,
    folder: folderPath.value,
    partitions: selected.value,
  })
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
  if (op.danger) {
    const ok = await sendIpcToMain('confirm_action', { message: `Run "${op.label}"? This may erase data.` })
    if (!ok) return
  }
  const values = form[opIndex(op)] || {}
  const cliOp = { name: op.name }
  for (const f of op.fields || []) {
    if (f.type === 'file' && !values[f.key]) {
      return alert(`${f.label} is required for ${op.label}`)
    }
    cliOp[f.key] = values[f.key] || undefined
  }
  setPending({
    type: 'cli',
    pkg_id: selectedPkg.value,
    device: selectedDevice.value || null,
    cli: {
      wait_secs: connWait.value,
      kick: connKick.value,
      kickto: connKickto.value || undefined,
      baudrate: connBaud.value || undefined,
      blk_size: connBlk.value || undefined,
      ops: [cliOp],
    },
  })
  router.push({ path: '/unisoc/run' })
}

async function stop() {
  await sendIpcToMain('stop_process')
}

onMounted(async () => {
  packageInstalled.value = await sendIpcToMain('get_packages')
})
</script>

<style scoped>
.unisoc-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.unisoc-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }
.unisoc-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.flash-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.right-col { flex: 1; display: flex; flex-direction: column; gap: 16px; min-width: 0; min-height: 0; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; align-items: center; }
.file-row { flex: 1; min-width: 0; }
.field-label { font-size: 11px; color: var(--text-secondary); white-space: nowrap; }
.checkbox { accent-color: var(--accent-primary); }
.select-input { background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } &.btn-danger { background: #c62828; color: #fff; border-color: #c62828; } }
.output-card { flex: 0 0 auto; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; max-height: 40%; }
.output-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } .header-actions { display: flex; align-items: center; gap: 10px; } .selection-count { color: var(--accent-primary); } .btn-link { background: none; border: none; color: var(--accent-primary); font-size: 12px; cursor: pointer; padding: 0; &:hover:not(:disabled) { text-decoration: underline; } &:disabled { opacity: 0.4; cursor: not-allowed; } } }
.partition-table-wrap { flex: 1; overflow-y: auto; }
.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; th, td { text-align: left; padding: 6px 10px; white-space: nowrap; } thead th { position: sticky; top: 0; background: var(--color-content-background); border-bottom: var(--color-list-header-border-bottom); color: var(--color-font); font-weight: 500; font-size: 12px; } tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); cursor: pointer; &:hover { background: var(--bg-tertiary); } &.selected { background: color-mix(in srgb, var(--accent-primary) 8%, transparent); } } td:first-child { font-weight: 500; } .col-check { width: 28px; text-align: center; input { accent-color: var(--accent-primary); cursor: pointer; } } .empty-row td { color: var(--text-secondary); font-style: italic; text-align: center; } }
.ops-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-height: 0; }
.ops-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } }
.ops-body { flex: 1; overflow-y: auto; padding: 10px 12px; display: flex; flex-direction: column; gap: 14px; }
.ops-group { display: flex; flex-direction: column; gap: 6px; }
.ops-group-title { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); font-weight: 600; }
.ops-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 8px; }
.op-item { display: flex; flex-direction: column; gap: 6px; padding: 8px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-tertiary); }
.op-item .btn { width: 100%; }
</style>