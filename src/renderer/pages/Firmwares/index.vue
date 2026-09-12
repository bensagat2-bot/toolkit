<template>
  <div class="mtk-page">
    <div class="mtk-header">
      <h2>Firmwares</h2>
      <div class="header-right">
        <div class="source-tabs">
          <button :class="['tab', { active: source === 'xiaomi' }]" @click="switchSource('xiaomi')">Xiaomi</button>
          <button :class="['tab', { active: source === 'transsion' }]" @click="switchSource('transsion')">Transsion</button>
        </div>
        <div class="credits-badge">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M16 8h-6a2 2 0 1 0 0 4h4a2 2 0 1 1 0 4H8"/><path d="M12 18V6"/></svg>
          <span>{{ credits }} credits</span>
        </div>
      </div>
    </div>

    <div class="mtk-content">
      <div class="flash-panel">
        <div class="action-group">
          <h3>Search</h3>
          <input v-model="search" @input="debounceSearch" class="text-input" placeholder="Search device, version..." />
        </div>
        <div class="action-group">
          <h3>Results</h3>
          <div class="result-count">{{ total }} firmwares found</div>
        </div>
        <div class="action-group">
          <h3>Pagination</h3>
          <div class="row">
            <button class="btn btn-sm" :disabled="page <= 1" @click="goPage(page - 1)">Previous</button>
            <span class="page-info">{{ page }} / {{ pages }}</span>
            <button class="btn btn-sm" :disabled="page >= pages" @click="goPage(page + 1)">Next</button>
          </div>
        </div>
      </div>

      <div class="output-card">
        <div class="output-card-header">
          <span>{{ source === 'xiaomi' ? 'Xiaomi' : 'Transsion' }} Firmware</span>
        </div>

        <div class="partition-table-wrap" v-if="!loading && items.length">
          <table class="partition-table">
            <thead>
              <tr>
                <th>Device</th>
                <th>Version</th>
                <th>Android</th>
                <th>Method</th>
                <th>Date</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in items" :key="item.id" @click="openDetail(item)" class="clickable">
                <td class="col-device">{{ item.device_name || item.project || 'Unknown' }}</td>
                <td><span class="badge">{{ item.version }}</span></td>
                <td>{{ item.android || '-' }}</td>
                <td>{{ item.method || '-' }}</td>
                <td>{{ item.date || '-' }}</td>
                <td>
                  <span :class="item.has_link ? 'status-ok' : 'status-no'">
                    {{ item.has_link ? 'Available' : 'No link' }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="output-body welcome-body" v-else-if="loading">
          <div class="spinner"></div>
          <p>Loading firmwares...</p>
        </div>

        <div class="output-body welcome-body" v-else>
          <div class="welcome-icon">F</div>
          <h3>Firmware Browser</h3>
          <p>No firmwares loaded. Try searching or switching source.</p>
        </div>
      </div>
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
            <div class="info-row" v-if="selectedItem.method"><span>Method</span><span>{{ selectedItem.method }}</span></div>
            <div class="info-row" v-if="selectedItem.region"><span>Region</span><span>{{ selectedItem.region }}</span></div>
            <div class="info-row" v-if="selectedItem.date"><span>Date</span><span>{{ selectedItem.date }}</span></div>
          </div>

          <!-- Step 1: Show credit cost -->
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

          <!-- Step 2: Confirm purchase -->
          <div class="modal-link-section" v-if="confirmingPurchase">
            <div class="confirm-msg">
              <span>Are you sure you want to spend <strong>5 credits</strong> on this firmware?</span>
            </div>
            <div class="credit-actions">
              <button class="btn btn-primary" @click="confirmDownload">Yes, buy</button>
              <button class="btn" @click="confirmingPurchase = false">Cancel</button>
            </div>
          </div>

          <!-- Step 3: Show link -->
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
  } catch { items.value = []; total.value = 0 }
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
.mtk-page { height: 100%; display: flex; flex-direction: column; padding: 16px; gap: 16px; }
.mtk-header { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-primary-light-500); h2 { font-size: 16px; margin: 0; } }
.header-right { display: flex; align-items: center; gap: 12px; }
.source-tabs { display: flex; border: 1px solid var(--border-primary); border-radius: 4px; overflow: hidden; }
.tab { padding: 5px 12px; border: none; background: var(--bg-secondary); color: var(--text-secondary); font-size: 11px; cursor: pointer; transition: all 0.15s; }
.tab.active { background: var(--accent-primary); color: #fff; }
.tab:hover:not(.active) { background: var(--bg-tertiary); }
.credits-badge { display: flex; align-items: center; gap: 6px; padding: 5px 10px; border-radius: 4px; background: var(--bg-secondary); border: 1px solid var(--border-primary); font-size: 12px; color: var(--accent-primary); font-weight: 600; }
.credits-badge svg { width: 14px; height: 14px; }

.mtk-content { flex: 1; display: flex; gap: 16px; min-height: 0; }
.flash-panel { width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
.action-group { display: flex; flex-direction: column; gap: 6px; h3 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); margin: 0; } }
.row { display: flex; gap: 6px; align-items: center; }
.text-input { flex: 1; min-width: 0; background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: 4px; padding: 4px 8px; font-size: 12px; color: var(--text-primary); outline: none; }
.text-input:focus { border-color: var(--accent-primary); }
.result-count { font-size: 12px; color: var(--text-secondary); }
.page-info { font-size: 12px; color: var(--text-secondary); padding: 0 8px; }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: 4px; background: var(--bg-secondary); color: var(--text-primary); font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; svg { width: 14px; height: 14px; flex-shrink: 0; } &:hover:not(:disabled) { border-color: var(--accent-primary); } &:disabled { opacity: 0.4; cursor: not-allowed; } &.btn-sm { padding: 3px 8px; font-size: 11px; } &.btn-primary { background: var(--accent-primary); color: #fff; border-color: var(--accent-primary); } }

.badge { font-size: 10px; padding: 1px 6px; border-radius: 3px; background: var(--accent-primary); color: #fff; font-weight: 500; }
.status-ok { color: #4caf50; font-size: 11px; }
.status-no { color: #f44336; font-size: 11px; }

.output-card { flex: 1; display: flex; flex-direction: column; border: 1px solid var(--border-primary); border-radius: 6px; overflow: hidden; min-width: 0; }
.output-card-header { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; border-bottom: var(--color-list-header-border-bottom); span { font-size: 12px; color: var(--color-font); } }
.partition-table-wrap { flex: 1; overflow-y: auto; }
.welcome-body { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; text-align: center; padding: 40px; flex: 1; h3 { font-size: 16px; margin: 0; } p { font-size: 12px; color: var(--text-secondary); margin: 0; } }
.welcome-icon { width: 48px; height: 48px; border-radius: 12px; background: linear-gradient(135deg, #3b82f6, #60a5fa); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; }
.spinner { width: 24px; height: 24px; border: 2px solid var(--border-primary); border-top-color: var(--accent-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.partition-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.partition-table th, .partition-table td { text-align: left; padding: 6px 10px; white-space: nowrap; }
.partition-table thead th { position: sticky; top: 0; background: var(--color-content-background); border-bottom: var(--color-list-header-border-bottom); color: var(--color-font); font-weight: 500; font-size: 12px; }
.partition-table tbody tr { border-top: 1px solid var(--border-primary); color: var(--text-primary); }
.partition-table tbody tr.clickable { cursor: pointer; &:hover { background: var(--bg-tertiary); } }
.col-device { font-weight: 500; }

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
