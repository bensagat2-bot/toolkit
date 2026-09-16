<template>
  <div class="term-page">
    <div class="term-card">
      <div class="term-header">
        <span class="term-title">Terminal</span>
        <span class="term-device">{{ deviceLabel }}</span>
        <div class="term-actions">
          <button class="btn-link" @click="clearLog" :disabled="busy">Clear</button>
          <button class="btn-link" @click="copyLog" :disabled="busy">Copy</button>
          <button class="btn-link" @click="browseFile" :disabled="busy">Browse</button>
        </div>
      </div>
      <div class="term-body" ref="bodyRef" @drop.prevent="onHtmlDrop">
        <div v-for="(line, i) in lines" :key="i" :class="['term-line', line.kind]">
          <template v-if="line.kind === 'cmd'">
            <span class="prompt">$ v1per&gt;</span>
            <span class="cmd-text">{{ line.text }}</span>
          </template>
          <template v-else>{{ line.text }}</template>
        </div>
        <div class="term-line input-line">
          <span class="prompt">$ v1per&gt;</span>
          <input
            ref="inputRef"
            v-model="current"
            class="term-input"
            spellcheck="false"
            autocomplete="off"
            :disabled="busy"
            placeholder="type here"
            @keydown.enter.prevent="runCommand"
            @keydown.up.prevent="historyPrev"
            @keydown.down.prevent="historyNext"
          />
        </div>
        <div v-if="busy" class="term-line busy">running...</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { sendIpcToMain, rendererOn } from '@renderer/utils/ipc'

const bodyRef = ref(null)
const inputRef = ref(null)
const current = ref('')
const lines = ref([])
const busy = ref(false)
const history = ref([])
const historyIndex = ref(-1)
const deviceStatus = ref('') // '', 'adb', 'fastboot', or device serial info
let unlistenDragDrop = null

const deviceLabel = computed(() => {
  if (deviceStatus.value) return `device: ${deviceStatus.value}`
  return 'waiting for device'
})

const scrollBottom = () => {
  nextTick(() => {
    if (bodyRef.value) bodyRef.value.scrollTop = bodyRef.value.scrollHeight
  })
}

watch(() => lines.value.length, scrollBottom)

function push(kind, text) {
  lines.value.push({ kind, text })
}

function clearLog() {
  lines.value = []
  current.value = ''
  historyIndex.value = -1
  focusInput()
}

async function copyLog() {
  const text = lines.value.map((l) => (l.kind === 'cmd' ? `$ v1per> ${l.text}` : l.text)).join('\n')
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    // clipboard unavailable, ignore
  }
}

function focusInput() {
  nextTick(() => inputRef.value?.focus())
}

function historyPrev() {
  if (!history.value.length) return
  historyIndex.value = historyIndex.value < 0 ? history.value.length - 1 : Math.max(0, historyIndex.value - 1)
  current.value = history.value[historyIndex.value]
}

function historyNext() {
  if (historyIndex.value < 0) return
  historyIndex.value += 1
  if (historyIndex.value >= history.value.length) {
    historyIndex.value = -1
    current.value = ''
  } else {
    current.value = history.value[historyIndex.value]
  }
}

function onDropPaths(paths) {
  const quoted = paths.map((p) => `"${p}"`).join(' ')
  current.value = current.value ? `${current.value} ${quoted}` : quoted
  focusInput()
}



function onDragOver() {
  dragOver.value = true
}

function onDragLeave() {
  dragOver.value = false
}

function onHtmlDrop(e) {
  // Try multiple path extraction methods
  let paths = []
  // 1. file.path (works in some WebView2/Chrome configs)
  const files = e.dataTransfer?.files
  if (files?.length) {
    paths = Array.from(files).map((f) => f.path).filter(Boolean)
    if (paths.length) { onDropPaths(paths); return }
  }
  // 2. getData('text') - Windows shell sometimes drops path as text
  const text = e.dataTransfer?.getData('text')
  if (text && /^[a-zA-Z]:\\/.test(text)) {
    onDropPaths([text]); return
  }
  // 3. insert file name at cursor as placeholder
  if (files?.length) {
    push('info', 'Drag-drop path unavailable. Use Browse button or type path manually.')
    current.value = current.value ? `${current.value} ${files[0].name}` : files[0].name
  }
}

async function browseFile() {
  const { showSelectDialog } = await import("@renderer/utils/ipc")
  const result = await showSelectDialog({ title: "Select file", filters: [] })
  if (result.filePaths?.length) {
    const quoted = result.filePaths.map((p) => `"${p}"`).join(" ")
    current.value = current.value ? `${current.value} ${quoted}` : quoted
    focusInput()
  }
}

