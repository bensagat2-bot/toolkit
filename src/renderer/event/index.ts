import { rendererOn } from '@renderer/utils/ipc'
import { createAppEventHub } from './appEvent'

export const registerEvents = () => {
  window.app_event = createAppEventHub()

  rendererOn('theme_change', ({ params }) => {
    window.setTheme((params as LX.ThemeSetting).theme.colors)
  })
}

export { clearDownKeys, createKeyEventHub } from './keyEvent'
export type { AppEventTypes } from './appEvent'
export type { KeyEventTypes } from './keyEvent'

registerEvents()