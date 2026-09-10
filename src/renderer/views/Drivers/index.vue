<template>
  <div :class="$style.container">
    <div :class="$style.sidebar">
      <div :class="$style.sidebarHeader">
        <h2 :class="$style.title">Drivers</h2>
      </div>
      <ul :class="$style.categoryList">
        <li
          v-for="category in categories"
          :key="category"
          :class="[$style.categoryItem, { [$style.active]: selectedCategory === category }]"
          @click="selectedCategory = category"
        >
          <span :class="$style.categoryLabel">{{ category }}</span>
        </li>
      </ul>
    </div>
    <div :class="$style.content">
      <div :class="$style.grid">
        <div
          v-for="driver in filteredDrivers"
          :key="driver.name"
          :class="$style.card"
        >
          <div :class="$style.cardIcon">
            <img v-if="driver.image" :src="driver.image" :alt="driver.name">
            <svg v-else viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 6h-4V4c0-1.1-.9-2-2-2h-4c-1.1 0-2 .9-2 2v2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm-6 0h-4V4h4v2zm2 12H8V8h8v10z"/>
            </svg>
          </div>
          <div :class="$style.cardContent">
            <div :class="$style.cardHeader">
              <h3 :class="$style.cardName">{{ driver.name }}</h3>
              <span v-if="driver.recommended" :class="$style.recommendedBadge">Recommended</span>
            </div>
            <p :class="$style.cardDesc">{{ driver.description }}</p>
            <div :class="$style.cardFooter">
              <span :class="$style.tagBadge">{{ driver.tag }}</span>
              <button :class="$style.downloadBtn" @click="downloadDriver(driver)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
                Download
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { ref, computed } from '@common/utils/vueTools'
import { sendIpcToMain } from '@renderer/utils'

interface Driver {
  category: string
  tag: string
  recommended?: boolean
  name: string
  description: string
  url: string
  image?: string
}

