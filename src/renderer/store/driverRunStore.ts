import { ref } from 'vue'

export interface DriverDownloadJob {
  name: string
  url: string
}

const pending = ref<DriverDownloadJob | null>(null)

export function setPending(job: DriverDownloadJob) {
  pending.value = job
}

export function useDriverRunStore() {
  return { pending }
}