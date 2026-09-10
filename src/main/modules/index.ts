import registerHotKey from './hotKey'
import registerAppMenu from './appMenu'
import registerV1Per from './v1per'

let isRegistered = false
export default () => {
  if (isRegistered) return
  registerHotKey()
  registerAppMenu()
  registerV1Per()
  isRegistered = true
}
