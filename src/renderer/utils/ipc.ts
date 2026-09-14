import { invoke } from '@tauri-apps/api/core'

export const sendIpcToMain = <T = any>(channel: string, ...args: any[]): Promise<T> => {
  const params = args[0] ?? {}
  return invoke(channel, params) as Promise<T>
}

export const sendIpcWithTimeout = <T = any>(channel: string, args: any, timeoutMs = 120000): Promise<T> => {
  let timer: ReturnType<typeof setTimeout> | null = null
  const timeoutPromise = new Promise<T>((_, reject) => {
    timer = setTimeout(() => reject(new Error(`Operation timed out after ${timeoutMs / 1000}s`)), timeoutMs)
  })
  return Promise.race([sendIpcToMain<T>(channel, args), timeoutPromise]).finally(() => {
    if (timer) clearTimeout(timer)
  })
}

export async function rendererOn(name: string, listener: (event: any, params: any) => void): Promise<() => void> {
  const { listen } = await import('@tauri-apps/api/event')
  const unlisten = await listen(name, (event) => {
    listener(null, event.payload)
  })
  return unlisten
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

export const setWindowMinimize = () => invoke('window_minimize')
export const setWindowMaximize = () => invoke('window_toggle_maximize')
export const setWindowClose = () => invoke('window_close')
export const setWindowFullscreen = () => invoke('window_toggle_fullscreen')
