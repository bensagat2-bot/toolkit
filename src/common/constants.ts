export const URL_SCHEME_RXP = /^v1per:\/\//
export const SPLIT_CHAR = {} as const
export const STORE_NAMES = {
  APP_SETTINGS: 'config_v2',
  DATA: 'data',
  HOTKEY: 'hot_key',
  USER_API: 'user_api',
  THEME: 'theme',
} as const
export const APP_EVENT_NAMES = {
  winMainName: 'win_main',
  winLyricName: 'win_lyric',
  trayName: 'tray',
} as const
export const TRAY_AUTO_ID = -1

export const DATA_KEYS = {
  playInfo: 'playInfo',
  userApi: 'userApi',
  setting: 'setting',
  hotKey: 'hotKey',
  theme: 'theme',
  sync: 'sync',
  list: 'list',
  search: 'search',
} as const

export const LIST_IDS = {
  default: 'default',
  love: 'love',
} as const

export const DEFAULT_SETTING = {
  'common': {
    windowSizeId: 0,
    fontSize: 14,
    startInFullscreen: false,
    langId: null,
    font: 'Microsoft YaHei',
    isShowAnimation: true,
    isAgreePact: false,
    controlBtnPosition: 'left',
    transparentWindow: false,
    tryAutoUpdate: true,
    showChangeLog: true,
  },
  'network': {
    proxy: {
      enable: false,
      host: '',
      port: '',
    },
  },
  'tray': {
    enable: true,
    themeId: 0,
  },
  'theme': {
    id: 'default',
    lightId: 'default-light',
    darkId: 'default-dark',
  },
}
