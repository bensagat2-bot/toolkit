<template>
  <div class="mtk-page">
    <div class="mtk-header">
      <h2>MediaTek Tools</h2>
    </div>

    <div class="mtk-content">
      <div class="flash-panel">
        <div class="action-group">
          <h3>Download Agent</h3>
          <div class="row">
            <input :value="daName" class="text-input" placeholder="No DA selected" readonly />
            <button class="btn btn-sm" @click="browseDa">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Scatter Firmware</h3>
          <div class="row">
            <input :value="scatterName" class="text-input" placeholder="No scatter file" readonly />
            <button class="btn btn-sm" @click="browseScatter">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Auth File (Optional)</h3>
          <div class="row">
            <input :value="authName" class="text-input" placeholder="No auth file" readonly />
            <button class="btn btn-sm" @click="browseAuth">Browse</button>
          </div>
        </div>

        <div class="action-group">
          <h3>Connection Options</h3>
          <label class="check"><input type="checkbox" v-model="forceBrom" /> Force BROM</label>
          <label class="check"><input type="checkbox" v-model="usePreloaderFw" /> Use preloader from FW</label>
          <label class="check"><input type="checkbox" v-model="erasePreloader" /> Force BROM Erase preloader</label>
        </div>

        <button class="btn btn-primary start-btn" @click="startFlash" :disabled="!scatterPath || selected.length === 0">
          Start Flash
        </button>
      </div>

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
                <th>Start Address</th>
                <th>Size</th>
                <th>Filename</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="partitions.length === 0" class="empty-row">
                <td colspan="5">No scatter file selected.</td>
              </tr>
              <tr v-for="(p, i) in partitions" :key="i" :class="{ selected: isSelected(p) }" @click="toggle(p)">
                <td class="col-check" @click.stop><input type="checkbox" :checked="isSelected(p)" @change="toggle(p)" /></td>
                <td>{{ p.name }}</td>
                <td>{{ p.address }}</td>
                <td>{{ formatSize(p.size) }}</td>
                <td>{{ p.filename }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, showSelectDialog } from '@renderer/utils/ipc'
import { setPending } from '@renderer/store/mtkFlashStore'

const router = useRouter()
const daPath = ref('')
const scatterPath = ref('')
const authPath = ref('')
const forceBrom = ref(false)
const usePreloaderFw = ref(false)
const erasePreloader = ref(false)
const partitions = ref([])
const selected = ref([])

const daName = computed(() => daPath.value ? daPath.value.split(/[/\\]/).pop() : '')
const scatterName = computed(() => scatterPath.value ? scatterPath.value.split(/[/\\]/).pop() : '')
const authName = computed(() => authPath.value ? authPath.value.split(/[/\\]/).pop() : '')
const allChecked = computed(() => partitions.value.length > 0 && selected.value.length === partitions.value.length)

const pickFile = async (opts) => (await showSelectDialog(opts)).filePaths[0] || ''

const browseDa = async () => {
  const p = await pickFile({ title: 'Select Download Agent', filters: [{ name: 'DA Files', extensions: ['bin'] }] })
  if (p) daPath.value = p
}

const browseScatter = async () => {
  const p = await pickFile({ title: 'Select Scatter File', filters: [{ name: 'Scatter Files', extensions: ['txt', 'xml'] }] })
  if (p) { scatterPath.value = p; await loadScatter() }
}

const browseAuth = async () => {
  const p = await pickFile({ title: 'Select Auth File', filters: [{ name: 'Auth Files', extensions: ['bin', 'auth'] }] })
  if (p) authPath.value = p
}

async function loadScatter() {
  partitions.value = []
  selected.value = []
  try {
    const data = await sendIpcToMain('mtk_load_scatter', { path: scatterPath.value })
    const raw = data.partitions || []
    // Deduplicate by name: keep first occurrence
    const seen = new Set()
    const deduped = []
    for (const p of raw) {
      if (!seen.has(p.name)) {
        seen.add(p.name)
        deduped.push(p)
      }
    }
    partitions.value = deduped
    selected.value = deduped.filter((p) => p.download).map((p) => p.name)
  } catch (e) {
    partitions.value = []
  }
}

function isSelected(p) {
  return selected.value.includes(p.name)
}

function toggle(p) {
  const idx = selected.value.indexOf(p.name)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(p.name)
}

function toggleAll() {
  selected.value = allChecked.value ? [] : partitions.value.map((p) => p.name)
}

function selectAll() {
  selected.value = partitions.value.map((p) => p.name)
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

function startFlash() {
  setPending({
    da_path: daPath.value || null,
    scatter_path: scatterPath.value,
    auth_path: authPath.value || null,
    force_brom: forceBrom.value,
    use_preloader_from_fw: usePreloaderFw.value,
    force_brom_erase_preloader: erasePreloader.value,
    partitions: selected.value,
  })
  router.push({ path: '/mediatek/flash' })
}
</script>

<style scoped>
.mtk-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.mtk-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }
.mtk-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.flash-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; &:focus { border-color: var(--accent-primary); } }
.check { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-primary); cursor: pointer; input { accent-color: var(--accent-primary); } }
.start-btn { width: 100%; padding: 10px; font-size: 13px; font-weight: 600; margin-top: auto; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }
.output-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.output-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } .header-actions { display: flex; align-items: center; gap: 10px; } .selection-count { color: var(--accent-primary); } .btn-link { background: none; border: none; color: var(--accent-primary); font-size: 12px; cursor: pointer; padding: 0; &:hover:not(:disabled) { text-decoration: underline; } &:disabled { opacity: 0.4; cursor: not-allowed; } } }
.partition-table-wrap { flex: 1; overflow-y: auto; }
.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; th, td { text-align: left; padding: 6px 10px; white-space: nowrap; } thead th { position: sticky; top: 0; background: var(--color-content-background); border-bottom: var(--color-list-header-border-bottom); color: var(--color-font); font-weight: 500; font-size: 12px; } tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); cursor: pointer; &:hover { background: var(--bg-tertiary); } &.selected { background: color-mix(in srgb, var(--accent-primary) 8%, transparent); } } td:first-child { font-weight: 500; } .col-check { width: 28px; text-align: center; input { accent-color: var(--accent-primary); cursor: pointer; } } .empty-row td { color: var(--text-secondary); font-style: italic; text-align: center; } }
</style>