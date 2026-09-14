<template>
  <div class="backup-page">
    <div class="backup-header">
      <h2>Backup</h2>
      <span class="tagline">Partition backups saved to Downloads/v1per-user-backups</span>
    </div>

    <div v-if="errorMsg || toastMsg" class="toast-bar" :class="toastType">
      <span>{{ errorMsg || toastMsg }}</span>
      <button class="btn-link" @click="errorMsg = ''; toastMsg = ''">&times;</button>
    </div>

    <div v-if="!loaded" class="backup-loading">Loading backups...</div>

    <div v-else-if="cards.length === 0" class="backup-empty">
      <div class="empty-card">
        <h3>No backup firmwares is here yet</h3>
        <p>Start your first partition backup to keep a full copy of the firmware.</p>
        <button class="btn btn-primary" @click="startBackup('Backup 1')">Start Backup</button>
      </div>
    </div>

    <div v-else class="backup-grid">
      <div
        v-for="card in cards"
        :key="card.name"
        class="backup-card"
      >
        <div class="card-head">
          <input
            v-if="editingName === card.name"
            v-model="nameDraft"
            class="card-name-input"
            @keydown.enter="commitRename(card.name)"
            @blur="commitRename(card.name)"
          />
          <h3 v-else class="card-name" @dblclick="startRename(card.name)">{{ card.name }}</h3>
          <span class="card-edit" title="Rename" @click="startRename(card.name)">✎</span>
        </div>
        <div class="card-meta">
          <span class="card-model">{{ card.model || 'Unknown device' }}</span>
          <span class="card-date">{{ card.date }}</span>
        </div>
        <div class="card-stats">
          <span class="stat">{{ card.partitions }} partitions</span>
        </div>

        <div class="card-actions">
          <button class="btn btn-sm" @click="openCard(card)">View Partitions</button>
          <button class="btn btn-sm btn-primary" @click="startBackup(card.name)">Backup Again</button>
          <button class="btn btn-sm btn-danger" @click="deleteCard(card)">Delete</button>
        </div>
      </div>

      <div
        v-if="cards.length < 10"
        class="backup-card placeholder"
      >
        <h3 class="card-name">+ Add</h3>
        <p class="placeholder-text">New backup</p>
        <button class="btn btn-primary btn-sm" @click="startBackup(`Backup ${cards.length + 1}`)">Start Backup</button>
      </div>
    </div>

    <!-- Detail modal: partition list with select / download -->
    <div v-if="activeCard" class="modal-mask" @click.self="activeCard = null">
      <div class="modal">
        <div class="modal-head">
          <h3>{{ activeCard.name }}</h3>
          <button class="btn-link" @click="activeCard = null">Close</button>
        </div>
        <div class="modal-toolbar">
          <button class="btn btn-sm" @click="toggleAll">{{ allSelected ? 'Deselect All' : 'Select All' }}</button>
          <button class="btn btn-sm btn-primary" :disabled="!selected.length" @click="downloadSelected">
            Download Selected ({{ selected.length }})
          </button>
          <span class="modal-hint">saved to Downloads/v1per-user-backups/{{ activeCard.name }}</span>
        </div>
        <div class="modal-list">
          <label v-for="f in activeCard.files" :key="f.name" class="part-row">
            <input v-model="selectedSet" type="checkbox" :value="f.name" />
            <span class="part-name">{{ f.name }}</span>
            <span class="part-size">{{ fmtSize(f.size) }}</span>
          </label>
          <div v-if="!activeCard.files.length" class="modal-empty">No partitions in this backup.</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, showSelectFolder } from '@renderer/utils/ipc'
import { setPending } from './runStore'

const router = useRouter()
const cards = ref([])
const loaded = ref(false)
const activeCard = ref(null)
const selectedSet = ref([])
const editingName = ref(null)
const nameDraft = ref('')
const toastMsg = ref('')
const toastType = ref('')
const errorMsg = ref('')

const selected = computed(() => [...selectedSet.value])
const allSelected = computed(() => activeCard.value?.files.length > 0 && selectedSet.value.length === activeCard.value.files.length)

const fmtSize = (bytes) => {
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`
  if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${bytes} B`
}

const load = async () => {
  loaded.value = false
  try {
    cards.value = await sendIpcToMain('backup_list')
  } catch (e) {
    console.error('Failed to load backups:', e)
    errorMsg.value = 'Failed to load backups.'
    cards.value = []
  } finally {
    loaded.value = true
  }
}

const startBackup = (name) => {
  setPending({ name })
  router.push({ path: '/backup/run' })
}

const openCard = (card) => {
  activeCard.value = card
  selectedSet.value = []
}

const toggleAll = () => {
  if (allSelected.value) selectedSet.value = []
  else selectedSet.value = activeCard.value.files.map((f) => f.name)
}

