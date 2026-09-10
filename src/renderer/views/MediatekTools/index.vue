<template>
  <div class="mediatek-tools">
    <div class="tools-header">
      <h2>MediaTek Tools</h2>
      <div class="device-status">
        <span :class="['status-dot', deviceConnected ? 'connected' : 'disconnected']"></span>
        {{ deviceConnected ? 'Device Connected' : 'No Device' }}
      </div>
    </div>

    <div class="tools-section">
      <h3>Flash Tools</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="openScatterFlasher">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <path d="M3 9h18M9 3v18" />
            </svg>
          </div>
          <div class="tool-name">Scatter Flasher</div>
          <div class="tool-desc">Flash firmware using scatter file</div>
        </div>
        <div class="tool-card" @click="openDAFlasher">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 2v4m0 12v4M2 12h4m12 0h4" />
              <circle cx="12" cy="12" r="4" />
            </svg>
          </div>
          <div class="tool-name">DA Flasher</div>
          <div class="tool-desc">Flash via DA (Download Agent)</div>
        </div>
        <div class="tool-card" @click="openPreloaderFlash">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
            </svg>
          </div>
          <div class="tool-name">Preloader Flash</div>
          <div class="tool-desc">Flash via preloader mode</div>
        </div>
      </div>
    </div>

    <div class="tools-section">
      <h3>Device Operations</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="forceFastboot">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
            </svg>
          </div>
          <div class="tool-name">Force Fastboot</div>
          <div class="tool-desc">Force device to fastboot mode</div>
        </div>
        <div class="tool-card" @click="forceRecovery">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </div>
          <div class="tool-name">Force Recovery</div>
          <div class="tool-desc">Force device to recovery mode</div>
        </div>
        <div class="tool-card" @click="forceBrom">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div class="tool-name">Force BROM</div>
          <div class="tool-desc">Force device to BROM mode</div>
        </div>
        <div class="tool-card" @click="readInfo">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          <div class="tool-name">Read Info</div>
          <div class="tool-desc">Read device information</div>
        </div>
      </div>
    </div>

    <div class="tools-section">
      <h3>Partition Operations</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="readPartitions">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 6h16M4 10h16M4 14h16M4 18h16" />
            </svg>
          </div>
          <div class="tool-name">Read Partitions</div>
          <div class="tool-desc">List all partitions</div>
        </div>
        <div class="tool-card" @click="backupPartition">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" />
            </svg>
          </div>
          <div class="tool-name">Backup Partition</div>
          <div class="tool-desc">Backup selected partition</div>
        </div>
        <div class="tool-card" @click="erasePartition">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </div>
          <div class="tool-name">Erase Partition</div>
          <div class="tool-desc">Erase selected partition</div>
        </div>
      </div>
    </div>

    <div class="tools-section">
      <h3>Root & Unlock</h3>
      <div class="tools-grid">
        <div class="tool-card" @click="unlockBootloader">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0110 0v4" />
            </svg>
          </div>
          <div class="tool-name">Unlock Bootloader</div>
          <div class="tool-desc">Unlock device bootloader</div>
        </div>
        <div class="tool-card" @click="rootDevice">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 2L2 7l10 5 10-5-10-5z" />
              <path d="M2 17l10 5 10-5" />
              <path d="M2 12l10 5 10-5" />
            </svg>
          </div>
          <div class="tool-name">Root Device</div>
          <div class="tool-desc">Root with Magisk</div>
        </div>
        <div class="tool-card" @click="frpBypass">
          <div class="tool-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
            </svg>
          </div>
          <div class="tool-name">FRP Bypass</div>
          <div class="tool-desc">Bypass FRP lock</div>
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

const openScatterFlasher = () => sendIpcToMain('open-scatter-flasher')
const openDAFlasher = () => sendIpcToMain('open-da-flasher')
const openPreloaderFlash = () => sendIpcToMain('open-preloader-flash')

const forceFastboot = () => sendIpcToMain('mtk-force-fastboot')
const forceRecovery = () => sendIpcToMain('mtk-force-recovery')
const forceBrom = () => sendIpcToMain('mtk-force-brom')
const readInfo = () => sendIpcToMain('mtk-read-info')

const readPartitions = () => sendIpcToMain('mtk-read-partitions')
const backupPartition = () => sendIpcToMain('mtk-backup-partition')
const erasePartition = () => sendIpcToMain('mtk-erase-partition')

const unlockBootloader = () => sendIpcToMain('mtk-unlock-bootloader')
const rootDevice = () => sendIpcToMain('mtk-root')
const frpBypass = () => sendIpcToMain('mtk-frp-bypass')

const clearOutput = () => {
  commandOutput.value = ''
}
</script>

<style lang="less" scoped>
.mediatek-tools {
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
