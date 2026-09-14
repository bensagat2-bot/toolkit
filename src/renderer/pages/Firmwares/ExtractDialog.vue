<template>
  <div class="extract-overlay" @click.self="close">
    <div class="extract-modal">
      <div class="modal-header">
        <h3>Extract Partitions</h3>
        <button class="modal-close" @click="close">&times;</button>
      </div>

      <div class="modal-body">
        <div class="fastboot-form" v-if="isTgz">
          <p class="hint">Fastboot .tgz archive - the whole file streams, only the requested image is saved.</p>
          <div class="field">
            <label>Image name</label>
            <input v-model="imageName" class="text-input" placeholder="init_boot" />
          </div>
        </div>

        <template v-if="showPartList">
          <div class="part-header" v-if="loaded">
            <span class="selection-count">{{ selected.length }} / {{ partitions.length }} selected</span>
            <div class="part-actions">
              <button class="btn-link" :disabled="!partitions.length" @click="selectAll">Select All</button>
              <button class="btn-link" :disabled="!partitions.length" @click="deselectAll">Deselect All</button>
            </div>
          </div>

          <div class="part-loading" v-if="!loaded && !error">
            <div class="spinner"></div>
            <p>Reading partition manifest...</p>
          </div>

          <div class="part-error" v-if="error">
            <p>{{ error }}</p>
          </div>

          <div class="part-list scroll" v-if="loaded">
            <div
              v-for="p in partitions"
              :key="p.name"
              :class="['part-row', { selected: isSelected(p) }]"
              @click="toggle(p)"
            >
              <span class="col-check" @click.stop>
                <input type="checkbox" :checked="isSelected(p)" @change="toggle(p)" />
              </span>
              <span class="part-name">{{ p.name }}</span>
              <span class="part-size">{{ formatMb(p.size_bytes) }}</span>
            </div>
          </div>
        </template>

        <div class="field">
          <label>Output folder</label>
          <div class="row">
            <input v-model="outputDir" class="text-input" placeholder="Choose output folder" readonly />
            <button class="btn btn-sm" @click="browseFolder">Browse</button>
          </div>
        </div>

        <div class="extract-progress" v-if="extracting">
          <div class="processing-ring"></div>
          <p>{{ progressText }}</p>
          <div class="progress-bar-wrap" v-if="showPartList && selected.length > 1">
            <div class="progress-bar" :style="{ width: extractProgress + '%' }"></div>
          </div>
        </div>

        <div class="done-msg" v-if="done">
          <span class="status-ok">Done: {{ done }}</span>
        </div>

        <div class="error-msg" v-if="extractError">{{ extractError }}</div>

        <div class="extract-actions">
          <button class="btn btn-primary" @click="start" :disabled="!canStart || extracting">
            {{ isTgz ? 'Extract .img' : `Extract (${selected.length})` }}
          </button>
          <button class="btn" @click="close" :disabled="extracting">Close</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { sendIpcToMain, showSelectFolder } from '@renderer/utils/ipc'

const props = defineProps({
  url: { type: String, required: true },
  pwd: { type: String, default: '' },
})

const emit = defineEmits(['close'])

const partitions = ref([])
const loaded = ref(false)
const error = ref('')
const selectedNames = ref([])
const imageName = ref('init_boot')
const outputDir = ref('')
const extracting = ref(false)
const progressText = ref('')
const extractProgress = ref(0)
const done = ref('')
const extractError = ref('')

const isFrbox = computed(() => props.url.includes('/disk/s/'))
const isTgz = computed(() => !isFrbox.value && props.url.endsWith('.tgz'))
const isFastbootZip = computed(() => !isFrbox.value && !isTgz.value && (props.url.includes('images_') || props.url.includes('fastboot')))
const showPartList = computed(() => !isTgz.value)

const canStart = computed(() => {
  if (!outputDir.value) return false
  if (isTgz.value) return !!imageName.value.trim()
  return selectedNames.value.length > 0
})

const allChecked = computed(() => partitions.value.length > 0 && selectedNames.value.length === partitions.value.length)

function isSelected(p) {
  return selectedNames.value.includes(p.name)
}

function toggle(p) {
  const idx = selectedNames.value.indexOf(p.name)
  if (idx >= 0) selectedNames.value.splice(idx, 1)
  else selectedNames.value.push(p.name)
}

function toggleAll() {
  selectedNames.value = allChecked.value ? [] : partitions.value.map((p) => p.name)
}

function selectAll() {
  selectedNames.value = partitions.value.map((p) => p.name)
}

function deselectAll() {
  selectedNames.value = []
}

function formatMb(bytes) {
  const mb = (bytes || 0) / 1024 / 1024
  return `${mb >= 1024 ? (mb / 1024).toFixed(2) + ' GB' : mb.toFixed(1) + ' MB'}`
}

async function browseFolder() {
  const dir = await showSelectFolder('Choose output folder')
  if (dir) outputDir.value = dir
}

