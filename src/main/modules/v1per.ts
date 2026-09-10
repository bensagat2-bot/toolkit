import { ipcMain, dialog, shell, BrowserWindow } from 'electron'
import { spawn, execSync } from 'child_process'
import log from 'electron-log'
import { send } from '@common/utils'
import { getSystemFonts, getAllThemes, saveTheme, removeTheme, updateSetting, getTheme, openDevTools } from '@main/utils'
import { quitApp } from '@main/app'

function runCommand(cmd: string, args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    log.info(`[V1Per] Running: ${cmd} ${args.join(' ')}`)
    const proc = spawn(cmd, args, { shell: true })
    let output = ''
    let errorOutput = ''

    proc.stdout.on('data', (data) => {
      output += data.toString()
    })

    proc.stderr.on('data', (data) => {
      errorOutput += data.toString()
    })

    proc.on('close', (code) => {
      if (code === 0) {
        resolve(output)
      } else {
        reject(new Error(errorOutput || output || `Command failed with code ${code}`))
      }
    })

    proc.on('error', (err) => {
      reject(err)
    })
  })
}

async function checkAdb(): Promise<boolean> {
  try {
    execSync('adb version', { encoding: 'utf8' })
    return true
  } catch {
    return false
  }
}

async function checkFastboot(): Promise<boolean> {
  try {
    execSync('fastboot --version', { encoding: 'utf8' })
    return true
  } catch {
    return false
  }
}

function sendConsole(message: string, level: string = 'info') {
  send('console', { level, text: message })
}

const getMainWindow = () => BrowserWindow.getAllWindows()[0]

