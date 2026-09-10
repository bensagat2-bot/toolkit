declare namespace LX {
  interface CmdParams {
    /**
     * 禁用硬件加速启动
     */
    dha?: boolean

    /**
     * 以非透明模式启动
     */
    dt?: boolean

    /**
     * 启动后最小化到系统托盘
     */
    hidden?: boolean

    [key: string]: boolean | number | string
  }

  interface EnvParams {
    deeplink?: string | null
    cmdParams: CmdParams
    workAreaSize?: Electron.Size
  }

  type UpdateStatus = 'downloaded' | 'downloading' | 'error' | 'checking' | 'idle'
  interface VersionInfo {
    version: string
    desc: string
  }
}