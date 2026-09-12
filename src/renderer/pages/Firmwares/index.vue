<template>
  <div class="firmwares-page">
    <div class="fw-header">
      <h2>Firmwares</h2>
      <div class="header-actions">
        <div class="source-tabs">
          <button :class="['tab', { active: source === 'xiaomi' }]" @click="switchSource('xiaomi')">Xiaomi</button>
          <button :class="['tab', { active: source === 'transsion' }]" @click="switchSource('transsion')">Transsion</button>
        </div>
        <div class="search-box">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input v-model="search" @input="debounceSearch" placeholder="Search device, version..." />
        </div>
      </div>
    </div>

    <div class="credit-bar" v-if="credits <= 0">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      <span>You need credits to download firmware. Contact admin to purchase.</span>
    </div>

    <div class="credit-bar credit-bar--ok" v-else>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      <span>{{ credits }} credits available</span>
    </div>

    <div class="fw-grid" v-if="!loading && items.length">
      <div v-for="item in items" :key="item.id" class="fw-card" @click="openDetail(item)">
        <div class="fw-card-icon" :class="source === 'xiaomi' ? 'icon-xiaomi' : 'icon-transsion'">
          {{ source === 'xiaomi' ? 'X' : 'T' }}
        </div>
        <div class="fw-card-body">
          <h4 class="fw-card-name">{{ item.device_name || item.project || 'Unknown' }}</h4>
          <div class="fw-card-meta">
            <span class="fw-badge">{{ item.version }}</span>
            <span v-if="item.android" class="fw-android">Android {{ item.android }}</span>
            <span v-if="item.method" class="fw-method">{{ item.method }}</span>
          </div>
          <div class="fw-card-footer">
            <span v-if="item.date" class="fw-date">{{ item.date }}</span>
            <span v-if="item.region" class="fw-region">{{ item.region }}</span>
            <span :class="['fw-link-status', item.has_link ? 'has-link' : 'no-link']">
              {{ item.has_link ? 'Download available' : 'No link' }}
            </span>
          </div>
        </div>
        <div class="fw-card-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
        </div>
      </div>
    </div>

    <div class="fw-loading" v-if="loading">
      <div class="spinner"></div>
      <span>Loading firmwares...</span>
    </div>

    <div class="fw-empty" v-if="!loading && !items.length">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
      <span>No firmwares found</span>
    </div>

    <div class="fw-pagination" v-if="pages > 1">
      <button :disabled="page <= 1" @click="goPage(page - 1)">Previous</button>
      <span>Page {{ page }} of {{ pages }}</span>
      <button :disabled="page >= pages" @click="goPage(page + 1)">Next</button>
    </div>

    <!-- Floating Detail Modal -->
    <div class="fw-overlay" v-if="selectedItem" @click.self="selectedItem = null">
      <div class="fw-modal">
        <div class="modal-header">
          <h3>{{ selectedItem.device_name || selectedItem.project }}</h3>
          <button class="modal-close" @click="selectedItem = null">&times;</button>
        </div>
        <div class="modal-body">
          <div class="modal-info">
            <div class="info-row"><span>Version</span><span>{{ selectedItem.version }}</span></div>
            <div class="info-row" v-if="selectedItem.android"><span>Android</span><span>{{ selectedItem.android }}</span></div>
            <div class="info-row" v-if="selectedItem.codename"><span>Codename</span><span>{{ selectedItem.codename }}</span></div>
            <div class="info-row" v-if="selectedItem.method"><span>Method</span><span>{{ selectedItem.method }}</span></div>
            <div class="info-row" v-if="selectedItem.size"><span>Size</span><span>{{ selectedItem.size }}</span></div>
            <div class="info-row" v-if="selectedItem.region"><span>Region</span><span>{{ selectedItem.region }}</span></div>
            <div class="info-row" v-if="selectedItem.date"><span>Date</span><span>{{ selectedItem.date }}</span></div>
            <div class="info-row" v-if="selectedItem.mainboard"><span>Mainboard</span><span>{{ selectedItem.mainboard }}</span></div>
            <div class="info-row" v-if="selectedItem.platform"><span>Platform</span><span>{{ selectedItem.platform }}</span></div>
          </div>

          <div class="modal-link-section" v-if="selectedItem.has_link">
            <div v-if="credits > 0" class="link-available">
              <button class="copy-btn" @click="copyLink">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                {{ copied ? 'Copied!' : 'Copy Download Link' }}
              </button>
              <span class="link-cost">Costs 1 credit</span>
            </div>
            <div v-else class="link-locked">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
              <span>Insufficient credits. Contact admin to purchase.</span>
            </div>
          </div>

          <div class="modal-link-section" v-else>
            <div class="no-link-msg">No download link available for this firmware.</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { sendIpcToMain } from '@renderer/utils/ipc'

