import { ref } from 'vue'

export type UnisocJobType = 'unlock' | 'flash' | 'erase_frp' | 'dump'

export interface UnisocFlashPart {
  name: string
  file: string
}

export interface UnisocJob {
  type: UnisocJobType
  pkg_id: string
  device: string | null
  folder?: string
  partitions?: UnisocFlashPart[]
}

const pending = ref<UnisocJob | null>(null)

export function setPending(job: UnisocJob) {
  pending.value = job
}

export function useUnisocRunStore() {
  return { pending }
}