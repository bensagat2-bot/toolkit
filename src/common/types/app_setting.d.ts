declare namespace LX {
  interface AppSetting {
    'version': string

    'common.windowSizeId': number
    'common.fontSize': number
    'common.startInFullscreen': boolean
    'common.langId': string | null
    'common.font': string
    'common.isShowAnimation': boolean
    'common.isAgreePact': boolean
    'common.controlBtnPosition': 'left' | 'right'
    'common.transparentWindow': boolean
    'common.tryAutoUpdate': boolean
    'common.showChangeLog': boolean

    'network.proxy.enable': boolean
    'network.proxy.host': string
    'network.proxy.port': string

    'tray.enable': boolean
    'tray.themeId': number

    'theme.id': string
    'theme.lightId': string
    'theme.darkId': string
  }
}
