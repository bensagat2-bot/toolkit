import { ref } from 'vue'

export interface LogLine {
  text: string
  response: string | null
  type: string
}

const RESPONSE_RE = /^(.*\.\.\.)\s+([A-Z0-9][A-Z0-9 _\-]*?)\s*\.?\s*$/i

export function splitResponse(message: string): { text: string; response: string | null } {
  const m = message.match(RESPONSE_RE)
  if (m) {
    return { text: m[1].replace(/\s*$/, ''), response: m[2].trim() }
  }
  return { text: message, response: null }
}

export function isSuccessMessage(message: string): boolean {
  return /\.\.\.\s+(FOUND|NOT FOUND|DONE|OK|SUCCESS|UNLOCKED)\s*\.?\s*$/i.test(message)
}

// Shared output-log state with the "Waiting for device... FOUND" flow: the
// response after the dots is appended (green) onto the same line as its base
// text instead of creating a brand new log entry.
export function createRunLog() {
  const logLines = ref<LogLine[]>([])
  let logQueue: Array<{ text: string; response: string | null; type: string }> = []
  let logTimer: ReturnType<typeof setTimeout> | null = null
  const delay = 500

  function flushNext() {
    if (!logQueue.length) {
      logTimer = null
      return
    }
    const item = logQueue.shift()!
    addLog(item.text, item.type, item.response)
    logTimer = setTimeout(flushNext, delay)
  }

  function addLog(message: string, type = 'info', responseOverride?: string | null) {
    const { text, response } = splitResponse(message)
    const resp = responseOverride !== undefined ? responseOverride : response
    const last = logLines.value[logLines.value.length - 1]
    if (resp && last && last.text === text && last.type === type) {
      last.response = resp
    } else {
      logLines.value.push({ text, response: resp, type })
    }
  }

  function queueLog(message: string, type = 'info') {
    const { text, response } = splitResponse(message)
    logQueue.push({ text, response, type })
    if (!logTimer) flushNext()
  }

  function clear() {
    logLines.value = []
  }

  function stop() {
    if (logTimer) clearTimeout(logTimer)
    logTimer = null
  }

  return { logLines, addLog, queueLog, clear, stop }
}