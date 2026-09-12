import { ref } from 'vue'

export type UtilsJobType = 'root' | 'anykernel' | 'force_fastboot' | 'scrcpy'

export interface UtilsJob {
  type: UtilsJobType
  boot_img?: string
  zip?: string
}

const pending = ref<UtilsJob | null>(null)

export function setPending(job: UtilsJob) {
  pending.value = job
}

export function useUtilsRunStore() {
  return { pending }
}