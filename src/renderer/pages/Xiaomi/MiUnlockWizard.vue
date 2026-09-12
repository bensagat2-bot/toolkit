<template>
  <div class="wizard-overlay" v-if="visible">
    <div class="wizard-card">
      <div class="wizard-header">
        <h3>Mi Unlock Wizard</h3>
        <button class="close-btn" @click="close">&times;</button>
      </div>

      <div class="wizard-progress">
        <div v-for="(s, i) in steps" :key="i" :class="['step-dot', { active: i === step, done: i < step }]"></div>
      </div>

      <!-- Step 0: Welcome -->
      <div class="wizard-body" v-if="step === 0">
        <h4>Unlock your Xiaomi bootloader</h4>
        <p>This wizard walks you through the official Xiaomi unlock flow.</p>
        <div class="info-card">
          <div class="info-title">What will happen</div>
          <ol>
            <li>Connect your phone in fastboot mode.</li>
            <li>Read device token and product info.</li>
            <li>Send the unlock command to the device.</li>
          </ol>
        </div>
        <div class="warn-banner">
          Unlocking can ERASE ALL DATA on the device and will affect its security posture.
          Only continue with a device that belongs to you.
        </div>
        <label class="ack-check">
          <input type="checkbox" v-model="acknowledged" />
          <span>I understand the risks, and this is my own device.</span>
        </label>
      </div>

      <!-- Step 1: Device Check -->
      <div class="wizard-body" v-if="step === 1">
        <h4>Connect your device</h4>
        <p>Put your phone into fastboot mode and connect it by USB.</p>
        <div class="info-card">
          <div class="info-title">How to enter fastboot mode</div>
          <ol>
            <li>Power off the phone completely.</li>
            <li>Hold Volume Down + Power until the fastboot logo appears.</li>
            <li>Connect the phone to this computer with a USB cable.</li>
          </ol>
        </div>
        <button class="wizard-btn wizard-btn--primary" @click="detectDevice" :disabled="detecting">
          {{ detecting ? 'Detecting...' : 'Detect Device' }}
        </button>
        <div v-if="deviceDetected" class="ok-banner">
          Device detected - {{ detectedProduct }}
        </div>
        <div v-if="detectError" class="err-banner">
          {{ detectError }}
        </div>
      </div>

      <!-- Step 2: Unlock -->
      <div class="wizard-body" v-if="step === 2">
        <h4>Review and Unlock</h4>
        <div class="detail-grid">
          <div class="detail-row">
            <span class="detail-label">Product</span>
            <span class="detail-value">{{ detectedProduct }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Token</span>
            <span class="detail-value detail-value--mono">{{ deviceToken ? deviceToken.substring(0, 32) + '...' : 'N/A' }}</span>
          </div>
        </div>
        <div class="warn-banner">
          This will send the unlock command to your device. The bootloader will be unlocked
          and ALL USER DATA will be erased.
        </div>
        <label class="ack-check">
          <input type="checkbox" v-model="readyToUnlock" />
          <span>I have backed up everything important and want to proceed.</span>
        </label>
      </div>

      <!-- Step 3: Result -->
      <div class="wizard-body" v-if="step === 3">
        <h4>{{ unlockSuccess ? 'Unlock Complete' : 'Unlock Failed' }}</h4>
        <div v-if="unlockSuccess" class="ok-banner">
          Bootloader unlock command sent successfully. Your device may reboot on its own.
        </div>
        <div v-else class="err-banner">
          {{ unlockError }}
        </div>
        <div v-if="unlockSuccess" class="info-card">
          <div class="info-title">Next steps</div>
          <ol>
            <li>Device will reboot automatically.</li>
            <li>First boot may take 5-10 minutes.</li>
            <li>Set up your device as new.</li>
          </ol>
        </div>
      </div>

      <div class="wizard-footer">
        <button v-if="step > 0 && step < 3" class="wizard-btn" @click="step--">Back</button>
        <div class="footer-spacer"></div>
        <button
          v-if="step === 0"
          class="wizard-btn wizard-btn--primary"
          @click="step = 1"
          :disabled="!acknowledged"
        >
          Next
        </button>
        <button
          v-if="step === 1"
          class="wizard-btn wizard-btn--primary"
          @click="startUnlock"
          :disabled="!deviceDetected || unlocking"
        >
          {{ unlocking ? 'Unlocking...' : 'Unlock Now' }}
        </button>
        <button
          v-if="step === 3"
          class="wizard-btn wizard-btn--primary"
          @click="close"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { sendIpcToMain } from '@renderer/utils/ipc'

const props = defineProps({ visible: Boolean })
const emit = defineEmits(['close'])

const step = ref(0)
const acknowledged = ref(false)
const detecting = ref(false)
const deviceDetected = ref(false)
const detectError = ref('')
const detectedProduct = ref('')
const deviceToken = ref('')
const readyToUnlock = ref(false)
const unlocking = ref(false)
const unlockSuccess = ref(false)
const unlockError = ref('')

const steps = ['Welcome', 'Device', 'Unlock', 'Result']

function close() {
  step.value = 0
  acknowledged.value = false
  deviceDetected.value = false
  detectError.value = ''
  detectedProduct.value = ''
  deviceToken.value = ''
  readyToUnlock.value = false
  unlocking.value = false
  unlockSuccess.value = false
  unlockError.value = ''
  emit('close')
}

async function detectDevice() {
  detecting.value = true
  detectError.value = ''
  deviceDetected.value = false
  try {
    const info = await sendIpcToMain('xiaomi_detect_device')
    if (info.mode !== 'fastboot') {
      detectError.value = 'Device not in fastboot mode. Current mode: ' + info.mode
      return
    }
    detectedProduct.value = info.product || info.model || 'Unknown'
    deviceDetected.value = true

    try {
      deviceToken.value = await sendIpcToMain('xiaomi_get_token')
    } catch {
      deviceToken.value = ''
    }
  } catch (e) {
    detectError.value = 'Detection failed: ' + e
  } finally {
    detecting.value = false
  }
}

async function startUnlock() {
  if (!readyToUnlock.value) {
    step.value = 2
    return
  }
  unlocking.value = true
  unlockSuccess.value = false
  unlockError.value = ''
  try {
    await sendIpcToMain('xiaomi_fastboot_oem_unlock')
    unlockSuccess.value = true
    step.value = 3
  } catch (e) {
    unlockError.value = 'Unlock failed: ' + e
    unlockSuccess.value = false
    step.value = 3
  } finally {
    unlocking.value = false
  }
}
</script>

<style scoped>
.wizard-overlay {
  position: fixed; inset: 0; z-index: 1000;
  background: rgba(0, 0, 0, 0.6); backdrop-filter: blur(4px);
  display: flex; align-items: center; justify-content: center;
}
.wizard-card {
  width: 520px; max-height: 80vh; background: var(--bg-secondary);
  border: 1px solid var(--border-primary); border-radius: 10px;
  display: flex; flex-direction: column; overflow: hidden;
}
.wizard-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 18px; border-bottom: 1px solid var(--border-primary);
}
.wizard-header h3 { margin: 0; font-size: 14px; color: var(--text-primary); }
.close-btn { background: none; border: none; color: var(--text-secondary); font-size: 20px; cursor: pointer; padding: 0 4px; }
.close-btn:hover { color: var(--text-primary); }

