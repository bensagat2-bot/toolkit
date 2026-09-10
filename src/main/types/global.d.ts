/* eslint-disable no-var */

interface Lx {
  inited: boolean
  appSetting: LX.AppSetting
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