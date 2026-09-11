import { invoke } from '@tauri-apps/api/core'

export const sendIpcToMain = <T = any>(channel: string, ...args: any[]): Promise<T> => {
  const params = args[0] ?? {}
  return invoke(channel, params) as Promise<T>
}

export function rendererOn(name: string, listener: (event: any, params: any) => void): void {
  // Tauri v2 uses events via listen()
  import('@tauri-apps/api/event').then(({ listen }) => {
    listen(name, (event) => {
      listener(null, event.payload)
    })
  })
}

export function rendererOff(_name: string, _listener: any): void {
  // Tauri v2: unlisten handled by returned unlisten function
}

export const showSelectDialog = async (options: any): Promise<{ filePaths: string[] }> => {
  const filters = options?.filters?.map((f: any) => ({
    name: f.name || 'Files',
    extensions: f.extensions || [],
  })) || []
  const result = await invoke<string | null>('select_file', {
    title: options?.title || 'Select File',
    filters: filters.flatMap((f: any) => f.extensions),
  })
  return { filePaths: result ? [result] : [] }
}

export const showSaveDialog = async (_options: any): Promise<{ filePath: string | null }> => {
  const result = await invoke<string | null>('select_file', {
    title: 'Save File',
    filters: [],
  })
  return { filePath: result }
}

export const showSelectFolder = async (title = 'Select Folder'): Promise<string> => {
  const result = await invoke<string | null>('select_folder', { title })
  return result || ''
}

export const getSetting = async (): Promise<any> => {
  return {}
}

export const updateSetting = async (setting: any): Promise<any> => {
  return setting
}

export const getThemes = async (): Promise<any> => {
  return {}
}

export const saveTheme = async (_theme: any): Promise<void> => {}

export const removeTheme = async (_id: string): Promise<void> => {}

export const getSystemFonts = async (): Promise<string[]> => {
  return []
}

export const openDevTools = (): Promise<void> => {
  return Promise.resolve()
}

export const setWindowMinimize = () => invoke('window_minimize')
export const setWindowMaximize = () => invoke('window_toggle_maximize')
export const setWindowClose = () => invoke('window_close')
export const setWindowFullscreen = () => invoke('window_toggle_fullscreen')

export const getCacheSize = (): Promise<number> => {
  return Promise.resolve(0)
}

export const clearCache = (): Promise<void> => {
  return Promise.resolve()
}
