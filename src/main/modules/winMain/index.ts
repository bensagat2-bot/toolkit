import initRendererEvent, { handleKeyDown, hotKeyConfigUpdate } from './rendererEvent'

import { APP_EVENT_NAMES } from '@common/constants'
import { createWindow, minimize, setProxy, toggleHide, toggleMinimize } from './main'
import initUpdate from './autoUpdate'
import { HOTKEY_COMMON } from '@common/hotKey'
import { quitApp } from '@main/app'

export default () => {
  initRendererEvent()
  initUpdate()

  global.lx.event_app.on('hot_key_down', ({ type, key }: { type: string, key: string }) => {
    let info = global.lx.hotKey.config.global.keys[key]
    if (info?.type != APP_EVENT_NAMES.winMainName) return
    switch (info.action) {
      case HOTKEY_COMMON.close.action:
        quitApp()
        break
      case HOTKEY_COMMON.hide_toggle.action:
        toggleHide()
        break
      case HOTKEY_COMMON.min.action:
        minimize()
        break
      case HOTKEY_COMMON.min_toggle.action:
        toggleMinimize()
        break
      default:
        handleKeyDown(type, key)
        break
    }
  })
  global.lx.event_app.on('hot_key_config_update', (config: any) => {
    hotKeyConfigUpdate(config)
  })

  global.lx.event_app.on('app_inited', () => {
    createWindow()
  })

  global.lx.event_app.on('updated_config', (keys: any, setting: any) => {
    if (keys.includes('network.proxy.enable') || (global.lx.appSetting['network.proxy.enable'] && keys.some((k: string) => k.includes('network.proxy.')))) {
      setProxy()
    }
  })
}

export * from './main'
export * from './rendererEvent'

