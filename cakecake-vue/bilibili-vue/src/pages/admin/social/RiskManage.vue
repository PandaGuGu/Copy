<template>
  <div class="rk-page" v-loading="loading">
    <header class="rk-page__head">
      <h2 class="rk-page__title">风控管理</h2>
      <p class="rk-page__desc">管理风控规则与黑白名单</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 风控规则 -->
      <el-tab-pane label="风控规则" name="rules">
        <div class="rk-toolbar">
          <el-button type="primary" size="default" @click="openRuleDialog(null)">新建规则</el-button>
        </div>
        <el-table :data="rules" stripe size="default">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="name" label="规则名称" min-width="140" show-overflow-tooltip />
          <el-table-column label="分类" width="100">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ categoryLabel(row.category) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="类型" width="100">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ ruleTypeLabel(row.rule_type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="匹配模式" min-width="180" show-overflow-tooltip>
            <template #default="{ row }">
              <code class="rk-code">{{ row.pattern }}</code>
            </template>
          </el-table-column>
          <el-table-column label="动作" width="90">
            <template #default="{ row }">
              <el-tag :type="actionTag(row.action)" size="small">{{ actionLabel(row.action) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="优先级" width="70">
            <template #default="{ row }">{{ row.priority }}</template>
          </el-table-column>
          <el-table-column label="启用" width="70">
            <template #default="{ row }">
              <el-switch v-model="row.enabled" @change="toggleRule(row)" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openRuleDialog(row)">编辑</el-button>
              <el-popconfirm title="确认删除？" @confirm="deleteRule(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 黑白名单 -->
      <el-tab-pane label="黑白名单" name="lists">
        <div class="rk-toolbar">
          <el-select v-model="filterListType" placeholder="类型" clearable size="default" style="width: 120px" @change="fetchLists">
            <el-option label="全部" value="" />
            <el-option label="黑名单" value="blacklist" />
            <el-option label="白名单" value="whitelist" />
          </el-select>
          <el-button type="primary" size="default" @click="openListDialog(null)">新建条目</el-button>
        </div>
        <el-table :data="listItems" stripe size="default">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column label="类型" width="80">
            <template #default="{ row }">
              <el-tag :type="row.list_type === 'blacklist' ? 'danger' : 'success'" size="small" effect="dark">
                {{ row.list_type === 'blacklist' ? '黑名单' : '白名单' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="目标" min-width="160" show-overflow-tooltip>
            <template #default="{ row }">
              <span class="rk-target">{{ row.target }}</span>
              <el-tag size="small" effect="plain" style="margin-left: 4px">{{ targetTypeLabel(row.target_type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="原因" min-width="160" show-overflow-tooltip>
            <template #default="{ row }">{{ row.reason || '—' }}</template>
          </el-table-column>
          <el-table-column label="过期时间" width="160">
            <template #default="{ row }">
              <span v-if="row.expires_at">{{ fmtTime(row.expires_at) }}</span>
              <span v-else class="rk-muted">永久</span>
            </template>
          </el-table-column>
          <el-table-column label="创建时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openListDialog(row)">编辑</el-button>
              <el-popconfirm title="确认删除？" @confirm="deleteListItem(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <!-- 规则编辑弹窗 -->
    <el-dialog v-model="ruleDialogVisible" :title="ruleForm.id ? '编辑规则' : '新建规则'" width="560px" destroy-on-close>
      <el-form :model="ruleForm" label-width="90px" size="default">
        <el-form-item label="规则名称">
          <el-input v-model="ruleForm.name" placeholder="输入规则名称" />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="ruleForm.category" style="width: 100%">
            <el-option label="关键词匹配" value="keyword" />
            <el-option label="频率限制" value="rate_limit" />
            <el-option label="设备指纹" value="device_fingerprint" />
            <el-option label="行为分析" value="behavior" />
          </el-select>
        </el-form-item>
        <el-form-item label="规则类型">
          <el-select v-model="ruleForm.rule_type" style="width: 100%">
            <el-option label="正则匹配" value="regex" />
            <el-option label="阈值检测" value="threshold" />
            <el-option label="频率限制" value="rate_limit" />
            <el-option label="关键词" value="keyword" />
          </el-select>
        </el-form-item>
        <el-form-item label="匹配模式">
          <el-input v-model="ruleForm.pattern" type="textarea" :rows="3" placeholder="正则表达式或JSON阈值配置，如 {&quot;max_count&quot;: 100, &quot;window&quot;: 3600}" />
        </el-form-item>
        <el-form-item label="动作">
          <el-select v-model="ruleForm.action" style="width: 100%">
            <el-option label="拦截删除" value="reject" />
            <el-option label="隔离待审" value="quarantine" />
            <el-option label="告警通知" value="notify_admin" />
            <el-option label="自动封禁" value="auto_ban" />
          </el-select>
        </el-form-item>
        <el-form-item label="优先级">
          <el-input-number v-model="ruleForm.priority" :min="0" :max="999" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="ruleForm.enabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="ruleDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveRule">保存</el-button>
      </template>
    </el-dialog>

    <!-- 名单编辑弹窗 -->
    <el-dialog v-model="listDialogVisible" :title="listForm.id ? '编辑条目' : '新建条目'" width="480px" destroy-on-close>
      <el-form :model="listForm" label-width="80px" size="default">
        <el-form-item label="名单类型">
          <el-select v-model="listForm.list_type" style="width: 100%">
            <el-option label="黑名单" value="blacklist" />
            <el-option label="白名单" value="whitelist" />
          </el-select>
        </el-form-item>
        <el-form-item label="目标类型">
          <el-select v-model="listForm.target_type" style="width: 100%">
            <el-option label="用户ID" value="user" />
            <el-option label="IP地址" value="ip" />
            <el-option label="设备ID" value="device" />
            <el-option label="内容ID" value="content" />
          </el-select>
        </el-form-item>
        <el-form-item label="目标值">
          <el-input v-model="listForm.target" placeholder="输入用户ID / IP / 设备ID" />
        </el-form-item>
        <el-form-item label="原因">
          <el-input v-model="listForm.reason" type="textarea" :rows="2" placeholder="加入名单的原因" />
        </el-form-item>
        <el-form-item label="过期时间">
          <el-date-picker v-model="listForm.expires_at" type="datetime" placeholder="留空为永久" style="width: 100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="listDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveListItem">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage } from 'element-plus'

const ADMIN_API = '/api/v1/admin'

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
const activeTab = ref('rules')

// Rules
const rules = ref([])
const ruleDialogVisible = ref(false)
const ruleForm = reactive({
  id: null, name: '', category: 'keyword', rule_type: 'keyword',
  pattern: '', action: 'reject', priority: 10, enabled: true,
})

// Lists
const listItems = ref([])
const filterListType = ref('')
const listDialogVisible = ref(false)
const listForm = reactive({
  id: null, list_type: 'blacklist', target_type: 'user', target: '', reason: '', expires_at: null,
})

async function fetchRules() {
  loading.value = true
  try {
    const d = await api('/risk/rules')
    rules.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchLists() {
  loading.value = true
  try {
    const params = new URLSearchParams()
    if (filterListType.value) params.set('list_type', filterListType.value)
    const d = await api(`/risk/bw-list?${params}`)
    listItems.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function onTabChange(tab) {
  if (tab === 'rules' && rules.value.length === 0) fetchRules()
  if (tab === 'lists' && listItems.value.length === 0) fetchLists()
}

async function toggleRule(row) {
  try {
    await api(`/risk/rules/${row.id}/toggle`, { method: 'POST' })
    ElMessage.success(row.enabled ? '已启用' : '已禁用')
  } catch (e) {
    row.enabled = !row.enabled
    ElMessage.error(e.message || '操作失败')
  }
}

function openRuleDialog(row) {
  if (row) {
    Object.assign(ruleForm, {
      id: row.id, name: row.name, category: row.category, rule_type: row.rule_type,
      pattern: row.pattern || '', action: row.action, priority: row.priority || 10, enabled: row.enabled,
    })
  } else {
    Object.assign(ruleForm, {
      id: null, name: '', category: 'keyword', rule_type: 'keyword',
      pattern: '', action: 'reject', priority: 10, enabled: true,
    })
  }
  ruleDialogVisible.value = true
}

async function saveRule() {
  if (!ruleForm.name.trim()) {
    ElMessage.warning('请输入规则名称')
    return
  }
  saving.value = true
  try {
    if (ruleForm.id) {
      await api(`/risk/rules/${ruleForm.id}`, { method: 'PUT', body: { ...ruleForm } })
    } else {
      await api('/risk/rules', { method: 'POST', body: { ...ruleForm } })
    }
    ElMessage.success('已保存')
    ruleDialogVisible.value = false
    fetchRules()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteRule(row) {
  try {
    await api(`/risk/rules/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    fetchRules()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function openListDialog(row) {
  if (row) {
    Object.assign(listForm, {
      id: row.id, list_type: row.list_type, target_type: row.target_type,
      target: row.target, reason: row.reason || '', expires_at: row.expires_at || null,
    })
  } else {
    Object.assign(listForm, {
      id: null, list_type: 'blacklist', target_type: 'user', target: '', reason: '', expires_at: null,
    })
  }
  listDialogVisible.value = true
}

async function saveListItem() {
  if (!listForm.target.trim()) {
    ElMessage.warning('请输入目标值')
    return
  }
  saving.value = true
  try {
    const payload = { ...listForm }
    if (payload.expires_at) payload.expires_at = new Date(payload.expires_at).toISOString()
    if (listForm.id) {
      await api(`/risk/bw-list/${listForm.id}`, { method: 'PUT', body: payload })
    } else {
      await api('/risk/bw-list', { method: 'POST', body: payload })
    }
    ElMessage.success('已保存')
    listDialogVisible.value = false
    fetchLists()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteListItem(row) {
  try {
    await api(`/risk/bw-list/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    fetchLists()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function categoryLabel(c) {
  return { keyword: '关键词匹配', rate_limit: '频率限制', device_fingerprint: '设备指纹', behavior: '行为分析' }[c] || c
}

function ruleTypeLabel(t) {
  return { keyword: '关键词', regex: '正则匹配', threshold: '阈值检测', rate_limit: '频率限制' }[t] || t
}

function actionLabel(a) {
  return { reject: '拦截删除', quarantine: '隔离待审', notify_admin: '告警通知', auto_ban: '自动封禁' }[a] || a
}

function actionTag(a) {
  return { reject: 'danger', quarantine: 'warning', notify_admin: 'info', auto_ban: 'danger' }[a] || ''
}

function targetTypeLabel(t) {
  return { user: '用户', ip: 'IP', device: '设备', content: '内容' }[t] || t
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => fetchRules())
</script>

<style scoped>
.rk-page { padding: 20px 24px; }
.rk-page__head { margin-bottom: 14px; }
.rk-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.rk-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.rk-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; }
.rk-muted { color: #9499a0; }
.rk-code { font-family: 'Courier New', monospace; font-size: 12px; color: #e6a23c; background: #fffaf3; padding: 2px 6px; border-radius: 3px; }
.rk-target { font-weight: 500; color: #18191c; }
</style>
