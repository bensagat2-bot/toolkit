export * from '@common/utils/renderer'
export * from '@common/utils/nodejs'
export * from '@common/utils/common'
export * from '@common/utils/tools'
export { sendIpcToMain } from './ipc'

/**
 * 设置标题
 */
let dom_title = document.getElementsByTagName('title')[0]
export const setTitle = (title: string | null) => {
  title ||= 'V1Per'
  dom_title.innerText = title
}

export const getFontSizeWithScreen = (screenWidth: number = window.innerWidth): number => {
  return screenWidth <= 1440
    ? 16
    : screenWidth <= 1920
      ? 18
      : screenWidth <= 2560
        ? 20
        : 22
}

export const decodeName = (str: string | null = '') => {
  if (!str) return ''
  return new window.DOMParser().parseFromString(str, 'text/html').body.textContent
}