/* eslint-disable no-var */
import { type AppEventTypes, type KeyEventTypes } from '@renderer/event'
import { type I18n } from '@renderer/plugins/i18n'

interface Lx {
  isEditingHotKey: boolean
  isProd: boolean
  rootOffset: number
}

declare global {
  interface Window {
    ELECTRON_DISABLE_SECURITY_WARNINGS?: string
    dt: boolean
    shouldUseDarkColors: boolean
    lx: Lx
    appSetting: any
    app_event: AppEventTypes
    key_event: KeyEventTypes
    i18n: I18n

    lxData: any

    ipcRenderer: Electron.IpcRenderer

    setTheme: (colors: Record<string, string>) => void
    setLang: (lang?: string) => void
  }

  module NodeJS {
    interface ProcessVersions {
      app: string
    }
  }

  namespace LX {
    interface KeyDownEevent {
      /**
       * 原始事件
       */
      event: KeyEvent | null

      /**
       * 按下的按键数组
       */
      keys: string[]

      /**
       * 按下的按键组合
       */
      key: string
      /**
       * 当前触发此事件的单个按键（不包括之前已按下的键）
       */
      eventKey: string
      /**
       * 按键操作类型
       */
      type: 'down' | 'up'
    }

    class KeyEvent extends KeyboardEvent {
      /**
       * 此事件是否标记为 已被处理，如果设置为`true`，则停止触发key event事件
       */
      lx_handled?: boolean
    }
  }
}

// declare const ELECTRON_DISABLE_SECURITY_WARNINGS: string