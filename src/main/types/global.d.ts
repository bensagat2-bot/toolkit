
/// <reference types="node" />

declare module NodeJS {
  interface Global {
    isDev: boolean
    envParams: LX.EnvParams
    staticPath: string
    lxDataPath: string
    lxOldDataPath: string
    lx: Lx
    appWorder: AppWorder
    [key: string]: any
  }
}

declare const webpackStaticPath: string
declare const webpackUserApiPath: string

