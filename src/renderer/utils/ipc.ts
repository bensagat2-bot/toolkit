import { ipcRenderer } from 'electron'

export const sendIpcToMain = <T = any>(channel: string, ...args: any[]): Promise<T> => {
  return ipcRenderer.invoke(channel, ...args)
}

export function rendererOn(name: string, listener: LX.IpcRendererEventListener): void
export function rendererOn<T>(name: string, listener: LX.IpcRendererEventListenerParams<T>): void
export function rendererOn<T>(name: string, listener: LX.IpcRendererEventListener | LX.IpcRendererEventListenerParams<T>): void {
  ipcRenderer.on(name, (event, params) => {
    ;(listener as LX.IpcRendererEventListenerParams<T>)({ event, params })
  })
}

export function rendererOnce(name: string, listener: LX.IpcRendererEventListener): void
export function rendererOnce<T>(name: string, listener: LX.IpcRendererEventListenerParams<T>): void
export function rendererOnce<T>(name: string, listener: LX.IpcRendererEventListener | LX.IpcRendererEventListenerParams<T>): void {
  ipcRenderer.once(name, (event, params) => {
    ;(listener as LX.IpcRendererEventListenerParams<T>)({ event, params })
  })
}

export const rendererOff = (name: string, listener: (...args: any[]) => any) => {
  ipcRenderer.removeListener(name, listener)
}

export const rendererSend = <T = any>(name: string, params?: T): void => {
  ipcRenderer.send(name, params)
}

export const rendererInvoke = <T = any, V = any>(name: string, params?: T): Promise<V> => {
  return ipcRenderer.invoke(name, params)
}

export const getSetting = async(): Promise<LX.AppSetting> => {
  return sendIpcToMain<LX.AppSetting>('get-setting')
}

export const updateSetting = (setting: Partial<LX.AppSetting>): Promise<LX.AppSetting> => {
  return sendIpcToMain<LX.AppSetting>('set-setting', setting)
}

export const getThemes = async(): Promise<LX.ThemeInfo> => {
  return sendIpcToMain<LX.ThemeInfo>('get-themes')
}

export const saveTheme = (theme: LX.Theme): Promise<void> => {
  return sendIpcToMain('save-theme', theme)
}

export const removeTheme = (id: string): Promise<void> => {
  return sendIpcToMain('remove-theme', id)
}

export const getSystemFonts = async(): Promise<string[]> => {
  return sendIpcToMain<string[]>('get-system-fonts')
}

export const openDevTools = (): Promise<void> => {
  return sendIpcToMain('open-dev-tools')
}

export const showSelectDialog = (options: Electron.OpenDialogOptions): Promise<Electron.OpenDialogReturnValue> => {
  return sendIpcToMain<Electron.OpenDialogReturnValue>('show-open-dialog', options)
}

export const showSaveDialog = (options: Electron.SaveDialogOptions): Promise<Electron.SaveDialogReturnValue> => {
  return sendIpcToMain<Electron.SaveDialogReturnValue>('show-save-dialog', options)
}

export const setWindowMinimize = () => sendIpcToMain('win-min')
export const setWindowMaximize = () => sendIpcToMain('win-max')
export const setWindowClose = () => sendIpcToMain('win-close')
export const setWindowFullscreen = () => sendIpcToMain('win-fullscreen')

export const getCacheSize = (): Promise<number> => {
  return sendIpcToMain<number>('get-cache-size')
}

export const clearCache = (): Promise<void> => {
  return sendIpcToMain('clear-cache')
}