const driversData: Driver[] = [
  {
    category: "CHIPSET DRIVERS",
    tag: "MTK",
    recommended: true,
    name: "MediaTek MT65xx VCOM",
    description: "MediaTek VCOM USB driver. Required for SP Flash Tool, MTK Flash Tool, and DA flashing on Tecno, Infinix, Itel, Redmi, Realme MTK chipsets.",
    url: "https://github.com/CIPHERCHAN/TTSD-Releases/releases/download/TTSD-V3/MediaTek.MT65xx.GSM.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "MTK",
    name: "MediaTek VCOM Driver (alt)",
    description: "Alternate VCOM driver — for Add Legacy Hardware install when the main package fails.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/android-vcom-driver.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "QC",
    recommended: true,
    name: "Qualcomm QDLoader 9008 (x64)",
    description: "EDL mode driver. 64-bit signed setup. Required for QPST / QFIL / EDL flash.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Qualcomm_QDLoader_HS-USB_Driver_64bit_Setup.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "QC",
    name: "Qualcomm QDLoader 9008 (x86)",
    description: "EDL mode driver. 32-bit setup for legacy Windows installations.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Qualcomm_QDLoader_HS-USB_Driver_32bit_Setup.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "SPD",
    recommended: true,
    name: "Spreadtrum / Unisoc R4.20.4201",
    description: "SPD upgrade port driver for Itel feature phones and entry-level Unisoc-based Android. Newer build.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/SPD_Driver_R4.20.4201.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "SPD",
    name: "Spreadtrum / Unisoc R4.20.0201",
    description: "SPD upgrade port driver. Older build — use this if R4.20.4201 fails on a particular device.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/SPD_Driver_R4.20.0201.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "SPD",
    name: "Spreadtrum SCI USB2Serial",
    description: "Alternate Spreadtrum SCI USB-to-Serial driver for older devices.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Spreadtrum_SCI_USB2Serial.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "SPD",
    name: "Spreadtrum Jungo USB2Serial",
    description: "Jungo-based Spreadtrum USB-to-Serial driver. Used by older SPD flash tools.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Spreadtrum_Jungo_USB2Serial.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "RK",
    name: "Rockchip USB Driver v4.8",
    description: "For Rockchip-based tablets and Android boxes. Supports Maskrom / Loader modes.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Rockchip_DriverAssitant_v4.8.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "RK",
    name: "Rockchip USB Driver v4.4",
    description: "Older Rockchip driver — fallback when v4.8 has compatibility issues.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Rockchip-DriverAssitant-v4.4.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "AML",
    name: "Amlogic USB Driver",
    description: "For Amlogic Android TV boxes (S905/S912 etc). Used with USB Burning Tool.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Amlogic_Driver.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "INTEL",
    name: "Intel Android Driver v1.10.0",
    description: "Intel-based Android tablets (Asus Zenfone, older Lenovo).",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/intel-android-driver-1.10.0.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "NV",
    name: "NVIDIA Tegra Android Driver",
    description: "For NVIDIA Shield and Tegra-based Android devices.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/android-nvidia-driver-201506.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "BCM",
    name: "Broadcom Android Driver",
    description: "For Broadcom-chipset Android devices.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/android-broadcom-driver.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "RDA",
    name: "RDA / Coolsand Driver v1.2",
    description: "For RDA / Coolsand feature phones and budget Android.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/RDA_Driver_v1.2.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "RDA",
    name: "RDA Driver v1.2 (signed)",
    description: "Signed variant of the RDA driver — for Win 10/11 with driver signature enforcement.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/rda-driver-v1.2-signed.zip"
  },
  {
    category: "CHIPSET DRIVERS",
    tag: "CDC",
    name: "Android CDC Driver",
    description: "Generic Android CDC serial driver — for unbranded ADB/Fastboot via CDC class.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/android-cdc-driver.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "SAMSUNG",
    recommended: true,
    name: "Samsung USB Driver v1.5.51",
    description: "Official Samsung USB driver for mobile phones. Required for Odin and Smart Switch.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/SAMSUNG_USB_Driver_for_Mobile_Phones_v1.5.51.0.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "XIAOMI",
    name: "Xiaomi USB Driver",
    description: "Mi USB driver for Mi Flash, Fastboot, and EDL modes on Xiaomi/Redmi/POCO.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/xiaomi_usb_driver.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "HUAWEI",
    name: "Huawei Android USB Driver",
    description: "Huawei mobile driver for ADB, Fastboot, and unlock tools.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Huawei-Android-USB-Driver.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "HUAWEI",
    name: "Huawei HiSuite Driver",
    description: "Huawei HiSuite USB driver bundle. For HiSuite-based unlock and backup.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Huawei-USB-Drivers-HiSuite.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "HUAWEI",
    name: "HiSuite v5.0.1.300 (full)",
    description: "Full HiSuite installer with bundled USB drivers.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/HiSuite_5.0.1.300_OVE.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "HTC",
    name: "HTC USB Driver v4.17",
    description: "HTC USB driver for sync, fastboot, and S-OFF tools.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/HTC_Driver_4.17.0.001.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "LG",
    name: "LG Mobile Driver v4.5 (WHQL)",
    description: "WHQL-signed LG USB driver for LGUP / KDZ flashing.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/LG-Mobile-Driver_WHQL_Ver_4.5.0.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "LG",
    name: "LG Mobile Driver v4.2 (WHQL)",
    description: "Older LG USB driver — fallback for legacy LG devices.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/LG-Mobile-Driver_WHQL_Ver_4.2.0.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "MOTO",
    name: "Motorola Mobile Driver (x64)",
    description: "Motorola USB driver for ADB / Fastboot / RSD Lite. 64-bit Windows.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Motorola_Mobile_Drivers_64bit.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "MOTO",
    name: "Motorola Mobile Driver (x86)",
    description: "Motorola USB driver, 32-bit Windows.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Motorola_Mobile_Drivers_32bit.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "1+",
    name: "OnePlus USB Driver",
    description: "OnePlus official USB driver for ADB and Fastboot.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/OnePlus_USB_Drivers_Setup.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "ASUS",
    name: "ASUS Android USB Driver",
    description: "ASUS Android USB driver for ZenFone and Padfone.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/ASUS_Android_USB_drivers_for_Windows_20150212.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "CP",
    name: "Coolpad Driver v2.03",
    description: "Coolpad/Yulong USB driver, current version.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Coolpad_Driver_v2.03.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "CP",
    name: "Coolpad Setup (140827)",
    description: "Older Coolpad bundle setup — fallback for legacy devices.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Coolpad-140827-Setup.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "ZTE",
    name: "ZTE Android USB Driver",
    description: "ZTE handset USB driver.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/ZTE-Android-USB-Driver.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "AMZ",
    name: "Amazon Kindle Fire USB Driver",
    description: "For Kindle Fire tablets — ADB / Fastboot / Recovery.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/kindle_fire_usb_driver.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "LE",
    name: "Lenovo LePhone Driver",
    description: "Lenovo LePhone USB driver bundle.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/LePhone2.0.zip"
  },
  {
    category: "BRAND DRIVERS",
    tag: "MOB",
    name: "Mobistel Driver v5.1417",
    description: "Mobistel Cynus USB driver.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/mobistel_driver_v5.1417.00.zip"
  },
  {
    category: "ADB / FASTBOOT TOOLKIT",
    tag: "GOOGLE",
    recommended: true,
    name: "Google USB Driver (r13)",
    description: "Official Google USB driver. Required for ADB on Pixel and AOSP devices.",
    url: "https://dl.google.com/android/repository/usb_driver_r13-windows.zip"
  },
  {
    category: "ADB / FASTBOOT TOOLKIT",
    tag: "ADB",
    recommended: true,
    name: "ADB & Fastboot Platform Tools",
    description: "Always-current Platform Tools from Google. ADB + Fastboot + dmtracedump + etc.",
    url: "https://dl.google.com/android/repository/platform-tools-latest-windows.zip"
  },
  {
    category: "ADB / FASTBOOT TOOLKIT",
    tag: "ADB",
    name: "Latest USB Driver (multi-OEM)",
    description: "Combined OEM USB driver pack from Google (older).",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/latest_usb_driver_windows.zip"
  },
  {
    category: "ADB / FASTBOOT TOOLKIT",
    tag: "ADB",
    name: "ADB Setup v1.4.3",
    description: "One-click ADB + Fastboot installer with PATH setup.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/adb-setup-1.4.3.zip"
  },
  {
    category: "ADB / FASTBOOT TOOLKIT",
    tag: "ADB",
    name: "Minimal ADB & Fastboot v1.4.3",
    description: "Lightweight ADB + Fastboot installer. Minimal footprint.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/minimal_adb_fastboot_v1.4.3_setup.zip"
  },
  {
    category: "ADB / FASTBOOT TOOLKIT",
    tag: "ADB",
    name: "Universal Naked Driver v0.73",
    description: "Generic ADB driver with no manufacturer-specific assumptions.",
    url: "https://github.com/gsmusbdrivers/usbdrivers/raw/master/Universal_Naked_Driver_0.73.zip"
  }
]

