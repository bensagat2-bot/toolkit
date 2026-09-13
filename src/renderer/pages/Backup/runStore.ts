import { ref } from 'vue'

export interface BackupJob {
  name: string
}

const pending = ref<BackupJob | null>(null)

export function setPending(job: BackupJob) {
  pending.value = job
}

export function useBackupRunStore() {
  return { pending }
}