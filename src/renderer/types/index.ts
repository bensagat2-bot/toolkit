// ── Device Types ─────────────────────────────────────────────

export type DeviceMode = 'none' | 'adb' | 'fastboot' | 'download'

export interface DetectResult {
  mode: DeviceMode
  serial: string | null
}

// ── Unisoc Types ─────────────────────────────────────────────

export interface UnisocPackage {
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

export interface UnisocPackages {
  [key: string]: boolean
}

// ── MediaTek Types ───────────────────────────────────────────

export interface ScatterEntry {
  name: string
  start: string
  size: string
  type: string
}

export interface ScatterFile {
  path: string
  entries: ScatterEntry[]
}

// ── Common Types ─────────────────────────────────────────────

export interface LogEntry {
  text: string
  type: 'info' | 'success' | 'warn' | 'error' | 'system'
  timestamp: number
}

export interface OperationState {
  running: boolean
  progress: number
  currentOp: string
}

// ── Settings Types ───────────────────────────────────────────

export interface AppSettings {
  theme: string
  language: string
  fontSize: number
}

// ── IPC Event Payloads ───────────────────────────────────────

export interface OutputPayload {
  type: 'output' | 'error' | 'done' | 'start' | 'partitions'
  data?: string
  code?: number
  operation?: string
}
