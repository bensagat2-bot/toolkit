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
    </div>

    <div class="fw-list-wrap scroll">
      <div class="fw-grid" v-if="!loading && items.length">
        <transition-group name="card">
          <div class="fw-card" v-for="(item, i) in items" :key="item.id" :style="{ '--i': i }" @click="openDetail(item)">
            <div class="fw-card-top">
              <div class="fw-dev-icon" :class="source === 'transsion' ? 'icon-transsion' : 'icon-xiaomi'">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect v-if="source === 'xiaomi'" x="5" y="2" width="14" height="20" rx="2" />
                  <circle v-if="source === 'xiaomi'" cx="12" cy="18" r="1" />
                  <path v-else d="M4 8l4-4 4 4M8 4v16M16 8l4 4-4 4M20 12H8" />
                </svg>
              </div>
              <div class="fw-card-info">
                <div class="fw-model" :title="item.device_name || item.project || 'Unknown'">{{ item.device_name || item.project || 'Unknown' }}</div>
                <div class="fw-version" :title="item.version">{{ item.version }}</div>
              </div>
              <span :class="['fw-status', item.has_link ? 'status-ok' : 'status-no']">{{ item.has_link ? 'Available' : 'No link' }}</span>
            </div>

            <div class="fw-meta">
              <span class="meta-pill" v-if="item.android"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="7" y="2" width="10" height="20" rx="2"/><path d="M11 18h2"/></svg>{{ item.android }}</span>
              <span class="meta-pill" v-if="item.method"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><path d="M3.3 7l8.7 5 8.7-5"/><path d="M12 22V12"/></svg>{{ item.method }}</span>
              <span class="meta-pill" v-if="item.date"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>{{ item.date }}</span>
              <span class="meta-pill" v-if="item.region"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>{{ item.region }}</span>
              <span class="meta-pill" v-if="item.mainboard"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M1 15h3M20 9h3M20 15h3"/></svg>{{ item.mainboard }}</span>
            </div>

            <div class="fw-footer">
              <span class="fw-brand" v-if="item.brand || item.market_type">{{ item.brand || item.market_type }}</span>
              <div class="fw-actions">
                <span class="btn btn-sm fw-more">Details</span>
              </div>
            </div>
          </div>
        </transition-group>
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
    <transition name="modal">
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
            <div class="info-row" v-if="selectedItem.market_type"><span>Market</span><span>{{ selectedItem.market_type }}</span></div>
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

          <div class="modal-link-section processing-section" v-if="processingOrder">
            <div class="processing-ring"></div>
            <div class="processing-msg">
              <p>Please wait, processing your order...</p>
              <p class="processing-sub">Preparing your download link</p>
            </div>
          </div>

          <transition name="fade">
            <div class="error-msg" v-if="showError">{{ errorMsg }}</div>
          </transition>

          <div class="modal-link-section" v-if="linkRevealed && revealedLink">
            <div class="purchase-actions">
              <button class="btn btn-primary" @click="copyLink">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                {{ copied ? 'Copied!' : 'Copy Download Link' }}
              </button>
              <button class="btn btn-primary" @click="openExtract" :disabled="!canExtract">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/></svg>
                Extract Partition
              </button>
            </div>
            <div class="link-note" v-if="revealedCode">
              Password: <strong>{{ revealedCode }}</strong>
            </div>
          </div>

          <div class="modal-link-section" v-if="!selectedItem.has_link">
            <div class="no-link-msg">No download link available for this firmware.</div>
          </div>
        </div>
      </div>
      </div>
    </transition>

    <ExtractDialog
      v-if="showExtract && revealedLink"
      :url="revealedLink"
      @close="closeExtract"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useAccountStore } from '@renderer/store/accountStore'
import ExtractDialog from './ExtractDialog.vue'

const store = useAccountStore()
const API_BASE = 'https://firmwaresss-devices.vercel.app'
const items = ref([])
const total = ref(0)
const page = ref(1)
const pages = ref(1)
const loading = ref(false)
const source = ref('xiaomi')
const search = ref('')
const selectedItem = ref(null)
const credits = ref(store.account?.credits ?? 0)
const copied = ref(false)
const confirmingPurchase = ref(false)
const linkRevealed = ref(false)
const processingOrder = ref(false)
const revealedLink = ref('')
const revealedCode = ref('')
const errorMsg = ref('')
const showError = ref(false)
const showExtract = ref(false)

const canExtract = computed(() => {
  const link = revealedLink.value
  return !!link && (link.includes('ota_full') || link.includes('ota-') || link.includes('images_') || link.endsWith('.tgz'))
})

function openExtract() {
  showExtract.value = true
}

function closeExtract() {
  showExtract.value = false
}

let searchTimer = null

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
  processingOrder.value = false
  copied.value = false
  revealedLink.value = ''
  revealedCode.value = ''
  errorMsg.value = ''
  showError.value = false
}

function startPurchase() {
  confirmingPurchase.value = true
}