const downloadSelected = async () => {
  const folder = await showSelectFolder('Select destination folder')
  if (!folder) return
  try {
    const n = await sendIpcToMain('backup_download_selected', {
      name: activeCard.value.name,
      selected: selected.value,
      dest: folder,
    })
    toastMsg.value = `Downloaded ${n} partition(s) to ${folder}`
    toastType.value = 'success'
  } catch (e) {
    errorMsg.value = `Download failed: ${e}`
    toastType.value = 'error'
  }
}

const startRename = (name) => {
  editingName.value = name
  nameDraft.value = name
}

const commitRename = async (oldName) => {
  const newName = nameDraft.value?.trim()
  if (editingName.value !== oldName) return
  editingName.value = null
  if (!newName || newName === oldName) return
  try {
    await sendIpcToMain('backup_rename', { old: oldName, new: newName })
    await load()
  } catch (e) {
    errorMsg.value = `Rename failed: ${e}`
    toastType.value = 'error'
  }
}

const deleteCard = async (card) => {
  try {
    const ok = await sendIpcToMain('confirm_action', { message: `Delete backup "${card.name}"? This removes its folder in Downloads.` })
    if (!ok) return
    await sendIpcToMain('backup_delete', { name: card.name })
    await load()
  } catch (e) {
    errorMsg.value = `Delete failed: ${e}`
    toastType.value = 'error'
  }
}

onMounted(load)
</script>

<style scoped>
.backup-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.backup-header { display: flex; align-items: center; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } .tagline { font-size: 12px; color: var(--text-secondary); } }
.backup-grid { flex: 1; display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 16px; align-content: start; overflow-y: auto; }
.backup-card { display: flex; flex-direction: column; gap: 8px; padding: 14px; border: 1px solid var(--border-primary); border-radius: 10px; background: var(--bg-tertiary); animation: cardIn 0.4s ease; }
.backup-card.placeholder { border-style: dashed; align-items: center; justify-content: center; text-align: center; min-height: 150px; }
.backup-empty { flex: 1; display: flex; align-items: center; justify-content: center; }
.empty-card { text-align: center; padding: 40px; border: 1px dashed var(--border-primary); border-radius: 12px; background: var(--bg-tertiary); display: flex; flex-direction: column; gap: 10px; align-items: center; h3 { font-size: 16px; margin: 0; } }
.card-head { display: flex; align-items: center; gap: 6px; }
.card-name { margin: 0; font-size: 16px; flex: 1; cursor: pointer; }
.card-name-input { flex: 1; font-size: 14px; background: var(--bg-secondary); border: 1px solid var(--accent-primary); border-radius: 4px; padding: 2px 6px; color: var(--text-primary); outline: none; }
.card-edit { cursor: pointer; color: var(--text-secondary); font-size: 13px; &:hover { color: var(--accent-primary); } }
.card-meta { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-secondary); }
.card-stats { font-size: 12px; font-weight: 600; color: var(--accent-primary); }
.card-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: auto; }
.placeholder-text { color: var(--text-secondary); font-size: 12px; margin: 0; }
.backup-loading { color: var(--text-secondary); font-size: 13px; }

.modal-mask { position: fixed; inset: 0; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal { width: min(560px, 90vw); max-height: 80vh; display: flex; flex-direction: column; background: var(--bg-primary); border: 1px solid var(--border-primary); border-radius: 12px; overflow: hidden; }
.modal-head { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; border-bottom: var(--color-list-header-border-bottom); h3 { margin: 0; font-size: 16px; } }
.modal-toolbar { display: flex; align-items: center; gap: 8px; padding: 10px 16px; border-bottom: var(--color-list-header-border-bottom); flex-wrap: wrap; }
.modal-hint { font-size: 11px; color: var(--text-secondary); }
.modal-list { overflow-y: auto; padding: 8px 16px; }
.part-row { display: flex; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; cursor: pointer; }
.part-name { flex: 1; font-family: 'Cascadia Code', monospace; }
.part-size { color: var(--text-secondary); font-size: 12px; }
.modal-empty { color: var(--text-secondary); padding: 20px 0; text-align: center; }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 4px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } &.btn-danger { color: var(--color-status-error); } }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 12px; cursor: pointer; padding: 0; &:hover { text-decoration: underline; } }
.toast-bar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-radius: 6px; font-size: 12px; margin-bottom: 8px; }
.toast-bar.success { background: rgba(76, 175, 80, 0.15); color: var(--color-status-success); }
.toast-bar.error { background: rgba(244, 67, 54, 0.15); color: var(--color-status-error); }
@keyframes cardIn { from { opacity: 0; transform: scale(0.98); } to { opacity: 1; transform: scale(1); } }
</style>