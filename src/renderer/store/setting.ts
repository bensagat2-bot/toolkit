import { reactive } from '@common/utils/vueTools'
import * as ipc from '@renderer/utils/ipc'
import { themeId } from './index'

export const appSetting = reactive(window.appSetting || {})

export const initSetting = (setting: any) => {
  Object.assign(appSetting, setting)
}

export const isShowAnimation = true

export const mergeSetting = (newSetting: any) => {
  Object.assign(appSetting, newSetting)
}

export const updateSetting = (setting: any) => {
  Object.assign(appSetting, setting)
  void ipc.updateSetting(setting).then(saved => {
    Object.assign(appSetting, saved)
  })
}

export const saveAgreePact = () => {
  updateSetting({ 'common.isAgreePact': true })
}

export const saveVolume = () => {}
export const saveVolumeIsMute = () => {}

export { themeId }