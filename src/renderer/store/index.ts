import { ref, reactive } from '@common/utils/vueTools'
import { windowSizeList as defaultWindowSizeList } from '@common/config'

export const appSetting = window.appSetting || {}

export const themeId = ref('default')
export const themeInfo = reactive<{
  id: string
  name: string
  isDark: boolean
  dataPath: string
  themes: LX.Theme[]
  userThemes: LX.Theme[]
}>({
  id: 'default',
  name: 'Default',
  isDark: false,
  dataPath: '',
  themes: [],
  userThemes: [],
})

export const isFullscreen = ref(false)
export const isShowChangeLog = ref(false)
export const isShowPact = ref(false)

export const versionInfo = {
  version: '1.0.0',
  desc: '',
}

export const windowSizeList = defaultWindowSizeList

export const proxy = {
  enable: false,
  host: '',
  port: '',
}

export const qualityList = {}

export const themeShouldUseDarkColors = ref(false)