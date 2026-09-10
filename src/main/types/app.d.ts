/* eslint-disable no-var */

interface Lx {
  inited: boolean
  appSetting: LX.AppSetting
  hotKey: {
    enable: boolean
    config: LX.HotKeyConfigAll
    state: LX.HotKeyState
  }
  isSkipTrayQuit: boolean
  event_app: any
  theme: LX.ThemeSetting
  [key: string]: any
}

declare global {
  var envParams: LX.EnvParams
  var staticPath: string
  var lxDataPath: string
  var lxOldDataPath: string
  var lx: Lx
  var appWorder: AppWorder
}
