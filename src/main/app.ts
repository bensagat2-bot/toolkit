import path from 'node:path'
import { app, BrowserWindow, shell, nativeTheme } from 'electron'
import { isMac, log, getPlatform, encodePath } from '@common/utils'
import defaultSetting from '@common/defaultSetting'
import registerModules from './modules'
import { initSetting, getTheme, parseEnvParams, updateSetting as mainUpdateSetting } from './utils'

let mainWindow: BrowserWindow | null = null

export const createMainWindow = () => {
  const wins = BrowserWindow.getAllWindows()
  if (wins.length) {
    mainWindow = wins[0]
    mainWindow.show()
    return mainWindow
  }

  const { shouldUseDarkColors, theme } = global.lx.theme

  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    show: false,
    frame: false,
    transparent: true,
    icon: path.join(__dirname, '../../resources/icons/icon.ico'),
    webPreferences: {
      nodeIntegrationInWorker: true,
      contextIsolation: false,
      nodeIntegration: true,
      webSecurity: false,
      sandbox: false,
      spellcheck: false,
    },
    backgroundColor: '#00000000',
  })

  mainWindow.on('ready-to-show', () => {
    mainWindow?.show()
  })

  mainWindow.on('closed', () => {
    mainWindow = null
  })

  const themeQuery = encodeURIComponent(JSON.stringify(theme))
  const winURL = process.env.NODE_ENV !== 'production' ? 'http://localhost:9080' : `file://${encodePath(path.join(__dirname, 'index.html'))}`
  void mainWindow.loadURL(winURL + `?os=${getPlatform()}&dt=false&dark=${shouldUseDarkColors}&theme=${themeQuery}`)

  return mainWindow
}

export const showMainWindow = () => {
  if (mainWindow) {
    if (mainWindow.isMinimized()) mainWindow.restore()
    mainWindow.show()
    mainWindow.focus()
  }
}

export const quitApp = () => {
  app.quit()
}

export default async () => {
  log.info('[V1Per] Starting V1Per app...')

  const envParams = parseEnvParams()
  global.envParams = envParams

  global.lx = {
    inited: false,
    isSkipTrayQuit: false,
    event_app: { on: () => {}, emit: () => {} },
    appSetting: defaultSetting,
    theme: {
      shouldUseDarkColors: nativeTheme.shouldUseDarkColors,
      theme: { id: 'default', name: 'default', isDark: false, colors: {} },
    },
  }
  global.lxDataPath = path.join(app.getPath('userData'), 'data')

  const result = await initSetting()
  global.lx.appSetting = result.setting

  global.lx.theme = getTheme()

  registerModules()

  nativeTheme.on('updated', () => {
    global.lx.theme = getTheme()
    mainWindow?.webContents.send('theme_change', global.lx.theme)
  })

  app.whenReady().then(() => {
    log.info('[V1Per] App ready, creating window...')
    createMainWindow()
  })

  app.on('window-all-closed', () => {
    if (!isMac) {
      app.quit()
    }
  })

  app.on('activate', () => {
    showMainWindow()
  })

  log.info('[V1Per] App initialized')
}

export { mainUpdateSetting }