<template>
  <div class="drivers-page">
    <div class="page-header">
      <h2>Drivers</h2>
      <p class="subtitle">Essential drivers for Android device servicing</p>
    </div>

    <div class="drivers-grid">
      <div v-for="driver in drivers" :key="driver.id" class="driver-card" @click="selectDriver(driver)">
        <div class="card-image" :style="{ background: driver.gradient }">
          <div class="floating-icon" v-html="driver.icon"></div>
          <div class="pulse-ring"></div>
        </div>
        <div class="card-content">
          <h3>{{ driver.name }}</h3>
          <p class="card-desc">{{ driver.description }}</p>
          <div class="card-meta">
            <span class="version">{{ driver.version }}</span>
            <span class="size">{{ driver.size }}</span>
          </div>
        </div>
        <div class="card-actions">
          <button class="btn-download" @click.stop="downloadDriver(driver)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
            </svg>
            Download
          </button>
          <button class="btn-info" @click.stop="showInfo(driver)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4M12 8h.01"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div v-if="selectedDriver" class="driver-detail">
      <div class="detail-header">
        <div class="detail-icon" :style="{ background: selectedDriver.gradient }" v-html="selectedDriver.icon"></div>
        <div>
          <h3>{{ selectedDriver.name }}</h3>
          <p>{{ selectedDriver.fullDescription }}</p>
        </div>
        <button class="btn-close" @click="selectedDriver = null">&times;</button>
      </div>
      <div class="detail-steps">
        <h4>Installation Steps</h4>
        <ol>
          <li v-for="(step, i) in selectedDriver.steps" :key="i">{{ step }}</li>
        </ol>
      </div>
      <div class="detail-files">
        <h4>Included Files</h4>
        <div v-for="file in selectedDriver.files" :key="file" class="file-item">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
            <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8"/>
          </svg>
          {{ file }}
        </div>
      </div>
      <button class="btn-download-lg" @click="downloadDriver(selectedDriver)">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
        </svg>
        Download {{ selectedDriver.name }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const selectedDriver = ref(null)

const drivers = [
  {
    id: 'cdc',
    name: 'USB CDC Driver',
    description: 'Universal serial bus driver for Android devices in fastboot/download mode',
    fullDescription: 'Required for Windows to recognize Android devices in fastboot and download mode. Install before connecting your device.',
    version: 'v1.0',
    size: '2.1 MB',
    gradient: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    icon: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M15 9H9v6h6V9zm-2 4h-2v-2h2v2zm8-2V9h-2V7c0-1.1-.9-2-2-2h-2V3h-2v2h-2V3H9v2H7c-1.1 0-2 .9-2 2v2H3v2h2v2H3v2h2v2c0 1.1.9 2 2 2h2v2h2v-2h2v2h2v-2h2c1.1 0 2-.9 2-2v-2h2v-2h-2v-2h2z"/></svg>',
    steps: [
      'Download and extract the driver package',
      'Open Device Manager on your PC',
      'Connect your Android device in fastboot mode',
      'Right-click the unknown device and select "Update driver"',
      'Browse to the extracted folder and install',
    ],
    files: ['android_winusb.inf', 'WdfCoInstaller01009.dll', 'WUDFUpdate_01009.dll', 'dpinst.exe'],
  },
  {
    id: 'spreadsrd',
    name: 'Spreadtrum SPD Driver',
    description: 'Official SPD/Unisoc USB driver for download mode communication',
    fullDescription: 'Required for Unisoc/Spreadtrum devices to communicate in download mode. Used for flashing firmware and bootloader unlock.',
    version: 'v1.0',
    size: '5.8 MB',
    gradient: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
    icon: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>',
    steps: [
      'Extract the SPD driver package',
      'Run dpinst.exe as administrator',
      'Follow the installation wizard',
      'Connect your Unisoc device in download mode',
      'Windows will automatically detect and install the driver',
    ],
    files: ['spd_dump.exe', 'dpinst.exe', 'android.inf', 'WdfCoInstaller.dll'],
  },
  {
    id: 'mtk',
    name: 'MediaTek VCOM Driver',
    description: 'VCOM port driver for MediaTek preloader and BROM mode',
    fullDescription: 'Required for MediaTek devices to communicate in preloader/BROM mode. Used for deep-level flashing and repairs.',
    version: 'v1.0',
    size: '3.2 MB',
    gradient: 'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
    icon: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>',
    steps: [
      'Download the MediaTek driver package',
      'Run the installer as administrator',
      'Restart your computer after installation',
      'Connect your MediaTek device in preloader mode',
      'Device Manager should show "MediaTek USB VCOM" port',
    ],
    files: ['mtk_driver.inf', 'usbvcom.sys', 'WdfCoInstaller.dll', 'dpinst.exe'],
  },
  {
    id: 'qualcomm',
    name: 'Qualcomm QDLoader 9008',
    description: 'EDL mode driver for Qualcomm devices emergency download',
    fullDescription: 'Required for Qualcomm devices in EDL (Emergency Download) mode. Used for unbricking and deep-level repairs.',
    version: 'v1.0',
    size: '4.5 MB',
    gradient: 'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
    icon: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>',
    steps: [
      'Extract the Qualcomm driver package',
      'Open Device Manager',
      'Boot your device into EDL mode (Vol+ Vol- or test points)',
      'Connect USB cable',
      'Right-click unknown device, select "Update driver"',
      'Browse to the extracted Qualcomm driver folder',
    ],
    files: ['qcser.inf', 'qusb_bser.sys', 'WdfCoInstaller.dll', 'dpinst.exe'],
  },
  {
    id: 'adb',
    name: 'ADB & Fastboot',
    description: 'Android Debug Bridge and Fastboot tools for device communication',
    fullDescription: 'Essential command-line tools for Android device communication. Used across all Android servicing operations.',
    version: 'v35.0',
    size: '12 MB',
    gradient: 'linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)',
    icon: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M20 3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H4V5h16v14zM6 7h5v2H6V7zm0 4h5v2H6v-2zm0 4h5v2H6v-2zm7-8h5v10h-5V7z"/></svg>',
    steps: [
      'Download the platform-tools package',
      'Extract to C:\\platform-tools',
      'Add C:\\platform-tools to system PATH',
      'Open terminal and run "adb version" to verify',
    ],
    files: ['adb.exe', 'fastboot.exe', 'AdbWinApi.dll', 'AdbWinUsbApi.dll'],
  },
  {
    id: 'libusb',
    name: 'LibUSB Filter Driver',
    description: 'USB filter driver for direct device communication bypassing Windows drivers',
    fullDescription: 'Required for tools that need direct USB access to devices, bypassing standard Windows drivers. Used by some exploit tools.',
    version: 'v1.2',
    size: '1.8 MB',
    gradient: 'linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%)',
    icon: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/></svg>',
    steps: [
      'Download and run the LibUSB installer',
      'Connect your device',
      'Open Filter Wizard from the start menu',
      'Select your device and install the filter driver',
    ],
    files: ['libusb-win32-devel-filter-1.2.6.0.exe', 'install-filter.exe', 'libusb0.dll'],
  },
]

function selectDriver(driver) {
  selectedDriver.value = driver
}

async function downloadDriver(driver) {
  try {
    await sendIpcToMain('open_url', `https://github.com/coolishsec0175/toolkit/releases/tag/drivers-${driver.id}`)
  } catch {
    console.log('Download:', driver.name)
  }
}

function showInfo(driver) {
  selectedDriver.value = driver
}
</script>

<style lang="less" scoped>
.drivers-page {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.page-header {
  margin-bottom: 24px;

  h2 {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 4px 0;
  }

  .subtitle {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
  }
}

.drivers-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.driver-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s ease;

  &:hover {
    border-color: var(--accent-primary);
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }
}

.card-image {
  height: 100px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;

  .floating-icon {
    width: 56px;
    height: 56px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(10px);
    animation: float 3s ease-in-out infinite;
    position: relative;
    z-index: 1;

    svg {
      width: 32px;
      height: 32px;
      color: white;
    }
  }

  .pulse-ring {
    position: absolute;
    width: 80px;
    height: 80px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-radius: 50%;
    animation: pulse 2s ease-out infinite;
  }
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

@keyframes pulse {
  0% { transform: scale(0.8); opacity: 1; }
  100% { transform: scale(1.5); opacity: 0; }
}

.card-content {
  padding: 16px;

  h3 {
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 8px 0;
  }

  .card-desc {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0 0 12px 0;
  }

  .card-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--text-secondary);

    span {
      display: flex;
      align-items: center;
      gap: 4px;
    }
  }
}

