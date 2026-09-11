import { ref } from 'vue'

export interface MtkFlashOptions {
  da_path: string | null
  scatter_path: string
  auth_path: string | null
  force_brom: boolean
  use_preloader_from_fw: boolean
  force_brom_erase_preloader: boolean
  partitions: string[] | null
}

const pending = ref<MtkFlashOptions | null>(null)

export function setPending(opts: MtkFlashOptions) {
  pending.value = opts
}

export function useMtkFlashStore() {
  return { pending }
}