<template>
  <div class="cfg-page" v-loading="loading">
    <header class="cfg-page__head">
      <h2 class="cfg-page__title">配置与发布</h2>
      <p class="cfg-page__desc">功能开关 · 版本发布 · 一键部署上线</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 模块注册：一键新建模块 + 发布 -->
      <el-tab-pane label="模块注册" name="modules">
        <div class="cfg-module-wizard">
          <div class="cfg-module-wizard__header">
            <h3>一键注册新模块</h3>
            <p>填入模块信息，自动创建功能开关、快照配置并部署上线。</p>
          </div>
          <el-form :model="modForm" label-width="100px" size="default" style="max-width: 520px;">
            <el-form-item label="功能开关 Key">
              <el-input v-model="modForm.key" placeholder="如: heart_anim_enabled">
                <template #prepend>flag</template>
              </el-input>
            </el-form-item>
            <el-form-item label="功能描述">
              <el-input v-model="modForm.desc" placeholder="这个模块做什么" />
            </el-form-item>
            <el-form-item label="生效位置">
              <el-input v-model="modForm.location" placeholder="如: 首页右侧小电视、播放器控制栏" />
            </el-form-item>
            <el-form-item label="默认启用">
              <el-switch v-model="modForm.enabled" />
            </el-form-item>
            <el-form-item label="发布版本号">
              <el-input v-model="modForm.version" placeholder="如: 1.2.0" style="width: 160px" />
            </el-form-item>
            <el-form-item label="发布说明">
              <el-input v-model="modForm.notes" type="textarea" :rows="2" placeholder="本次变更说明" />
            </el-form-item>
          </el-form>
          <!-- 执行进度 -->
          <div v-if="modProgress.length > 0" class="cfg-module-progress">
            <div v-for="(step, i) in modProgress" :key="i" class="cfg-module-progress__step" :class="{ done: step.ok, fail: step.fail, active: step.active }">
              <span class="step-dot">{{ step.fail ? '✗' : step.ok ? '✓' : '○' }}</span>
              <span class="step-text">{{ step.text }}</span>
            </div>
          </div>
          <div style="margin-top: 20px;">
            <el-button type="primary" size="large" :loading="modRunning" @click="runModuleWizard">
              {{ modRunning ? '执行中...' : '一键注册并上线' }}
            </el-button>
            <el-button v-if="modDone && modReleaseId" type="success" size="large" @click="activeTab='releases';fetchReleases()">
              查看发布列表 →
            </el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- 功能开关 -->
      <el-tab-pane label="功能开关" name="flags">
        <div class="cfg-toolbar">
          <el-button type="primary" size="default" @click="openFlagDialog(null)">新建开关</el-button>
        </div>
        <el-table :data="flags" stripe size="default" empty-text="暂无功能开关">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="key" label="Key" min-width="150">
            <template #default="{ row }">
              <code class="cfg-code">{{ row.key }}</code>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述" min-width="160" show-overflow-tooltip />
          <el-table-column label="状态" width="80">
            <template #default="{ row }">
              <el-switch v-model="row.enabled" @change="toggleFlag(row)" />
            </template>
          </el-table-column>
          <el-table-column label="灰度比例" width="180">
            <template #default="{ row }">
              <div class="cfg-rollout">
                <el-slider
                  v-model="row.rollout_pct"
                  :min="0" :max="100" :step="5"
                  style="width: 120px"
                  @change="updateRollout(row)"
                />
                <span class="cfg-rollout__val">{{ row.rollout_pct }}%</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="白名单" width="120">
            <template #default="{ row }">
              <span v-if="row.whitelist && row.whitelist.length > 0" class="cfg-muted">
                {{ row.whitelist.length }} 人
              </span>
              <span v-else class="cfg-muted">—</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openFlagDialog(row)">编辑</el-button>
              <el-popconfirm title="确认删除？" @confirm="deleteFlag(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 版本发布 -->
      <el-tab-pane label="版本发布" name="releases">
        <div class="cfg-toolbar">
          <el-button type="primary" size="default" @click="openReleaseDialog(null)">新建发布</el-button>
          <el-button size="default" @click="fetchReleases">刷新</el-button>
        </div>
        <el-table :data="releases" stripe size="default" empty-text="暂无发布记录">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column label="版本号" width="100">
            <template #default="{ row }">
              <span class="cfg-version">v{{ row.version }}</span>
            </template>
          </el-table-column>
          <el-table-column label="标题" min-width="150" show-overflow-tooltip>
            <template #default="{ row }">{{ row.title || '—' }}</template>
          </el-table-column>
          <el-table-column label="类型" width="80">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ releaseTypeLabel(row.type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="90">
            <template #default="{ row }">
              <el-tag :type="releaseStatusTag(row.status)" size="small" effect="plain">
                {{ releaseStatusLabel(row.status) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="部署时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.released_at) || '—' }}</template>
          </el-table-column>
          <el-table-column label="创建时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <!-- 部署按钮：draft 状态可部署，deployed 是当前线上 -->
              <el-button
                v-if="row.status === 'draft'"
                size="small" type="success"
                :loading="deployingId === row.id"
                @click="deployRelease(row)"
              >部署上线</el-button>
              <el-tag v-else-if="row.status === 'deployed'" type="success" size="small" effect="dark">当前线上</el-tag>
              <!-- 导出快照 -->
              <el-button size="small" text type="primary" @click="exportRelease(row)">导出</el-button>
              <el-button size="small" text type="info" @click="openReleaseDialog(row)">详情</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <!-- 功能开关编辑弹窗 -->
    <el-dialog v-model="flagDialogVisible" :title="flagForm.id ? '编辑开关' : '新建开关'" width="520px" destroy-on-close>
      <el-form :model="flagForm" label-width="90px" size="default">
        <el-form-item label="Key">
          <el-input v-model="flagForm.key" placeholder="如: new_homepage_enabled" :disabled="!!flagForm.id" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="flagForm.description" placeholder="功能描述" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="flagForm.enabled" />
        </el-form-item>
        <el-form-item label="灰度比例">
          <div class="cfg-form-rollout">
            <el-slider v-model="flagForm.rollout_pct" :min="0" :max="100" :step="5" style="width: 300px" />
            <span class="cfg-form-rollout__val">{{ flagForm.rollout_pct }}%</span>
          </div>
        </el-form-item>
        <el-form-item label="白名单用户">
          <el-input
            v-model="flagForm.whitelistInput"
            type="textarea"
            :rows="3"
            placeholder="用户ID，逗号分隔"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="flagDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveFlag">保存</el-button>
      </template>
    </el-dialog>

    <!-- 发布详情 / 新建弹窗 -->
    <el-dialog v-model="releaseDialogVisible" :title="releaseForm.id ? '发布详情' : '新建发布'" width="560px" destroy-on-close>
      <p v-if="!releaseForm.id" class="cfg-hint">新建发布将自动快照当前所有功能开关状态。</p>
      <el-form :model="releaseForm" label-width="90px" size="default" :disabled="!!releaseForm.id">
        <el-form-item label="版本号">
          <el-input v-model="releaseForm.version" placeholder="如: 1.2.0" :disabled="!!releaseForm.id" />
        </el-form-item>
        <el-form-item label="标题">
          <el-input v-model="releaseForm.title" placeholder="本次发布名称" :disabled="!!releaseForm.id" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="releaseForm.type" style="width: 100%" :disabled="!!releaseForm.id">
            <el-option label="灰度发布" value="canary" />
            <el-option label="全量发布" value="full" />
            <el-option label="热修复" value="hotfix" />
          </el-select>
        </el-form-item>
        <el-form-item label="发布说明">
          <el-input v-model="releaseForm.notes" type="textarea" :rows="4" :disabled="!!releaseForm.id" />
        </el-form-item>
      </el-form>
      <!-- 详情视图（查看已有发布时） -->
      <template v-if="releaseForm.id">
        <el-descriptions :column="1" border size="small" style="margin-top: 12px">
          <el-descriptions-item label="状态">
            <el-tag :type="releaseStatusTag(releaseForm.status)" size="small">{{ releaseStatusLabel(releaseForm.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="类型">
            <el-tag size="small" effect="plain">{{ releaseTypeLabel(releaseForm.type) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="部署时间">{{ fmtTime(releaseForm.released_at) || '尚未部署' }}</el-descriptions-item>
          <el-descriptions-item label="包含开关数">
            {{ releaseForm.flag_count ?? '—' }}
          </el-descriptions-item>
          <el-descriptions-item label="发布说明">{{ releaseForm.notes || '—' }}</el-descriptions-item>
        </el-descriptions>
        <div style="margin-top: 12px; display: flex; gap: 8px;">
          <el-button v-if="releaseForm.status === 'draft'" size="small" type="success" @click="deployRelease(releaseForm)">部署上线</el-button>
          <el-button size="small" @click="viewSnapshot(releaseForm.id)">查看快照</el-button>
          <el-button size="small" @click="exportRelease(releaseForm)">下载快照</el-button>
        </div>
      </template>
      <template #footer>
        <el-button @click="releaseDialogVisible = false">关闭</el-button>
        <el-button v-if="!releaseForm.id" type="primary" :loading="saving" @click="createRelease">创建</el-button>
      </template>
    </el-dialog>

    <!-- 快照查看弹窗 -->
    <el-dialog v-model="snapshotVisible" title="配置快照" width="700px" destroy-on-close>
      <pre class="cfg-snapshot">{{ snapshotContent }}</pre>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage, ElMessageBox } from 'element-plus'

const API_BASE = import.meta.env.VITE_API_BASE || '/api/v1'
const ADMIN_API = API_BASE.replace('/api/v1', '/api/v1/admin')

async function api(path, opts = {}) {
  const m = (opts.method || 'GET').toLowerCase()
  const url = ADMIN_API + path
  if (m === 'post') return http.post(url, opts.body || {})
  if (m === 'put') return http.put(url, opts.body || {})
  if (m === 'delete') return http.delete(url)
  return http.get(url)
}

const loading = ref(false)
const saving = ref(false)
const deployingId = ref(null)
const activeTab = ref('flags')

// Flags
const flags = ref([])
const flagDialogVisible = ref(false)
const flagForm = reactive({
  id: null, key: '', description: '', enabled: false, rollout_pct: 0, whitelist: [], whitelistInput: '',
})

// Releases
const releases = ref([])
const releaseDialogVisible = ref(false)
const releaseForm = reactive({
  id: null, version: '', title: '', type: 'canary', notes: '', status: '', released_at: null, has_snapshot: false, flag_count: null,
})

// Snapshot viewer
const snapshotVisible = ref(false)
const snapshotContent = ref('')

// Module wizard
const modForm = reactive({ key: '', desc: '', location: '', enabled: true, version: '', notes: '' })
const modRunning = ref(false)
const modDone = ref(false)
const modReleaseId = ref(null)
const modProgress = ref([])

// ── Module wizard: 一键注册 + 发布 ──
async function runModuleWizard() {
  if (!modForm.key.trim()) return ElMessage.warning('请输入功能开关 Key')
  if (!modForm.version.trim()) return ElMessage.warning('请输入发布版本号')
  const ver = modForm.version.trim()
  const key = modForm.key.trim().toLowerCase().replace(/[^a-z0-9_]/g, '_')

  modProgress.value = [
    { text: `创建功能开关 ${key}`, active: true, ok: false, fail: false },
    { text: `新建发布 v${ver}`, active: false, ok: false, fail: false },
    { text: `部署上线`, active: false, ok: false, fail: false },
  ]
  modRunning.value = true
  modDone.value = false

  let flagId = null
  let releaseId = null

  // Step 1: Create flag
  try {
    const r = await api('/config/feature-flags', {
      method: 'POST',
      body: { key, description: modForm.desc, enabled: modForm.enabled, rollout_pct: modForm.enabled ? 100 : 0 },
    })
    flagId = r.id
    modProgress.value[0] = { ...modProgress.value[0], active: false, ok: true }
  } catch (e) {
    modProgress.value[0] = { ...modProgress.value[0], active: false, fail: true, text: `创建开关失败: ${e.message}` }
    modRunning.value = false
    return
  }

  // Step 2: Create release (auto-snapshot)
  try {
    modProgress.value[1] = { ...modProgress.value[1], active: true }
    const r = await api('/config/releases', {
      method: 'POST',
      body: { version: ver, title: modForm.desc || key, type: 'canary', notes: modForm.notes || `新模块: ${key} (位置: ${modForm.location || '—'})` },
    })
    releaseId = r.id
    modProgress.value[1] = { ...modProgress.value[1], active: false, ok: true }
  } catch (e) {
    modProgress.value[1] = { ...modProgress.value[1], active: false, fail: true, text: `创建发布失败: ${e.message}` }
    modRunning.value = false
    return
  }

  // Step 3: Deploy
  try {
    modProgress.value[2] = { ...modProgress.value[2], active: true }
    const r = await api(`/config/releases/${releaseId}/deploy`, { method: 'POST' })
    modProgress.value[2] = { ...modProgress.value[2], active: false, ok: true, text: `部署成功！已应用 ${r.applied} 个开关` }
  } catch (e) {
    modProgress.value[2] = { ...modProgress.value[2], active: false, fail: true, text: `部署失败: ${e.message}` }
    modRunning.value = false
    return
  }

  modRunning.value = false
  modDone.value = true
  modReleaseId.value = releaseId
  ElMessage.success('模块注册并上线完成！')
  // 刷新发布列表并自动切换 tab
  fetchReleases()
  activeTab.value = 'releases'
}

async function fetchFlags() {
  loading.value = true
  try {
    const res = await http.get(ADMIN_API + '/config/feature-flags')
    const data = res && res.data ? res.data : (res || {})
    const items = Array.isArray(data.items) ? data.items : 
                  Array.isArray(data) ? data : []
    flags.value = items
    console.log('Flags loaded:', items.length)
    if (!items.length) ElMessage.warning('功能开关列表为空')
  } catch (e) {
    console.error('fetchFlags error:', e)
    ElMessage.error('加载失败: ' + (e.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

async function fetchReleases() {
  loading.value = true
  try {
    const res = await http.get(ADMIN_API + '/config/releases')
    const data = res && res.data ? res.data : (res || {})
    const items = Array.isArray(data.items) ? data.items : 
                  Array.isArray(data) ? data : []
    releases.value = items
    console.log('Releases loaded:', items.length)
    if (!items.length) ElMessage.warning('发布列表为空，请先创建发布')
  } catch (e) {
    console.error('fetchReleases error:', e)
    ElMessage.error('加载失败: ' + (e.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

function onTabChange(tab) {
  console.log('[ConfigManage] tab changed to:', tab)
  if (tab === 'flags') fetchFlags()
  if (tab === 'releases') fetchReleases()
}

async function toggleFlag(row) {
  const newVal = row.enabled
  try {
    await api(`/config/feature-flags/${row.id}`, { method: 'PUT', body: { enabled: newVal } })
    ElMessage({ message: newVal ? `✅ ${row.key} 已启用` : `⏸ ${row.key} 已关闭`, type: 'success', duration: 1500 })
    // 同步到白名单/灰度也保持一致
    await api(`/config/feature-flags/${row.id}`, { method: 'PUT', body: { rollout_pct: newVal ? 100 : 0 } })
  } catch (e) {
    row.enabled = !newVal // 回退
    ElMessage.error('操作失败: ' + (e.message || ''))
  }
}

async function updateRollout(row) {
  try {
    await api(`/config/feature-flags/${row.id}`, { method: 'PUT', body: { rollout_pct: row.rollout_pct } })
  } catch (e) {
    ElMessage.error(e.message || '更新失败')
  }
}

function openFlagDialog(row) {
  if (row) {
    Object.assign(flagForm, {
      id: row.id, key: row.key, description: row.description || '',
      enabled: row.enabled, rollout_pct: row.rollout_pct || 0,
      whitelist: row.whitelist || [], whitelistInput: (row.whitelist || []).join(', '),
    })
  } else {
    Object.assign(flagForm, {
      id: null, key: '', description: '', enabled: false, rollout_pct: 0, whitelist: [], whitelistInput: '',
    })
  }
  flagDialogVisible.value = true
}

async function saveFlag() {
  if (!flagForm.key.trim()) {
    ElMessage.warning('请输入Key')
    return
  }
  saving.value = true
  try {
    const whitelist = flagForm.whitelistInput
      .split(/[,\n\s]+/)
      .map(s => s.trim())
      .filter(Boolean)
    const payload = {
      key: flagForm.key,
      description: flagForm.description,
      enabled: flagForm.enabled,
      rollout_pct: flagForm.rollout_pct,
      whitelist,
    }
    if (flagForm.id) {
      await api(`/config/feature-flags/${flagForm.id}`, { method: 'PUT', body: payload })
    } else {
      await api('/config/feature-flags', { method: 'POST', body: payload })
    }
    ElMessage.success('已保存')
    flagDialogVisible.value = false
    fetchFlags()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteFlag(row) {
  try {
    await api(`/config/feature-flags/${row.id}`, { method: 'DELETE' })
    ElMessage({ message: `🗑 已删除 ${row.key}`, type: 'success' })
    fetchFlags()
  } catch (e) {
    ElMessage.error('删除失败: ' + (e.message || ''))
  }
}

// ── Release ──

function openReleaseDialog(row) {
  if (row) {
    Object.assign(releaseForm, {
      id: row.id, version: row.version, title: row.title || '',
      type: row.type || 'canary', notes: row.notes || '',
      status: row.status || '', released_at: row.released_at,
      has_snapshot: !!row.has_snapshot,
      flag_count: row.flag_count,
    })
  } else {
    Object.assign(releaseForm, {
      id: null, version: '', title: '', type: 'canary', notes: '', status: '', released_at: null, has_snapshot: false, flag_count: null,
    })
  }
  releaseDialogVisible.value = true
}

async function createRelease() {
  if (!releaseForm.version.trim() || !releaseForm.title.trim()) {
    ElMessage.warning('请填写版本号和标题')
    return
  }
  saving.value = true
  try {
    await api('/config/releases', {
      method: 'POST',
      body: {
        version: releaseForm.version,
        title: releaseForm.title,
        type: releaseForm.type,
        notes: releaseForm.notes,
      },
    })
    ElMessage.success('已创建发布，当前功能开关状态已快照')
    releaseDialogVisible.value = false
    fetchReleases()
  } catch (e) {
    ElMessage.error(e.message || '创建失败')
  } finally {
    saving.value = false
  }
}

// 部署上线：将发布快照中的配置应用到线上
async function deployRelease(row) {
  try {
    await ElMessageBox.confirm(
      `确认部署版本 v${row.version} 到线上？\n\n快照中的功能开关状态将被应用到当前系统，之前已部署的版本将自动回退。`,
      '确认部署',
      { type: 'warning', confirmButtonText: '部署', cancelButtonText: '取消' }
    )
  } catch { return }

  deployingId.value = row.id
  try {
    const r = await api(`/config/releases/${row.id}/deploy`, { method: 'POST' })
    ElMessage.success(`部署成功！已应用 ${r.applied} 个功能开关`)
    fetchFlags()
    fetchReleases()
  } catch (e) {
    ElMessage.error(e.message || '部署失败')
  } finally {
    deployingId.value = null
  }
}

// 导出配置快照（下载 JSON）
function exportRelease(row) {
  const url = `${ADMIN_API}/config/releases/${row.id}/export`
  const a = document.createElement('a')
  a.href = url
  a.download = `config-snapshot-v${row.version}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  ElMessage.success('正在下载快照...')
}

// 查看快照内容
async function viewSnapshot(id) {
  try {
    const d = await api(`/config/releases/${id}/snapshot`)
    snapshotContent.value = JSON.stringify(d.snapshot, null, 2)
    snapshotVisible.value = true
  } catch (e) {
    ElMessage.error('加载快照失败')
  }
}

function releaseStatusLabel(s) {
  return {
    draft: '草稿', deployed: '已部署', rolled_back: '已回退',
  }[s] || s
}

function releaseStatusTag(s) {
  return {
    draft: 'info', deployed: 'success', rolled_back: 'danger',
  }[s] || ''
}

function releaseTypeLabel(t) {
  return { canary: '灰度', full: '全量', hotfix: '热修' }[t] || t
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => fetchFlags())
</script>

<style scoped>
.cfg-page { padding: 20px 24px; }
.cfg-page__head { margin-bottom: 14px; }
.cfg-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.cfg-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.cfg-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; }
.cfg-muted { color: #9499a0; font-size: 12px; }
.cfg-code { font-family: 'Courier New', monospace; font-size: 12px; color: #00a1d6; background: #f0f7ff; padding: 2px 6px; border-radius: 3px; }
.cfg-version { font-weight: 600; color: #18191c; }
.cfg-rollout { display: flex; align-items: center; gap: 8px; }
.cfg-rollout__val { font-size: 12px; color: #61666d; min-width: 34px; }
.cfg-form-rollout { display: flex; align-items: center; gap: 10px; }
.cfg-form-rollout__val { font-size: 13px; font-weight: 600; color: #00a1d6; min-width: 40px; }
.cfg-snapshot { background: #f6f7f8; border: 1px solid #e3e5e7; border-radius: 6px; padding: 16px; font-size: 12px; line-height: 1.6; max-height: 500px; overflow-y: auto; white-space: pre-wrap; word-break: break-all; }
.cfg-hint { font-size: 13px; color: #9499a0; margin-bottom: 16px; }

/* Module wizard */
.cfg-module-wizard { padding: 10px 0; }
.cfg-module-wizard__header { margin-bottom: 20px; }
.cfg-module-wizard__header h3 { margin: 0 0 6px; font-size: 16px; font-weight: 600; }
.cfg-module-wizard__header p { margin: 0; font-size: 13px; color: #9499a0; }
.cfg-module-progress { margin: 16px 0; padding: 12px 16px; background: #f6f7f8; border-radius: 6px; }
.cfg-module-progress__step { display: flex; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; color: #9499a0; transition: color 0.3s; }
.cfg-module-progress__step.active { color: #00a1d6; font-weight: 500; }
.cfg-module-progress__step.done { color: #52c41a; }
.cfg-module-progress__step.fail { color: #ff4d4f; }
.step-dot { font-size: 14px; min-width: 16px; }
.step-text { flex: 1; }
</style>
