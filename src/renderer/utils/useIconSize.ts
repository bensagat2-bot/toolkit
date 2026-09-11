import { ref, onMounted, onBeforeUnmount, type Ref } from 'vue'

const onDomSizeChanged = (dom: HTMLElement, onChanged: (width: number, height: number) => void) => {
  const resizeObserver = new ResizeObserver(entries => {
    for (let entry of entries) {
      const { width, height } = entry.contentRect
      onChanged(Math.trunc(width), Math.trunc(height))
    }
  })

  resizeObserver.observe(dom)
  onChanged(dom.clientWidth, dom.clientHeight)

  return () => {
    resizeObserver.disconnect()
  }
}

export const useIconSize = (parentDom: Ref<HTMLElement | null>, size: number) => {
  const iconSize = ref('32px')
  let unsub: (() => void) | null = null

  onMounted(() => {
    if (!parentDom.value) return
    unsub = onDomSizeChanged(parentDom.value, (width) => {
      iconSize.value = Math.trunc(width * size) + 'px'
    })
  })
  onBeforeUnmount(() => {
    unsub?.()
  })

  return iconSize
}
