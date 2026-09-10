import path from 'node:path'
import os from 'node:os'

const isMac = process.platform == 'darwin'
const isWin = process.platform == 'win32'

const defaultSetting: LX.AppSetting = {
  version: '1.0.0',

  'common.windowSizeId': 3,
  'common.fontSize': 16,
  'common.startInFullscreen': false,
  'common.langId': null,
  'common.font': '',
  'common.isShowAnimation': true,
  'common.isAgreePact': false,
  'common.controlBtnPosition': isMac ? 'left' : 'right',
  'common.transparentWindow': !isMac,
  'common.tryAutoUpdate': true,
  'common.showChangeLog': true,

  'network.proxy.enable': false,
  'network.proxy.host': '',
  'network.proxy.port': '',

  'tray.enable': false,
  'tray.themeId': 0,

  'theme.id': 'green',
  'theme.lightId': 'green',
  'theme.darkId': 'black',

}

export default defaultSetting
