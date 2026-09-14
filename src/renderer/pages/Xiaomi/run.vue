<template>
  <div class="xiaomi-run-page">
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
        <div v-if="logLines.length === 0" class="log-empty">Preparing operation...</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { sendIpcToMain, sendIpcWithTimeout, rendererOn } from '@renderer/utils/ipc'
import { useXiaomiRunStore } from './runStore'
import { createRunLog } from '@renderer/utils/runLog'

const router = useRouter()
const { pending } = useXiaomiRunStore()
const { logLines, queueLog, addLog, clear: clearLog, stop: stopLog } = createRunLog()
const busy = ref(false)
const elapsed = ref(0)
const terminalRef = ref(null)
let elapsedTimer = null

const jobLabel = computed(() => {
  if (!pending.value) return ''
  const labels = {
    detect: 'Device Detection',
    unlock: 'Mi Unlock',
    flash: 'ROM Flash',
    fastboot_cmd: 'Fastboot Command',
    adb_cmd: 'ADB Command',
    reboot_bootloader: 'Reboot to Bootloader',
    rom_scan: 'ROM Scan',
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
  router.push({ path: '/xiaomi' })
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
    if (job.type === 'detect') {
      const info = await sendIpcWithTimeout('xiaomi_detect_device', {}, 30000)
      addLog(`Mode: ${info.mode}`, 'success')
      addLog(`Model: ${info.model}`)
      addLog(`Brand: ${info.brand}`)
      addLog(`Android: ${info.android}`)
      addLog(`Bootloader: ${info.bootloader}`, info.bootloader === 'Unlocked' ? 'success' : 'info')
      addLog(`Root: ${info.root_status}`)
    } else if (job.type === 'unlock') {
      await sendIpcWithTimeout('xiaomi_fastboot_oem_unlock', {}, 60000)
      queueLog('Unlock command sent.', 'success')
    } else if (job.type === 'reboot_bootloader') {
      await sendIpcWithTimeout('xiaomi_reboot_bootloader', {}, 30000)
      queueLog('Rebooting to bootloader...', 'success')
    } else if (job.type === 'fastboot_cmd') {
      const out = await sendIpcWithTimeout('xiaomi_run_fastboot', { args: job.args }, 60000)
      out.split('\n').forEach(l => { if (l.trim()) queueLog(l.trim()) })
    } else if (job.type === 'adb_cmd') {
      const out = await sendIpcWithTimeout('xiaomi_run_adb', { args: job.args }, 60000)
      out.split('\n').forEach(l => { if (l.trim()) queueLog(l.trim()) })
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

onMounted(() => { runJob() })

onBeforeUnmount(() => {
  if (elapsedTimer) clearInterval(elapsedTimer)
  stopLog()
})
</script>

<style scoped>
.xiaomi-run-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; animation: pageIn 0.35s ease; }
</style>