async function start() {
  done.value = ''
  extractError.value = ''
  extracting.value = true
  extractProgress.value = 0

  const names = isTgz.value ? [imageName.value] : selectedNames.value
  let total = names.length
  let completed = 0

  for (const name of names) {
    const isLast = names.length > 1
    progressText.value = isLast
      ? `Extracting ${name}.img... (${completed + 1}/${total})`
      : `Extracting ${name}.img...`
    extractProgress.value = total > 1 ? Math.round((completed / total) * 100) : 0
    try {
      let result
      const stem = name.replace(/\.img$/i, '')
      if (isFrbox.value) {
        const path = `${outputDir.value}\\${stem}`
        result = await sendIpcToMain('frbox_extract_partition', {
          url: props.url,
          pwd: props.pwd || null,
          name: name,
          outputPath: path,
        })
      } else if (isFastbootZip.value) {
        const path = `${outputDir.value}\\${stem}.img`
        result = await sendIpcToMain('ota_extract_fastboot_image', {
          url: props.url,
          imageName: name,
          outputPath: path,
        })
      } else if (isTgz.value) {
        const path = `${outputDir.value}\\${stem}.img`
        result = await sendIpcToMain('ota_extract_tgz', {
          url: props.url,
          imageName: name,
          outputPath: path,
        })
      } else {
        const path = `${outputDir.value}\\${stem}.img`
        result = await sendIpcToMain('ota_extract_partition', {
          url: props.url,
          partition: name,
          outputPath: path,
        })
      }
      completed++
      if (isLast || completed === total) {
        done.value = total > 1 ? `${completed}/${total} partitions extracted` : (result || 'Extracted')
      }
    } catch (e) {
      extractError.value = `${name}: ${e || 'Extraction failed'}`
      extracting.value = false
      return
    }
  }
  extractProgress.value = 100
  extracting.value = false
}

function close() {
  if (extracting.value) return
  emit('close')
}

onMounted(async () => {
  if (isTgz.value) {
    loaded.value = true
    return
  }
  try {
    let list
    if (isFrbox.value) {
      const entries = await sendIpcToMain('frbox_list_partitions', {
        url: props.url,
        pwd: props.pwd || null,
      })
      list = (entries || [])
        .filter((e) => e.uncompressed_size > 0)
        .map((e) => ({ name: e.name, size_bytes: e.uncompressed_size }))
    } else if (isFastbootZip.value) {
      const images = await sendIpcToMain('ota_list_fastboot_images', { url: props.url })
      list = (images || []).map((e) => ({ name: e.name, size_bytes: e.size_bytes }))
    } else {
      list = await sendIpcToMain('ota_list_partitions', { url: props.url })
    }
    partitions.value = list
    selectedNames.value = list.map((p) => p.name)
    loaded.value = true
  } catch (e) {
    error.value = e || 'Failed to read partitions'
  }
})
</script>

<style scoped>
.extract-overlay { position: fixed; inset: 0; z-index: 1100; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; }
.extract-modal { width: 520px; max-height: 85vh; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 10px; overflow: hidden; display: flex; flex-direction: column; }
.modal-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border-primary); }
.modal-header h3 { margin: 0; font-size: 14px; }
.modal-close { background: none; border: none; color: var(--text-secondary); font-size: 20px; cursor: pointer; padding: 0 4px; }
.modal-close:hover { color: var(--text-primary); }
.modal-body { flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 14px; }

.hint { font-size: 12px; color: var(--text-secondary); margin: 0; }
.field { display: flex; flex-direction: column; gap: 4px; label { font-size: 11px; color: var(--text-secondary); } }
.text-input { width: 100%; background: var(--bg-primary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 8px 10px; font-size: 12px; color: var(--text-primary); outline: none; }
.text-input:focus { border-color: var(--accent-primary); }
.row { display: flex; gap: 6px; .text-input { flex: 1; } }

.part-header { display: flex; align-items: center; justify-content: space-between; padding: 4px 0; }
.selection-count { font-size: 11px; color: var(--accent-primary); font-weight: 600; }
.part-actions { display: flex; gap: 10px; }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 11px; cursor: pointer; padding: 0; }
.btn-link:hover:not(:disabled) { text-decoration: underline; }
.btn-link:disabled { opacity: 0.4; cursor: not-allowed; }

.part-list { max-height: 280px; overflow-y: auto; border: 1px solid var(--border-primary); border-radius: 6px; }
.part-row { display: flex; align-items: center; gap: 8px; padding: 8px 10px; font-size: 12px; cursor: pointer; border-bottom: 1px solid var(--border-primary); transition: background 0.15s; }
.part-row:last-child { border-bottom: none; }
.part-row:hover { background: var(--bg-tertiary); }
.part-row.selected { background: color-mix(in srgb, var(--accent-primary) 8%, transparent); }
.col-check { flex: none; width: 20px; display: flex; align-items: center; input { accent-color: var(--accent-primary); cursor: pointer; } }
.part-name { flex: 1; font-weight: 500; font-family: monospace; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.part-size { flex: none; color: var(--text-secondary); font-size: 11px; white-space: nowrap; }

.part-loading { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 20px; p { font-size: 12px; color: var(--text-secondary); margin: 0; } }
.spinner { width: 22px; height: 22px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
.part-error p { font-size: 12px; color: #f44336; }

.extract-progress { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 12px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-tertiary); p { font-size: 12px; margin: 0; } }
.progress-bar-wrap { width: 100%; height: 4px; background: var(--border-primary); border-radius: 2px; overflow: hidden; }
.progress-bar { height: 100%; background: var(--accent-primary); border-radius: 2px; transition: width 0.3s ease; }
.processing-ring { width: 20px; height: 20px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
.done-msg { font-size: 12px; }
.status-ok { color: #4caf50; }
.error-msg { font-size: 12px; color: #f44336; }

.extract-actions { display: flex; gap: 8px; .btn { flex: 1; } }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 8px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 5px 10px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }
@keyframes spin { to { transform: rotate(360deg); } }
</style>