import '@common/error'
import { createApp } from 'vue'

import '@renderer/event'

import mountComponents from './components'

import initPlugins from './plugins'
import { i18nPlugin } from './plugins/i18n'

import App from './App.vue'
import router from './router'

import { initSetting } from './store/setting'
import { themeShouldUseDarkColors } from './store'
import { getSetting } from './utils/ipc'
import { setLanguage } from '@root/lang'

const app = createApp(App)

router.afterEach(() => {})

window.appSetting = {}
initSetting({
  'version': '1.0.0',
  'common.windowSizeId': 3,
  'common.fontSize': 16,
  'common.startInFullscreen': false,
  'common.langId': 'en-us',
  'common.font': 'Microsoft YaHei',
  'common.isShowAnimation': true,
  'common.isAgreePact': false,
  'common.controlBtnPosition': 'right',
  'common.transparentWindow': true,
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
})

themeShouldUseDarkColors.value = window.shouldUseDarkColors

void getSetting().then(setting => {
  initSetting(setting)
  setLanguage((setting['common.langId'] ?? 'en-us') as any)
})

app.use(router).use(i18nPlugin)
initPlugins(app)
mountComponents(app)
app.mount('#root')