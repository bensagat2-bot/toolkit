import { WIN_MAIN_RENDERER_EVENT_NAME } from '@common/ipcNames'
import { mainHandle } from '@common/mainIpc'
import { sendEvent } from '@main/modules/winMain/main'

const noop = () => {}

export default () => {
  mainHandle<string, LX.UserApi.ImportUserApi>(WIN_MAIN_RENDERER_EVENT_NAME.import_user_api, async({ params: script }) => {
    return { apiInfo: { id: '', name: '', enable: false, api: null, allowShowUpdateAlert: true }, apiList: [] }
  })

  mainHandle<string[], LX.UserApi.UserApiInfo[]>(WIN_MAIN_RENDERER_EVENT_NAME.remove_user_api, async({ params: apiIds }) => {
    return []
  })

  mainHandle<LX.UserApi.UserApiSetApiParams, void>(WIN_MAIN_RENDERER_EVENT_NAME.set_user_api, async({ params: apiId }) => {
    return
  })

  mainHandle<any, LX.UserApi.UserApiInfo[]>(WIN_MAIN_RENDERER_EVENT_NAME.get_user_api_list, async() => {
    return []
  })

  mainHandle<any, LX.UserApi.UserApiStatus>(WIN_MAIN_RENDERER_EVENT_NAME.get_user_api_status, async() => {
    return { status: 'disable' as const, isEnable: false }
  })

  mainHandle<LX.UserApi.UserApiSetAllowUpdateAlertParams, void>(WIN_MAIN_RENDERER_EVENT_NAME.user_api_set_allow_update_alert, async({ params: { id, enable } }) => {
    noop()
  })

  mainHandle<LX.UserApi.UserApiRequestParams, any>(WIN_MAIN_RENDERER_EVENT_NAME.request_user_api, async({ params }) => {
    return { success: false, message: 'User API not available' }
  })
  mainHandle<LX.UserApi.UserApiRequestCancelParams, void>(WIN_MAIN_RENDERER_EVENT_NAME.request_user_api_cancel, async({ params: requestKey }) => {
    noop()
  })
}

export const sendStatusChange = (status: LX.UserApi.UserApiStatus) => {
  sendEvent(WIN_MAIN_RENDERER_EVENT_NAME.user_api_status, status)
}
export const sendShowUpdateAlert = (info: LX.UserApi.UserApiUpdateInfo) => {
  sendEvent(WIN_MAIN_RENDERER_EVENT_NAME.user_api_show_update_alert, info)
}

