<template>
  <div class="extract-overlay" @click.self="close">
    <div class="extract-modal">
      <div class="modal-header">
        <h3>Extract Partition</h3>
        <button class="modal-close" @click="close">&times;</button>
      </div>

      <div class="modal-body">
        <!-- is fastboot -->
        <div class="fastboot-form" v-if="isTgz">
          <p class="hint">Fastboot archive - the whole file streams, only the requested image is saved.</p>
          <div class="field">
            <label>Image name</label>
            <input v-model="imageName" class="text-input" placeholder="init_boot" />
          </div>
        </div>

        <!-- is OTA: partition list -->
        <template v-else>
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
              :class="['part-row', { selected: selected === p.name }]"
              @click="selected = p.name"
            >
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
        </div>

        <div class="done-msg" v-if="done">
          <span class="status-ok">Done: {{ done }}</span>
        </div>

        <div class="error-msg" v-if="extractError">{{ extractError }}</div>

        <div class="extract-actions">
          <button class="btn btn-primary" @click="start" :disabled="!canStart || extracting">
            {{ isTgz ? 'Extract .img' : 'Extract Selected' }}
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
const selected = ref('')
const imageName = ref('init_boot')
const outputDir = ref('')
const extracting = ref(false)
const progressText = ref('')
const done = ref('')
const extractError = ref('')

const isFrbox = computed(() => props.url.includes('/disk/s/'))
const isTgz = computed(() => !isFrbox.value && (props.url.includes('images_') || props.url.endsWith('.tgz')))

const canStart = computed(() => {
  if (!outputDir.value) return false
  if (isTgz.value) return !!imageName.value.trim()
  return !!selected.value
})

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
  progressText.value = isTgz.value
    ? `Streaming ${imageName.value}.img...`
    : `Extracting ${selected.value}.img...`
  try {
    let result
    if (isFrbox.value) {
      const path = `${outputDir.value}\\${selected.value}`
      result = await sendIpcToMain('frbox_extract_partition', {
        url: props.url,
        pwd: props.pwd || null,
        name: selected.value,
        outputPath: path,
      })
    } else if (isTgz.value) {
      const path = `${outputDir.value}\\${imageName.value.replace(/\.img$/, '')}.img`
      result = await sendIpcToMain('ota_extract_tgz', {
        url: props.url,
        imageName: imageName.value,
        outputPath: path,
      })
    } else {
      const path = `${outputDir.value}\\${selected.value}.img`
      result = await sendIpcToMain('ota_extract_partition', {
        url: props.url,
        partition: selected.value,
        outputPath: path,
      })
    }
    done.value = result || 'Extracted'
  } catch (e) {
    extractError.value = e || 'Extraction failed'
  }
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
    if (isFrbox.value) {
      const entries = await sendIpcToMain('frbox_list_partitions', {
        url: props.url,
        pwd: props.pwd || null,
      })
      partitions.value = (entries || [])
        .filter((e) => e.uncompressed_size > 0)
        .map((e) => ({ name: e.name, size_bytes: e.uncompressed_size }))
    } else {
      partitions.value = await sendIpcToMain('ota_list_partitions', { url: props.url })
    }
    loaded.value = true
  } catch (e) {
    error.value = e || 'Failed to read partitions'
  }
})
</script>

<style scoped>
.extract-overlay { position: fixed; inset: 0; z-index: 1100; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; }
.extract-modal { width: 480px; max-height: 80vh; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 10px; overflow: hidden; display: flex; flex-direction: column; }
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

.part-list { max-height: 240px; overflow-y: auto; border: 1px solid var(--border-primary); border-radius: 6px; }
.part-row { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; font-size: 12px; cursor: pointer; border-bottom: 1px solid var(--border-primary); transition: background 0.15s; }
.part-row:last-child { border-bottom: none; }
.part-row:hover { background: var(--bg-tertiary); }
.part-row.selected { background: rgba(77,131,175,0.15); }
.part-name { font-weight: 500; font-family: monospace; }
.part-size { color: var(--text-secondary); font-size: 11px; }

.part-loading { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 20px; p { font-size: 12px; color: var(--text-secondary); margin: 0; } }
.spinner { width: 22px; height: 22px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
.part-error p { font-size: 12px; color: #f44336; }

.extract-progress { display: flex; align-items: center; gap: 10px; padding: 12px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-tertiary); p { font-size: 12px; margin: 0; } }
.processing-ring { width: 20px; height: 20px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
.done-msg { font-size: 12px; }
.status-ok { color: #4caf50; }
.error-msg { font-size: 12px; color: #f44336; }

.extract-actions { display: flex; gap: 8px; .btn { flex: 1; } }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 8px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 5px 10px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }
@keyframes spin { to { transform: rotate(360deg); } }
</style>