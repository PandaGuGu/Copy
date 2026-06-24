<template>
  <div class="cfg-page" v-loading="loading">
    <header class="cfg-page__head">
      <h2 class="cfg-page__title">配置与发布</h2>
      <p class="cfg-page__desc">功能开关管理与版本发布控制</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
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
          <el-table-column label="版本号" width="110">
            <template #default="{ row }">
              <span class="cfg-version">v{{ row.version }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="title" label="标题" min-width="160" show-overflow-tooltip />
          <el-table-column label="状态" width="90">
            <template #default="{ row }">
              <el-tag :type="releaseStatusTag(row.status)" size="small" effect="plain">
                {{ releaseStatusLabel(row.status) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="发布类型" width="90">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ releaseTypeLabel(row.type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="发布时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.released_at) }}</template>
          </el-table-column>
          <el-table-column label="创建时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="140" fixed="right">
            <template #default="{ row }">
              <el-button
                v-if="row.status === 'released' || row.status === 'rolled_out'"
                size="small" text type="warning"
                @click="rollbackRelease(row)"
              >回滚</el-button>
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

    <!-- 发布详情弹窗 -->
    <el-dialog v-model="releaseDialogVisible" :title="releaseForm.id ? '发布详情' : '新建发布'" width="560px" destroy-on-close>
      <el-form :model="releaseForm" label-width="90px" size="default" :disabled="!!releaseForm.id">
        <el-form-item label="版本号">
          <el-input v-model="releaseForm.version" placeholder="如: 1.2.0" :disabled="!!releaseForm.id" />
        </el-form-item>
        <el-form-item label="标题">
          <el-input v-model="releaseForm.title" :disabled="!!releaseForm.id" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="releaseForm.type" style="width: 100%" :disabled="!!releaseForm.id">
            <el-option label="灰度发布" value="canary" />
            <el-option label="全量发布" value="full" />
            <el-option label="紧急发布" value="hotfix" />
          </el-select>
        </el-form-item>
        <el-form-item label="发布说明">
          <el-input v-model="releaseForm.notes" type="textarea" :rows="4" :disabled="!!releaseForm.id" />
        </el-form-item>
      </el-form>
      <template v-if="releaseForm.id">
        <el-descriptions :column="1" border size="small" style="margin-top: 12px">
          <el-descriptions-item label="状态">
            <el-tag :type="releaseStatusTag(releaseForm.status)" size="small">{{ releaseStatusLabel(releaseForm.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="发布时间">{{ fmtTime(releaseForm.released_at) }}</el-descriptions-item>
          <el-descriptions-item label="发布说明">{{ releaseForm.notes || '—' }}</el-descriptions-item>
        </el-descriptions>
      </template>
      <template #footer>
        <el-button @click="releaseDialogVisible = false">关闭</el-button>
        <el-button v-if="!releaseForm.id" type="primary" :loading="saving" @click="createRelease">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage, ElMessageBox } from 'element-plus'

const API_BASE = import.meta.env.VITE_API_BASE || '/api/v1'
const ADMIN_API = API_BASE.replace('/api/v1', '/api/v1/admin')

function getToken() { return localStorage.getItem("minibili_admin_access_token") || "" }
async function api(path, opts = {}) {
  const res = await fetch(ADMIN_API + path, {
    method: opts.method || "GET",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + getToken(), ...(opts.headers || {}) },
    body: opts.body ? JSON.stringify(opts.body) : undefined,
  })
  const body = await res.json()
  if (!res.ok || (body.code != null && body.code !== 0)) { throw new Error(body.msg || body.message || "请求失败") }
  return body.data || body
}

const loading = ref(false)
const saving = ref(false)
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
  id: null, version: '', title: '', type: 'canary', notes: '', status: '', released_at: null,
})

async function fetchFlags() {
  loading.value = true
  try {
    const d = await api('/config/feature-flags')
    flags.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchReleases() {
  loading.value = true
  try {
    const d = await api('/config/releases')
    releases.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function onTabChange(tab) {
  if (tab === 'flags' && flags.value.length === 0) fetchFlags()
  if (tab === 'releases' && releases.value.length === 0) fetchReleases()
}

async function toggleFlag(row) {
  try {
    await api(`/config/feature-flags/${row.id}`, { method: 'PUT', body: { enabled: row.enabled } })
    ElMessage.success(row.enabled ? '已启用' : '已禁用')
  } catch (e) {
    row.enabled = !row.enabled
    ElMessage.error(e.message || '操作失败')
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
    ElMessage.success('已删除')
    fetchFlags()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function openReleaseDialog(row) {
  if (row) {
    Object.assign(releaseForm, {
      id: row.id, version: row.version, title: row.title || '',
      type: row.type || 'canary', notes: row.notes || '',
      status: row.status || '', released_at: row.released_at,
    })
  } else {
    Object.assign(releaseForm, {
      id: null, version: '', title: '', type: 'canary', notes: '', status: '', released_at: null,
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
    ElMessage.success('已创建')
    releaseDialogVisible.value = false
    fetchReleases()
  } catch (e) {
    ElMessage.error(e.message || '创建失败')
  } finally {
    saving.value = false
  }
}

async function rollbackRelease(row) {
  try {
    await ElMessageBox.confirm(`确认回滚版本 v${row.version}？此操作将恢复到上一版本。`, '回滚确认', {
      type: 'warning',
    })
    await api(`/config/releases/${row.id}/rollback`, { method: 'POST' })
    ElMessage.success('已回滚')
    fetchReleases()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(e.message || '回滚失败')
  }
}

function releaseStatusLabel(s) {
  return { draft: '草稿', released: '已发布', rolled_out: '全量发布', rolled_back: '已回滚' }[s] || s
}

function releaseStatusTag(s) {
  return { draft: 'info', released: 'success', rolled_out: 'success', rolled_back: 'warning' }[s] || ''
}

function releaseTypeLabel(t) {
  return { canary: '灰度', full: '全量', hotfix: '紧急' }[t] || t
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
</style>
