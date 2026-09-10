import { exec, spawn } from 'child_process'
import { promisify } from 'util'

const execAsync = promisify(exec)

export interface CommandResult {
  success: boolean
  output: string
  error?: string
}

export const runAdbCommand = async (command: string): Promise<CommandResult> => {
  try {
    const { stdout, stderr } = await execAsync(`adb ${command}`)
    return {
      success: true,
      output: stdout.trim(),
      error: stderr.trim() || undefined
    }
  } catch (error: any) {
    return {
      success: false,
      output: error.stdout?.trim() || '',
      error: error.stderr?.trim() || error.message
    }
  }
}

export const runFastbootCommand = async (command: string): Promise<CommandResult> => {
  try {
    const { stdout, stderr } = await execAsync(`fastboot ${command}`)
    return {
      success: true,
      output: stdout.trim(),
      error: stderr.trim() || undefined
    }
  } catch (error: any) {
    return {
      success: false,
      output: error.stdout?.trim() || '',
      error: error.stderr?.trim() || error.message
    }
  }
}

export const getAdbDevices = async (): Promise<CommandResult> => {
  return runAdbCommand('devices -l')
}

export const getFastbootDevices = async (): Promise<CommandResult> => {
  return runFastbootCommand('devices')
}

export const adbReboot = async (mode?: 'bootloader' | 'recovery' | 'sideload'): Promise<CommandResult> => {
  const cmd = mode ? `reboot ${mode}` : 'reboot'
  return runAdbCommand(cmd)
}

export const fastbootReboot = async (): Promise<CommandResult> => {
  return runFastbootCommand('reboot')
}

export const adbSideload = async (filePath: string): Promise<CommandResult> => {
  return runAdbCommand(`sideload "${filePath}"`)
}

export const fastbootFlash = async (partition: string, filePath: string): Promise<CommandResult> => {
  return runFastbootCommand(`flash ${partition} "${filePath}"`)
}

export const adbShell = async (command: string): Promise<CommandResult> => {
  return runAdbCommand(`shell ${command}`)
}

export const fastbootOem = async (command: string): Promise<CommandResult> => {
  return runFastbootCommand(`oem ${command}`)
}

export const getDeviceInfo = async (): Promise<CommandResult> => {
  const result = await runAdbCommand('shell getprop')
  if (result.success) {
    const props: Record<string, string> = {}
    result.output.split('\n').forEach(line => {
      const match = line.match(/\[(.+?)\]:\s*\[(.+?)\]/)
      if (match) {
        props[match[1]] = match[2]
      }
    })
    return {
      success: true,
      output: JSON.stringify(props, null, 2)
    }
  }
  return result
}

export const getBatteryInfo = async (): Promise<CommandResult> => {
  const result = await runAdbCommand('shell dumpsys battery')
  if (result.success) {
    const lines = result.output.split('\n')
    const info: Record<string, string> = {}
    lines.forEach(line => {
      const match = line.match(/(\w+):\s*(\d+)/)
      if (match) {
        info[match[1]] = match[2]
      }
    })
    return {
      success: true,
      output: JSON.stringify(info, null, 2)
    }
  }
  return result
}

export const getStorageInfo = async (): Promise<CommandResult> => {
  const result = await runAdbCommand('shell df -h')
  return result
}

export const checkRoot = async (): Promise<CommandResult> => {
  const result = await runAdbCommand('shell su -c id')
  if (result.success && result.output.includes('uid=0')) {
    return {
      success: true,
      output: 'Root access available'
    }
  }
  return {
    success: false,
    output: 'No root access',
    error: 'Device is not rooted or su not available'
  }
}
