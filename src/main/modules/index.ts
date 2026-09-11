import registerAppMenu from './appMenu'
import registerV1Per from './v1per'
import { registerUnisocHandlers } from './unisoc'

let isRegistered = false
export default () => {
  if (isRegistered) return
  registerAppMenu()
  registerV1Per()
  registerUnisocHandlers()
  isRegistered = true
}
