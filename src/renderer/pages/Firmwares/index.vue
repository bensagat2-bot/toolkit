<template>
  <div class="fw-page">
    <div class="fw-header">
      <h2>Firmwares</h2>
      <div class="source-tabs">
        <button :class="['tab', { active: source === 'xiaomi' }]" @click="switchSource('xiaomi')">Xiaomi</button>
        <button :class="['tab', { active: source === 'transsion' }]" @click="switchSource('transsion')">Transsion</button>
      </div>
      <div class="search-box">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
        <input v-model="search" @input="debounceSearch" placeholder="Search device, version..." />
      </div>
      <div class="credits-badge">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M16 8h-6a2 2 0 1 0 0 4h4a2 2 0 1 1 0 4H8"/><path d="M12 18V6"/></svg>
        <span>{{ credits }} credits</span>
      </div>
    </div>

    <div class="fw-list-wrap scroll">
      <div class="fw-grid" v-if="!loading && items.length">
        <div class="fw-card" v-for="item in items" :key="item.id" @click="openDetail(item)">
          <div class="card-top">
            <span class="fw-name" :title="item.device_name || item.project || 'Unknown'">{{ item.device_name || item.project || 'Unknown' }}</span>
            <span :class="item.has_link ? 'status-ok' : 'status-no'">{{ item.has_link ? 'Available' : 'No link' }}</span>
          </div>
          <div class="fw-sub">{{ fwSub(item) }}</div>
          <div class="fw-version"><span class="badge">{{ item.version }}</span></div>
          <div class="fw-meta">
            <span v-if="item.android">Android {{ item.android }}</span>
            <span v-if="item.method">{{ item.method }}</span>
            <span v-if="item.date">{{ item.date }}</span>
            <span v-if="item.region">{{ item.region }}</span>
          </div>
        </div>
      </div>

      <div class="fw-loading" v-else-if="loading">
        <div class="spinner"></div>
        <p>Loading firmwares...</p>
      </div>

      <div class="fw-empty" v-else>
        <div class="welcome-icon">F</div>
        <h3>Firmware Browser</h3>
        <p>No firmwares loaded. Try searching or switching source.</p>
      </div>
    </div>

    <div class="fw-pagination" v-if="pages > 1">
      <button class="page-btn" :disabled="page <= 1" @click="goPage(page - 1)">Previous</button>
      <span class="page-info">{{ page }} / {{ pages }} <em class="total">({{ total }} total)</em></span>
      <button class="page-btn" :disabled="page >= pages" @click="goPage(page + 1)">Next</button>
    </div>

    <!-- Detail Modal -->
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
            <div class="info-row" v-if="selectedItem.mainboard"><span>Mainboard</span><span>{{ selectedItem.mainboard }}</span></div>
            <div class="info-row" v-if="selectedItem.brand"><span>Brand</span><span>{{ selectedItem.brand }}</span></div>
            <div class="info-row" v-if="selectedItem.method"><span>Method</span><span>{{ selectedItem.method }}</span></div>
            <div class="info-row" v-if="selectedItem.region"><span>Region</span><span>{{ selectedItem.region }}</span></div>
            <div class="info-row" v-if="selectedItem.date"><span>Date</span><span>{{ selectedItem.date }}</span></div>
            <div class="info-row" v-if="selectedItem.platform"><span>Platform</span><span>{{ selectedItem.platform }}</span></div>
          </div>

          <div class="modal-link-section" v-if="!confirmingPurchase && !linkRevealed">
            <div class="credit-cost-msg">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M16 8h-6a2 2 0 1 0 0 4h4a2 2 0 1 1 0 4H8"/><path d="M12 18V6"/></svg>
              <span>This firmware requires <strong>5 credits</strong> to download.</span>
            </div>
            <div class="credit-actions">
              <button class="btn btn-primary" @click="startPurchase" :disabled="credits < 5">Yes, download</button>
              <button class="btn" @click="selectedItem = null">No, cancel</button>
            </div>
            <div v-if="credits < 5" class="insufficient-msg">Insufficient credits. You have {{ credits }} credits.</div>
          </div>

          <div class="modal-link-section" v-if="confirmingPurchase">
            <div class="confirm-msg">
              <span>Are you sure you want to spend <strong>5 credits</strong> on this firmware?</span>
            </div>
            <div class="credit-actions">
              <button class="btn btn-primary" @click="confirmDownload">Yes, buy</button>
              <button class="btn" @click="confirmingPurchase = false">Cancel</button>
            </div>
          </div>

          <div class="modal-link-section" v-if="linkRevealed && selectedItem.has_link">
            <div class="link-available">
              <button class="btn btn-primary" @click="copyLink">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                {{ copied ? 'Copied!' : 'Copy Download Link' }}
              </button>
            </div>
          </div>

          <div class="modal-link-section" v-if="!selectedItem.has_link">
            <div class="no-link-msg">No download link available for this firmware.</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

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
const confirmingPurchase = ref(false)
const linkRevealed = ref(false)

