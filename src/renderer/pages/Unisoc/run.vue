<template>
  <div class="unisoc-run-page">
    <div class="run-header">
      <button class="btn" @click="goBack">Back</button>
      <span>{{ jobLabel }}</span>
      <span class="elapsed">Elapsed: {{ elapsed }}s</span>
    </div>

    <div class="run-log-card">
      <div class="terminal-header">
        <span>Output</span>
        <button class="btn-link" @click="clearLog" :disabled="busy">Clear</button>
      </div>
      <div class="terminal-body" ref="terminalRef">
        <div v-for="(line, i) in logLines" :key="i" :class="['log-line', line.type]">
          <span class="line-text">{{ line.text }}</span><template v-if="line.response"><span class="resp"> {{ line.response }}</span></template>
        </div>
        <div v-if="logLines.length === 0" class="log-empty">Waiting for device...</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, sendIpcWithTimeout, rendererOn } from '@renderer/utils/ipc'
import { useUnisocRunStore } from './runStore'
import { createRunLog } from '@renderer/utils/runLog'
import { useOperationStore } from '@renderer/store/operationStore'

const router = useRouter()
const opStore = useOperationStore()
const { pending } = useUnisocRunStore()
const { logLines, queueLog, addLog, clear: clearLog, stop: stopLog } = createRunLog()
const busy = ref(false)
const elapsed = ref(0)
const terminalRef = ref(null)
let elapsedTimer = null
let unlistenProgress = null

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

watch(() => logLines.value.length, () => {
  nextTick(() => { if (terminalRef.value) terminalRef.value.scrollTop = terminalRef.value.scrollHeight })
})

const goBack = async () => {
  if (busy.value) {
    try { await sendIpcToMain('stop_process') } catch {}
  }
  router.push({ path: '/unisoc' })
}

async function runJob() {
  if (!pending.value) {
    addLog('No operation pending.', 'error')
    return
  }
  busy.value = true
  opStore.setBusy(true)
  elapsed.value = 0
  elapsedTimer = setInterval(() => { elapsed.value += 1 }, 1000)
  try {
    const job = pending.value
    if (job.type === 'unlock') {
      await sendIpcWithTimeout('unisoc_unlock', { pkg_id: job.pkg_id, device: job.device }, 120000)
     } else if (job.type === 'flash') {
       await sendIpcWithTimeout('unisoc_flash', {
         pkg_id: job.pkg_id, device: job.device, folder: job.folder, partitions: job.partitions,
       }, 300000)
     } else if (job.type === 'erase_frp') {
       await sendIpcWithTimeout('unisoc_erase_frp', { pkg_id: job.pkg_id, device: job.device }, 120000)
     } else if (job.type === 'dump') {
       await sendIpcWithTimeout('unisoc_dump', { pkg_id: job.pkg_id, device: job.device }, 120000)
     } else if (job.type === 'cli') {
       await sendIpcWithTimeout('unisoc_run_cli', {
        pkg_id: job.pkg_id,
        device: job.device,
        wait_secs: job.cli?.wait_secs,
        kick: job.cli?.kick,
        kickto: job.cli?.kickto,
        baudrate: job.cli?.baudrate,
        blk_size: job.cli?.blk_size,
        ops: job.cli?.ops || [],
      }, 300000)
    }
    queueLog('Done.', 'success')
  } catch (e) {
    queueLog(`Failed: ${e}`, 'error')
  } finally {
    clearInterval(elapsedTimer)
    elapsedTimer = null
    busy.value = false
    opStore.setBusy(false)
  }
}

const onProgress = (_event, data) => {
  if (!data || !data.message) return
  queueLog(data.message, 'info')
}

onMounted(async () => {
  unlistenProgress = await rendererOn('unisoc:progress', onProgress)
  runJob()
})

onBeforeUnmount(() => {
  if (elapsedTimer) clearInterval(elapsedTimer)
  if (unlistenProgress) unlistenProgress()
  stopLog()
  opStore.setBusy(false)
})
</script>

<style scoped>
.unisoc-run-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; animation: pageIn 0.35s ease; }
</style>