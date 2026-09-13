import { defineStore } from 'pinia'
import { ref } from 'vue'

// Global operation lock. While busy, the whole UI is covered by an overlay so
// the user cannot click other tools/nav or disturb a running operation.
export const useOperationStore = defineStore('operation', () => {
  const busy = ref(false)

  function setBusy(value: boolean) {
    busy.value = value
  }

  return { busy, setBusy }
})