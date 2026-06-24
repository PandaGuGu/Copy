<template>
  <div class="op-page" v-loading="loading">
    <header class="op-page__head">
      <h2 class="op-page__title">运维监控</h2>
      <p class="op-page__desc">任务队列、告警、链路追踪、CDN/存储与系统健康</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 任务队列 -->
      <el-tab-pane label="任务队列" name="tasks">
        <div class="op-toolbar">
          <el-select v-model="taskFilter.status" placeholder="状态" clearable size="small" style="width: 110px" @change="fetchTasks">
            <el-option label="全部" value="" />
            <el-option label="成功" value="success" />
            <el-option label="失败" value="failed" />
            <el-option label="重试中" value="retrying" />
            <el-option label="处理中" value="processing" />
          </el-select>
          <el-button type="primary" size="small" @click="fetchTasks">刷新</el-button>
        </div>
        <el-table :data="tasks" stripe size="default" empty-text="暂无任务">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column label="类型" width="140">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ row.type }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="90">
            <template #default="{ row }">
              <el-tag :type="taskStatusTag(row.status)" size="small">{{ taskStatusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="错误信息" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">
              <span v-if="row.error" class="op-error">{{ row.error }}</span>
              <span v-else class="op-muted">—</span>
            </template>
          </el-table-column>
          <el-table-column label="耗时" width="80">
            <template #default="{ row }">{{ row.duration ? row.duration + 'ms' : '—' }}</template>
          </el-table-column>
          <el-table-column label="创建时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="90" fixed="right">
            <template #default="{ row }">
              <el-button
                v-if="row.status === 'failed' || row.status === 'retrying'"
                size="small" text type="primary"
                @click="retryTask(row)"
              >重试</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 告警 -->
      <el-tab-pane label="告警" name="alerts">
        <div class="op-subsection">
          <h4 class="op-subsection__title">告警规则</h4>
          <div class="op-toolbar">
            <el-button type="primary" size="small" @click="openAlertRuleDialog(null)">新建规则</el-button>
          </div>
          <el-table :data="alertRules" stripe size="small" empty-text="暂无规则">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column prop="name" label="名称" min-width="120" />
            <el-table-column label="指标" width="120">
              <template #default="{ row }">{{ row.metric }}</template>
            </el-table-column>
            <el-table-column label="条件" width="100">
              <template #default="{ row }">{{ row.operator }} {{ row.threshold }}</template>
            </el-table-column>
            <el-table-column label="启用" width="70">
              <template #default="{ row }">
                <el-switch v-model="row.enabled" @change="toggleAlertRule(row)" />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="openAlertRuleDialog(row)">编辑</el-button>
                <el-popconfirm title="确认删除？" @confirm="deleteAlertRule(row)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <div class="op-subsection">
          <h4 class="op-subsection__title">告警记录</h4>
          <el-table :data="alertRecords" stripe size="default" empty-text="暂无告警">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column label="规则" width="120">
              <template #default="{ row }">{{ row.rule_name || row.rule_id }}</template>
            </el-table-column>
            <el-table-column label="级别" width="80">
              <template #default="{ row }">
                <el-tag :type="alertLevelTag(row.level)" size="small" effect="dark">{{ row.level }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="内容" min-width="200" show-overflow-tooltip>
              <template #default="{ row }">{{ row.message }}</template>
            </el-table-column>
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.acknowledged ? 'success' : 'danger'" size="small" effect="plain">
                  {{ row.acknowledged ? '已确认' : '待确认' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="时间" width="155">
              <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="80" fixed="right">
              <template #default="{ row }">
                <el-button
                  v-if="!row.acknowledged"
                  size="small" text type="primary"
                  @click="ackAlert(row)"
                >确认</el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-tab-pane>

      <!-- 链路追踪 -->
      <el-tab-pane label="链路追踪" name="traces">
        <div class="op-toolbar">
          <el-input v-model="traceSearch.trace_id" placeholder="Trace ID" clearable size="small" style="width: 180px" @keyup.enter="searchTraces" />
          <el-input v-model="traceSearch.request_id" placeholder="Request ID" clearable size="small" style="width: 180px" @keyup.enter="searchTraces" />
          <el-input v-model="traceSearch.user_id" placeholder="User ID" clearable size="small" style="width: 140px" @keyup.enter="searchTraces" />
          <el-button type="primary" size="small" @click="searchTraces">搜索</el-button>
        </div>
        <el-table :data="traces" stripe size="default" empty-text="输入条件搜索">
          <el-table-column prop="trace_id" label="Trace ID" width="180" show-overflow-tooltip />
          <el-table-column prop="request_id" label="Request ID" width="180" show-overflow-tooltip />
          <el-table-column label="接口" min-width="160" show-overflow-tooltip>
            <template #default="{ row }">{{ row.method }} {{ row.path }}</template>
          </el-table-column>
          <el-table-column label="状态码" width="80">
            <template #default="{ row }">
              <el-tag :type="row.status_code >= 400 ? 'danger' : 'success'" size="small">{{ row.status_code }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="耗时" width="80">
            <template #default="{ row }">{{ row.duration }}ms</template>
          </el-table-column>
          <el-table-column label="用户ID" width="90">
            <template #default="{ row }">{{ row.user_id || '—' }}</template>
          </el-table-column>
          <el-table-column label="时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- CDN/存储 -->
      <el-tab-pane label="CDN / 存储" name="cdn">
        <div class="op-subsection">
          <h4 class="op-subsection__title">CDN 刷新任务</h4>
          <div class="op-toolbar">
            <el-button type="primary" size="small" @click="openCdnDialog">新建刷新</el-button>
            <el-button size="small" @click="fetchCdnTasks">刷新列表</el-button>
          </div>
          <el-table :data="cdnTasks" stripe size="default" empty-text="暂无刷新任务">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column label="类型" width="90">
              <template #default="{ row }">
                <el-tag size="small" effect="plain">{{ row.refresh_type === 'directory' ? '目录' : 'URL' }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="URLs" min-width="200" show-overflow-tooltip>
              <template #default="{ row }">{{ Array.isArray(row.urls) ? row.urls.join(', ') : row.urls }}</template>
            </el-table-column>
            <el-table-column label="状态" width="90">
              <template #default="{ row }">
                <el-tag :type="cdnStatusTag(row.status)" size="small">{{ cdnStatusLabel(row.status) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="创建时间" width="155">
              <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
            </el-table-column>
          </el-table>
        </div>

        <div class="op-subsection">
          <h4 class="op-subsection__title">存储生命周期规则</h4>
          <div class="op-toolbar">
            <el-button type="primary" size="small" @click="openLifecycleDialog(null)">新建规则</el-button>
          </div>
          <el-table :data="lifecycleRules" stripe size="default" empty-text="暂无规则">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column prop="name" label="名称" min-width="120" />
            <el-table-column prop="bucket" label="Bucket" width="120" />
            <el-table-column label="前缀" width="100">
              <template #default="{ row }">{{ row.prefix || '/' }}</template>
            </el-table-column>
            <el-table-column label="转入IA" width="80">
              <template #default="{ row }">{{ row.ia_days ? row.ia_days + '天' : '—' }}</template>
            </el-table-column>
            <el-table-column label="转入Archive" width="90">
              <template #default="{ row }">{{ row.archive_days ? row.archive_days + '天' : '—' }}</template>
            </el-table-column>
            <el-table-column label="删除" width="70">
              <template #default="{ row }">{{ row.delete_days ? row.delete_days + '天' : '—' }}</template>
            </el-table-column>
            <el-table-column label="启用" width="70">
              <template #default="{ row }">
                <el-switch v-model="row.enabled" @change="toggleLifecycle(row)" />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="openLifecycleDialog(row)">编辑</el-button>
                <el-popconfirm title="确认删除？" @confirm="deleteLifecycle(row)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-tab-pane>

      <!-- 系统健康 -->
      <el-tab-pane label="系统健康" name="health">
        <div class="op-health-grid">
          <div
            v-for="h in healthStatus"
            :key="h.name"
            class="op-health-card"
            :class="{ 'op-health-card--ok': h.status === 'ok', 'op-health-card--err': h.status !== 'ok' }"
          >
            <div class="op-health-card__icon" :class="h.status === 'ok' ? 'op-health-card__icon--ok' : 'op-health-card__icon--err'">
              {{ h.status === 'ok' ? '✓' : '✗' }}
            </div>
            <div class="op-health-card__info">
              <div class="op-health-card__name">{{ h.name }}</div>
              <div class="op-health-card__detail">{{ h.detail }}</div>
              <div class="op-health-card__latency" v-if="h.latency">{{ h.latency }}ms</div>
            </div>
          </div>
        </div>
        <div class="op-health-actions">
          <el-button type="primary" size="small" @click="fetchHealth">刷新状态</el-button>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- CDN刷新弹窗 -->
    <el-dialog v-model="cdnDialogVisible" title="新建CDN刷新" width="480px" destroy-on-close>
      <el-form :model="cdnForm" label-width="80px" size="default">
        <el-form-item label="刷新类型">
          <el-radio-group v-model="cdnForm.refresh_type">
            <el-radio value="url">URL刷新</el-radio>
            <el-radio value="directory">目录刷新</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="URL列表">
          <el-input
            v-model="cdnForm.urls"
            type="textarea"
            :rows="5"
            placeholder="每行一个URL"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="cdnDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="createCdnTask">提交</el-button>
      </template>
    </el-dialog>

    <!-- 告警规则弹窗 -->
    <el-dialog v-model="alertRuleDialogVisible" :title="alertRuleForm.id ? '编辑规则' : '新建规则'" width="480px" destroy-on-close>
      <el-form :model="alertRuleForm" label-width="70px" size="default">
        <el-form-item label="名称">
          <el-input v-model="alertRuleForm.name" placeholder="规则名称" />
        </el-form-item>
        <el-form-item label="指标">
          <el-input v-model="alertRuleForm.metric" placeholder="如 cpu_usage, memory_usage" />
        </el-form-item>
        <el-form-item label="条件">
          <el-select v-model="alertRuleForm.operator" style="width: 90px">
            <el-option label=">" value=">" />
            <el-option label="<" value="<" />
            <el-option label=">=" value=">=" />
            <el-option label="<=" value="<=" />
            <el-option label="==" value="==" />
          </el-select>
          <el-input v-model="alertRuleForm.threshold" placeholder="阈值" style="width: 140px; margin-left: 8px" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="alertRuleForm.enabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="alertRuleDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveAlertRule">保存</el-button>
      </template>
    </el-dialog>

    <!-- 生命周期规则弹窗 -->
    <el-dialog v-model="lifecycleDialogVisible" :title="lifecycleForm.id ? '编辑规则' : '新建规则'" width="480px" destroy-on-close>
      <el-form :model="lifecycleForm" label-width="100px" size="default">
        <el-form-item label="名称">
          <el-input v-model="lifecycleForm.name" />
        </el-form-item>
        <el-form-item label="Bucket">
          <el-input v-model="lifecycleForm.bucket" />
        </el-form-item>
        <el-form-item label="前缀">
          <el-input v-model="lifecycleForm.prefix" placeholder="/" />
        </el-form-item>
        <el-form-item label="转入IA(天)">
          <el-input-number v-model="lifecycleForm.ia_days" :min="0" />
        </el-form-item>
        <el-form-item label="转入Archive(天)">
          <el-input-number v-model="lifecycleForm.archive_days" :min="0" />
        </el-form-item>
        <el-form-item label="删除(天)">
          <el-input-number v-model="lifecycleForm.delete_days" :min="0" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="lifecycleForm.enabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="lifecycleDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveLifecycle">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage } from 'element-plus'

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
const activeTab = ref('tasks')

// Tasks
const tasks = ref([])
const taskFilter = reactive({ status: '' })

// Alerts
const alertRules = ref([])
const alertRecords = ref([])
const alertRuleDialogVisible = ref(false)
const alertRuleForm = reactive({ id: null, name: '', metric: '', operator: '>', threshold: '', enabled: true })

// Traces
const traces = ref([])
const traceSearch = reactive({ trace_id: '', request_id: '', user_id: '' })

// CDN
const cdnTasks = ref([])
const cdnDialogVisible = ref(false)
const cdnForm = reactive({ refresh_type: 'url', urls: '' })

// Lifecycle
const lifecycleRules = ref([])
const lifecycleDialogVisible = ref(false)
const lifecycleForm = reactive({ id: null, name: '', bucket: '', prefix: '/', ia_days: 0, archive_days: 0, delete_days: 0, enabled: true })

// Health
const healthStatus = ref([])

async function fetchTasks() {
  loading.value = true
  try {
    const params = new URLSearchParams()
    if (taskFilter.status) params.set('status', taskFilter.status)
    const d = await api(`/ops/tasks?${params}`)
    tasks.value = d.items || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function retryTask(row) {
  try {
    await api(`/ops/tasks/${row.id}/retry`, { method: 'POST' })
    ElMessage.success('已提交重试')
    fetchTasks()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

async function fetchAlerts() {
  loading.value = true
  try {
    const [rulesD, recordsD] = await Promise.all([
      api('/ops/alert-rules'),
      api('/ops/alert-records'),
    ])
    alertRules.value = rulesD.items || rulesD || []
    alertRecords.value = recordsD.items || recordsD || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function ackAlert(row) {
  try {
    await api(`/ops/alert-records/${row.id}/ack`, { method: 'POST' })
    ElMessage.success('已确认')
    fetchAlerts()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

function openAlertRuleDialog(row) {
  if (row) {
    Object.assign(alertRuleForm, { id: row.id, name: row.name, metric: row.metric, operator: row.operator, threshold: row.threshold, enabled: row.enabled })
  } else {
    Object.assign(alertRuleForm, { id: null, name: '', metric: '', operator: '>', threshold: '', enabled: true })
  }
  alertRuleDialogVisible.value = true
}

async function saveAlertRule() {
  saving.value = true
  try {
    if (alertRuleForm.id) {
      await api(`/ops/alert-rules/${alertRuleForm.id}`, { method: 'PUT', body: { ...alertRuleForm } })
    } else {
      await api('/ops/alert-rules', { method: 'POST', body: { ...alertRuleForm } })
    }
    ElMessage.success('已保存')
    alertRuleDialogVisible.value = false
    fetchAlerts()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteAlertRule(row) {
  try {
    await api(`/ops/alert-rules/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    fetchAlerts()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

async function toggleAlertRule(row) {
  try {
    await api(`/ops/alert-rules/${row.id}`, { method: 'PUT', body: { enabled: row.enabled } })
  } catch (e) {
    row.enabled = !row.enabled
    ElMessage.error(e.message || '操作失败')
  }
}

async function searchTraces() {
  if (!traceSearch.trace_id && !traceSearch.request_id && !traceSearch.user_id) {
    ElMessage.warning('请输入搜索条件')
    return
  }
  loading.value = true
  try {
    const params = new URLSearchParams()
    if (traceSearch.trace_id) params.set('trace_id', traceSearch.trace_id)
    if (traceSearch.request_id) params.set('request_id', traceSearch.request_id)
    if (traceSearch.user_id) params.set('user_id', traceSearch.user_id)
    const d = await api(`/ops/traces?${params}`)
    traces.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '搜索失败')
  } finally {
    loading.value = false
  }
}

async function fetchCdnTasks() {
  loading.value = true
  try {
    const d = await api('/ops/cdn/refresh-tasks')
    cdnTasks.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchLifecycleRules() {
  try {
    const d = await api('/ops/storage/lifecycle-rules')
    lifecycleRules.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  }
}

function openCdnDialog() {
  cdnForm.refresh_type = 'url'
  cdnForm.urls = ''
  cdnDialogVisible.value = true
}

async function createCdnTask() {
  const urls = cdnForm.urls.split('\n').map(s => s.trim()).filter(Boolean)
  if (urls.length === 0) {
    ElMessage.warning('请输入至少一个URL')
    return
  }
  saving.value = true
  try {
    await api('/ops/cdn/refresh', { method: 'POST', body: { refresh_type: cdnForm.refresh_type, urls } })
    ElMessage.success('刷新任务已创建')
    cdnDialogVisible.value = false
    fetchCdnTasks()
  } catch (e) {
    ElMessage.error(e.message || '创建失败')
  } finally {
    saving.value = false
  }
}

function openLifecycleDialog(row) {
  if (row) {
    Object.assign(lifecycleForm, { id: row.id, name: row.name, bucket: row.bucket, prefix: row.prefix || '/', ia_days: row.ia_days || 0, archive_days: row.archive_days || 0, delete_days: row.delete_days || 0, enabled: row.enabled })
  } else {
    Object.assign(lifecycleForm, { id: null, name: '', bucket: '', prefix: '/', ia_days: 0, archive_days: 0, delete_days: 0, enabled: true })
  }
  lifecycleDialogVisible.value = true
}

async function saveLifecycle() {
  saving.value = true
  try {
    if (lifecycleForm.id) {
      await api(`/ops/storage/lifecycle-rules/${lifecycleForm.id}`, { method: 'PUT', body: { ...lifecycleForm } })
    } else {
      await api('/ops/storage/lifecycle-rules', { method: 'POST', body: { ...lifecycleForm } })
    }
    ElMessage.success('已保存')
    lifecycleDialogVisible.value = false
    fetchLifecycleRules()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteLifecycle(row) {
  try {
    await api(`/ops/storage/lifecycle-rules/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    fetchLifecycleRules()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

async function toggleLifecycle(row) {
  try {
    await api(`/ops/storage/lifecycle-rules/${row.id}`, { method: 'PUT', body: { enabled: row.enabled } })
  } catch (e) {
    row.enabled = !row.enabled
    ElMessage.error(e.message || '操作失败')
  }
}

async function fetchHealth() {
  loading.value = true
  try {
    const d = await api('/ops/health')
    healthStatus.value = d.items || d || [
      { name: 'Database', status: 'ok', detail: 'Connected', latency: 2 },
      { name: 'Redis', status: 'ok', detail: 'Connected', latency: 1 },
      { name: 'OSS', status: 'ok', detail: 'Available', latency: 15 },
      { name: 'RabbitMQ', status: 'ok', detail: 'Connected', latency: 3 },
    ]
  } catch (e) {
    ElMessage.error(e.message || '获取健康状态失败')
  } finally {
    loading.value = false
  }
}

function onTabChange(tab) {
  if (tab === 'tasks' && tasks.value.length === 0) fetchTasks()
  if (tab === 'alerts' && alertRules.value.length === 0) fetchAlerts()
  if (tab === 'cdn') {
    if (cdnTasks.value.length === 0) fetchCdnTasks()
    if (lifecycleRules.value.length === 0) fetchLifecycleRules()
  }
  if (tab === 'health') fetchHealth()
}

function taskStatusLabel(s) {
  return { success: '成功', failed: '失败', retrying: '重试中', processing: '处理中' }[s] || s
}

function taskStatusTag(s) {
  return { success: 'success', failed: 'danger', retrying: 'warning', processing: 'info' }[s] || ''
}

function alertLevelTag(l) {
  return { critical: 'danger', warning: 'warning', info: 'info' }[l] || ''
}

function cdnStatusLabel(s) {
  return { pending: '处理中', done: '已完成', failed: '失败' }[s] || s
}

function cdnStatusTag(s) {
  return { pending: 'warning', done: 'success', failed: 'danger' }[s] || ''
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => fetchTasks())
</script>

<style scoped>
.op-page { padding: 20px 24px; }
.op-page__head { margin-bottom: 14px; }
.op-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.op-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.op-toolbar { margin-bottom: 12px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.op-muted { color: #9499a0; }
.op-error { color: #f56c6c; font-size: 12px; }

.op-subsection { margin-bottom: 20px; }
.op-subsection__title { margin: 0 0 10px; font-size: 14px; font-weight: 600; color: #18191c; }

.op-health-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 14px; margin-bottom: 16px; }
.op-health-card { display: flex; align-items: center; gap: 14px; padding: 18px 20px; border-radius: 10px; border: 2px solid; }
.op-health-card--ok { border-color: #d4edda; background: #f0faf3; }
.op-health-card--err { border-color: #f5c6cb; background: #fdf0f0; }
.op-health-card__icon { width: 36px; height: 36px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; color: #fff; flex-shrink: 0; }
.op-health-card__icon--ok { background: #67c23a; }
.op-health-card__icon--err { background: #f56c6c; }
.op-health-card__name { font-size: 15px; font-weight: 600; color: #18191c; }
.op-health-card__detail { font-size: 12px; color: #61666d; margin-top: 2px; }
.op-health-card__latency { font-size: 11px; color: #9499a0; margin-top: 2px; }
.op-health-actions { margin-top: 8px; }
</style>
