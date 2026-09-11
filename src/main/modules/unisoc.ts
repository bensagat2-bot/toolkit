import { ipcMain, dialog, BrowserWindow } from 'electron'
import { spawn, execFile, ChildProcess } from 'child_process'
import path from 'path'
import fs from 'fs'
import os from 'os'
import log from 'electron-log'

const SPRD_VID = 0x1782
const SPRD_KEYWORDS = ['Spreadtrum', 'Unisoc', 'sdboot', 'ums', 'sc']

let currentProcess: ChildProcess | null = null

interface UnisocPackage {
  id: string
  name: string
  execAddr: number
  fdl1: string
  fdl1Addr: number
  fdl2: string
  fdl2Addr: number
  cboot: string
  splLoaderBk: string | null
  miscDone: string
  chsizeUboot: boolean
  toolsGen: 'gen1' | 'gen2'
  erasePersist: boolean
  backupPartitions: string[]
  files: string[]
}

const PACKAGES: Record<string, UnisocPackage> = {
  ums9230: {
    id: 'ums9230', name: 'UMS9230 (T606/T612)', execAddr: 0x65015f08,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x65000800,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin', splLoaderBk: 'splloader_bk.bin',
    miscDone: 'misc-ubldone.bin', chsizeUboot: false, toolsGen: 'gen1', erasePersist: true,
    backupPartitions: ['boot', 'init_boot', 'vendor_boot', 'prodnv'],
    files: ['custom_exec_no_verify_65015f08.bin', 'fdl1-dl.bin', 'fdl2-dl.bin', 'fdl2-cboot.bin', 'splloader_bk.bin', 'misc-ubldone.bin', 'misc-wipe.bin'],
  },
  sc9863a: {
    id: 'sc9863a', name: 'SC9863A', execAddr: 0x4ee8,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x5000,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin', splLoaderBk: null,
    miscDone: 'misc-wipe.bin', chsizeUboot: true, toolsGen: 'gen1', erasePersist: false,
    backupPartitions: ['boot', 'prodnv'],
    files: ['custom_exec_no_verify_4ee8.bin', 'fdl1-dl.bin', 'fdl2-dl.bin', 'fdl2-cboot.bin', 'misc-wipe.bin'],
  },
  ums512: {
    id: 'ums512', name: 'UMS512 (T610/T700)', execAddr: 0x3ee8,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x5500,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin', splLoaderBk: null,
    miscDone: 'misc-wipe.bin', chsizeUboot: false, toolsGen: 'gen2', erasePersist: false,
    backupPartitions: ['boot', 'prodnv'],
    files: ['custom_exec_no_verify_3ee8.bin', 'fdl1-dl.bin', 'fdl2-dl.bin', 'fdl2-cboot.bin', 'misc-wipe.bin'],
  },
  ums9620: {
    id: 'ums9620', name: 'UMS9620', execAddr: 0x65012f48,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x65000800,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin', splLoaderBk: null,
    miscDone: 'misc-wipe.bin', chsizeUboot: false, toolsGen: 'gen2', erasePersist: false,
    backupPartitions: ['boot', 'prodnv'],
    files: ['custom_exec_no_verify_65012f48.bin', 'fdl1-dl.bin', 'fdl2-dl.bin', 'fdl2-cboot.bin', 'misc-wipe.bin'],
  },
}

function getUnisocRoot(): string | null {
  const appPath = process.env.PORTABLE_EXECUTABLE_DIR || path.dirname(process.execPath)
  let dir = appPath
  for (let i = 0; i < 6; i++) {
    const candidate = path.join(dir, 'Unisoc')
    if (fs.existsSync(candidate)) return candidate
    dir = path.dirname(dir)
  }
  const devPath = path.join(process.cwd(), 'resources', 'Unisoc')
  if (fs.existsSync(devPath)) return devPath
  return null
}