export default {
  name: 'Drivers',
  setup() {
    const selectedCategory = ref('All')

    const categories = computed(() => {
      const cats = [...new Set(driversData.map(d => d.category))]
      return ['All', ...cats]
    })

    const filteredDrivers = computed(() => {
      if (selectedCategory.value === 'All') {
        return driversData
      }
      return driversData.filter(d => d.category === selectedCategory.value)
    })

    const downloadDriver = (driver: Driver) => {
      sendIpcToMain('util-open-url', driver.url)
    }

    return {
      categories,
      selectedCategory,
      filteredDrivers,
      downloadDriver,
    }
  },
}
</script>

<style lang="less" module>
@import '@renderer/assets/styles/layout.less';

.container {
  overflow: hidden;
  height: 100%;
  display: flex;
  position: relative;
}

.sidebar {
  flex: none;
  width: 220px;
  display: flex;
  flex-flow: column nowrap;
  border-right: 1px solid var(--color-border);
  background-color: var(--color-main-background);
}

.sidebarHeader {
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.title {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-primary);
  margin: 0;
}

.categoryList {
  flex: auto;
  overflow-y: auto;
  padding: 8px 0;
  margin: 0;
  list-style: none;
}

.categoryItem {
  padding: 10px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
  color: var(--color-font);
  
  &:hover {
    background-color: var(--color-button-background-hover);
  }
  
  &.active {
    background-color: var(--color-primary-background-hover);
    color: var(--color-primary);
    border-left: 3px solid var(--color-primary);
    padding-left: 13px;
  }
}

.categoryLabel {
  font-size: 13px;
  display: block;
  .mixin-ellipsis-1();
}

.content {
  flex: auto;
  overflow-y: auto;
  padding: 20px;
  background-color: var(--color-main-background);
}

.grid {
  display: flex;
  flex-flow: row wrap;
  justify-content: flex-start;
  gap: 16px;
}

.card {
  width: calc(33.333% - 12px);
  min-width: 280px;
  background-color: var(--color-secondary-background);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: transform 0.2s, box-shadow 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  
  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
}

.cardIcon {
  width: 100%;
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-primary-background);
  color: var(--color-primary);
  
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  
  svg {
    width: 48px;
    height: 48px;
    opacity: 0.7;
  }
}

.cardContent {
  padding: 12px;
  display: flex;
  flex-direction: column;
  flex: auto;
}

.cardHeader {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 8px;
}

.cardName {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-font);
  margin: 0;
  flex: auto;
  .mixin-ellipsis-2();
}

.recommendedBadge {
  flex: none;
  font-size: 10px;
  padding: 2px 6px;
  background-color: var(--color-primary);
  color: var(--color-primary-font);
  border-radius: 4px;
  white-space: nowrap;
}

.cardDesc {
  font-size: 12px;
  color: var(--color-font-label);
  line-height: 1.4;
  margin: 0 0 12px;
  flex: auto;
  .mixin-ellipsis-3();
}

.cardFooter {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: auto;
}

.tagBadge {
  font-size: 11px;
  padding: 3px 8px;
  background-color: var(--color-button-background);
  color: var(--color-font-label);
  border-radius: 4px;
}

.downloadBtn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background-color: var(--color-primary);
  color: var(--color-primary-font);
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: opacity 0.2s;
  
  &:hover {
    opacity: 0.8;
  }
  
  svg {
    width: 14px;
    height: 14px;
  }
}
</style>