async function confirmDownload() {
  confirmingPurchase.value = false
  processingOrder.value = true
  showError.value = false
  errorMsg.value = ''
  try {
    const result = await store.purchase(source.value, selectedItem.value.id)
    revealedLink.value = result.link
    revealedCode.value = result.extraction_code || ''
    credits.value = store.account?.credits ?? 0
    setTimeout(() => {
      processingOrder.value = false
      linkRevealed.value = true
    }, 800)
  } catch (e) {
    processingOrder.value = false
    errorMsg.value = e?.message || 'Purchase failed. Please try again.'
    showError.value = true
    if (e?.message === 'Not signed in' || e?.message === 'Invalid session') {
      store.logout()
    }
  }
}

function copyLink() {
  if (!revealedLink.value) return
  let text = revealedLink.value
  if (revealedCode.value) text += `\nPassword: ${revealedCode.value}`
  navigator.clipboard.writeText(text)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}

onMounted(() => {
  credits.value = store.account?.credits ?? 0
  fetchFirmwares()
})
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

.fw-list-wrap { flex: auto; min-height: 0; overflow-y: auto; padding: 15px 0 0; }
.fw-grid { display: flex; flex-flow: row wrap; justify-content: space-between; gap: 16px 0; }

.fw-card {
  width: 32%;
  max-width: 360px;
  box-sizing: border-box;
  display: flex;
  flex-flow: column nowrap;
  padding: 14px 16px;
  border: 1px solid var(--border-primary);
  border-radius: 10px;
  background: var(--bg-primary);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: opacity 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease, border-color 0.15s ease;
  animation: cardIn 0.4s ease both;
  animation-delay: calc(var(--i) * 0.04s);
}
@keyframes cardIn { from { opacity: 0; transform: translateY(16px); } to { opacity: 1; transform: translateY(0); } }
.card-enter-active { transition: opacity 0.3s ease, transform 0.3s ease; }
.card-enter-from { opacity: 0; transform: translateY(14px); }
.card-leave-to { opacity: 0; transform: translateY(-8px); }
.card-move { transition: transform 0.3s ease; }
.fw-card:hover { opacity: 0.92; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12); transform: translateY(-1px); border-color: var(--accent-primary); }

.fw-card-top { display: flex; align-items: center; gap: 10px; }
.fw-dev-icon { flex: none; width: 36px; height: 36px; border-radius: 8px; display: flex; align-items: center; justify-content: center; }
.fw-dev-icon svg { width: 18px; height: 18px; }
.icon-xiaomi { background: rgba(255, 103, 0, 0.12); color: #ff6f00; }
.icon-transsion { background: rgba(63, 81, 181, 0.12); color: #3f51b5; }
.fw-card-info { flex: 1; min-width: 0; }
.fw-model { font-size: 13px; font-weight: 600; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fw-version { font-size: 11px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-top: 2px; }
.fw-status { flex: none; font-size: 10px; font-weight: 500; padding: 2px 8px; border-radius: 10px; }
.status-ok { color: #4caf50; background: rgba(76, 175, 80, 0.12); }
.status-no { color: #f44336; background: rgba(244, 67, 54, 0.12); }

.fw-meta { display: flex; flex-flow: row wrap; gap: 6px; margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--border-primary); }
.meta-pill { display: inline-flex; align-items: center; gap: 4px; font-size: 10px; color: var(--text-secondary); background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 10px; padding: 2px 8px; }
.meta-pill svg { width: 11px; height: 11px; flex-shrink: 0; }

.fw-footer { display: flex; align-items: center; justify-content: space-between; margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border-primary); }
.fw-brand { font-size: 10px; font-weight: 600; color: var(--accent-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fw-actions { flex: none; }
.fw-more { font-size: 10px; padding: 3px 12px; border-radius: 10px; }
.fw-more:hover { border-color: var(--accent-primary); color: var(--accent-primary); }

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
.purchase-actions { display: flex; flex-direction: column; gap: 8px; width: 100%; .btn { width: 100%; } }
.link-note { font-size: 12px; color: var(--text-secondary); strong { font-family: monospace; color: var(--accent-primary); } }
.no-link-msg { font-size: 12px; color: var(--text-secondary); text-align: center; }
.processing-section { gap: 14px; padding: 22px 14px; }
.processing-ring { width: 34px; height: 34px; border: 3px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.9s linear infinite; }
.processing-msg { display: flex; flex-direction: column; align-items: center; gap: 4px; p { font-size: 13px; margin: 0; color: var(--text-primary); font-weight: 600; } }
.processing-sub { font-size: 11px !important; font-weight: 400 !important; color: var(--text-secondary); animation: pulse 1.4s ease-in-out infinite; }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
.modal-enter-active, .modal-leave-active { transition: opacity 0.25s ease; }
.modal-enter-active .fw-modal, .modal-leave-active .fw-modal { transition: transform 0.25s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .fw-modal, .modal-leave-to .fw-modal { transform: scale(0.94); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.25s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.error-msg { font-size: 12px; color: #f44336; text-align: center; padding: 8px; }
</style>