async function runCommand() {
  let cmd = current.value.trim()
  if (!cmd || busy.value) return

  // Auto-prepend adb/fastboot when device is known and command is not
  // already prefixed and doesn't start with a known tool keyword.
  const knownTools = ['adb', 'fastboot', 'scrcpy', 'clear', 'cd', 'dir', 'echo', 'help', 'exit']
  const first = cmd.split(/\s+/)[0].toLowerCase()
  if (deviceStatus.value && !knownTools.includes(first)) {
    if (deviceStatus.value.startsWith('fastboot')) {
      cmd = `fastboot ${cmd}`
    } else if (deviceStatus.value.startsWith('adb')) {
      cmd = `adb ${cmd}`
    }
  }

  history.value.push(cmd)
  historyIndex.value = -1
  push('cmd', cmd)
  current.value = ''
  busy.value = true
  scrollBottom()
  try {
    if (cmd === 'clear') {
      lines.value = []
      return
    }
    await sendIpcToMain('term_run', { command: cmd })
  } catch (e) {
    push('err', `error: ${e}`)
  } finally {
    busy.value = false
    scrollBottom()
    focusInput()
  }
}

const onOutput = (_event, data) => {
  if (!data || typeof data.line !== 'string') return
  const text = data.line
  if (/^\[exit code:/.test(text)) {
    push('err', text)
  } else if (/error:|failed|not found|unknown|denied/i.test(text)) {
    push('err', text)
  } else {
    push('out', text)
  }
}

onMounted(async () => {
  push('info', 'V1per terminal. Type adb/fastboot commands. Drag & drop a file to insert its path.')
  push('info', 'Try: adb devices | fastboot devices | adb shell getprop ro.product.model')
  push('info', 'Device polling active - connected devices appear here.')
  await rendererOn('term:output', onOutput)
  const { listen } = await import('@tauri-apps/api/event')
  unlistenDragDrop = await listen('tauri://drag-drop', (event) => {
    const payload = event.payload
    if (payload.type === 'drop') {
      onDropPaths(payload.paths)
    }
  })
  focusInput()

  // Detect device mode on mount and poll every 3s
  const pollDevice = async () => {
    try {
      const info = await sendIpcToMain('detect_device')
      if (info?.mode && info.mode !== 'none') {
        const serial = info.serial || 'connected'
        deviceStatus.value = `${info.mode} (${serial})`
      } else {
        deviceStatus.value = ''
      }
    } catch {
      deviceStatus.value = ''
    }
  }
  await pollDevice()
  const pollTimer = setInterval(pollDevice, 3000)
  onBeforeUnmount(() => {
    if (typeof unlistenDragDrop === 'function') unlistenDragDrop()
    clearInterval(pollTimer)
    lines.value = []
  })
  return // prevent the old onBeforeUnmount from overwriting
})
</script>

<style scoped>
.term-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  animation: pageIn 0.35s ease;
}

.term-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  background: var(--bg-tertiary);
  overflow: hidden;
  box-shadow: 0 2px 12px rgba(77, 131, 175, 0.08);
}

.term-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  border-bottom: var(--color-list-header-border-bottom);
  background: var(--bg-secondary);
  font-size: 13px;
}

.term-title {
  font-weight: 700;
  color: var(--text-primary);
}

.term-device {
  font-size: 11px;
  color: var(--accent-primary);
  font-weight: 600;
}

.term-actions {
  margin-left: auto;
  display: flex;
  gap: 10px;
}

.term-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  font-size: 15px;
  line-height: 1.7;
  cursor: text;
  user-select: text;
  -webkit-user-select: text;
  transition: background 0.2s, border-color 0.2s;
}
.term-body.drag-over {
  background: color-mix(in srgb, var(--accent-primary) 8%, transparent);
  outline: 2px dashed var(--accent-primary);
  outline-offset: -4px;
}

.term-line {
  white-space: pre-wrap;
  word-break: break-all;
  user-select: text;
  -webkit-user-select: text;
}

.term-line.cmd {
  color: var(--text-primary);
  font-weight: 600;
}

.term-line.out {
  color: var(--text-primary);
}

.term-line.info {
  color: var(--text-secondary);
  font-style: italic;
}

.term-line.err {
  color: #f44336;
}

.prompt {
  color: var(--accent-primary);
  font-weight: 700;
  margin-right: 6px;
}

.term-line.busy {
  color: var(--accent-primary);
  font-style: italic;
  opacity: 0.7;
}

.input-line {
  display: flex;
  align-items: center;
  min-height: 28px;
  margin-top: 4px;
}

.term-input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  font-size: 15px;
  caret-color: var(--accent-primary);
  padding: 0;
}

.btn-link {
  background: none;
  border: none;
  color: var(--accent-primary);
  font-size: 12px;
  cursor: pointer;
  padding: 0;
  &:hover:not(:disabled) {
    text-decoration: underline;
  }
  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

@keyframes pageIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>