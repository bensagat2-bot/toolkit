import { compareVer } from './index'

const oldThemeMap = {
  0: 'green',
  1: 'blue',
  2: 'yellow',
  3: 'orange',
  4: 'red',
  10: 'pink',
  5: 'purple',
  6: 'grey',
  11: 'ming',
  12: 'blue2',
  13: 'black',
  7: 'mid_autumn',
  8: 'naruto',
  9: 'happy_new_year',
} as const

export default (setting: any): Partial<LX.AppSetting> => {
  setting = { ...setting }

  if (compareVer(setting.version, '1.0.0') < 0) {
    if (setting.tray?.isShow != null) setting.tray.enable = setting.tray?.isShow

    setting['common.windowSizeId'] = setting.windowSizeId
    setting['common.startInFullscreen'] = setting.startInFullscreen
    setting['common.langId'] = setting.langId
    setting['common.font'] = setting.font
    setting['common.isShowAnimation'] = setting.isShowAnimation
    setting['common.isAgreePact'] = setting.isAgreePact
    setting['common.controlBtnPosition'] = setting.controlBtnPosition

    setting['network.proxy.enable'] = setting.network?.proxy?.enable
    setting['network.proxy.host'] = setting.network?.proxy?.host
    setting['network.proxy.port'] = setting.network?.proxy?.port

    setting['tray.enable'] = setting.tray?.enable
    setting['tray.themeId'] = setting.tray?.themeId

    setting['theme.id'] = oldThemeMap[setting.theme?.id as keyof typeof oldThemeMap]
    setting['theme.lightId'] = oldThemeMap[setting.theme?.lightId as keyof typeof oldThemeMap]
    setting['theme.darkId'] = oldThemeMap[setting.theme?.darkId as keyof typeof oldThemeMap]

    setting.version = '1.0.0'
  }

  return setting
}
