import './utils/logInit'
import '@common/error'
import { app } from 'electron'
import initApp from './app'
import { isLinux } from '@common/utils'

initApp()

app.whenReady().then(() => {
  isLinux ? setTimeout(() => {}, 300) : void 0
})
