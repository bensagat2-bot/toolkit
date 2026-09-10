import app, { sendConfigChange } from './app'
import hotKey from './hotKey'
import { sendEvent } from '../main'

export * from './app'
export * from './hotKey'
export * from './userApi'
export * from './sync'
export * from './process'

let isInitialized = false
export default () => {
  if (isInitialized) return
  isInitialized = true

  app()
  hotKey()

  global.lx.event_app.on('updated_config', (keys: any, setting: any) => {
    sendConfigChange(setting)
  })
}