const API_BASE = 'https://firmwaresss-devices.vercel.app'

const items = ref([])
const total = ref(0)
const page = ref(1)
const pages = ref(1)
const loading = ref(false)
const source = ref('xiaomi')
const search = ref('')
const selectedItem = ref(null)
const credits = ref(0)
const copied = ref(false)

let searchTimer = null

function debounceSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    page.value = 1
    fetchFirmwares()
  }, 400)
}

async function fetchFirmwares() {
  loading.value = true
  try {
    const params = new URLSearchParams({
      limit: '50',
      page: String(page.value),
      only_links: 'true',
    })
    if (search.value) params.set('search', search.value)

    const res = await fetch(`${API_BASE}/api/${source.value}/firmware?${params}`)
    const data = await res.json()

    items.value = data.data || []
    total.value = data.total || 0
    pages.value = data.pages || 1
  } catch {
    items.value = []
    total.value = 0
  }
  loading.value = false
}

function switchSource(src) {
  source.value = src
  page.value = 1
  search.value = ''
  fetchFirmwares()
}

function goPage(p) {
  page.value = p
  fetchFirmwares()
}

function openDetail(item) {
  selectedItem.value = item
  copied.value = false
}

function copyLink() {
  const link = selectedItem.value?.link || selectedItem.value?.network_disk_link
  if (!link) return
  navigator.clipboard.writeText(link)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}

onMounted(() => {
  fetchFirmwares()
  loadCredits()
})

async function loadCredits() {
  try {
    const info = await sendIpcToMain('xiaomi_detect_device')
    // credits would come from API in real implementation
    credits.value = 0
  } catch {
    credits.value = 0
  }
}
</script>

<style scoped>
.firmwares-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 12px; overflow: hidden; }

.fw-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px; flex-shrink: 0; }
.fw-header h2 { font-size: 15px; margin: 0; }
.header-actions { display: flex; align-items: center; gap: 10px; }

.source-tabs { display: flex; border: 1px solid var(--border-primary); border-radius: 4px; overflow: hidden; }
.tab { padding: 5px 12px; border: none; background: var(--bg-secondary); color: var(--text-secondary); font-size: 11px; cursor: pointer; transition: all 0.15s; }
.tab.active { background: var(--accent-primary); color: #fff; }
.tab:hover:not(.active) { background: var(--bg-tertiary); }

.search-box { display: flex; align-items: center; gap: 6px; padding: 5px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); }
.search-box svg { width: 14px; height: 14px; color: var(--text-secondary); flex-shrink: 0; }
.search-box input { border: none; background: none; outline: none; font-size: 12px; color: var(--text-primary); width: 180px; }

