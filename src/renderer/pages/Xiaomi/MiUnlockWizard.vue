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

      <div class="wizard-body scroll">

        <!-- Step 0: Welcome -->
        <template v-if="step === 0">
          <h4>Unlock your Xiaomi bootloader (Official)</h4>
          <p>This wizard uses the official Xiaomi unlock API - the same flow as MiUnlock desktop app.</p>
          <div class="info-card">
            <div class="info-title">What will happen</div>
            <ol>
              <li>Sign in with your Xiaomi account in your browser.</li>
              <li>Confirm your account region.</li>
              <li>Connect your phone in fastboot mode.</li>
              <li>Send the unlock command to your device.</li>
            </ol>
          </div>
          <div class="warn-banner">
            Unlocking will ERASE ALL DATA on the device. Only proceed with a device and Xiaomi account that belong to you.
          </div>
          <label class="ack-check">
            <input type="checkbox" v-model="acknowledged" />
            <span>I understand the risks, and this is my own device and account.</span>
          </label>
        </template>

        <!-- Step 1: Browser Login -->
        <template v-if="step === 1">
          <h4>Sign in to your Xiaomi account</h4>
          <p>Your password is entered on Xiaomi's own login page - never in this app.</p>

          <div class="info-card">
            <div class="info-title">Steps</div>
            <ol>
              <li>Click "Sign in" below to open the Xiaomi login page in your browser.</li>
              <li>Sign in with your Xiaomi account credentials.</li>
              <li>After login, you will be redirected to a success page.</li>
              <li>Return to this window - it will continue automatically.</li>
            </ol>
          </div>

          <div class="btn-row">
            <button class="wizard-btn wizard-btn--primary" @click="startBrowserLogin" :disabled="loginBusy">
              {{ loginBusy ? 'Waiting for browser login...' : 'Sign in with Xiaomi Account' }}
            </button>
          </div>

          <div v-if="loginBusy" class="loading-indicator">
            <div class="processing-ring"></div>
            <p>Waiting for you to sign in the browser...</p>
          </div>

          <div v-if="loginError" class="err-banner">{{ loginError }}</div>
          <div v-if="loginDone" class="ok-banner">Signed in successfully!</div>
        </template>

        <!-- Step 2: Region -->
        <template v-if="step === 2">
          <h4>Data-center region</h4>
          <p>Confirm which Xiaomi server region your account uses.</p>

          <div v-if="regionBusy" class="loading-indicator">
            <div class="processing-ring"></div>
            <p>Detecting your account region...</p>
          </div>

          <div v-if="regionError" class="err-banner">{{ regionError }}</div>

          <div v-if="regionResolved && !regionBusy">
            <div class="detail-grid">
              <div class="detail-row">
                <span class="detail-label">Account ID</span>
                <span class="detail-value">{{ sessionInfo?.userId }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Region</span>
                <span class="detail-value">{{ regionInfo?.region || 'Auto-detected' }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Zone</span>
                <span class="detail-value">{{ regionInfo?.zone }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Server</span>
                <span class="detail-value detail-value--mono">{{ regionInfo?.domain }}</span>
              </div>
            </div>
          </div>

          <div class="field-group" v-if="regionResolved && !regionBusy">
            <label>Override zone (if auto-detection is wrong):</label>
            <select v-model="selectedZone" class="text-input">
              <option v-for="z in zoneOptions" :key="z" :value="z">{{ z }}</option>
            </select>
          </div>
        </template>

        <!-- Step 3: Device Detection -->
        <template v-if="step === 3">
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

          <div class="btn-row">
            <button class="wizard-btn wizard-btn--primary" @click="detectDevice" :disabled="deviceBusy">
              {{ deviceBusy ? 'Detecting...' : 'Detect Device' }}
            </button>
          </div>

          <div v-if="deviceBusy" class="loading-indicator">
            <div class="processing-ring"></div>
            <p>Waiting for fastboot device...</p>
          </div>

          <div v-if="deviceError" class="err-banner">{{ deviceError }}</div>

          <div v-if="deviceDetected && !deviceBusy" class="detail-grid">
            <div class="detail-row">
              <span class="detail-label">Product</span>
              <span class="detail-value">{{ deviceProduct }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Token</span>
              <span class="detail-value detail-value--mono">{{ deviceToken ? deviceToken.substring(0, 32) + '...' : 'N/A' }}</span>
            </div>
          </div>
        </template>

        <!-- Step 4: Confirm and Unlock -->
        <template v-if="step === 4">
          <h4>Review &amp; Unlock</h4>

          <div v-if="unlockBusy" class="loading-indicator">
            <div class="processing-ring"></div>
            <p>{{ unlockStatusText }}</p>
          </div>

          <div v-if="unlockError" class="err-banner">{{ unlockError }}</div>

          <div v-if="!unlockBusy && !unlockDone">
            <div class="detail-grid">
              <div class="detail-row">
                <span class="detail-label">Account</span>
                <span class="detail-value">{{ sessionInfo?.userId }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Device</span>
                <span class="detail-value">{{ deviceProduct }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Zone</span>
                <span class="detail-value">{{ selectedZone || regionInfo?.zone }}</span>
              </div>
            </div>

            <div v-if="clearInfo" class="warn-banner" :class="{ 'ok-banner': clearInfo.cleanOrNot !== 1 }">
              <template v-if="clearInfo.cleanOrNot === 1">
                Unlocking this device will <strong>ERASE ALL USER DATA</strong>.
              </template>
              <template v-else>
                Unlocking this device will <strong>NOT</strong> erase user data.
              </template>
              <div v-if="clearInfo.description" class="notice-text">{{ clearInfo.description }}</div>
            </div>

            <label class="ack-check">
              <input type="checkbox" v-model="readyToUnlock" />
              <span>I have backed up everything important and want to proceed.</span>
            </label>

            <div class="btn-row">
              <button class="wizard-btn wizard-btn--primary" @click="startUnlock" :disabled="!readyToUnlock">
                Unlock Now
              </button>
            </div>
          </div>

          <div v-if="unlockDone && !unlockBusy" class="ok-banner">
            Unlock command sent successfully! Your device may reboot on its own.
          </div>
        </template>

      </div>

      <div class="wizard-footer">
        <button v-if="step > 0 && step < 4 && !unlockBusy" class="wizard-btn" @click="step--">Back</button>
        <div class="footer-spacer"></div>
        <button v-if="step === 0" class="wizard-btn wizard-btn--primary" @click="step = 1" :disabled="!acknowledged">Next</button>
        <button v-if="step === 1 && loginDone && !loginBusy" class="wizard-btn wizard-btn--primary" @click="resolveRegion">Next</button>
        <button v-if="step === 2 && regionResolved && !regionBusy" class="wizard-btn wizard-btn--primary" @click="step = 3">Next</button>
        <button v-if="step === 4 && unlockDone" class="wizard-btn wizard-btn--primary" @click="close">Done</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onBeforeUnmount } from 'vue'
import { sendIpcToMain, sendIpcWithTimeout } from '@renderer/utils/ipc'

const props = defineProps({ visible: Boolean })
const emit = defineEmits(['close'])

const step = ref(0)
const acknowledged = ref(false)
const steps = ['Welcome', 'Login', 'Region', 'Device', 'Unlock']

const loginBusy = ref(false)
const loginDone = ref(false)
const loginError = ref('')
const authCode = ref('')
const sessionInfo = ref(null)

const regionBusy = ref(false)
const regionResolved = ref(false)
const regionError = ref('')
const regionInfo = ref(null)
const selectedZone = ref('')
const zoneOptions = ['Singapore', 'China', 'India', 'Russia', 'Europe']

const deviceBusy = ref(false)
const deviceDetected = ref(false)
const deviceError = ref('')
const deviceProduct = ref('')
const deviceToken = ref('')

const readyToUnlock = ref(false)
const unlockBusy = ref(false)
const unlockDone = ref(false)
const unlockError = ref('')
const unlockStatusText = ref('')
const clearInfo = ref(null)

async function startBrowserLogin() {
  loginBusy.value = true
  loginError.value = ''
  loginDone.value = false
  try {
    const result = await sendIpcWithTimeout('miunlock_start_login', {}, 360000)
    authCode.value = result.code
    loginDone.value = true
  } catch (e) {
    loginError.value = e?.message || e || 'Login failed'
  }
  loginBusy.value = false
}

async function resolveRegion() {
  regionBusy.value = true
  regionError.value = ''
  regionResolved.value = false
  try {
    sessionInfo.value = await sendIpcWithTimeout('miunlock_create_session', {
      auth_code: authCode.value,
    }, 30000)
    const region = await sendIpcWithTimeout('miunlock_resolve_region', {
      session: sessionInfo.value,
    }, 30000)
    regionInfo.value = region
    selectedZone.value = region.zone
    regionResolved.value = true
  } catch (e) {
    regionError.value = e?.message || e || 'Region detection failed'
  }
  regionBusy.value = false
}

async function detectDevice() {
  deviceBusy.value = true
  deviceError.value = ''
  deviceDetected.value = false
  try {
    const info = await sendIpcWithTimeout('xiaomi_detect_device', {}, 30000)
    if (info.mode !== 'fastboot') {
      deviceError.value = 'Device not in fastboot mode. Current mode: ' + info.mode + '. Put your phone in fastboot mode and try again.'
      return
    }
    deviceProduct.value = info.product || info.model || 'Unknown'
    try {
      deviceToken.value = await sendIpcWithTimeout('miunlock_get_device_token', {}, 30000)
    } catch {
      deviceToken.value = ''
    }
    deviceDetected.value = true
  } catch (e) {
    deviceError.value = 'Detection failed: ' + (e?.message || e)
  }
  deviceBusy.value = false
}

async function startUnlock() {
  unlockBusy.value = true
  unlockError.value = ''
  unlockDone.value = false
  unlockStatusText.value = 'Preparing unlock request...'
  try {
    unlockStatusText.value = 'Sending unlock request to Xiaomi servers...'
    const result = await sendIpcWithTimeout('miunlock_perform_unlock', {
      session: sessionInfo.value,
      region: { ...regionInfo.value, zone: selectedZone.value },
      product: deviceProduct.value,
      device_token: deviceToken.value,
    }, 180000)
    unlockDone.value = true
  } catch (e) {
    unlockError.value = e?.message || e || 'Unlock failed'
  }
  unlockBusy.value = false
}

function close() {
  step.value = 0
  acknowledged.value = false
  loginDone.value = false
  loginError.value = ''
  regionResolved.value = false
  regionError.value = ''
  deviceDetected.value = false
  deviceError.value = ''
  deviceToken.value = ''
  readyToUnlock.value = false
  unlockDone.value = false
  unlockError.value = ''
  emit('close')
}
</script>

<style scoped>
.wizard-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; }
.wizard-card { width: 540px; max-height: 80vh; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 10px; display: flex; flex-direction: column; overflow: hidden; }
.wizard-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border-primary); }
.wizard-header h3 { margin: 0; font-size: 14px; color: var(--text-primary); }
.close-btn { background: none; border: none; color: var(--text-secondary); font-size: 20px; cursor: pointer; padding: 0 4px; }
.close-btn:hover { color: var(--text-primary); }
.wizard-progress { display: flex; gap: 6px; padding: 10px 18px; justify-content: center; }
.step-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--border-primary); transition: all 0.2s; }
.step-dot.active { background: var(--accent-primary); transform: scale(1.3); }
.step-dot.done { background: #4caf50; }
.wizard-body { flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 12px; }
.wizard-body h4 { margin: 0; font-size: 15px; color: var(--text-primary); }
.wizard-body p { margin: 0; font-size: 12px; color: var(--text-secondary); }
.info-card { padding: 12px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-tertiary); }
.info-title { font-size: 12px; font-weight: 600; color: var(--accent-primary); margin-bottom: 6px; }
.info-card ol { margin: 0; padding-left: 18px; font-size: 12px; color: var(--text-primary); line-height: 1.8; }
.warn-banner { padding: 10px 12px; border-radius: 6px; font-size: 12px; background: #ff980015; color: #ffab40; border: 1px solid #ff980033; }
.ok-banner { padding: 10px 12px; border-radius: 6px; font-size: 12px; background: #4caf5015; color: #81c784; border: 1px solid #4caf5033; }
.err-banner { padding: 10px 12px; border-radius: 6px; font-size: 12px; background: #f4433615; color: #e57373; border: 1px solid #f4433633; }
.ack-check { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-primary); cursor: pointer; }
.ack-check input { accent-color: var(--accent-primary); }
.detail-grid { display: flex; flex-direction: column; gap: 6px; }
.detail-row { display: flex; justify-content: space-between; font-size: 12px; padding: 4px 0; }
.detail-label { color: var(--text-secondary); }
.detail-value { color: var(--text-primary); font-weight: 500; }
.detail-value--mono { font-family: monospace; font-size: 11px; max-width: 300px; overflow: hidden; text-overflow: ellipsis; }
.btn-row { display: flex; gap: 8px; }
.loading-indicator { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 16px 0; }
.loading-indicator p { font-size: 12px; color: var(--text-secondary); margin: 0; }
.processing-ring { width: 24px; height: 24px; border: 3px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.9s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.field-group { display: flex; flex-direction: column; gap: 4px; }
.field-group label { font-size: 11px; color: var(--text-secondary); }
.text-input { width: 100%; background: var(--bg-primary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 8px 10px; font-size: 12px; color: var(--text-primary); outline: none; box-sizing: border-box; }
.text-input:focus { border-color: var(--accent-primary); }
.notice-text { margin-top: 6px; font-style: italic; opacity: 0.8; }
.wizard-footer { display: flex; align-items: center; padding: 12px 18px; border-top: 1px solid var(--border-primary); gap: 8px; }
.footer-spacer { flex: 1; }
.wizard-btn { padding: 7px 16px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-tertiary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; }
.wizard-btn:hover:not(:disabled) { border-color: var(--accent-primary); }
.wizard-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.wizard-btn--primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); }
.wizard-btn--primary:hover:not(:disabled) { opacity: 0.9; }
</style>