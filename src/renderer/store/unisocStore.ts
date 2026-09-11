import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UnisocPackage, UnisocPackages, LogEntry } from '@renderer/types'
import { PACKAGES, DEVICE_ALIASES } from '@renderer/pages/Unisoc/unisoc-data'
import { sendIpcToMain } from '@renderer/utils/ipc'

export const useUnisocStore = defineStore('unisoc', () => {
  const selectedPkg = ref('ums9230')
  const selectedDevice = ref('')
  const packageInstalled = ref<UnisocPackages>({})
  const isRunning = ref(false)
  const logLines = ref<LogEntry[]>([])
  const flashPartition = ref('')
  const flashImage = ref('')

  const packageList = Object.values(PACKAGES)
  const filteredAliases = computed(() =>
    Object.entries(DEVICE_ALIASES)
      .filter(([_, pkgId]) => pkgId === selectedPkg.value)
      .map(([alias]) => alias)
  )

  function appendLog(text: string, type: LogEntry['type'] = 'info') {
    logLines.value.push({ text, type, timestamp: Date.now() })
  }

  function clearLog() { logLines.value = [] }

  async function loadPackages() {
    packageInstalled.value = await sendIpcToMain('get_packages')
  }

  async function unlock() {
    isRunning.value = true; clearLog(); appendLog('Starting unlock...', 'system')
    try {
      await sendIpcToMain('unlock_bootloader', { pkgId: selectedPkg.value, device: selectedDevice.value || null })
      appendLog('Unlock completed.', 'success')
    } catch (e: any) { appendLog(`Error: ${e}`, 'error') }
    isRunning.value = false
  }

  async function dump() {
    isRunning.value = true; clearLog(); appendLog('Starting dump...', 'system')
    try {
      await sendIpcToMain('dump_partitions', { pkgId: selectedPkg.value, device: selectedDevice.value || null })
      appendLog('Dump completed.', 'success')
    } catch (e: any) { appendLog(`Error: ${e}`, 'error') }
    isRunning.value = false
  }

  async function flash() {
    isRunning.value = true; clearLog(); appendLog('Flashing...', 'system')
    try {
      await sendIpcToMain('flash_partition', {
        pkgId: selectedPkg.value, device: selectedDevice.value || null,
        partition: flashPartition.value, image: flashImage.value
      })
      appendLog('Flash completed.', 'success')
    } catch (e: any) { appendLog(`Error: ${e}`, 'error') }
    isRunning.value = false
  }

  async function erase() {
    isRunning.value = true; clearLog(); appendLog('Erasing...', 'system')
    try {
      await sendIpcToMain('erase_partition', {
        pkgId: selectedPkg.value, device: selectedDevice.value || null,
        partition: flashPartition.value
      })
      appendLog('Erase completed.', 'success')
    } catch (e: any) { appendLog(`Error: ${e}`, 'error') }
    isRunning.value = false
  }

  async function listParts() {
    isRunning.value = true; clearLog(); appendLog('Reading partitions...', 'system')
    try {
      const result: string = await sendIpcToMain('list_partitions', { pkgId: selectedPkg.value, device: selectedDevice.value || null })
      for (const line of result.split('\n')) if (line.trim()) appendLog(line.trim())
      appendLog('Done.', 'success')
    } catch (e: any) { appendLog(`Error: ${e}`, 'error') }
    isRunning.value = false
  }

  async function eraseFrp() {
    isRunning.value = true; clearLog(); appendLog('Erasing FRP...', 'system')
    try {
      await sendIpcToMain('erase_frp', { pkgId: selectedPkg.value, device: selectedDevice.value || null })
      appendLog('FRP erase completed.', 'success')
    } catch (e: any) { appendLog(`Error: ${e}`, 'error') }
    isRunning.value = false
  }

  async function stop() {
    await sendIpcToMain('stop_process')
    isRunning.value = false
    appendLog('Stopped.', 'warn')
  }

  async function selectImage() {
    const result = await sendIpcToMain('select_file', { title: 'Select image file', filters: ['img', 'bin', 'pac'] })
    if (result) flashImage.value = result
  }

  return {
    selectedPkg, selectedDevice, packageInstalled, isRunning,
    logLines, flashPartition, flashImage,
    packageList, filteredAliases,
    appendLog, clearLog, loadPackages,
    unlock, dump, flash, erase, listParts, eraseFrp, stop, selectImage,
  }
})
