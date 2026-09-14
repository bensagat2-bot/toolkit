import { defineStore } from 'pinia'
import { ref } from 'vue'

// Global operation lock. Used by run pages for local busy state and UI feedback.
export const useOperationStore = defineStore('operation', () => {
  const busy = ref(false)

  function setBusy(value: boolean) {
    busy.value = value
  }

  return { busy, setBusy }
})