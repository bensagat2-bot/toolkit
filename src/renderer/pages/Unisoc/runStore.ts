import { ref } from 'vue'

export type UnisocJobType = 'unlock' | 'flash' | 'erase_frp' | 'dump' | 'cli'

export interface UnisocFlashPart {
  name: string
  file: string
}

export interface UnisocCliOp {
  name: string
  part?: string
  file?: string
  offset?: string
  size?: string
  mode?: string
  value?: string
}

export interface UnisocCliRequest {
  wait_secs?: number
  kick?: boolean
  kickto?: string
  baudrate?: string
  blk_size?: string
  ops: UnisocCliOp[]
}

export interface UnisocJob {
  type: UnisocJobType
  pkg_id: string
  device: string | null
  folder?: string
  partitions?: UnisocFlashPart[]
  cli?: UnisocCliRequest
}

const pending = ref<UnisocJob | null>(null)

export function setPending(job: UnisocJob) {
  pending.value = job
}

export function useUnisocRunStore() {
  return { pending }
}