.wizard-progress {
  display: flex; gap: 6px; padding: 10px 18px; justify-content: center;
}
.step-dot {
  width: 8px; height: 8px; border-radius: 50%; background: var(--border-primary); transition: all 0.2s;
}
.step-dot.active { background: var(--accent-primary); transform: scale(1.3); }
.step-dot.done { background: #4caf50; }

.wizard-body {
  flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 12px;
}
.wizard-body h4 { margin: 0; font-size: 15px; color: var(--text-primary); }
.wizard-body p { margin: 0; font-size: 12px; color: var(--text-secondary); }

.info-card {
  padding: 12px; border: 1px solid var(--border-primary); border-radius: 6px;
  background: var(--bg-tertiary);
}
.info-title { font-size: 12px; font-weight: 600; color: var(--accent-primary); margin-bottom: 6px; }
.info-card ol { margin: 0; padding-left: 18px; font-size: 12px; color: var(--text-primary); line-height: 1.8; }

.warn-banner {
  padding: 10px 12px; border-radius: 6px; font-size: 12px;
  background: #ff980015; color: #ffab40; border: 1px solid #ff980033;
}
.ok-banner {
  padding: 10px 12px; border-radius: 6px; font-size: 12px;
  background: #4caf5015; color: #81c784; border: 1px solid #4caf5033;
}
.err-banner {
  padding: 10px 12px; border-radius: 6px; font-size: 12px;
  background: #f4433615; color: #e57373; border: 1px solid #f4433633;
}

.ack-check {
  display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-primary); cursor: pointer;
}
.ack-check input { accent-color: var(--accent-primary); }

.detail-grid { display: flex; flex-direction: column; gap: 6px; }
.detail-row { display: flex; justify-content: space-between; font-size: 12px; padding: 4px 0; }
.detail-label { color: var(--text-secondary); }
.detail-value { color: var(--text-primary); font-weight: 500; }
.detail-value--mono { font-family: monospace; font-size: 11px; max-width: 300px; overflow: hidden; text-overflow: ellipsis; }

.wizard-footer {
  display: flex; align-items: center; padding: 12px 18px;
  border-top: 1px solid var(--border-primary); gap: 8px;
}
.footer-spacer { flex: 1; }

.wizard-btn {
  padding: 7px 16px; border: 1px solid var(--border-primary); border-radius: 4px;
  background: var(--bg-tertiary); color: var(--text-primary); font-size: 12px;
  cursor: pointer; transition: all 0.15s;
}
.wizard-btn:hover:not(:disabled) { border-color: var(--accent-primary); }
.wizard-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.wizard-btn--primary {
  background: var(--accent-primary); color: #fff; border-color: var(--accent-primary);
}
.wizard-btn--primary:hover:not(:disabled) { opacity: 0.9; }
</style>