.card-actions {
  padding: 0 16px 16px;
  display: flex;
  gap: 8px;
}

.btn-download {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--accent-primary);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: opacity 0.2s;

  &:hover { opacity: 0.9; }

  svg { width: 14px; height: 14px; }
}

.btn-info {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.2s;

  &:hover { border-color: var(--accent-primary); }

  svg { width: 16px; height: 16px; color: var(--text-secondary); }
}

.driver-detail {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  padding: 24px;
}

.detail-header {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 20px;

  .detail-icon {
    width: 64px;
    height: 64px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;

    svg { width: 32px; height: 32px; color: white; }
  }

  div { flex: 1; }
  h3 { font-size: 18px; margin: 0 0 8px 0; }
  p { font-size: 13px; color: var(--text-secondary); margin: 0; line-height: 1.5; }
}

.btn-close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  font-size: 20px;
  cursor: pointer;

  &:hover { background: var(--bg-tertiary); }
}

.detail-steps, .detail-files {
  margin-bottom: 20px;

  h4 {
    font-size: 13px;
    font-weight: 600;
    margin: 0 0 12px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  ol, ul {
    margin: 0;
    padding-left: 20px;
  }

  li {
    font-size: 13px;
    line-height: 1.8;
    color: var(--text-primary);
  }
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  font-size: 12px;
  font-family: monospace;
  margin-bottom: 6px;

  svg { width: 14px; height: 14px; color: var(--text-secondary); flex-shrink: 0; }
}

.btn-download-lg {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 20px;
  background: var(--accent-primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.2s;

  &:hover { opacity: 0.9; }

  svg { width: 18px; height: 18px; }
}
</style>
