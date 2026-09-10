import path from 'node:path'
import { app, BrowserWindow, shell, screen, nativeTheme, dialog, ipcMain } from 'electron'
import { isMac, log } from '@common/utils'
import defaultSetting from '@common/defaultSetting'
import registerModules from './modules'
import { initSetting } from './utils'

let mainWindow: BrowserWindow | null = null

export const createMainWindow = () => {
  const { width: screenWidth, height: screenHeight } = screen.getPrimaryDisplay().workAreaSize
  const wins = BrowserWindow.getAllWindows()
  if (wins.length) {
    mainWindow = wins[0]
    mainWindow.show()
    return mainWindow
  }

  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    show: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
    backgroundColor: nativeTheme.shouldUseDarkColors ? '#1a1a1a' : '#ffffff',
  })

  mainWindow.on('ready-to-show', () => {
    mainWindow?.show()
  })

  mainWindow.on('closed', () => {
    mainWindow = null
  })

  if (process.env.ELECTRON_RENDERER_URL) {
    mainWindow.loadURL(process.env.ELECTRON_RENDERER_URL)
  } else {
    mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'))
  }

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

  global.lx = {
    inited: false,
    isSkipTrayQuit: false,
    event_app: { on: () => {}, emit: () => {} },
    event_list: { on: () => {}, emit: () => {} },
    event_dislike: { on: () => {}, emit: () => {} },
    worker: { dbService: {} } as any,
    player_status: { status: 'stoped' } as any,
    appSetting: defaultSetting,
    hotKey: {
      enable: true,
      config: {
        local: { enable: false, keys: {} },
        global: { enable: false, keys: {} },
      },
      state: new Map(),
    },
    theme: {
      shouldUseDarkColors: nativeTheme.shouldUseDarkColors,
      theme: { id: 'default', name: 'default', isDark: false, colors: {} },
    },
  }
  global.lxDataPath = path.join(app.getPath('userData'), 'data')

  initSetting()

  registerModules()

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

  ipcMain.handle('app:open-external', async (_, url: string) => {
    await shell.openExternal(url)
  })

  ipcMain.handle('app:open-path', async (_, filePath: string) => {
    await shell.openPath(filePath)
  })

  log.info('[V1Per] App initialized')
}
