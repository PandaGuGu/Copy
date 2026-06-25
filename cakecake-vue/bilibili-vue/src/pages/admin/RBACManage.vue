<template>
  <div class="rb-page" v-loading="loading">
    <header class="rb-page__head">
      <h2 class="rb-page__title">RBAC 与审计</h2>
      <p class="rb-page__desc">角色权限管理、审计日志与审批流程</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 角色管理 -->
      <el-tab-pane label="角色" name="roles">
        <div class="rb-toolbar">
          <el-button type="primary" size="default" @click="openRoleDialog(null)">新建角色</el-button>
        </div>
        <el-table :data="roles" stripe size="default" empty-text="暂无角色">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="name" label="角色名称" width="140" />
          <el-table-column prop="description" label="描述" min-width="160" show-overflow-tooltip />
          <el-table-column label="权限数" width="80">
            <template #default="{ row }">{{ row.permissions?.length || 0 }}</template>
          </el-table-column>
          <el-table-column label="管理员数" width="90">
            <template #default="{ row }">{{ row.admin_count || 0 }}</template>
          </el-table-column>
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openRoleDialog(row)">编辑</el-button>
              <el-button size="small" text type="success" @click="openPermissionDialog(row)">权限</el-button>
              <el-popconfirm title="确认删除此角色？" @confirm="deleteRole(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>

        <!-- 管理员列表 -->
        <div class="rb-subsection">
          <h4 class="rb-subsection__title">管理员列表</h4>
          <div class="rb-toolbar">
            <el-button type="primary" size="small" @click="openAssignRoleDialog(null)">分配角色</el-button>
          </div>
          <el-table :data="adminList" stripe size="small" empty-text="暂无管理员">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column label="管理员" min-width="140">
              <template #default="{ row }">
                <span>{{ row.nickname || row.username }}</span>
                <span class="rb-muted" style="margin-left: 4px">@{{ row.username }}</span>
              </template>
            </el-table-column>
            <el-table-column label="角色" width="140">
              <template #default="{ row }">
                <el-tag v-if="row.role" size="small" effect="plain">{{ row.role.name }}</el-tag>
                <span v-else class="rb-muted">未分配</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.status === 'active' ? 'success' : 'info'" size="small">
                  {{ row.status === 'active' ? '正常' : '禁用' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="openAssignRoleDialog(row)">分配角色</el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-tab-pane>

      <!-- 审计日志 -->
      <el-tab-pane label="审计日志" name="audit">
        <div class="rb-toolbar">
          <el-input v-model="auditFilter.admin_id" placeholder="管理员ID" clearable size="small" style="width: 120px" @keyup.enter="searchAudit" />
          <el-input v-model="auditFilter.action" placeholder="操作类型" clearable size="small" style="width: 140px" @keyup.enter="searchAudit" />
          <el-input v-model="auditFilter.resource" placeholder="资源类型" clearable size="small" style="width: 140px" @keyup.enter="searchAudit" />
          <el-date-picker
            v-model="auditFilter.time_range"
            type="datetimerange"
            range-separator="至"
            start-placeholder="开始时间"
            end-placeholder="结束时间"
            size="small"
            style="width: 340px"
            @change="searchAudit"
          />
          <el-button type="primary" size="small" @click="searchAudit">搜索</el-button>
        </div>
        <el-table :data="auditLogs" stripe size="default" empty-text="暂无日志">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column label="管理员" width="130">
            <template #default="{ row }">
              <span v-if="row.admin">{{ row.admin.nickname || row.admin.username }}</span>
              <span v-else class="rb-muted">#{{ row.admin_id }}</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120">
            <template #default="{ row }">
              <el-tag :type="auditActionTag(row.action)" size="small" effect="plain">{{ row.action }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="resource" label="资源" width="120" />
          <el-table-column prop="resource_id" label="资源ID" width="90" />
          <el-table-column label="详情" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">{{ row.detail || '—' }}</template>
          </el-table-column>
          <el-table-column label="IP" width="120">
            <template #default="{ row }">{{ row.ip || '—' }}</template>
          </el-table-column>
          <el-table-column label="时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
        </el-table>
        <div class="rb-pager" v-if="auditTotal > auditPageSize">
          <el-pagination
            v-model:current-page="auditPage"
            :page-size="auditPageSize"
            :total="auditTotal"
            layout="prev, pager, next, total"
            @current-change="fetchAudit"
          />
        </div>
      </el-tab-pane>

      <!-- 审批流程 -->
      <el-tab-pane label="审批流程" name="approvals">
        <div class="rb-toolbar">
          <el-select v-model="approvalFilter.status" placeholder="状态" clearable size="default" style="width: 120px" @change="fetchApprovals">
            <el-option label="全部" value="" />
            <el-option label="待审批" value="pending" />
            <el-option label="已通过" value="approved" />
            <el-option label="已驳回" value="rejected" />
          </el-select>
          <el-button type="primary" size="default" @click="fetchApprovals">刷新</el-button>
        </div>
        <el-table :data="approvals" stripe size="default" empty-text="暂无审批">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="title" label="标题" min-width="160" show-overflow-tooltip />
          <el-table-column label="类型" width="100">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ approvalTypeLabel(row.type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="发起人" width="120">
            <template #default="{ row }">
              <span v-if="row.applicant">{{ row.applicant.nickname || row.applicant.username }}</span>
              <span v-else class="rb-muted">#{{ row.applicant_id }}</span>
            </template>
          </el-table-column>
          <el-table-column label="当前步骤" width="100">
            <template #default="{ row }">{{ row.current_step || '—' }}</template>
          </el-table-column>
          <el-table-column label="状态" width="90">
            <template #default="{ row }">
              <el-tag :type="approvalStatusTag(row.status)" size="small" effect="plain">
                {{ approvalStatusLabel(row.status) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="发起时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="130" fixed="right">
            <template #default="{ row }">
              <template v-if="row.status === 'pending'">
                <el-button size="small" text type="success" @click="doApprove(row, true)">通过</el-button>
                <el-button size="small" text type="danger" @click="doApprove(row, false)">驳回</el-button>
              </template>
              <span v-else class="rb-muted">已完成</span>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <!-- 角色编辑弹窗 -->
    <el-dialog v-model="roleDialogVisible" :title="roleForm.id ? '编辑角色' : '新建角色'" width="480px" destroy-on-close>
      <el-form :model="roleForm" label-width="70px" size="default">
        <el-form-item label="名称">
          <el-input v-model="roleForm.name" placeholder="角色名称" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="roleForm.description" type="textarea" :rows="2" placeholder="角色描述" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="roleDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveRole">保存</el-button>
      </template>
    </el-dialog>

    <!-- 权限分配弹窗 -->
    <el-dialog v-model="permissionDialogVisible" title="权限分配" width="520px" destroy-on-close>
      <template v-if="permissionForm.role">
        <div class="rb-perm-role">
          <span class="rb-perm-role__name">{{ permissionForm.role.name }}</span>
          <span class="rb-muted">{{ permissionForm.role.description }}</span>
        </div>
        <el-divider />
        <div class="rb-perm-groups">
          <div v-for="group in permissionGroups" :key="group.name" class="rb-perm-group">
            <div class="rb-perm-group__head">
              <el-checkbox
                :model-value="isGroupAllChecked(group)"
                :indeterminate="isGroupIndeterminate(group)"
                @change="toggleGroup(group, $event)"
              >
                <span class="rb-perm-group__name">{{ group.label }}</span>
              </el-checkbox>
            </div>
            <div class="rb-perm-group__items">
              <el-checkbox
                v-for="perm in group.items"
                :key="perm.key"
                :model-value="permissionForm.permissions.includes(perm.key)"
                @change="togglePermission(perm.key, $event)"
              >
                {{ perm.label }}
              </el-checkbox>
            </div>
          </div>
        </div>
      </template>
      <template #footer>
        <el-button @click="permissionDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="savePermissions">保存</el-button>
      </template>
    </el-dialog>

    <!-- 分配角色弹窗 -->
    <el-dialog v-model="assignDialogVisible" title="分配角色" width="420px" destroy-on-close>
      <template v-if="assignForm.admin">
        <p class="rb-assign-info">
          管理员：<b>{{ assignForm.admin.nickname || assignForm.admin.username }}</b>
        </p>
      </template>
      <el-form label-width="70px" size="default">
        <el-form-item label="角色">
          <el-select v-model="assignForm.role_id" placeholder="选择角色" clearable style="width: 100%">
            <el-option v-for="r in roles" :key="r.id" :label="r.name" :value="r.id" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="assignDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveAssign">保存</el-button>
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
const activeTab = ref('roles')

// Roles
const roles = ref([])
const adminList = ref([])
const roleDialogVisible = ref(false)
const roleForm = reactive({ id: null, name: '', description: '' })

// Permissions
const permissionDialogVisible = ref(false)
const permissionForm = reactive({ role: null, permissions: [] })

const permissionGroups = [
  {
    name: 'content', label: '内容管理',
    items: [
      { key: 'video:review', label: '视频审核' },
      { key: 'article:review', label: '文章审核' },
      { key: 'dynamic:manage', label: '动态管理' },
      { key: 'comment:manage', label: '评论管理' },
    ],
  },
  {
    name: 'user', label: '用户管理',
    items: [
      { key: 'user:list', label: '查看用户' },
      { key: 'user:ban', label: '封禁用户' },
      { key: 'user:delete', label: '删除用户' },
    ],
  },
  {
    name: 'report', label: '举报与版权',
    items: [
      { key: 'report:handle', label: '处理举报' },
      { key: 'copyright:manage', label: '版权管理' },
    ],
  },
  {
    name: 'system', label: '系统管理',
    items: [
      { key: 'config:manage', label: '配置管理' },
      { key: 'rbac:manage', label: '权限管理' },
      { key: 'ops:monitor', label: '运维监控' },
    ],
  },
]

// Audit
const auditLogs = ref([])
const auditTotal = ref(0)
const auditPage = ref(1)
const auditPageSize = 20
const auditFilter = reactive({ admin_id: '', action: '', resource: '', time_range: null })

// Approvals
const approvals = ref([])
const approvalFilter = reactive({ status: '' })

// Assign role
const assignDialogVisible = ref(false)
const assignForm = reactive({ admin: null, role_id: null })

async function fetchRoles() {
  loading.value = true
  try {
    const d = await api('/rbac/roles')
    roles.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchAdmins() {
  try {
    const d = await api('/rbac/admins')
    adminList.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  }
}

function onTabChange(tab) {
  if (tab === 'roles') {
    if (roles.value.length === 0) fetchRoles()
    if (adminList.value.length === 0) fetchAdmins()
  }
  if (tab === 'audit' && auditLogs.value.length === 0) fetchAudit()
  if (tab === 'approvals' && approvals.value.length === 0) fetchApprovals()
}

function openRoleDialog(row) {
  if (row) {
    Object.assign(roleForm, { id: row.id, name: row.name, description: row.description || '' })
  } else {
    Object.assign(roleForm, { id: null, name: '', description: '' })
  }
  roleDialogVisible.value = true
}

async function saveRole() {
  if (!roleForm.name.trim()) {
    ElMessage.warning('请输入角色名称')
    return
  }
  saving.value = true
  try {
    if (roleForm.id) {
      await api(`/rbac/roles/${roleForm.id}`, { method: 'PUT', body: { ...roleForm } })
    } else {
      await api('/rbac/roles', { method: 'POST', body: { ...roleForm } })
    }
    ElMessage.success('已保存')
    roleDialogVisible.value = false
    fetchRoles()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteRole(row) {
  try {
    await api(`/rbac/roles/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    fetchRoles()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

async function openPermissionDialog(row) {
  try {
    const d = await api(`/rbac/roles/${row.id}`)
    permissionForm.role = d.role
    permissionForm.permissions = (d.permissions || []).map(p => typeof p === 'string' ? p : p.code)
    permissionDialogVisible.value = true
  } catch (e) {
    ElMessage.error(e.message || '获取权限失败')
  }
}

function isGroupAllChecked(group) {
  return group.items.every(p => permissionForm.permissions.includes(p.key))
}

function isGroupIndeterminate(group) {
  const checked = group.items.filter(p => permissionForm.permissions.includes(p.key))
  return checked.length > 0 && checked.length < group.items.length
}

function toggleGroup(group, checked) {
  const keys = group.items.map(p => p.key)
  if (checked) {
    for (const k of keys) {
      if (!permissionForm.permissions.includes(k)) permissionForm.permissions.push(k)
    }
  } else {
    permissionForm.permissions = permissionForm.permissions.filter(k => !keys.includes(k))
  }
}

function togglePermission(key, checked) {
  if (checked && !permissionForm.permissions.includes(key)) {
    permissionForm.permissions.push(key)
  } else if (!checked) {
    permissionForm.permissions = permissionForm.permissions.filter(k => k !== key)
  }
}

async function savePermissions() {
  if (!permissionForm.role) return
  saving.value = true
  try {
    await api(`/rbac/roles/${permissionForm.role.id}/permissions`, {
      method: 'POST',
      body: { permissions: permissionForm.permissions },
    })
    ElMessage.success('权限已更新')
    permissionDialogVisible.value = false
    fetchRoles()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

function openAssignRoleDialog(admin) {
  assignForm.admin = admin
  assignForm.role_id = admin?.role_id || null
  assignDialogVisible.value = true
}

async function saveAssign() {
  if (!assignForm.admin) return
  saving.value = true
  try {
    await api(`/rbac/admins/${assignForm.admin.id}/role`, {
      method: 'POST',
      body: { role_id: assignForm.role_id },
    })
    ElMessage.success('已分配')
    assignDialogVisible.value = false
    fetchAdmins()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function fetchAudit() {
  loading.value = true
  try {
    const params = new URLSearchParams({ page: auditPage.value, page_size: auditPageSize })
    if (auditFilter.admin_id) params.set('admin_id', auditFilter.admin_id)
    if (auditFilter.action) params.set('action', auditFilter.action)
    if (auditFilter.resource) params.set('resource', auditFilter.resource)
    if (auditFilter.time_range && auditFilter.time_range.length === 2) {
      params.set('start_time', new Date(auditFilter.time_range[0]).toISOString())
      params.set('end_time', new Date(auditFilter.time_range[1]).toISOString())
    }
    const d = await api(`/rbac/audit-logs?${params}`)
    auditLogs.value = d.items || []
    auditTotal.value = d.total || 0
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function searchAudit() {
  auditPage.value = 1
  fetchAudit()
}

async function fetchApprovals() {
  loading.value = true
  try {
    const params = new URLSearchParams()
    if (approvalFilter.status) params.set('status', approvalFilter.status)
    const d = await api(`/rbac/approval-flows?${params}`)
    approvals.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function doApprove(row, approved) {
  const label = approved ? '通过' : '驳回'
  try {
    await ElMessageBox.confirm(`确认${label}此审批？`, '提示', {
      type: approved ? 'info' : 'warning',
    })
    await api(`/rbac/approval-flows/${row.id}/${approved ? 'approve' : 'reject'}`, {
      method: 'POST',
    })
    ElMessage.success(`已${label}`)
    fetchApprovals()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(e.message || '操作失败')
  }
}

function auditActionTag(a) {
  if (a?.includes('create') || a?.includes('add')) return 'success'
  if (a?.includes('delete') || a?.includes('remove')) return 'danger'
  if (a?.includes('update') || a?.includes('edit')) return 'warning'
  return 'info'
}

function approvalStatusLabel(s) {
  return { pending: '待审批', approved: '已通过', rejected: '已驳回' }[s] || s
}

function approvalStatusTag(s) {
  return { pending: 'warning', approved: 'success', rejected: 'danger' }[s] || ''
}

function approvalTypeLabel(t) {
  return { content_takedown: '内容下架', user_ban: '用户封禁', config_change: '配置变更', other: '其他' }[t] || t
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => {
  fetchRoles()
  fetchAdmins()
})
</script>

<style scoped>
.rb-page { padding: 20px 24px; }
.rb-page__head { margin-bottom: 14px; }
.rb-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.rb-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.rb-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.rb-muted { color: #9499a0; }
.rb-pager { margin-top: 16px; display: flex; justify-content: flex-end; }
.rb-subsection { margin-top: 20px; }
.rb-subsection__title { margin: 0 0 10px; font-size: 14px; font-weight: 600; color: #18191c; }

.rb-perm-role { display: flex; align-items: center; gap: 10px; }
.rb-perm-role__name { font-size: 16px; font-weight: 600; color: #18191c; }
.rb-perm-groups { display: flex; flex-direction: column; gap: 14px; }
.rb-perm-group { border: 1px solid #e3e5e7; border-radius: 8px; padding: 12px 16px; }
.rb-perm-group__head { margin-bottom: 8px; }
.rb-perm-group__name { font-weight: 600; color: #18191c; }
.rb-perm-group__items { display: flex; flex-wrap: wrap; gap: 12px; padding-left: 24px; }

.rb-assign-info { font-size: 14px; color: #61666d; }
.rb-assign-info b { color: #18191c; }
</style>