function resolveSpdDump(pkgDir: string): string | null {
  const root = getUnisocRoot()
  if (root) {
    const shared = path.join(root, 'spd_dump.exe')
    if (fs.existsSync(shared)) return shared
  }
  const local = path.join(pkgDir, 'spd_dump.exe')
  return fs.existsSync(local) ? local : null
}

function resolveHelper(pkgDir: string, toolsGen: string, exe: string): string | null {
  const exeName = exe.endsWith('.exe') ? exe : exe + '.exe'
  const root = getUnisocRoot()
  if (root) {
    const shared = path.join(root, 'tools', toolsGen, exeName)
    if (fs.existsSync(shared)) return shared
  }
  const local = path.join(pkgDir, exeName)
  return fs.existsSync(local) ? local : null
}

function prepareWork(pkgDir: string, deviceDir: string | null, files: string[], fdl1: string, fdl2: string, cboot: string): string {
  const work = path.join(os.tmpdir(), 'v1per_unisoc')
  if (fs.existsSync(work)) {
    try { fs.rmSync(work, { recursive: true, force: true }) } catch {}
  }
  fs.mkdirSync(work, { recursive: true })
  for (const name of files) {
    const src = path.join(pkgDir, name)
    if (fs.existsSync(src)) fs.copyFileSync(src, path.join(work, name))
  }
  if (deviceDir) {
    for (const name of [fdl1, fdl2, cboot]) {
      const src = path.join(deviceDir, name)
      if (fs.existsSync(src)) fs.copyFileSync(src, path.join(work, name))
    }
  }
  return work
}

function buildBaseTokens(wait: boolean, kick: boolean, pkg: UnisocPackage): string[] {
  const t: string[] = []
  if (wait && kick) t.push('--kick')
  else if (wait) t.push('--wait', '300')
  t.push('exec_addr', `0x${pkg.execAddr.toString(16)}`)
  t.push('fdl', pkg.fdl1, `0x${pkg.fdl1Addr.toString(16)}`)
  t.push('fdl', pkg.fdl2, `0x${pkg.fdl2Addr.toString(16)}`)
  t.push('exec')
  return t
}

function runSpdDumpLive(exePath: string, tokens: string[], cwd: string, win: BrowserWindow, channel: string): Promise<string> {
  return new Promise((resolve) => {
    let output = ''
    log.info(`[Unisoc] spd_dump ${tokens.join(' ')}`)
    currentProcess = spawn(exePath, tokens, { cwd, windowsHide: true })
    currentProcess.stdout?.on('data', (data: Buffer) => {
      for (const line of data.toString().split('\n').filter(Boolean)) {
        output += line + '\n'
        if (!win.isDestroyed()) win.webContents.send(channel, { type: 'output', data: line.trim() })
      }
    })
    currentProcess.stderr?.on('data', (data: Buffer) => {
      for (const line of data.toString().split('\n').filter(Boolean)) {
        output += line + '\n'
        if (!win.isDestroyed()) win.webContents.send(channel, { type: 'output', data: line.trim() })
      }
    })
    currentProcess.on('close', (code) => {
      currentProcess = null
      if (!win.isDestroyed()) win.webContents.send(channel, { type: 'done', code })
      resolve(output)
    })
    currentProcess.on('error', (err) => {
      currentProcess = null
      if (!win.isDestroyed()) win.webContents.send(channel, { type: 'error', data: err.message })
      resolve(output)
    })
  })
}

function runHelperLive(exePath: string, arg: string, cwd: string, win: BrowserWindow, channel: string): Promise<boolean> {
  return new Promise((resolve) => {
    let output = ''
    const proc = spawn(exePath, [arg], { cwd, windowsHide: true })
    proc.stdout?.on('data', (data: Buffer) => {
      const line = data.toString().trim()
      if (line) { output += line + '\n'; if (!win.isDestroyed()) win.webContents.send(channel, { type: 'output', data: line }) }
    })
    proc.stderr?.on('data', (data: Buffer) => {
      const line = data.toString().trim()
      if (line) { output += line + '\n'; if (!win.isDestroyed()) win.webContents.send(channel, { type: 'output', data: line }) }
    })
    proc.on('close', () => resolve(output.length > 0))
    proc.on('error', () => resolve(false))
  })
}