let searchTimer = null

function fwSub(item) {
  if (source.value === 'transsion') {
    return [item.brand, item.mainboard, item.market_type].filter(Boolean).join(' \u00b7 ')
  }
  return [item.codename, item.region].filter(Boolean).join(' \u00b7 ')
}

function debounceSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { page.value = 1; fetchFirmwares() }, 400)
}

async function fetchFirmwares() {
  loading.value = true
  try {
    const params = new URLSearchParams({ limit: '50', page: String(page.value), only_links: 'true' })
    if (search.value) params.set('search', search.value)
    const res = await fetch(`${API_BASE}/api/${source.value}/firmware?${params}`)
    const data = await res.json()
    items.value = data.data || []
    total.value = data.total || 0
    pages.value = data.pages || 1
  } catch { items.value = []; total.value = 0; pages.value = 1 }
  loading.value = false
}

function switchSource(src) { source.value = src; page.value = 1; search.value = ''; fetchFirmwares() }
function goPage(p) { page.value = p; fetchFirmwares() }

function openDetail(item) {
  selectedItem.value = item
  confirmingPurchase.value = false
  linkRevealed.value = false
  copied.value = false
}

function startPurchase() {
  confirmingPurchase.value = true
}

function confirmDownload() {
  confirmingPurchase.value = false
  linkRevealed.value = true
  credits.value = Math.max(0, credits.value - 5)
}

function copyLink() {
  const link = selectedItem.value?.link || selectedItem.value?.network_disk_link
  if (!link) return
  navigator.clipboard.writeText(link)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}

onMounted(() => { fetchFirmwares() })
</script>

<style scoped>
.fw-page { position: relative; height: 100%; display: flex; flex-flow: column nowrap; padding: 15px 15px 0; gap: 12px; }

.fw-header { display: flex; align-items: center; gap: 12px; flex: none; }
.fw-header h2 { font-size: 16px; margin: 0; }
.source-tabs { display: flex; border: 1px solid var(--border-primary); border-radius: 4px; overflow: hidden; }
.tab { padding: 5px 12px; border: none; background: var(--bg-secondary); color: var(--text-secondary); font-size: 11px; cursor: pointer; transition: all 0.15s; }
.tab.active { background: var(--accent-primary); color: #fff; }
.tab:hover:not(.active) { background: var(--bg-tertiary); }
.search-box { flex: 1; display: flex; align-items: center; gap: 8px; max-width: 380px; margin-left: auto; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 5px 10px; }
.search-box svg { width: 14px; height: 14px; color: var(--text-secondary); flex-shrink: 0; }
.search-box input { flex: 1; min-width: 0; background: none; border: none; outline: none; font-size: 12px; color: var(--text-primary); }
.credits-badge { display: flex; align-items: center; gap: 6px; padding: 5px 10px; border-radius: 4px; background: var(--bg-secondary); border: 1px solid var(--border-primary); font-size: 12px; color: var(--accent-primary); font-weight: 600; }
.credits-badge svg { width: 14px; height: 14px; }

.fw-list-wrap { flex: auto; min-height: 0; overflow-y: auto; padding: 15px 0 0; }
.fw-grid { display: flex; flex-flow: row wrap; justify-content: space-between; gap: 16px 0; }

.fw-card {
  width: 32%;
  max-width: 360px;
  box-sizing: border-box;
  display: flex;
  flex-flow: column nowrap;
  gap: 8px;
  padding: 14px 16px;
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  background: var(--bg-primary);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: opacity 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
}
.fw-card:hover { opacity: 0.85; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12); transform: translateY(-1px); }