.credit-bar { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; font-size: 11px; background: #ff980015; color: #ffab40; border: 1px solid #ff980033; flex-shrink: 0; }
.credit-bar svg { width: 16px; height: 16px; flex-shrink: 0; }
.credit-bar--ok { background: #4caf5015; color: #81c784; border-color: #4caf5033; }

.fw-grid { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; padding-right: 4px; }

.fw-card { display: flex; align-items: center; gap: 12px; padding: 12px; border: 1px solid var(--border-primary); border-radius: 8px; background: var(--bg-secondary); cursor: pointer; transition: all 0.15s; }
.fw-card:hover { border-color: var(--accent-primary); background: var(--bg-tertiary); }

.fw-card-icon { width: 40px; height: 40px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 16px; font-weight: 700; color: #fff; flex-shrink: 0; }
.icon-xiaomi { background: linear-gradient(135deg, #ff6b00, #ff9800); }
.icon-transsion { background: linear-gradient(135deg, #3b82f6, #60a5fa); }

.fw-card-body { flex: 1; min-width: 0; }
.fw-card-name { font-size: 13px; font-weight: 600; margin: 0 0 4px 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.fw-card-meta { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.fw-badge { font-size: 10px; padding: 1px 6px; border-radius: 3px; background: var(--accent-primary); color: #fff; font-weight: 500; }
.fw-android { font-size: 10px; color: var(--text-secondary); }
.fw-method { font-size: 10px; padding: 1px 5px; border-radius: 3px; background: var(--bg-tertiary); color: var(--text-secondary); }
.fw-card-footer { display: flex; gap: 8px; margin-top: 4px; font-size: 10px; color: var(--text-secondary); }
.fw-link-status.has-link { color: #4caf50; }
.fw-link-status.no-link { color: #f44336; }

.fw-card-arrow { color: var(--text-secondary); flex-shrink: 0; }
.fw-card-arrow svg { width: 16px; height: 16px; }

.fw-loading { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 40px; color: var(--text-secondary); }
.spinner { width: 24px; height: 24px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.fw-empty { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 60px; color: var(--text-secondary); }
.fw-empty svg { width: 40px; height: 40px; opacity: 0.3; }

.fw-pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 10px 0; flex-shrink: 0; font-size: 12px; color: var(--text-secondary); }
.fw-pagination button { padding: 5px 12px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 11px; cursor: pointer; }
.fw-pagination button:hover:not(:disabled) { border-color: var(--accent-primary); }
.fw-pagination button:disabled { opacity: 0.35; cursor: not-allowed; }

/* Modal */
.fw-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; }
.fw-modal { width: 480px; max-height: 80vh; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 10px; overflow: hidden; display: flex; flex-direction: column; }
.modal-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border-primary); }
.modal-header h3 { margin: 0; font-size: 14px; }
.modal-close { background: none; border: none; color: var(--text-secondary); font-size: 20px; cursor: pointer; padding: 0 4px; }
.modal-close:hover { color: var(--text-primary); }

.modal-body { flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 16px; }
.modal-info { display: flex; flex-direction: column; gap: 6px; }
.info-row { display: flex; justify-content: space-between; font-size: 12px; padding: 3px 0; border-bottom: 1px solid var(--border-primary); }
.info-row span:first-child { color: var(--text-secondary); }
.info-row span:last-child { font-weight: 500; }

.modal-link-section { padding: 12px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-tertiary); }
.link-available { display: flex; flex-direction: column; align-items: center; gap: 8px; }
.copy-btn { display: flex; align-items: center; gap: 6px; padding: 8px 16px; border: none; border-radius: 4px; background: var(--accent-primary); color: #fff; font-size: 12px; cursor: pointer; transition: opacity 0.15s; }
.copy-btn:hover { opacity: 0.9; }
.copy-btn svg { width: 14px; height: 14px; }
.link-cost { font-size: 10px; color: var(--text-secondary); }

.link-locked { display: flex; align-items: center; gap: 8px; color: #ffab40; font-size: 12px; }
.link-locked svg { width: 16px; height: 16px; flex-shrink: 0; }

.no-link-msg { font-size: 12px; color: var(--text-secondary); text-align: center; }
</style>