function runCommand(cmd: string, args: string[], timeout = 8000): Promise<string> {
  return new Promise((resolve) => {
    execFile(cmd, args, { timeout, windowsHide: true }, (_err, stdout) => {
      resolve(stdout || '')
    })
  })
}

function parseDeviceMode(adbOutput: string, fbOutput: string, comPorts: string): { mode: string; serial: string | null } {
  for (const line of adbOutput.split('\n').slice(1)) {
    const parts = line.trim().split(/\s+/)
    if (parts.length >= 2 && parts[1] === 'device') return { mode: 'adb', serial: parts[0] }
  }
  for (const line of fbOutput.split('\n')) {
    const parts = line.trim().split(/\s+/)
    if (parts.length >= 1 && parts[0] && !line.includes('waiting')) return { mode: 'fastboot', serial: parts[0] }
  }
  if (comPorts.includes('1782') || SPRD_KEYWORDS.some(k => comPorts.includes(k))) return { mode: 'download', serial: null }
  return { mode: 'none', serial: null }
}

const getMainWindow = () => BrowserWindow.getAllWindows()[0]

export function registerUnisocHandlers() {
  ipcMain.handle('unisoc:detect', async () => {
    const adb = await runCommand('adb', ['devices'])
    const fb = await runCommand('fastboot', ['devices'])
    let comPorts = ''
    try { comPorts = await runCommand('powershell', ['-Command', 'Get-CimInstance Win32_SerialPort | Select DeviceID,Description | Format-List']) } catch {}
    return parseDeviceMode(adb, fb, comPorts)
  })

  ipcMain.handle('unisoc:stop', async () => {
    if (currentProcess) { currentProcess.kill(); currentProcess = null; return { success: true } }
    return { success: false }
  })

  ipcMain.handle('unisoc:get-packages', async () => {
    const root = getUnisocRoot()
    const result: Record<string, boolean> = {}
    for (const id of Object.keys(PACKAGES)) {
      const dir = root ? path.join(root, id) : null
      result[id] = dir ? fs.existsSync(dir) : false
    }
    return result
  })

  ipcMain.handle('unisoc:unlock', async (event, args: { pkgId: string; device?: string }) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (!win) return { success: false }
    const ch = 'unisoc:unlock:output'
    const pkg = PACKAGES[args.pkgId]
    if (!pkg) return { success: false, error: 'Unknown package' }
    const root = getUnisocRoot()
    const pkgDir = root ? path.join(root, pkg.id) : null
    if (!pkgDir || !fs.existsSync(pkgDir)) return { success: false, error: 'Package not installed' }
    const deviceDir = args.device ? path.join(pkgDir, args.device) : null
    const work = prepareWork(pkgDir, deviceDir && fs.existsSync(deviceDir) ? deviceDir : null, pkg.files, pkg.fdl1, pkg.fdl2, pkg.cboot)
    const spdDump = resolveSpdDump(pkgDir)
    if (!spdDump) return { success: false, error: 'spd_dump.exe not found' }

    win.webContents.send(ch, { type: 'start', operation: 'unlock' })
    win.webContents.send(ch, { type: 'output', data: '[1/3] Backing up and erasing splloader + uboot...' })
    const t1 = [...buildBaseTokens(true, pkg), 'r', 'splloader', 'r', 'uboot', 'e', 'splloader', 'e', 'splloader_bak', 'reset']
    const out1 = await runSpdDumpLive(spdDump, t1, work, win, ch)
    if (!out1) { win.webContents.send(ch, { type: 'error', data: 'Connection failed.' }); return { success: false } }

    const splSource = pkg.splLoaderBk || 'splloader.bin'
    const unlocker = path.join(work, 'spl-unlock.bin')
    if (!fs.existsSync(unlocker)) {
      win.webContents.send(ch, { type: 'output', data: `Generating spl-unlock.bin from ${splSource}...` })
      const helper = resolveHelper(pkgDir, pkg.toolsGen, 'gen_spl-unlock')
      if (helper) await runHelperLive(helper, splSource, work, win, ch)
    }

    const spl16k = path.join(work, 'u-boot-spl-16k-sign.bin')
    const splBin = path.join(work, 'splloader.bin')
    if (fs.existsSync(splBin)) fs.renameSync(splBin, spl16k)
    if (pkg.chsizeUboot) { const ch = resolveHelper(pkgDir, pkg.toolsGen, 'chsize'); if (ch) await runHelperLive(ch, 'uboot.bin', work, win, ch) }
    const ubBin = path.join(work, 'uboot.bin'); const ubBak = path.join(work, 'uboot_bak.bin')
    if (fs.existsSync(ubBin)) fs.renameSync(ubBin, ubBak)

    win.webContents.send(ch, { type: 'output', data: '[2/3] Flashing modified uboot...' })
    await runSpdDumpLive(spdDump, [...buildBaseTokens(true, pkg), 'w', 'uboot', pkg.cboot, 'reset'], work, win, ch)
    win.webContents.send(ch, { type: 'output', data: 'Waiting 10s...' })
    await new Promise(r => setTimeout(r, 10000))

    win.webContents.send(ch, { type: 'output', data: 'Sending unlocker...' })
    await runSpdDumpLive(spdDump, ['exec_addr', `0x${pkg.execAddr.toString(16)}`, 'fdl', 'spl-unlock.bin', `0x${pkg.fdl1Addr.toString(16)}`], work, win, ch)

    win.webContents.send(ch, { type: 'output', data: 'Checking unlock status...' })
    const statusT = [...buildBaseTokens(false, pkg), 'verbose', '2', 'read_part', 'miscdata', '8192', '64', 'm.bin', 'reset']
    await runSpdDumpLive(spdDump, statusT, work, win, ch)

    win.webContents.send(ch, { type: 'output', data: `Backing up ${pkg.backupPartitions.join(' / ')}...` })
    const bkpT = [...buildBaseTokens(false, pkg), ...pkg.backupPartitions.flatMap(p => ['r', p]), 'reset']
    await runSpdDumpLive(spdDump, bkpT, work, win, ch)

    win.webContents.send(ch, { type: 'output', data: '[3/3] Restoring splloader and uboot...' })
    const splR = fs.existsSync(spl16k) ? 'u-boot-spl-16k-sign.bin' : (pkg.splLoaderBk || 'splloader_bk.bin')
    const ubR = fs.existsSync(ubBak) ? 'uboot_bak.bin' : 'uboot_bk.bin'
    const persist = pkg.erasePersist ? ['e', 'persist'] : []
    await runSpdDumpLive(spdDump, [...buildBaseTokens(false, pkg), 'w', 'splloader', splR, 'w', 'uboot', ubR, ...persist, 'w', 'misc', pkg.miscDone, 'reset'], work, win, ch)

    const miscFile = path.join(work, 'm.bin')
    if (fs.existsSync(miscFile)) {
      const data = fs.readFileSync(miscFile)
      const allZero = data.every(b => b === 0)
      win.webContents.send(ch, { type: 'output', data: allZero ? 'Bootloader LOCKED.' : 'Bootloader UNLOCKED.' })
    }
    win.webContents.send(ch, { type: 'done', code: 0 })
    return { success: true }
  })

  ipcMain.handle('unisoc:dump', async (event, args: { pkgId: string; device?: string }) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (!win) return { success: false }
    const ch = 'unisoc:dump:output'
    const pkg = PACKAGES[args.pkgId]
    if (!pkg) return { success: false, error: 'Unknown package' }
    const root = getUnisocRoot()
    const pkgDir = root ? path.join(root, pkg.id) : null
    if (!pkgDir || !fs.existsSync(pkgDir)) return { success: false, error: 'Package not installed' }
    const deviceDir = args.device ? path.join(pkgDir, args.device) : null
    const work = prepareWork(pkgDir, deviceDir && fs.existsSync(deviceDir) ? deviceDir : null, pkg.files, pkg.fdl1, pkg.fdl2, pkg.cboot)
    const spdDump = resolveSpdDump(pkgDir)
    if (!spdDump) return { success: false, error: 'spd_dump.exe not found' }
    const dumpDir = path.join(os.homedir(), 'Downloads', 'v1per_unisoc_dump')
    fs.mkdirSync(dumpDir, { recursive: true })
    win.webContents.send(ch, { type: 'start', operation: 'dump' })
    win.webContents.send(ch, { type: 'output', data: `Dumping to ${dumpDir}` })
    await runSpdDumpLive(spdDump, [...buildBaseTokens(true, pkg), 'path', dumpDir, 'r', 'all', 'reset'], work, win, ch)
    win.webContents.send(ch, { type: 'output', data: 'Dump finished.' })
    win.webContents.send(ch, { type: 'done', code: 0 })
    return { success: true }
  })

  ipcMain.handle('unisoc:flash', async (event, args: { pkgId: string; device?: string; partition: string; image: string }) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (!win) return { success: false }
    const ch = 'unisoc:flash:output'
    if (!fs.existsSync(args.image)) return { success: false, error: 'Image not found' }
    const pkg = PACKAGES[args.pkgId]
    if (!pkg) return { success: false, error: 'Unknown package' }
    const root = getUnisocRoot()
    const pkgDir = root ? path.join(root, pkg.id) : null
    if (!pkgDir || !fs.existsSync(pkgDir)) return { success: false, error: 'Package not installed' }
    const deviceDir = args.device ? path.join(pkgDir, args.device) : null
    const work = prepareWork(pkgDir, deviceDir && fs.existsSync(deviceDir) ? deviceDir : null, pkg.files, pkg.fdl1, pkg.fdl2, pkg.cboot)
    const spdDump = resolveSpdDump(pkgDir)
    if (!spdDump) return { success: false, error: 'spd_dump.exe not found' }
    win.webContents.send(ch, { type: 'start', operation: 'flash' })
    win.webContents.send(ch, { type: 'output', data: `Flashing ${args.partition}` })
    await runSpdDumpLive(spdDump, [...buildBaseTokens(true, pkg), 'w', args.partition, args.image, 'reset'], work, win, ch)
    win.webContents.send(ch, { type: 'output', data: `Write to ${args.partition} finished.` })
    win.webContents.send(ch, { type: 'done', code: 0 })
    return { success: true }
  })

  ipcMain.handle('unisoc:erase', async (event, args: { pkgId: string; device?: string; partition: string }) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (!win) return { success: false }
    const ch = 'unisoc:erase:output'
    const pkg = PACKAGES[args.pkgId]
    if (!pkg) return { success: false, error: 'Unknown package' }
    const root = getUnisocRoot()
    const pkgDir = root ? path.join(root, pkg.id) : null
    if (!pkgDir || !fs.existsSync(pkgDir)) return { success: false, error: 'Package not installed' }
    const deviceDir = args.device ? path.join(pkgDir, args.device) : null
    const work = prepareWork(pkgDir, deviceDir && fs.existsSync(deviceDir) ? deviceDir : null, pkg.files, pkg.fdl1, pkg.fdl2, pkg.cboot)
    const spdDump = resolveSpdDump(pkgDir)
    if (!spdDump) return { success: false, error: 'spd_dump.exe not found' }
    win.webContents.send(ch, { type: 'start', operation: 'erase' })
    win.webContents.send(ch, { type: 'output', data: `Erasing ${args.partition}` })
    await runSpdDumpLive(spdDump, [...buildBaseTokens(true, pkg), 'e', args.partition, 'reset'], work, win, ch)
    win.webContents.send(ch, { type: 'output', data: `Erase of ${args.partition} finished.` })
    win.webContents.send(ch, { type: 'done', code: 0 })
    return { success: true }
  })

  ipcMain.handle('unisoc:parts', async (event, args: { pkgId: string; device?: string }) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (!win) return { success: false }
    const ch = 'unisoc:parts:output'
    const pkg = PACKAGES[args.pkgId]
    if (!pkg) return { success: false, error: 'Unknown package' }
    const root = getUnisocRoot()
    const pkgDir = root ? path.join(root, pkg.id) : null
    if (!pkgDir || !fs.existsSync(pkgDir)) return { success: false, error: 'Package not installed' }
    const deviceDir = args.device ? path.join(pkgDir, args.device) : null
    const work = prepareWork(pkgDir, deviceDir && fs.existsSync(deviceDir) ? deviceDir : null, pkg.files, pkg.fdl1, pkg.fdl2, pkg.cboot)
    const spdDump = resolveSpdDump(pkgDir)
    if (!spdDump) return { success: false, error: 'spd_dump.exe not found' }
    const listFile = path.join(work, 'partition_list.txt')
    win.webContents.send(ch, { type: 'start', operation: 'parts' })
    await runSpdDumpLive(spdDump, [...buildBaseTokens(true, pkg), 'path', work, 'partition_list', listFile, 'p', 'reset'], work, win, ch)
    if (fs.existsSync(listFile)) {
      const content = fs.readFileSync(listFile, 'utf-8')
      win.webContents.send(ch, { type: 'partitions', data: content })
    }
    win.webContents.send(ch, { type: 'done', code: 0 })
    return { success: true }
  })

  ipcMain.handle('unisoc:erasefrp', async (event, args: { pkgId: string; device?: string }) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (!win) return { success: false }
    const ch = 'unisoc:erasefrp:output'
    const pkg = PACKAGES[args.pkgId]
    if (!pkg) return { success: false, error: 'Unknown package' }
    const root = getUnisocRoot()
    const pkgDir = root ? path.join(root, pkg.id) : null
    if (!pkgDir || !fs.existsSync(pkgDir)) return { success: false, error: 'Package not installed' }
    const deviceDir = args.device ? path.join(pkgDir, args.device) : null
    const work = prepareWork(pkgDir, deviceDir && fs.existsSync(deviceDir) ? deviceDir : null, pkg.files, pkg.fdl1, pkg.fdl2, pkg.cboot)
    const spdDump = resolveSpdDump(pkgDir)
    if (!spdDump) return { success: false, error: 'spd_dump.exe not found' }
    win.webContents.send(ch, { type: 'start', operation: 'erasefrp' })
    win.webContents.send(ch, { type: 'output', data: 'Erasing FRP...' })
    await runSpdDumpLive(spdDump, [...buildBaseTokens(true, pkg), 'e', 'frp', 'reset'], work, win, ch)
    win.webContents.send(ch, { type: 'output', data: 'FRP erase finished.' })
    win.webContents.send(ch, { type: 'done', code: 0 })
    return { success: true }
  })

  ipcMain.handle('unisoc:select-file', async () => {
    const win = getMainWindow()
    if (!win) return null
    const result = await dialog.showOpenDialog(win, {
      title: 'Select image file',
      filters: [{ name: 'Image files', extensions: ['img', 'bin', 'pac'] }],
      properties: ['openFile'],
    })
    return result.canceled ? null : result.filePaths[0]
  })

  ipcMain.handle('unisoc:confirm', async (_, message: string) => {
    const win = getMainWindow()
    if (!win) return false
    const result = await dialog.showMessageBox(win, {
      type: 'warning',
      buttons: ['Cancel', 'Continue'],
      defaultId: 0,
      title: 'Confirm Action',
      message,
    })
    return result.response === 1
  })

  log.info('[Unisoc] IPC handlers registered')
}
