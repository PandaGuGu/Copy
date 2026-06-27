<template>
  <div class="tk-page" v-loading="loading">
    <header class="tk-page__head">
      <h2 class="tk-page__title">工单管理</h2>
      <p class="tk-page__desc">管理用户提交的工单与客服请求</p>
    </header>

    <!-- 统计卡片 -->
    <div class="tk-stats">
      <div class="tk-stat tk-stat--warn" @click="filterStatus = 'open'; search()">
        <span class="tk-stat__val">{{ stats.open_count || 0 }}</span>
        <span class="tk-stat__label">待处理</span>
      </div>
      <div class="tk-stat tk-stat--info" @click="filterStatus = 'processing'; search()">
        <span class="tk-stat__val">{{ stats.processing_count || 0 }}</span>
        <span class="tk-stat__label">处理中</span>
      </div>
      <div class="tk-stat tk-stat--ok" @click="filterStatus = 'resolved'; search()">
        <span class="tk-stat__val">{{ stats.resolved_count || 0 }}</span>
        <span class="tk-stat__label">已解决</span>
      </div>
      <div class="tk-stat tk-stat--dim" @click="filterStatus = 'closed'; search()">
        <span class="tk-stat__val">{{ stats.closed_count || 0 }}</span>
        <span class="tk-stat__label">已关闭</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tk-toolbar">
      <el-select v-model="filterStatus" placeholder="状态" clearable size="default" style="width: 120px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="待处理" value="open" />
        <el-option label="已指派" value="assigned" />
        <el-option label="处理中" value="processing" />
        <el-option label="已解决" value="resolved" />
        <el-option label="已关闭" value="closed" />
      </el-select>
      <el-select v-model="filterCategory" placeholder="分类" clearable size="default" style="width: 120px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="账号问题" value="account" />
        <el-option label="内容问题" value="content" />
        <el-option label="支付问题" value="payment" />
        <el-option label="技术故障" value="technical" />
        <el-option label="其他" value="other" />
      </el-select>
      <el-select v-model="filterPriority" placeholder="优先级" clearable size="default" style="width: 110px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="紧急" value="urgent" />
        <el-option label="高" value="high" />
        <el-option label="中" value="medium" />
        <el-option label="低" value="low" />
      </el-select>
      <el-button type="primary" size="default" @click="search">搜索</el-button>
    </div>

    <!-- 工单列表 -->
    <el-table :data="items" stripe size="default" empty-text="暂无工单">
      <el-table-column prop="id" label="ID" width="65" />
      <el-table-column label="标题" min-width="200" show-overflow-tooltip>
        <template #default="{ row }">
          <span class="tk-link" @click="openDetail(row)">{{ row.title }}</span>
        </template>
      </el-table-column>
      <el-table-column label="分类" width="100">
        <template #default="{ row }">
          <el-tag size="small" effect="plain">{{ categoryLabel(row.category) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="优先级" width="80">
        <template #default="{ row }">
          <el-tag :type="priorityTag(row.priority)" size="small" effect="dark">{{ priorityLabel(row.priority) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="SLA" width="130">
        <template #default="{ row }">
          <span v-if="row.sla_deadline" :class="slaClass(row.sla_deadline, row.status)">
            {{ slaCountdown(row.sla_deadline, row.status) }}
          </span>
          <span v-else class="tk-muted">—</span>
        </template>
      </el-table-column>
      <el-table-column label="提交人" width="110">
        <template #default="{ row }">
          <span v-if="row.user">{{ row.user.nickname || row.user.username }}</span>
          <span v-else class="tk-muted">#{{ row.user_id }}</span>
        </template>
      </el-table-column>
      <el-table-column label="处理人" width="110">
        <template #default="{ row }">
          <span v-if="row.assignee">{{ row.assignee.nickname || row.assignee.username }}</span>
          <span v-else class="tk-muted">未指派</span>
        </template>
      </el-table-column>
      <el-table-column label="创建时间" width="155">
        <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="160" fixed="right">
        <template #default="{ row }">
          <el-button size="small" text type="primary" @click="openDetail(row)">详情</el-button>
          <el-button v-if="row.status === 'open'" size="small" text type="success" @click="doAutoAssign(row)">自动分配</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="tk-pager" v-if="total > pageSize">
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="prev, pager, next, total"
        @current-change="fetch"
      />
    </div>

    <!-- 工单详情弹窗 -->
    <el-dialog v-model="detailVisible" title="工单详情" width="720px" destroy-on-close>
      <template v-if="detail">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="工单ID">{{ detail.id }}</el-descriptions-item>
          <el-descriptions-item label="标题">{{ detail.title }}</el-descriptions-item>
          <el-descriptions-item label="分类">{{ categoryLabel(detail.category) }}</el-descriptions-item>
          <el-descriptions-item label="优先级">{{ priorityLabel(detail.priority) }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="statusTag(detail.status)" size="small">{{ statusLabel(detail.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="SLA截止">
            <span v-if="detail.sla_deadline" :class="slaClass(detail.sla_deadline, detail.status)">
              {{ fmtTime(detail.sla_deadline) }} ({{ slaCountdown(detail.sla_deadline, detail.status) }})
            </span>
            <span v-else>—</span>
          </el-descriptions-item>
          <el-descriptions-item label="提交人">
            <span v-if="detail.user">{{ detail.user.nickname || detail.user.username }}</span>
            <span v-else>#{{ detail.user_id }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="处理人">
            <span v-if="detail.assignee">{{ detail.assignee.nickname || detail.assignee.username }}</span>
            <span v-else class="tk-muted">未指派</span>
          </el-descriptions-item>
          <el-descriptions-item label="描述" :span="2">
            <div class="tk-desc">{{ detail.description }}</div>
          </el-descriptions-item>
        </el-descriptions>

        <!-- 满意度评分 -->
        <div v-if="detail.satisfaction" class="tk-sat">
          <el-divider>用户评价</el-divider>
          <div class="tk-sat__stars">
            <span v-for="s in 5" :key="s" class="tk-sat__star" :class="{ 'tk-sat__star--active': s <= detail.satisfaction.score }">★</span>
            <span class="tk-sat__label">{{ detail.satisfaction.score }} 分</span>
          </div>
          <p v-if="detail.satisfaction.comment" class="tk-sat__comment">{{ detail.satisfaction.comment }}</p>
          <span class="tk-sat__time">{{ fmtTime(detail.satisfaction.created_at) }}</span>
        </div>

        <!-- 消息线程 -->
        <div class="tk-thread">
          <h4 class="tk-thread__title">消息记录</h4>
          <div class="tk-thread__list">
            <div
              v-for="msg in detail.messages"
              :key="msg.id"
              class="tk-msg"
              :class="{ 'tk-msg--admin': msg.is_admin }"
            >
              <div class="tk-msg__head">
                <span class="tk-msg__author">
                  {{ msg.is_admin ? (msg.admin?.nickname || '管理员') : (msg.user?.nickname || '用户') }}
                  <el-tag v-if="msg.is_admin" size="small" type="info" effect="plain">客服</el-tag>
                </span>
                <span class="tk-msg__time">{{ fmtTime(msg.created_at) }}</span>
              </div>
              <div class="tk-msg__body">{{ msg.content }}</div>
            </div>
            <div v-if="!detail.messages || detail.messages.length === 0" class="tk-muted tk-thread__empty">
              暂无消息
            </div>
          </div>
        </div>

        <!-- 操作区 -->
        <el-divider />
        <div class="tk-actions">
          <div class="tk-actions__row">
            <el-select v-model="actionAssignee" placeholder="指派处理人" size="small" style="width: 160px">
              <el-option v-for="a in adminList" :key="a.id" :label="a.nickname || a.username" :value="a.id" />
            </el-select>
            <el-button size="small" type="primary" @click="doAssign" :disabled="!actionAssignee">指派</el-button>
            <el-button size="small" type="success" @click="doAutoAssignFromDetail" :disabled="detail.status === 'closed'" style="margin-left: 6px">自动分配</el-button>

            <el-select v-model="actionStatus" placeholder="更新状态" size="small" style="width: 130px; margin-left: 12px">
              <el-option label="处理中" value="processing" />
              <el-option label="已解决" value="resolved" />
            </el-select>
            <el-button size="small" @click="doUpdateStatus" :disabled="!actionStatus">更新状态</el-button>
          </div>

          <div class="tk-actions__row">
            <el-input
              v-model="replyContent"
              type="textarea"
              :rows="3"
              placeholder="输入回复内容..."
              maxlength="2000"
              show-word-limit
              size="small"
            />
          </div>
          <div class="tk-actions__row">
            <el-button size="small" type="primary" @click="doReply" :disabled="!replyContent.trim()">发送回复</el-button>
            <el-button v-if="detail.status !== 'closed'" size="small" type="warning" @click="doClose">关闭工单</el-button>
            <el-button v-if="detail.status === 'closed'" size="small" type="success" @click="doReopen">重新打开</el-button>
          </div>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  adminListTickets,
  adminGetTicket,
  adminAssignTicket,
  adminAutoAssignTicket,
  adminUpdateTicketStatus,
  adminTicketSendMessage,
  adminCloseTicket,
  adminReopenTicket,
  adminListAdmins,
} from '@/api/admin'

const loading = ref(false)
const items = ref([])
const total = ref(0)
const page = ref(1)
const pageSize = 20
const filterStatus = ref('')
const filterCategory = ref('')
const filterPriority = ref('')
const stats = reactive({ open_count: 0, processing_count: 0, resolved_count: 0, closed_count: 0 })

const detailVisible = ref(false)
const detail = ref(null)
const adminList = ref([])
const actionAssignee = ref(null)
const actionStatus = ref('')
const replyContent = ref('')

async function fetch() {
  loading.value = true
  try {
    const params = { page: page.value, page_size: pageSize }
    if (filterStatus.value) params.status = filterStatus.value
    if (filterCategory.value) params.category = filterCategory.value
    if (filterPriority.value) params.priority = filterPriority.value
    const d = await adminListTickets(params)
    items.value = d.data.items || []
    total.value = d.data.total || 0
    Object.assign(stats, {
      open_count: d.data.open_count || 0,
      processing_count: d.data.processing_count || 0,
      resolved_count: d.data.resolved_count || 0,
      closed_count: d.data.closed_count || 0,
    })
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function search() {
  page.value = 1
  fetch()
}

async function openDetail(row) {
  try {
    const d = await adminGetTicket(row.id)
    detail.value = d.data
    detailVisible.value = true
    actionAssignee.value = d.data.assignee_id || null
    actionStatus.value = ''
    replyContent.value = ''
    if (adminList.value.length === 0) {
      try {
        const ad = await adminListAdmins()
        adminList.value = ad.data.items || []
      } catch { /* ignore */ }
    }
  } catch (e) {
    ElMessage.error(e.message || '获取详情失败')
  }
}

async function doAssign() {
  if (!detail.value || !actionAssignee.value) return
  try {
    await adminAssignTicket(detail.value.id, Number(actionAssignee.value))
    ElMessage.success('已指派')
    await openDetail({ id: detail.value.id })
    fetch()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

async function doAutoAssign(row) {
  try {
    await adminAutoAssignTicket(row.id)
    ElMessage.success('已自动分配')
    fetch()
  } catch (e) {
    ElMessage.error(e.message || '自动分配失败')
  }
}

async function doAutoAssignFromDetail() {
  if (!detail.value) return
  try {
    await adminAutoAssignTicket(detail.value.id)
    ElMessage.success('已自动分配给工作量最少的客服')
    await openDetail({ id: detail.value.id })
    fetch()
  } catch (e) {
    ElMessage.error(e.message || '自动分配失败')
  }
}

async function doUpdateStatus() {
  if (!detail.value || !actionStatus.value) return
  try {
    await adminUpdateTicketStatus(detail.value.id, actionStatus.value)
    ElMessage.success('状态已更新')
    await openDetail(detail.value)
    fetch()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

async function doReply() {
  if (!detail.value || !replyContent.value.trim()) return
  try {
    await adminTicketSendMessage(detail.value.id, replyContent.value.trim())
    replyContent.value = ''
    ElMessage.success('已发送')
    await openDetail(detail.value)
  } catch (e) {
    ElMessage.error(e.message || '发送失败')
  }
}

async function doClose() {
  if (!detail.value) return
  try {
    await ElMessageBox.confirm('确认关闭此工单？')
    await adminCloseTicket(detail.value.id)
    ElMessage.success('工单已关闭')
    await openDetail(detail.value)
    fetch()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(e.message || '操作失败')
  }
}

async function doReopen() {
  if (!detail.value) return
  try {
    await adminReopenTicket(detail.value.id)
    ElMessage.success('工单已重新打开')
    await openDetail(detail.value)
    fetch()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

function categoryLabel(c) {
  return { account: '账号问题', content: '内容问题', payment: '支付问题', technical: '技术故障', other: '其他' }[c] || c
}

function priorityLabel(p) {
  return { urgent: '紧急', high: '高', medium: '中', low: '低' }[p] || p
}

function priorityTag(p) {
  return { urgent: 'danger', high: 'danger', medium: 'warning', low: 'info' }[p] || ''
}

function statusLabel(s) {
  return { open: '待处理', assigned: '已指派', processing: '处理中', resolved: '已解决', closed: '已关闭' }[s] || s
}

function statusTag(s) {
  return { open: 'warning', assigned: 'info', processing: '', resolved: 'success', closed: 'info' }[s] || ''
}

function slaCountdown(deadline, status) {
  if (status === 'closed' || status === 'resolved') return '已完成'
  const diff = new Date(deadline).getTime() - Date.now()
  if (diff <= 0) return '已超时'
  const h = Math.floor(diff / 3600000)
  const m = Math.floor((diff % 3600000) / 60000)
  return `剩 ${h}h ${m}m`
}

function slaClass(deadline, status) {
  if (status === 'closed' || status === 'resolved') return 'tk-sla--done'
  const diff = new Date(deadline).getTime() - Date.now()
  if (diff <= 0) return 'tk-sla--over'
  if (diff < 3600000) return 'tk-sla--warn'
  return 'tk-sla--ok'
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => fetch())
</script>

<style scoped>
.tk-page { padding: 20px 24px; }
.tk-page__head { margin-bottom: 14px; }
.tk-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.tk-page__desc { margin: 0; font-size: 13px; color: #9499a0; }

.tk-stats { display: flex; gap: 10px; margin-bottom: 14px; }
.tk-stat { flex: 1; max-width: 140px; padding: 12px 16px; border-radius: 8px; background: #fff; border: 1px solid #e3e5e7; cursor: pointer; transition: all .15s; }
.tk-stat:hover { transform: translateY(-1px); box-shadow: 0 2px 8px rgba(0,0,0,.06); }
.tk-stat--warn { border-color: #ffe0b0; background: #fffaf3; }
.tk-stat--info { border-color: #d0e8ff; background: #f0f7ff; }
.tk-stat--ok { border-color: #d4edda; background: #f0faf3; }
.tk-stat--dim { border-color: #e3e5e7; background: #fafafa; }
.tk-stat__val { font-size: 24px; font-weight: 700; color: #18191c; }
.tk-stat--warn .tk-stat__val { color: #e6a23c; }
.tk-stat--info .tk-stat__val { color: #409eff; }
.tk-stat--ok .tk-stat__val { color: #67c23a; }
.tk-stat__label { font-size: 12px; color: #9499a0; }

.tk-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.tk-pager { margin-top: 16px; display: flex; justify-content: flex-end; }
.tk-muted { color: #9499a0; }
.tk-link { color: #00a1d6; cursor: pointer; }
.tk-link:hover { text-decoration: underline; }

.tk-sla--over { color: #f56c6c; font-weight: 600; font-size: 12px; }
.tk-sla--warn { color: #e6a23c; font-weight: 600; font-size: 12px; }
.tk-sla--ok { color: #67c23a; font-size: 12px; }
.tk-sla--done { color: #9499a0; font-size: 12px; }

.tk-desc { white-space: pre-wrap; word-break: break-word; line-height: 1.6; }

.tk-thread { margin-top: 16px; }
.tk-thread__title { margin: 0 0 10px; font-size: 14px; font-weight: 600; color: #18191c; }
.tk-thread__list { max-height: 260px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
.tk-thread__empty { text-align: center; padding: 20px; }
.tk-msg { padding: 10px 14px; border-radius: 8px; background: #f6f7f8; }
.tk-msg--admin { background: #e6f7ff; }
.tk-msg__head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
.tk-msg__author { font-size: 13px; font-weight: 600; color: #18191c; display: flex; align-items: center; gap: 4px; }
.tk-msg__time { font-size: 11px; color: #9499a0; }
.tk-msg__body { font-size: 13px; color: #61666d; line-height: 1.6; white-space: pre-wrap; word-break: break-word; }

.tk-actions { margin-top: 4px; }
.tk-actions__row { display: flex; gap: 8px; align-items: center; margin-bottom: 10px; flex-wrap: wrap; }
.tk-actions__row .el-textarea { flex: 1; }

.tk-sat { text-align: center; margin-bottom: 8px; }
.tk-sat__stars { display: flex; align-items: center; justify-content: center; gap: 4px; }
.tk-sat__star { font-size: 28px; color: #ddd; }
.tk-sat__star--active { color: #f5a623; }
.tk-sat__label { font-size: 16px; font-weight: 600; color: #f5a623; margin-left: 6px; }
.tk-sat__comment { font-size: 13px; color: #61666d; margin: 6px 0 2px; }
.tk-sat__time { font-size: 11px; color: #9499a0; }
</style>
