/// <reference types="node" />

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

interface AppWorder {
  [key: string]: any
}

declare var isDev: boolean
declare var envParams: LX.EnvParams
declare var staticPath: string
declare var lxDataPath: string
declare var lxOldDataPath: string
declare var lx: Lx
declare var appWorder: AppWorder
declare var webpackStaticPath: string
declare var webpackUserApiPath: string
