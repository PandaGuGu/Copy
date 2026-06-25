<template>
  <div class="op-page" v-loading="loading">
    <div class="op-page-title">运维监控</div>
    <el-tabs v-model="activeTab" @tab-change="onTabChange" class="op-tabs">

      <!-- ==================== 任务队列 ==================== -->
      <el-tab-pane label="任务队列" name="tasks">
        <div class="op-panel">
          <div class="op-panel-head">
            <b class="op-panel-title">异步任务列表</b>
            <div class="op-panel-actions">
              <el-select v-model="taskFilter.status" placeholder="状态筛选" clearable size="small" style="width: 110px" @change="fetchTasks">
                <el-option label="全部" value="" />
                <el-option label="处理中" value="running" />
                <el-option label="成功" value="success" />
                <el-option label="失败" value="failed" />
                <el-option label="重试中" value="retrying" />
              </el-select>
              <el-button size="small" @click="fetchTasks">刷新</el-button>
            </div>
          </div>
          <div class="op-panel-body">
            <el-table :data="tasks" stripe size="small" empty-text="暂无任务记录" max-height="480">
            <el-table-column prop="id" label="ID" width="64" align="center" />
            <el-table-column label="类型" width="100">
              <template #default="{ row }">
                <el-tag size="small" effect="plain" type="info">{{ row.type || row.task_type }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="taskStatusTag(row.status)" size="small">{{ taskStatusLabel(row.status) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="错误信息" min-width="200" show-overflow-tooltip>
              <template #default="{ row }">
                <span v-if="row.error || row.error_msg" class="op-error">{{ row.error || row.error_msg }}</span>
                <span v-else class="op-muted">—</span>
              </template>
            </el-table-column>
            <el-table-column label="耗时" width="80" align="right">
              <template #default="{ row }">{{ row.duration ? row.duration + 'ms' : '—' }}</template>
            </el-table-column>
            <el-table-column label="时间" width="148">
              <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="72" align="center" fixed="right">
              <template #default="{ row }">
                <el-button
                  v-if="row.status === 'failed' || row.status === 'retrying'"
                  size="small" text type="primary"
                  @click="retryTask(row)"
                >重试</el-button>
              </template>
            </el-table-column>
          </el-table>
          </div>
        </div>
      </el-tab-pane>

      <!-- ==================== 告警 ==================== -->
      <el-tab-pane label="告警" name="alerts">
        <div class="op-section">
          <div class="op-panel">
            <div class="op-panel-head">
              <b class="op-panel-title">告警规则</b>
              <div class="op-panel-actions">
                <el-button size="small" @click="evaluateAlerts" :loading="evaluating">立即评估</el-button>
                <el-button type="primary" size="small" @click="openAlertRuleDialog(null)">新建规则</el-button>
              </div>
            </div>
            <div class="op-panel-body">
            <el-table :data="alertRules" stripe size="small" empty-text="暂无规则">
              <el-table-column prop="id" label="ID" width="64" align="center" />
              <el-table-column prop="name" label="名称" min-width="120" />
              <el-table-column label="指标" width="140">
                <template #default="{ row }">
                  <code class="op-code">{{ row.metric }}</code>
                </template>
              </el-table-column>
              <el-table-column label="条件" width="120" align="center">
                <template #default="{ row }">{{ row.operator }} {{ row.threshold }}</template>
              </el-table-column>
              <el-table-column label="启用" width="64" align="center">
                <template #default="{ row }">
                  <el-switch v-model="row.enabled" size="small" @change="toggleAlertRule(row)" />
                </template>
              </el-table-column>
              <el-table-column label="操作" width="120" align="center">
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
        </div>

          <div class="op-panel">
            <div class="op-panel-head">
              <b class="op-panel-title">告警记录</b>
            </div>
            <div class="op-panel-body">
            <el-table :data="alertRecords" stripe size="small" empty-text="暂无告警记录" max-height="360">
              <el-table-column prop="id" label="ID" width="64" align="center" />
              <el-table-column label="规则" width="120">
                <template #default="{ row }">{{ row.rule_name || row.rule_id }}</template>
              </el-table-column>
              <el-table-column label="级别" width="72" align="center">
                <template #default="{ row }">
                  <el-tag :type="alertLevelTag(row.level)" size="small" effect="dark">{{ row.level || 'info' }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="内容" min-width="200" show-overflow-tooltip>
                <template #default="{ row }">{{ row.message }}</template>
              </el-table-column>
              <el-table-column label="状态" width="80" align="center">
                <template #default="{ row }">
                  <el-tag :type="row.acknowledged ? 'success' : 'danger'" size="small" effect="plain">
                    {{ row.acknowledged ? '已确认' : '待确认' }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="时间" width="148">
                <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
              </el-table-column>
              <el-table-column label="操作" width="72" align="center" fixed="right">
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
          </div>
        </div>
      </el-tab-pane>

      <!-- ==================== 链路追踪 ==================== -->
      <el-tab-pane label="链路追踪" name="traces">
        <div class="op-panel">
          <div class="op-panel-head">
            <b class="op-panel-title">请求链路查询</b>
            <div class="op-panel-actions">
              <el-input v-model="traceSearch.trace_id" placeholder="Trace ID" clearable size="small" style="width: 200px" @keyup.enter="searchTraces" />
              <el-input v-model="traceSearch.request_id" placeholder="Request ID" clearable size="small" style="width: 200px" @keyup.enter="searchTraces" />
              <el-input v-model="traceSearch.user_id" placeholder="User ID" clearable size="small" style="width: 140px" @keyup.enter="searchTraces" />
              <el-button type="primary" size="small" @click="searchTraces">搜索</el-button>
            </div>
          </div>
          <div class="op-panel-body">
          <el-table :data="traces" stripe size="small" empty-text="输入条件后点击搜索" max-height="480">
            <el-table-column prop="trace_id" label="Trace ID" width="260" show-overflow-tooltip>
              <template #default="{ row }">
                <code class="op-code">{{ row.trace_id }}</code>
              </template>
            </el-table-column>
            <el-table-column label="接口" min-width="160">
              <template #default="{ row }">
                <el-tag size="small" effect="plain" type="info" style="margin-right:6px">{{ row.method }}</el-tag>
                <span class="op-path">{{ row.path }}</span>
              </template>
            </el-table-column>
            <el-table-column label="状态码" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="row.status_code >= 400 ? 'danger' : 'success'" size="small">{{ row.status_code || row.status }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="耗时" width="80" align="right">
              <template #default="{ row }">{{ row.duration || row.duration_ms }}ms</template>
            </el-table-column>
            <el-table-column label="用户ID" width="80" align="center">
              <template #default="{ row }">{{ row.user_id || '—' }}</template>
            </el-table-column>
            <el-table-column label="时间" width="148">
              <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
            </el-table-column>
          </el-table>
          </div>
        </div>
      </el-tab-pane>

      <!-- ==================== 系统健康 ==================== -->
      <el-tab-pane label="系统健康" name="health">
        <div class="op-panel">
          <div class="op-panel-head">
            <b class="op-panel-title">组件健康状态</b>
            <el-button size="small" @click="fetchHealth">刷新状态</el-button>
          </div>
          <div class="op-panel-body">
          <div class="op-health-grid">
            <div
              v-for="h in healthStatus"
              :key="h.name"
              class="op-health-card"
              :class="{ 'op-health-card--ok': h.status === 'ok', 'op-health-card--err': h.status === 'unavailable' || h.status === 'error' }"
            >
              <div class="op-health-card__icon" :class="h.status === 'ok' ? 'op-health-card__icon--ok' : 'op-health-card__icon--err'">
                {{ h.status === 'ok' ? '✓' : '✗' }}
              </div>
              <div class="op-health-card__info">
                <div class="op-health-card__name">{{ h.name }}</div>
                <div class="op-health-card__detail">{{ h.detail || (h.status === 'ok' ? 'Normal' : 'Unavailable') }}</div>
                <div class="op-health-card__latency" v-if="h.latency">{{ h.latency }}ms</div>
              </div>
            </div>
          </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- ==================== CDN/存储 ==================== -->
      <el-tab-pane label="CDN / 存储" name="cdn">
        <div class="op-section">
          <div class="op-panel">
            <div class="op-panel-head">
              <b class="op-panel-title">CDN 刷新任务</b>
              <div class="op-panel-actions">
                <el-button size="small" @click="fetchCdnTasks">刷新列表</el-button>
                <el-button type="primary" size="small" @click="openCdnDialog">新建刷新</el-button>
              </div>
            </div>
            <div class="op-panel-body">
            <el-table :data="cdnTasks" stripe size="small" empty-text="暂无刷新任务">
              <el-table-column prop="id" label="ID" width="64" align="center" />
              <el-table-column label="类型" width="80" align="center">
                <template #default="{ row }">
                  <el-tag size="small" effect="plain">{{ row.refresh_type === 'directory' ? '目录' : 'URL' }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="URLs" min-width="200" show-overflow-tooltip>
                <template #default="{ row }">{{ Array.isArray(row.urls) ? row.urls.join(', ') : row.urls }}</template>
              </el-table-column>
              <el-table-column label="状态" width="80" align="center">
                <template #default="{ row }">
                  <el-tag :type="cdnStatusTag(row.status)" size="small">{{ cdnStatusLabel(row.status) }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="时间" width="148">
                <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
              </el-table-column>
            </el-table>
            </div>
          </div>

          <div class="op-panel">
            <div class="op-panel-head">
              <b class="op-panel-title">存储生命周期规则</b>
              <el-button type="primary" size="small" @click="openLifecycleDialog(null)">新建规则</el-button>
            </div>
            <div class="op-panel-body">
            <el-table :data="lifecycleRules" stripe size="small" empty-text="暂无规则">
              <el-table-column prop="id" label="ID" width="64" align="center" />
              <el-table-column prop="name" label="名称" min-width="120" />
              <el-table-column prop="bucket" label="Bucket" width="120" />
              <el-table-column label="前缀" width="100" align="center">
                <template #default="{ row }">{{ row.prefix || '/' }}</template>
              </el-table-column>
              <el-table-column label="转入IA" width="80" align="center">
                <template #default="{ row }">{{ row.ia_days ? row.ia_days + '天' : '—' }}</template>
              </el-table-column>
              <el-table-column label="Archive" width="90" align="center">
                <template #default="{ row }">{{ row.archive_days ? row.archive_days + '天' : '—' }}</template>
              </el-table-column>
              <el-table-column label="删除" width="72" align="center">
                <template #default="{ row }">{{ row.delete_days ? row.delete_days + '天' : '—' }}</template>
              </el-table-column>
              <el-table-column label="启用" width="64" align="center">
                <template #default="{ row }">
                  <el-switch v-model="row.enabled" size="small" @change="toggleLifecycle(row)" />
                </template>
              </el-table-column>
              <el-table-column label="操作" width="120" align="center">
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
          </div>
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

async function api(path, opts = {}) {
  const m = (opts.method || 'GET').toLowerCase();
  let r;
  if (m === 'get') r = await http.get(ADMIN_API + path);
  else if (m === 'post') r = await http.post(ADMIN_API + path, opts.body || {});
  else if (m === 'put') r = await http.put(ADMIN_API + path, opts.body || {});
  else if (m === 'delete') r = await http.delete(ADMIN_API + path);
  else r = await http.get(ADMIN_API + path);
  return r.data;
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
const evaluating = ref(false)

async function evaluateAlerts() {
  evaluating.value = true
  try {
    const d = await api('/ops/alerts/evaluate', { method: 'POST' })
    ElMessage.success(`评估完成：检测 ${d.rules_checked || 0} 条规则，触发 ${d.fired || 0} 条告警`)
    fetchAlerts()
  } catch (e) {
    ElMessage.error(e.message || '评估失败')
  } finally {
    evaluating.value = false
  }
}

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
    const d = await api('/ops/cdn/refresh')
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
    healthStatus.value = d.items || []
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
.op-page {
  padding: 0;
  max-width: 100%;
}

/* Page title */
.op-page-title {
  font-size: 18px;
  font-weight: 700;
  color: #1a1a2e;
  padding: 16px 20px 0;
  letter-spacing: 0.5px;
}
.op-card-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

/* Tabs — card style */
.op-tabs {
  --el-tabs-header-height: 44px;
}
.op-tabs :deep(.el-tabs__header) {
  margin: 0 0 16px;
  padding: 0 20px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
}
.op-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}
.op-tabs :deep(.el-tabs__content) {
  padding: 0 20px;
}

/* Panel cards — plain divs, no Element Plus dependency */
.op-panel {
  border: 1px solid #dcdfe6;
  border-radius: 8px;
  margin-bottom: 20px;
  overflow: hidden;
}
.op-panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  background: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  min-height: 44px;
}
.op-panel-title {
  font-size: 15px;
  font-weight: 700;
  color: #303133;
}
.op-panel-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}
.op-panel-body {
  padding: 14px 18px;
}

/* Section spacing */
.op-section {
  display: flex;
  flex-direction: column;
}

/* Table fine-tuning */
.op-page :deep(.el-table) {
  font-size: 13px;
}
.op-page :deep(.el-table th.el-table__cell) {
  background: #f5f7fa;
  color: #606266;
  font-weight: 600;
  height: 40px;
}
.op-page :deep(.el-table td.el-table__cell) {
  padding: 8px 0;
}

/* Inline elements */
.op-muted {
  color: #c0c4cc;
  font-size: 12px;
}
.op-error {
  color: #f56c6c;
  font-size: 12px;
  word-break: break-all;
}
.op-code {
  font-family: 'SF Mono', 'Menlo', 'Monaco', monospace;
  font-size: 12px;
  color: #409eff;
  background: #ecf5ff;
  padding: 2px 6px;
  border-radius: 3px;
}
.op-path {
  font-size: 13px;
  color: #606266;
}

/* Health grid */
.op-health-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
}
.op-health-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 20px 24px;
  border-radius: 10px;
  border: 1px solid;
  transition: box-shadow 0.2s;
}
.op-health-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}
.op-health-card--ok {
  border-color: #b7ebc4;
  background: #f6ffed;
}
.op-health-card--err {
  border-color: #f5c6cb;
  background: #fff2f0;
}
.op-health-card__icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
}
.op-health-card__icon--ok {
  background: #52c41a;
}
.op-health-card__icon--err {
  background: #ff4d4f;
}
.op-health-card__name {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 2px;
}
.op-health-card__detail {
  font-size: 12px;
  color: #8c8c8c;
  line-height: 1.4;
}
.op-health-card__latency {
  font-size: 11px;
  color: #bfbfbf;
  margin-top: 2px;
}

/* Dialogs */
.op-page :deep(.el-dialog) {
  border-radius: 8px;
}
</style>
