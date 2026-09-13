<template>
  <div class="unisoc-run-page">
    <div class="run-header">
      <button class="btn" @click="goBack" :disabled="busy">Back</button>
      <span>{{ jobLabel }}</span>
      <span class="elapsed">Elapsed: {{ elapsed }}s</span>
    </div>

    <div class="run-log-card">
      <div class="terminal-header">
        <span>Output</span>
        <button class="btn-link" @click="clearLog" :disabled="busy">Clear</button>
      </div>
      <div class="terminal-body" ref="terminalRef">
        <div v-for="(line, i) in logLines" :key="i" :class="['log-line', line.type]">{{ line.text }}</div>
        <div v-if="logLines.length === 0" class="log-empty">Waiting for device...</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, rendererOn } from '@renderer/utils/ipc'
import { useUnisocRunStore } from './runStore'

const router = useRouter()
const { pending } = useUnisocRunStore()
const logLines = ref([])
const busy = ref(false)
const elapsed = ref(0)
const terminalRef = ref(null)
let elapsedTimer = null
let logQueue = []
let logTimer = null

const jobLabel = computed(() => {
  if (!pending.value) return ''
  const labels = {
    unlock: 'Unlock Bootloader',
    flash: 'Flash Firmware',
    erase_frp: 'Erase FRP',
    dump: 'Dump Partitions',
    cli: 'Unisoc CLI',
  }
  return labels[pending.value.type] || ''
})

function addLog(text, type = 'info') {
  logLines.value.push({ text, type })
}

function queueLog(text, type = 'info') {
  logQueue.push({ text, type })
  if (!logTimer) flushNext()
}

function flushNext() {
  if (!logQueue.length) { logTimer = null; return }
  const item = logQueue.shift()
  addLog(item.text, item.type)
  logTimer = setTimeout(flushNext, 500)
}

watch(() => logLines.value.length, () => {
  nextTick(() => { if (terminalRef.value) terminalRef.value.scrollTop = terminalRef.value.scrollHeight })
})

const clearLog = () => { logLines.value = [] }

const goBack = () => {
  if (busy.value) return
  router.push({ path: '/unisoc' })
}

async function runJob() {
  if (!pending.value) {
    addLog('No operation pending.', 'error')
    return
  }
  busy.value = true
  elapsed.value = 0
  elapsedTimer = setInterval(() => { elapsed.value += 1 }, 1000)
  try {
    const job = pending.value
    if (job.type === 'unlock') {
      await sendIpcToMain('unisoc_unlock', { pkg_id: job.pkg_id, device: job.device })
    } else if (job.type === 'flash') {
      await sendIpcToMain('unisoc_flash', {
        pkg_id: job.pkg_id, device: job.device, folder: job.folder, partitions: job.partitions,
      })
    } else if (job.type === 'erase_frp') {
      await sendIpcToMain('unisoc_erase_frp', { pkg_id: job.pkg_id, device: job.device })
    } else if (job.type === 'dump') {
      await sendIpcToMain('unisoc_dump', { pkg_id: job.pkg_id, device: job.device })
    } else if (job.type === 'cli') {
      await sendIpcToMain('unisoc_run_cli', {
        pkg_id: job.pkg_id,
        device: job.device,
        wait_secs: job.cli?.wait_secs,
        kick: job.cli?.kick,
        kickto: job.cli?.kickto,
        baudrate: job.cli?.baudrate,
        blk_size: job.cli?.blk_size,
        ops: job.cli?.ops || [],
      })
    }
    queueLog('Done.', 'success')
  } catch (e) {
    queueLog(`Failed: ${e}`, 'error')
  } finally {
    clearInterval(elapsedTimer)
    elapsedTimer = null
    busy.value = false
  }
}

const onProgress = (_event, data) => {
  if (!data || !data.message) return
  queueLog(data.message, String(data.message).endsWith('OK') ? 'success' : 'info')
}

onMounted(async () => {
  await rendererOn('unisoc:progress', onProgress)
  runJob()
})

onBeforeUnmount(() => {
  if (elapsedTimer) clearInterval(elapsedTimer)
  if (logTimer) clearTimeout(logTimer)
})
</script>

<style scoped>
.unisoc-run-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; animation: pageIn 0.35s ease; }
.run-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); font-size: 12px; color: var(--text-secondary); span.elapsed { color: var(--accent-primary); font-weight: 600; } }
.run-log-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-height: 0; animation: cardIn 0.45s ease; }
.terminal-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } }
.terminal-body { flex: 1; overflow-y: auto; padding: 8px 12px; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.5; }
.log-line { white-space: pre-wrap; word-break: break-all; &.info { color: var(--text-primary); } &.success { color: #4caf50; } &.warn { color: #ff9800; } &.error { color: #f44336; } }
.log-empty { color: var(--text-secondary); font-style: italic; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } }
.btn-link { background: none; border: none; color: var(--accent-primary); font-size: 12px; cursor: pointer; padding: 0; &:hover:not(:disabled) { text-decoration: underline; } &:disabled { opacity: 0.4; cursor: not-allowed; } }
@keyframes pageIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
@keyframes cardIn { from { opacity: 0; transform: scale(0.98); } to { opacity: 1; transform: scale(1); } }
</style>