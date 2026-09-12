import { ref } from 'vue'

export type XiaomiJobType = 'detect' | 'unlock' | 'flash' | 'fastboot_cmd' | 'adb_cmd' | 'reboot_bootloader' | 'rom_scan'

export interface XiaomiJob {
  type: XiaomiJobType
  args?: string[]
  folder?: string
}

const pending = ref<XiaomiJob | null>(null)

export function setPending(job: XiaomiJob) {
  pending.value = job
}

export function useXiaomiRunStore() {
  return { pending }
}
