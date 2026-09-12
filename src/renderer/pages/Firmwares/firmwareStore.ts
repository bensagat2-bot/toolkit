import { ref } from 'vue'

export interface FirmwareItem {
  id: number
  codename?: string
  device_name?: string
  project?: string
  brand?: string
  version: string
  android?: string
  method?: string
  size?: string
  link?: string
  date?: string
  region?: string
  mainboard?: string
  network_disk_link?: string
  extraction_code?: string
  platform?: string
  has_link: boolean
}

export interface FirmwareState {
  items: FirmwareItem[]
  total: number
  page: number
  pages: number
  loading: boolean
  source: 'xiaomi' | 'transsion'
  search: string
  credits: number
}

const state = ref<FirmwareState>({
  items: [],
  total: 0,
  page: 1,
  pages: 1,
  loading: false,
  source: 'xiaomi',
  search: '',
  credits: 0,
})

export function useFirmwareStore() {
  return { state }
}
