import { defineStore } from 'pinia'
import { ref } from 'vue'

// Global operation lock. While busy, the whole UI is covered by an invisible
// click-blocker so the user cannot click other tools/nav or disturb a running
// operation. No visible overlay is shown.
export const useOperationStore = defineStore('operation', () => {
  const busy = ref(false)

  function setBusy(value: boolean) {
    busy.value = value
  }

  return { busy, setBusy }
})