import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DeviceMode, DetectResult } from '@renderer/types'
import { sendIpcToMain } from '@renderer/utils/ipc'

export const useDeviceStore = defineStore('device', () => {
  const mode = ref<DeviceMode>('none')
  const serial = ref<string | null>(null)
  const isDetecting = ref(false)

  const modeLabel = computed(() => {
    const labels: Record<DeviceMode, string> = {
      adb: 'ADB', fastboot: 'Fastboot', download: 'Download', none: 'No Device'
    }
    return labels[mode.value]
  })

  const isConnected = computed(() => mode.value !== 'none')

  async function detect() {
    isDetecting.value = true
    try {
      const result: DetectResult = await sendIpcToMain('detect_device')
      mode.value = result.mode
      serial.value = result.serial
    } catch {
      mode.value = 'none'
      serial.value = null
    } finally {
      isDetecting.value = false
    }
  }

  return { mode, serial, isDetecting, modeLabel, isConnected, detect }
})
