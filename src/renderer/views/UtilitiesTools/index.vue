<template>
  <div class="utilities-tools">
    <div class="tools-header">
      <h2>Utilities Tools</h2>
      <div class="device-status">
        <span :class="['status-dot', deviceConnected ? 'connected' : 'disconnected']"></span>
        {{ deviceConnected ? 'Device Connected' : 'No Device' }}
      </div>
    </div>

    <div class="tools-section">
      <h3>ADB Tools</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="adbDevices">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="4" y="4" width="16" height="16" rx="2" />
              <rect x="9" y="9" width="6" height="6" />
            </svg>
          </div>
          <div class="tool-name">ADB Devices</div>
          <div class="tool-desc">List connected devices</div>
        </div>
        <div class="tool-card" @click="adbShell">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </div>
          <div class="tool-name">ADB Shell</div>
          <div class="tool-desc">Open ADB shell</div>
        </div>
        <div class="tool-card" @click="adbReboot">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </div>
          <div class="tool-name">ADB Reboot</div>
          <div class="tool-desc">Reboot device</div>
        </div>
        <div class="tool-card" @click="adbSideload">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l3-3m-3 3H9" />
            </svg>
          </div>
          <div class="tool-name">ADB Sideload</div>
          <div class="tool-desc">Sideload ZIP file</div>
        </div>
      </div>
    </div>

    <div class="tools-section">
      <h3>Fastboot Tools</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="fastbootDevices">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="4" y="4" width="16" height="16" rx="2" />
              <rect x="9" y="9" width="6" height="6" />
            </svg>
          </div>
          <div class="tool-name">Fastboot Devices</div>
          <div class="tool-desc">List fastboot devices</div>
        </div>
        <div class="tool-card" @click="fastbootFlash">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
            </svg>
          </div>
          <div class="tool-name">Fastboot Flash</div>
          <div class="tool-desc">Flash partition via fastboot</div>
        </div>
        <div class="tool-card" @click="fastbootReboot">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </div>
          <div class="tool-name">Fastboot Reboot</div>
          <div class="tool-desc">Reboot from fastboot</div>
        </div>
        <div class="tool-card" @click="fastbootOem">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <div class="tool-name">Fastboot OEM</div>
          <div class="tool-desc">Run OEM commands</div>
        </div>
      </div>
    </div>

    <div class="tools-section">
      <h3>Device Info</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="rootCheck">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div class="tool-name">Root Check</div>
          <div class="tool-desc">Check root status</div>
        </div>
        <div class="tool-card" @click="deviceInfo">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          <div class="tool-name">Device Info</div>
          <div class="tool-desc">Get device information</div>
        </div>
        <div class="tool-card" @click="batteryInfo">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <div class="tool-name">Battery Info</div>
          <div class="tool-desc">Get battery status</div>
        </div>
        <div class="tool-card" @click="storageInfo">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
            </svg>
          </div>
          <div class="tool-name">Storage Info</div>
          <div class="tool-desc">Get storage information</div>
        </div>
      </div>
    </div>

    <div class="tools-section">
      <h3>Utilities</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="openScrcpy">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
              <line x1="12" y1="18" x2="12" y2="18" />
            </svg>
          </div>
          <div class="tool-name">Scrcpy</div>
          <div class="tool-desc">Mirror Android screen</div>
        </div>
        <div class="tool-card" @click="openDrivers">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" />
            </svg>
          </div>
          <div class="tool-name">Drivers</div>
          <div class="tool-desc">Download device drivers</div>
        </div>
        <div class="tool-card" @click="openScatter">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="7" height="7" />
              <rect x="14" y="3" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" />
            </svg>
          </div>
          <div class="tool-name">Scatter Editor</div>
          <div class="tool-desc">Parse scatter file</div>
        </div>
        <div class="tool-card" @click="openAnykernel">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
              <line x1="8" y1="21" x2="16" y2="21" />
              <line x1="12" y1="17" x2="12" y2="21" />
            </svg>
          </div>
          <div class="tool-name">Anykernel</div>
          <div class="tool-desc">Pack anykernel flashable zip</div>
        </div>
      </div>
    </div>

    <div class="command-output" v-if="commandOutput">
      <div class="output-header">
        <span>Command Output</span>
        <button @click="clearOutput" class="clear-btn">Clear</button>
      </div>
      <pre>{{ commandOutput }}</pre>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { sendIpcToMain } from '@renderer/utils'

const deviceConnected = ref(false)
const commandOutput = ref('')

const updateDeviceStatus = (connected) => {
  deviceConnected.value = connected
}

onMounted(() => {
  window.api?.on?.('device-status', updateDeviceStatus)
})

onUnmounted(() => {
  window.api?.off?.('device-status', updateDeviceStatus)
})

const adbDevices = () => sendIpcToMain('util-adb-devices')
const adbShell = () => sendIpcToMain('util-adb-shell')
const adbReboot = () => sendIpcToMain('util-adb-reboot')
const adbSideload = () => sendIpcToMain('util-adb-sideload')

const fastbootDevices = () => sendIpcToMain('util-fastboot-devices')
const fastbootFlash = () => sendIpcToMain('util-fastboot-flash')
const fastbootReboot = () => sendIpcToMain('util-fastboot-reboot')
const fastbootOem = () => sendIpcToMain('util-fastboot-oem')

const rootCheck = () => sendIpcToMain('util-root-check')
const deviceInfo = () => sendIpcToMain('util-device-info')
const batteryInfo = () => sendIpcToMain('util-battery-info')
const storageInfo = () => sendIpcToMain('util-storage-info')

const openScrcpy = () => sendIpcToMain('util-scrcpy')
const openDrivers = () => sendIpcToMain('util-drivers')
const openScatter = () => sendIpcToMain('util-scatter')
const openAnykernel = () => sendIpcToMain('util-anykernel')

const clearOutput = () => {
  commandOutput.value = ''
}
</script>

<style lang="less" scoped>
.utilities-tools {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.tools-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  h2 {
    font-size: 18px;
    font-weight: 500;
    color: var(--text-primary);
  }
}

.device-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;

  &.connected {
    background: #4caf50;
  }

  &.disconnected {
    background: #f44336;
  }
}

.tools-section {
  margin-bottom: 24px;

  h3 {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.tool-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover {
    border-color: var(--accent-primary);
    background: var(--bg-tertiary);
    transform: translateY(-2px);
  }
}

.tool-icon {
  width: 40px;
  height: 40px;
  margin-bottom: 12px;
  color: var(--accent-secondary);

  svg {
    width: 100%;
    height: 100%;
  }
}

.tool-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.tool-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

.command-output {
  margin-top: 24px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  overflow: hidden;
}

.output-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-primary);

  span {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
}

.clear-btn {
  background: none;
  border: none;
  color: var(--accent-primary);
  cursor: pointer;
  font-size: 12px;

  &:hover {
    text-decoration: underline;
  }
}

pre {
  padding: 16px;
  margin: 0;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
  color: var(--text-primary);
  overflow-x: auto;
  max-height: 200px;
  overflow-y: auto;
}
</style>
