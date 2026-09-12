<template>
  <div class="unisoc-page">
    <aside class="sidebar">
      <div class="sidebar-brand">
        <div class="brand-icon">S</div>
        <span class="brand-text">SPD Tools</span>
      </div>

      <div class="sidebar-section">
        <div class="section-title">Device</div>
        <select v-model="selectedPkg" class="sidebar-select">
          <option v-for="pkg in packageList" :key="pkg.id" :value="pkg.id">
            {{ pkg.name }}
          </option>
        </select>
        <select v-model="selectedDevice" class="sidebar-select">
          <option value="">Auto-detect Port</option>
          <option v-for="alias in filteredAliases" :key="alias" :value="alias">{{ alias }}</option>
        </select>
      </div>

      <div class="sidebar-section">
        <div class="section-title">Connection</div>
        <div class="field-row">
          <label>Wait</label>
          <input v-model.number="connWait" type="number" min="0" class="field-input" />
          <span class="field-unit">s</span>
        </div>
        <div class="field-row">
          <label>Baud</label>
          <input v-model="connBaud" class="field-input" placeholder="auto" />
        </div>
        <div class="field-row">
          <label>Blk</label>
          <input v-model="connBlk" class="field-input" placeholder="auto" />
        </div>
        <div class="field-row">
          <label>Kick</label>
          <input v-model="connKick" type="checkbox" class="field-check" />
          <input v-model="connKickto" class="field-input field-input--sm" placeholder="dl_diag" />
        </div>
      </div>

      <div class="sidebar-section">
        <div class="section-title">Operations</div>
        <button class="op-btn op-btn--primary" @click="runUnlock">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          Unlock Bootloader
        </button>
        <button class="op-btn op-btn--accent" @click="runFlash" :disabled="!folderPath || selected.length === 0">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
          Flash Firmware
        </button>
        <button class="op-btn" @click="runEraseFrp">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
          Erase FRP
        </button>
        <button class="op-btn" @click="runDump">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          Dump Partitions
        </button>
        <button class="op-btn op-btn--danger" @click="stop">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
          Stop
        </button>
      </div>

      <div class="sidebar-section sidebar-section--grow">
        <div class="section-title">CLI Tools</div>
        <div class="cli-list">
          <div v-for="grp in opsGroups" :key="grp.id" class="cli-group">
            <div class="cli-group-label">{{ grp.label }}</div>
            <div v-for="op in opsByGroup(grp.id)" :key="op.name + op.label" class="cli-item">
              <div v-for="f in op.fields || []" :key="f.key" class="cli-field">
                <label>{{ f.label }}</label>
                <input v-if="f.type === 'text'" v-model="form[opIndex(op)][f.key]" class="field-input" :placeholder="f.placeholder || ''" />
                <select v-else-if="f.type === 'select'" v-model="form[opIndex(op)][f.key]" class="field-input">
                  <option v-for="o in f.options || []" :key="o" :value="o">{{ o }}</option>
                </select>
                <div v-else class="field-row">
                  <input :value="form[opIndex(op)][f.key] || ''" class="field-input" readonly />
                  <button class="field-btn" @click="pickFile(op, f)">...</button>
                </div>
              </div>
              <button class="cli-run-btn" :class="{ 'cli-run-btn--danger': op.danger }" @click="runOp(op)">
                {{ op.label }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <main class="main-content">
      <div class="main-header">
        <div class="header-left">
          <h2>Firmware Partitions</h2>
          <span v-if="partitions.length" class="sel-badge">{{ selected.length }}/{{ partitions.length }}</span>
        </div>
        <div class="header-right">
          <button class="hdr-btn" :disabled="!partitions.length" @click="selectAll">Select All</button>
          <button class="hdr-btn" :disabled="!partitions.length" @click="deselectAll">Deselect All</button>
        </div>
      </div>

      <div class="firmware-bar">
        <div class="firmware-info">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
          <span>{{ folderName || 'No firmware folder selected' }}</span>
        </div>
        <button class="browse-btn" @click="browseFolder">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          Browse
        </button>
      </div>

      <div class="partition-card">
        <table class="partition-table">
          <thead>
            <tr>
              <th class="col-check">
                <input type="checkbox" :checked="allChecked" @change="toggleAll" />
              </th>
              <th>Partition</th>
              <th>File</th>
              <th>Size</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="partitions.length === 0" class="empty-row">
              <td colspan="4">
                <div class="empty-state">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                  <span>Select a firmware folder to view partitions</span>
                </div>
              </td>
            </tr>
            <tr v-for="(p, i) in partitions" :key="i" :class="{ selected: isSelected(p) }" @click="toggle(p)">
              <td class="col-check" @click.stop>
                <input type="checkbox" :checked="isSelected(p)" @change="toggle(p)" />
              </td>
              <td class="col-name">{{ p.name }}</td>
              <td class="col-file">{{ p.file }}</td>
              <td class="col-size">{{ formatSize(p.size) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </main>
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
.unisoc-page { height: 100%; display: flex; overflow: hidden; }

/* ── Sidebar ─────────────────────────────────── */
.sidebar {
  width: 260px; flex-shrink: 0; display: flex; flex-direction: column;
  background: var(--bg-secondary); border-right: 1px solid var(--border-primary);
  overflow-y: auto; overflow-x: hidden;
}
.sidebar-brand { display: flex; align-items: center; gap: 10px; padding: 14px 16px; border-bottom: 1px solid var(--border-primary); }
.brand-icon { width: 28px; height: 28px; border-radius: 6px; background: linear-gradient(135deg, #7c3aed, #a855f7); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700; }
.brand-text { font-size: 13px; font-weight: 600; color: var(--text-primary); }
.sidebar-section { padding: 12px 14px; border-bottom: 1px solid var(--border-primary); display: flex; flex-direction: column; gap: 6px; }
.sidebar-section--grow { flex: 1; overflow-y: auto; }
.section-title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-secondary); font-weight: 600; margin-bottom: 2px; }
.sidebar-select { width: 100%; background: var(--bg-tertiary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 5px 8px; font-size: 11px; color: var(--text-primary); outline: none; }
.sidebar-select:focus { border-color: var(--accent-primary); }

/* ── Fields ──────────────────────────────────── */
.field-row { display: flex; align-items: center; gap: 6px; }
.field-row label { font-size: 11px; color: var(--text-secondary); white-space: nowrap; min-width: 30px; }
.field-input { flex: 1; min-width: 0; background: var(--bg-tertiary); border: 1px solid var(--border-primary); border-radius: 3px; padding: 4px 6px; font-size: 11px; color: var(--text-primary); outline: none; }
.field-input:focus { border-color: var(--accent-primary); }
.field-input--sm { max-width: 70px; }
.field-unit { font-size: 10px; color: var(--text-secondary); }
.field-check { accent-color: var(--accent-primary); }
.field-btn { padding: 3px 8px; font-size: 10px; border: 1px solid var(--border-primary); border-radius: 3px; background: var(--bg-tertiary); color: var(--text-primary); cursor: pointer; }
.field-btn:hover { border-color: var(--accent-primary); }

/* ── Operation Buttons ───────────────────────── */
.op-btn {
  display: flex; align-items: center; gap: 8px; width: 100%; padding: 7px 10px;
  border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-tertiary);
  color: var(--text-primary); font-size: 11px; cursor: pointer; transition: all 0.15s;
}
.op-btn svg { width: 14px; height: 14px; flex-shrink: 0; }
.op-btn:hover:not(:disabled) { border-color: var(--accent-primary); background: var(--bg-secondary); }
.op-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.op-btn--primary { border-color: #7c3aed33; background: #7c3aed15; color: #a78bfa; }
.op-btn--primary:hover { background: #7c3aed25; border-color: #7c3aed; }
.op-btn--accent { border-color: #22c55e33; background: #22c55e15; color: #4ade80; }
.op-btn--accent:hover { background: #22c55e25; border-color: #22c55e; }
.op-btn--danger { border-color: #ef444433; background: #ef444415; color: #f87171; }
.op-btn--danger:hover { background: #ef444425; border-color: #ef4444; }

/* ── CLI Tools ───────────────────────────────── */
.cli-list { display: flex; flex-direction: column; gap: 10px; }
.cli-group { display: flex; flex-direction: column; gap: 4px; }
.cli-group-label { font-size: 10px; color: var(--accent-primary); font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }
.cli-item { display: flex; flex-direction: column; gap: 4px; padding: 6px 8px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-tertiary); }
.cli-field { display: flex; flex-direction: column; gap: 2px; }
.cli-field label { font-size: 10px; color: var(--text-secondary); }
.cli-run-btn { width: 100%; padding: 5px 8px; border: 1px solid var(--border-primary); border-radius: 3px; background: var(--bg-secondary); color: var(--text-primary); font-size: 11px; cursor: pointer; transition: all 0.15s; }
.cli-run-btn:hover { border-color: var(--accent-primary); }
.cli-run-btn--danger { color: #f87171; border-color: #ef444444; }
.cli-run-btn--danger:hover { background: #ef444415; }

/* ── Main Content ────────────────────────────── */
.main-content { flex: 1; display: flex; flex-direction: column; min-width: 0; padding: 16px; gap: 12px; overflow: hidden; }
.main-header { display: flex; align-items: center; justify-content: space-between; flex-shrink: 0; }
.header-left { display: flex; align-items: center; gap: 10px; }
.header-left h2 { font-size: 15px; margin: 0; color: var(--text-primary); }
.sel-badge { font-size: 11px; padding: 2px 8px; border-radius: 10px; background: var(--accent-primary); color: #fff; font-weight: 600; }
.header-right { display: flex; gap: 6px; }
.hdr-btn { padding: 4px 10px; border: 1px solid var(--border-primary); border-radius: 3px; background: var(--bg-secondary); color: var(--text-secondary); font-size: 11px; cursor: pointer; transition: all 0.15s; }
.hdr-btn:hover:not(:disabled) { color: var(--accent-primary); border-color: var(--accent-primary); }
.hdr-btn:disabled { opacity: 0.35; cursor: not-allowed; }

/* ── Firmware Bar ────────────────────────────── */
.firmware-bar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-secondary); flex-shrink: 0; }
.firmware-info { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-secondary); min-width: 0; }
.firmware-info svg { width: 16px; height: 16px; flex-shrink: 0; color: var(--accent-primary); }
.firmware-info span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.browse-btn { display: flex; align-items: center; gap: 5px; padding: 5px 12px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-tertiary); color: var(--text-primary); font-size: 11px; cursor: pointer; transition: all 0.15s; flex-shrink: 0; }
.browse-btn svg { width: 13px; height: 13px; }
.browse-btn:hover { border-color: var(--accent-primary); }

/* ── Partition Table ─────────────────────────── */
.partition-card { flex: 1; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; display: flex; flex-direction: column; min-height: 0; }
.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.partition-table th, .partition-table td { text-align: left; padding: 7px 12px; white-space: nowrap; }
.partition-table thead th { position: sticky; top: 0; background: var(--color-content-background); border-bottom: 1px solid var(--border-primary); color: var(--text-secondary); font-weight: 500; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; z-index: 1; }
.partition-table tbody { overflow-y: auto; }
.partition-table tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); cursor: pointer; transition: background 0.1s; }
.partition-table tbody tr:hover { background: var(--bg-tertiary); }
.partition-table tbody tr.selected { background: color-mix(in srgb, var(--accent-primary) 8%, transparent); }
.col-check { width: 32px; text-align: center; }
.col-check input { accent-color: var(--accent-primary); cursor: pointer; }
.col-name { font-weight: 500; }
.col-file { color: var(--text-secondary); }
.col-size { color: var(--text-secondary); font-variant-numeric: tabular-nums; }
.empty-row td { padding: 40px 0; }
.empty-state { display: flex; flex-direction: column; align-items: center; gap: 8px; color: var(--text-secondary); }
.empty-state svg { width: 32px; height: 32px; opacity: 0.3; }
.empty-state span { font-size: 12px; }
</style>
