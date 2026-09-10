import { ipcMain, dialog, shell } from 'electron'
import { spawn, execSync } from 'child_process'
import log from 'electron-log'
import { send } from '@common/utils'

const isWindows = process.platform === 'win32'

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

export default () => {
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

  ipcMain.handle('util:open-url', async (_, url: string) => {
    try {
      await shell.openExternal(url)
      return 'OK'
    } catch (err: any) {
      return 'Error: ' + err.message
    }
  })

  log.info('[V1Per] IPC handlers registered')
}
