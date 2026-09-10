import fs from 'node:fs'
import { checkPath, joinPath } from '@common/utils/nodejs'
import { log } from '@common/utils'
import { APP_EVENT_NAMES, STORE_NAMES } from '@common/constants'

export const parseDataFile = async<T>(name: string): Promise<T | null> => {
  const path = joinPath(global.lxOldDataPath, name)
  if (await checkPath(path)) {
    try {
      return JSON.parse((await fs.promises.readFile(path)).toString())
    } catch (err) {
      log.error(err)
    }
  }
  return null
}

const migrateFile = async(name: string, targetName: string) => {
  let path = joinPath(global.lxDataPath, targetName)
  let oldPath = joinPath(global.lxOldDataPath, name)
  if (!await checkPath(path) && await checkPath(oldPath)) {
    await fs.promises.copyFile(oldPath, path).catch(err => {
      log.error(err)
    })
  }
}

export const migrateHotKey = async() => {
  const oldConfig = await parseDataFile<LX.HotKeyConfigAll>('hotKey.json')
  if (oldConfig) {
    const updateHotKeyTypeName = (config: LX.HotKeyConfig) => {
      const hotKeyNameMap = {
        mainWindow: APP_EVENT_NAMES.winMainName,
      }
      for (const keyConfig of Object.values(config.keys)) {
        if (hotKeyNameMap[keyConfig.type as keyof typeof hotKeyNameMap]) {
          keyConfig.type = hotKeyNameMap[keyConfig.type as keyof typeof hotKeyNameMap]
        }
      }
    }
    updateHotKeyTypeName(oldConfig.local)
    updateHotKeyTypeName(oldConfig.global)

    if (oldConfig.global.keys.VolumeUp) {
      delete oldConfig.global.keys.VolumeUp
      delete oldConfig.global.keys.VolumeDown
      delete oldConfig.global.keys.VolumeMute
    }
    return {
      local: oldConfig.local,
      global: oldConfig.global,
    }
  }
  return null
}

export const migrateUserApi = async() => migrateFile('userApi.json', STORE_NAMES.USER_API + '.json')
