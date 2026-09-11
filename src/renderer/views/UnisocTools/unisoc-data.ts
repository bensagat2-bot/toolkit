export const SPRD_VID = 0x1782

export const NV_PROTECTED = ['nvram', 'nvdata', 'nvcfg', 'prodnv', 'modem', 'miscdata', 'dsp', 'wcn', 'lte']

export const SPRD_KEYWORDS = ['Spreadtrum', 'Unisoc', 'sdboot', 'ums', 'sc']

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

export const PACKAGES: Record<string, UnisocPackage> = {
  ums9230: {
    id: 'ums9230',
    name: 'UMS9230 (T606/T612)',
    execAddr: 0x65015f08,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x65000800,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin',
    splLoaderBk: 'splloader_bk.bin',
    miscDone: 'misc-ubldone.bin',
    chsizeUboot: false,
    toolsGen: 'gen1',
    erasePersist: true,
    backupPartitions: ['boot', 'init_boot', 'vendor_boot', 'prodnv'],
    files: [
      'custom_exec_no_verify_65015f08.bin', 'fdl1-dl.bin', 'fdl2-dl.bin',
      'fdl2-cboot.bin', 'splloader_bk.bin', 'misc-ubldone.bin', 'misc-wipe.bin',
    ],
  },
  sc9863a: {
    id: 'sc9863a',
    name: 'SC9863A',
    execAddr: 0x4ee8,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x5000,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin',
    splLoaderBk: null,
    miscDone: 'misc-wipe.bin',
    chsizeUboot: true,
    toolsGen: 'gen1',
    erasePersist: false,
    backupPartitions: ['boot', 'prodnv'],
    files: [
      'custom_exec_no_verify_4ee8.bin', 'fdl1-dl.bin', 'fdl2-dl.bin',
      'fdl2-cboot.bin', 'misc-wipe.bin',
    ],
  },
  ums512: {
    id: 'ums512',
    name: 'UMS512 (T610/T700)',
    execAddr: 0x3ee8,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x5500,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin',
    splLoaderBk: null,
    miscDone: 'misc-wipe.bin',
    chsizeUboot: false,
    toolsGen: 'gen2',
    erasePersist: false,
    backupPartitions: ['boot', 'prodnv'],
    files: [
      'custom_exec_no_verify_3ee8.bin', 'fdl1-dl.bin', 'fdl2-dl.bin',
      'fdl2-cboot.bin', 'misc-wipe.bin',
    ],
  },
  ums9620: {
    id: 'ums9620',
    name: 'UMS9620',
    execAddr: 0x65012f48,
    fdl1: 'fdl1-dl.bin', fdl1Addr: 0x65000800,
    fdl2: 'fdl2-dl.bin', fdl2Addr: 0x9efffe00,
    cboot: 'fdl2-cboot.bin',
    splLoaderBk: null,
    miscDone: 'misc-wipe.bin',
    chsizeUboot: false,
    toolsGen: 'gen2',
    erasePersist: false,
    backupPartitions: ['boot', 'prodnv'],
    files: [
      'custom_exec_no_verify_65012f48.bin', 'fdl1-dl.bin', 'fdl2-dl.bin',
      'fdl2-cboot.bin', 'misc-wipe.bin',
    ],
  },
}

export const DEVICE_ALIASES: Record<string, string> = {
  hot40i: 'ums9230', hot30i: 'ums9230', smart8: 'ums9230',
  smart10: 'ums9230', hot12pro: 'ums9230',
  s25: 'ums9230', s23: 'ums9230', s25ultra: 'ums9230',
  city100: 'ums9230', p40plus: 'ums9230', p55nfc: 'ums9230',
  p65: 'ums9230', s23plus: 'ums9230', vision3plus: 'ums9230',
  vision5plus: 'ums9230', sketsa_3: 'ums9230',
  c31: 'ums9230', c33: 'ums9230', c35: 'ums9230',
  c51: 'ums9230', c53: 'ums9230', c61: 'ums9230',
  c71: 'ums9230', narzo50i_prime: 'ums9230', note50: 'ums9230',
  note60: 'ums9230', note60x: 'ums9230',
  kl4: 'ums9230', spark_8c: 'ums9230', spark_10c: 'ums9230',
  km4: 'ums9230', kn3: 'ums9230', a5: 'ums9230',
  y19s: 'ums9230', a35: 'ums9230', c32: 'ums9230',
  a70: 'sc9863a', a60s: 'sc9863a', a50: 'sc9863a',
  a80: 'sc9863a', a90: 'sc9863a', vision3: 'sc9863a',
  smart7: 'sc9863a', c11: 'sc9863a', narzo50i: 'sc9863a',
  blade_a31: 'sc9863a', blade_a5: 'sc9863a', blade_a51: 'sc9863a',
  blade_a52: 'sc9863a', blade_a7: 'sc9863a', blade_v2020: 'sc9863a',
  hot12play_nfc: 'ums512', c21y: 'ums512', c25y: 'ums512',
  tab_vx_lite_1013: 'ums512',
  nubia_neo_gt_3: 'ums9620', nubia_neo_2: 'ums9620',
}

export type DeviceMode = 'none' | 'adb' | 'fastboot' | 'download'

export interface DetectResult {
  mode: DeviceMode
  serial: string | null
}
