import { getThemes as getThemesFromIpc } from '@renderer/utils/ipc'
import { isUrl, encodePath } from '@common/utils/common'
import { themeInfo, themeShouldUseDarkColors } from './index'

let themesData: LX.ThemeInfo | null = null

export const findTheme = (info: LX.ThemeInfo, id: string): LX.Theme | undefined => {
  return info.themes.find(theme => theme.id == id) ?? info.userThemes.find(theme => theme.id == id)
}

export const getThemes = (callback?: (info: LX.ThemeInfo) => void): Promise<LX.ThemeInfo> => {
  return getThemesFromIpc().then(info => {
    themesData = info
    themeInfo.themes = info.themes
    themeInfo.userThemes = info.userThemes
    themeInfo.dataPath = info.dataPath
    callback?.(info)
    return info
  })
}

export const buildBgUrl = (imageName: string, dataPath: string): string => {
  return isUrl(imageName)
    ? `url(${imageName})`
    : `url(file:///${encodePath(dataPath)}/${encodePath(imageName)})`
}

export const buildThemeColors = (theme: LX.Theme, dataPath: string): Record<string, string> => {
  const colors: Record<string, string> = {
    ...theme.config.themeColors,
    ...theme.config.extInfo,
  }
  const bg = theme.config.extInfo['--background-image']
  if (bg && bg != 'none' && !isUrl(bg) && !bg.startsWith('url(')) {
    colors['--background-image'] = buildBgUrl(bg, dataPath)
  }
  return colors
}

export const copyTheme = (theme: LX.Theme): LX.Theme => {
  return {
    ...theme,
    config: {
      ...theme.config,
      extInfo: { ...theme.config.extInfo },
      themeColors: { ...theme.config.themeColors },
    },
  }
}

export const applyTheme = (id: string, lightId: string, darkId: string, dataPath: string) => {
  if (!themesData) return
  themeId.value = id
  let theme: LX.Theme | undefined
  if (id == 'auto') {
    const light = findTheme(themesData, lightId)
    const dark = findTheme(themesData, darkId)
    theme = themeShouldUseDarkColors.value ? dark ?? light : light ?? dark
  } else {
    theme = findTheme(themesData, id)
  }
  if (!theme) return
  window.setTheme(buildThemeColors(theme, dataPath))
  themeInfo.id = theme.id
  themeInfo.name = theme.name
  themeInfo.isDark = theme.isDark
}