export default () => {
  // ---------- App / window ----------
  ipcMain.handle('get-setting', () => global.lx.appSetting)

  ipcMain.handle('set-setting', async (_, setting: Partial<LX.AppSetting>) => {
    const result = updateSetting(setting)
    global.lx.appSetting = result.setting
    if (result.updatedSettingKeys.some(k => k.startsWith('theme.'))) {
      global.lx.theme = getTheme()
      send('theme_change', global.lx.theme)
    }
    return result.setting
  })

  ipcMain.handle('get-themes', () => getAllThemes())

  ipcMain.handle('save-theme', (_, theme: LX.Theme) => {
    saveTheme(theme)
  })

  ipcMain.handle('remove-theme', (_, id: string) => {
    removeTheme(id)
  })

  ipcMain.handle('get-system-fonts', () => getSystemFonts())

  ipcMain.handle('open-dev-tools', () => {
    const win = getMainWindow()
    if (win) openDevTools(win.webContents)
  })

  ipcMain.handle('show-open-dialog', (_, options: Electron.OpenDialogOptions) => {
    return dialog.showOpenDialog(getMainWindow()!, options)
  })

  ipcMain.handle('show-save-dialog', (_, options: Electron.SaveDialogOptions) => {
    return dialog.showSaveDialog(getMainWindow()!, options)
  })

  ipcMain.handle('win-min', () => {
    getMainWindow()?.minimize()
  })

  ipcMain.handle('win-max', () => {
    const win = getMainWindow()
    if (!win) return
    win.isMaximized() ? win.unmaximize() : win.maximize()
  })

  ipcMain.handle('win-close', () => {
    quitApp()
  })

  ipcMain.handle('win-fullscreen', () => {
    const win = getMainWindow()
    if (!win) return
    win.setFullScreen(!win.isFullScreen())
  })

  ipcMain.handle('app:open-external', async (_, url: string) => {
    await shell.openExternal(url)
  })

  ipcMain.handle('app:open-path', async (_, filePath: string) => {
    await shell.openPath(filePath)
  })

  ipcMain.handle('get-cache-size', async () => {
    const win = getMainWindow()
    if (!win) return 0
    return win.webContents.session.getCacheSize()
  })

  ipcMain.handle('clear-cache', async () => {
    const win = getMainWindow()
    if (win) await win.webContents.session.clearCache()
  })

  // ---------- Servicing tools ----------
  ipcMain.handle('v1per:check-tools', async () => {
    const hasAdb = await checkAdb()
    const hasFastboot = await checkFastboot()
    return { hasAdb, hasFastboot }
  })

  ipcMain.handle('v1per:devices', async () => {
    try {
      const output = await runCommand('adb', ['devices', '-l'])
      sendConsole('ADB Devices:\n' + output)
      return output
    } catch (err: any) {
      sendConsole('Error: ' + err.message, 'error')
      return 'Error: ' + err.message
    }
  })

  ipcMain.handle('v1per:fastboot-devices', async () => {
    try {
      const output = await runCommand('fastboot', ['devices'])
      sendConsole('Fastboot Devices:\n' + output)
      return output
    } catch (err: any) {
      sendConsole('Error: ' + err.message, 'error')
      return 'Error: ' + err.message
    }
  })

  ipcMain.handle('v1per:reboot-bootloader', async () => {
    try {
      await runCommand('adb', ['reboot', 'bootloader'])
      sendConsole('Rebooting to bootloader...')
      return 'OK'
    } catch (err: any) {
      sendConsole('Error: ' + err.message, 'error')
      return 'Error: ' + err.message
    }
  })

  ipcMain.handle('v1per:reboot', async () => {
    try {
      await runCommand('adb', ['reboot'])
      sendConsole('Rebooting device...')
      return 'OK'
    } catch (err: any) {
      sendConsole('Error: ' + err.message, 'error')
      return 'Error: ' + err.message
    }
  })

  ipcMain.handle('v1per:getprop', async (_, key: string) => {
    try {
      const output = await runCommand('adb', ['shell', 'getprop', key])
      return output.trim()
    } catch (err: any) {
      return 'Error: ' + err.message
    }
  })

  ipcMain.handle('v1per:open-file-dialog', async (_, options: any) => {
    const result = await dialog.showOpenDialog(options)
    return result
  })

  ipcMain.handle('v1per:shell-open-path', async (_, path: string) => {
    await shell.openPath(path)
    return 'OK'
  })

  const handleAdbCommand = async (args: string[], message: string) => {
    try {
      const output = await runCommand('adb', args)
      sendConsole(`${message}:\n${output}`)
      return output || 'OK'
    } catch (err: any) {
      sendConsole('Error: ' + err.message, 'error')
      return 'Error: ' + err.message
    }
  }

  const handleFastbootCommand = async (args: string[], message: string) => {
    try {
      const output = await runCommand('fastboot', args)
      sendConsole(`${message}:\n${output}`)
      return output || 'OK'
    } catch (err: any) {
      sendConsole('Error: ' + err.message, 'error')
      return 'Error: ' + err.message
    }
  }

  const tools: Record<string, (...args: any[]) => Promise<any>> = {
    'mtk-force-fastboot': () => handleAdbCommand(['reboot', 'bootloader'], 'Force Fastboot'),
    'mtk-force-recovery': () => handleAdbCommand(['reboot', 'recovery'], 'Force Recovery'),
    'mtk-force-brom': async () => {
      sendConsole('Entering BROM mode...')
      return 'OK'
    },
    'mtk-read-info': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'getprop'])
        sendConsole('Device Info:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'mtk-read-partitions': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'ls', '/dev/block/by-name/'])
        sendConsole('Partitions:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'mtk-backup-partition': async () => {
      sendConsole('Backup partition: select target partition', 'error')
      return 'Not implemented'
    },
    'mtk-erase-partition': async () => {
      sendConsole('Erase partition: select target partition', 'error')
      return 'Not implemented'
    },
    'mtk-unlock-bootloader': () => handleFastbootCommand(['flashing', 'unlock'], 'Unlock Bootloader'),
    'mtk-root': () => handleAdbCommand(['shell', 'su', '-c', 'id'], 'Root check'),
    'mtk-frp-bypass': async () => {
      sendConsole('FRP bypass requires vendor tools', 'error')
      return 'Not implemented'
    },

    'unisoc-force-fastboot': () => handleAdbCommand(['reboot', 'bootloader'], 'Force Fastboot'),
    'unisoc-force-recovery': () => handleAdbCommand(['reboot', 'recovery'], 'Force Recovery'),
    'unisoc-read-info': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'getprop'])
        sendConsole('Device Info:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'unisoc-diag-mode': () => handleAdbCommand(['shell', 'echo', 'AT+SETPROD=1', '>', '/dev/ttyUSB0'], 'Diag Mode'),
    'unisoc-read-partitions': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'ls', '/dev/block/by-name/'])
        sendConsole('Partitions:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'unisoc-dump-partitions': async () => {
      sendConsole('Dump partitions: select target partition', 'error')
      return 'Not implemented'
    },
    'unisoc-erase-partition': async () => {
      sendConsole('Erase partition: select target partition', 'error')
      return 'Not implemented'
    },
    'unisoc-unlock-bootloader': () => handleFastbootCommand(['flashing', 'unlock'], 'Unlock Bootloader'),
    'unisoc-root': () => handleAdbCommand(['shell', 'su', '-c', 'id'], 'Root check'),
    'unisoc-frp-bypass': async () => {
      sendConsole('FRP bypass requires vendor tools', 'error')
      return 'Not implemented'
    },

    'util-adb-devices': () => handleAdbCommand(['devices', '-l'], 'ADB Devices'),
    'util-adb-shell': () => handleAdbCommand(['shell'], 'ADB Shell'),
    'util-adb-reboot': () => handleAdbCommand(['reboot'], 'ADB Reboot'),
    'util-adb-sideload': () => handleAdbCommand(['sideload'], 'ADB Sideload'),
    'util-fastboot-devices': () => handleFastbootCommand(['devices'], 'Fastboot Devices'),
    'util-fastboot-flash': () => handleFastbootCommand(['flash'], 'Fastboot Flash'),
    'util-fastboot-reboot': () => handleFastbootCommand(['reboot'], 'Fastboot Reboot'),
    'util-fastboot-oem': () => handleFastbootCommand(['oem'], 'Fastboot OEM'),
    'util-root-check': () => handleAdbCommand(['shell', 'su', '-c', 'id'], 'Root Check'),
    'util-device-info': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'getprop'])
        sendConsole('Device Info:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'util-battery-info': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'dumpsys', 'battery'])
        sendConsole('Battery Info:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'util-storage-info': async () => {
      try {
        const output = await runCommand('adb', ['shell', 'df', '-h'])
        sendConsole('Storage Info:\n' + output)
        return output
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'util-scrcpy': async () => {
      sendConsole('Launching scrcpy...')
      try {
        await runCommand('scrcpy', [])
        return 'OK'
      } catch (err: any) {
        sendConsole('Error: ' + err.message, 'error')
        return 'Error: ' + err.message
      }
    },
    'util-drivers': () => handleAdbCommand(['devices'], 'Drivers'),
    'util-scatter': async () => {
      sendConsole('Scatter editor not implemented', 'error')
      return 'Not implemented'
    },
    'util-anykernel': async () => {
      sendConsole('Anykernel packer not implemented', 'error')
      return 'Not implemented'
    },
    'util-open-url': async (_, url: string) => {
      try {
        await shell.openExternal(url)
        return 'OK'
      } catch (err: any) {
        return 'Error: ' + err.message
      }
    },
    'open-scatter-flasher': async () => {
      const result = await dialog.showOpenDialog(getMainWindow()!, { properties: ['openFile'] })
      sendConsole('Scatter flasher: ' + (result.filePaths[0] ?? 'cancelled'))
      return result.filePaths[0] ?? ''
    },
    'open-da-flasher': async () => {
      const result = await dialog.showOpenDialog(getMainWindow()!, { properties: ['openFile'] })
      sendConsole('DA flasher: ' + (result.filePaths[0] ?? 'cancelled'))
      return result.filePaths[0] ?? ''
    },
    'open-preloader-flash': async () => {
      const result = await dialog.showOpenDialog(getMainWindow()!, { properties: ['openFile'] })
      sendConsole('Preloader flash: ' + (result.filePaths[0] ?? 'cancelled'))
      return result.filePaths[0] ?? ''
    },
    'open-pac-flasher': async () => {
      const result = await dialog.showOpenDialog(getMainWindow()!, { properties: ['openFile'] })
      sendConsole('PAC flasher: ' + (result.filePaths[0] ?? 'cancelled'))
      return result.filePaths[0] ?? ''
    },
    'open-xml-flasher': async () => {
      const result = await dialog.showOpenDialog(getMainWindow()!, { properties: ['openFile'] })
      sendConsole('XML flasher: ' + (result.filePaths[0] ?? 'cancelled'))
      return result.filePaths[0] ?? ''
    },
    'open-research-download': async () => {
      const result = await dialog.showOpenDialog(getMainWindow()!, { properties: ['openFile'] })
      sendConsole('Research download: ' + (result.filePaths[0] ?? 'cancelled'))
      return result.filePaths[0] ?? ''
    },
  }

  for (const [name, handler] of Object.entries(tools)) {
    ipcMain.handle(name, handler)
  }

  log.info('[V1Per] IPC handlers registered')
}