.card-top { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.fw-name { font-size: 14px; font-weight: 600; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fw-sub { font-size: 12px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fw-version { display: flex; }
.badge { font-size: 10px; padding: 2px 8px; border-radius: 3px; background: var(--accent-primary); color: #fff; font-weight: 500; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fw-meta { display: flex; flex-flow: row wrap; gap: 6px 12px; font-size: 11px; color: var(--text-secondary); }
.status-ok { color: #4caf50; font-size: 11px; }
.status-no { color: #f44336; font-size: 11px; }

.fw-loading, .fw-empty { display: flex; flex-flow: column nowrap; align-items: center; justify-content: center; gap: 8px; text-align: center; padding: 60px 0; }
.fw-empty h3 { font-size: 16px; margin: 0; }
.fw-empty p { font-size: 12px; color: var(--text-secondary); margin: 0; }
.welcome-icon { width: 48px; height: 48px; border-radius: 12px; background: linear-gradient(135deg, #3b82f6, #60a5fa); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; }
.spinner { width: 24px; height: 24px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.fw-pagination { flex: none; display: flex; align-items: center; justify-content: center; gap: 12px; padding: 12px 0; }
.page-btn { padding: 5px 14px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; }
.page-btn:hover:not(:disabled) { border-color: var(--accent-primary); }
.page-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.page-info { font-size: 12px; color: var(--text-secondary); }
.page-info .total { font-style: normal; color: var(--text-secondary); opacity: 0.8; }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }

/* Modal */
.fw-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; }
.fw-modal { width: 460px; max-height: 80vh; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 10px; overflow: hidden; display: flex; flex-direction: column; }
.modal-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border-primary); }
.modal-header h3 { margin: 0; font-size: 14px; }
.modal-close { background: none; border: none; color: var(--text-secondary); font-size: 20px; cursor: pointer; padding: 0 4px; }
.modal-close:hover { color: var(--text-primary); }
.modal-body { flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 16px; }
.modal-info { display: flex; flex-direction: column; gap: 6px; }
.info-row { display: flex; justify-content: space-between; font-size: 12px; padding: 3px 0; border-bottom: 1px solid var(--border-primary); span:first-child { color: var(--text-secondary); } span:last-child { font-weight: 500; } }

.modal-link-section { padding: 14px; border: 1px solid var(--border-primary); border-radius: 6px; background: var(--bg-tertiary); display: flex; flex-direction: column; gap: 12px; align-items: center; }
.credit-cost-msg { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--text-primary); svg { width: 20px; height: 20px; color: var(--accent-primary); flex-shrink: 0; } }
.credit-actions { display: flex; gap: 8px; width: 100%; .btn { flex: 1; } }
.insufficient-msg { font-size: 11px; color: #f44336; }
.confirm-msg { font-size: 13px; text-align: center; }
.link-available { display: flex; flex-direction: column; align-items: center; gap: 8px; width: 100%; .btn { width: 100%; } }
.no-link-msg { font-size: 12px; color: var(--text-secondary); text-align: center; }
</style>