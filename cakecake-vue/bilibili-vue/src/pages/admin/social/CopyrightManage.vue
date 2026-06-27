<template>
  <div class="cp-page" v-loading="loading">
    <header class="cp-page__head">
      <h2 class="cp-page__title">版权管理</h2>
      <p class="cp-page__desc">处理版权投诉与内容下架请求</p>
    </header>

    <!-- 筛选栏 -->
    <div class="cp-toolbar">
      <el-select v-model="filterStatus" placeholder="状态" clearable size="default" style="width: 130px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="待处理" value="pending" />
        <el-option label="已受理" value="accepted" />
        <el-option label="已驳回" value="rejected" />
        <el-option label="已下架" value="takedown" />
        <el-option label="已恢复" value="restored" />
      </el-select>
      <el-input
        v-model="filterQ"
        placeholder="搜索投诉标题..."
        clearable
        size="default"
        style="width: 240px"
        @keyup.enter="search"
        @clear="search"
      />
      <el-button type="primary" size="default" @click="search">搜索</el-button>
    </div>

    <!-- 投诉列表 -->
    <el-table :data="items" stripe size="default" empty-text="暂无投诉">
      <el-table-column prop="id" label="ID" width="65" />
      <el-table-column label="投诉标题" min-width="200" show-overflow-tooltip>
        <template #default="{ row }">
          <span class="cp-link" @click="openDetail(row)">{{ row.title }}</span>
        </template>
      </el-table-column>
      <el-table-column label="投诉人" width="130">
        <template #default="{ row }">
          <span v-if="row.complainant">{{ row.complainant.name || row.complainant.email }}</span>
          <span v-else class="cp-muted">—</span>
        </template>
      </el-table-column>
      <el-table-column label="关联内容" min-width="180" show-overflow-tooltip>
        <template #default="{ row }">
          <span v-if="row.content">{{ row.content.title }}</span>
          <span v-else class="cp-muted">—</span>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="提交时间" width="155">
        <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="80" fixed="right">
        <template #default="{ row }">
          <el-button size="small" text type="primary" @click="openDetail(row)">详情</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="cp-pager" v-if="total > pageSize">
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="prev, pager, next, total"
        @current-change="fetch"
      />
    </div>

    <!-- 详情弹窗 -->
    <el-dialog v-model="detailVisible" title="版权投诉详情" width="640px" destroy-on-close>
      <template v-if="detail">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="投诉ID">{{ detail.id }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="statusTag(detail.status)" size="small">{{ statusLabel(detail.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="标题" :span="2">{{ detail.title }}</el-descriptions-item>
          <el-descriptions-item label="投诉人" v-if="detail.complainant">
            {{ detail.complainant.name }} ({{ detail.complainant.email }})
          </el-descriptions-item>
          <el-descriptions-item label="投诉人" v-else>
            {{ detail.complainant_name || '—' }}
          </el-descriptions-item>
          <el-descriptions-item label="投诉类型">
            <el-tag size="small" effect="plain">{{ typeLabel(detail.complaint_type) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="提交时间">{{ fmtTime(detail.created_at) }}</el-descriptions-item>
          <el-descriptions-item label="描述" :span="2">
            <div class="cp-desc">{{ detail.description }}</div>
          </el-descriptions-item>
        </el-descriptions>

        <!-- 关联内容 -->
        <div class="cp-section" v-if="detail.content">
          <h4 class="cp-section__title">关联内容</h4>
          <el-descriptions :column="2" border size="small">
            <el-descriptions-item label="内容ID">{{ detail.content.id }}</el-descriptions-item>
            <el-descriptions-item label="类型">{{ detail.content.type || '—' }}</el-descriptions-item>
            <el-descriptions-item label="标题" :span="2">{{ detail.content.title }}</el-descriptions-item>
            <el-descriptions-item label="上传者" v-if="detail.content.uploader">
              {{ detail.content.uploader.nickname || detail.content.uploader.username }}
            </el-descriptions-item>
            <el-descriptions-item label="上传者" v-else>{{ detail.content.uploader_name || '—' }}</el-descriptions-item>
          </el-descriptions>
        </div>

        <!-- 证据链接 -->
        <div class="cp-section" v-if="detail.evidence_urls && detail.evidence_urls.length > 0">
          <h4 class="cp-section__title">证据链接</h4>
          <ul class="cp-evidence">
            <li v-for="(url, i) in detail.evidence_urls" :key="i">
              <a :href="url" target="_blank" rel="noopener" class="cp-evidence__link">{{ url }}</a>
            </li>
          </ul>
        </div>

        <!-- 处理备注 -->
        <div class="cp-section" v-if="detail.handler_comment">
          <h4 class="cp-section__title">处理备注</h4>
          <div class="cp-comment">{{ detail.handler_comment }}</div>
        </div>

        <!-- 操作区 -->
        <el-divider />
        <div class="cp-actions" v-if="detail.status === 'pending' || detail.status === 'accepted'">
          <el-input
            v-model="handlerComment"
            placeholder="处理备注（选填）"
            size="default"
            style="margin-right: 8px"
          />
        </div>
        <div class="cp-actions">
          <el-button
            v-if="detail.status === 'pending'"
            type="success" size="small"
            @click="doAction('accept')"
          >受理</el-button>
          <el-button
            v-if="detail.status === 'pending'"
            type="danger" size="small"
            @click="doAction('reject')"
          >驳回</el-button>
          <el-button
            v-if="detail.status === 'accepted'"
            type="danger" size="small"
            @click="doAction('takedown')"
          >下架内容</el-button>
          <el-button
            v-if="detail.status === 'takedown'"
            type="success" size="small"
            @click="doAction('restore')"
          >恢复内容</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  adminListCopyrightComplaints,
  adminGetCopyrightComplaint,
  adminAcceptCopyrightComplaint,
  adminRejectCopyrightComplaint,
  adminTakedownCopyright,
  adminRestoreCopyright,
} from '@/api/admin'

const loading = ref(false)
const items = ref([])
const total = ref(0)
const page = ref(1)
const pageSize = 20
const filterStatus = ref('')
const filterQ = ref('')

const detailVisible = ref(false)
const detail = ref(null)
const handlerComment = ref('')

async function fetch() {
  loading.value = true
  try {
    const params = { page: page.value, page_size: pageSize }
    if (filterStatus.value) params.status = filterStatus.value
    if (filterQ.value) params.q = filterQ.value
    const d = await adminListCopyrightComplaints(params)
    items.value = d.data?.items || []
    total.value = d.data?.total || 0
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
    const d = await adminGetCopyrightComplaint(row.id)
    detail.value = d.data
    detailVisible.value = true
    handlerComment.value = ''
  } catch (e) {
    ElMessage.error(e.message || '获取详情失败')
  }
}

async function doAction(action) {
  if (!detail.value) return
  const actionLabels = {
    accept: '受理', reject: '驳回', takedown: '下架', restore: '恢复',
  }
  try {
    await ElMessageBox.confirm(`确认${actionLabels[action]}此投诉？`, '提示', {
      type: action === 'reject' || action === 'takedown' ? 'warning' : 'info',
    })
    let fn
    switch (action) {
      case 'accept': fn = adminAcceptCopyrightComplaint; break
      case 'reject': fn = () => adminRejectCopyrightComplaint(detail.value.id, handlerComment.value); break
      case 'takedown': fn = adminTakedownCopyright; break
      case 'restore': fn = adminRestoreCopyright; break
    }
    await fn(detail.value.id)
    ElMessage.success(`已${actionLabels[action]}`)
    await openDetail(detail.value)
    fetch()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(e.message || '操作失败')
  }
}

function statusLabel(s) {
  return { pending: '待处理', accepted: '已受理', rejected: '已驳回', takedown: '已下架', restored: '已恢复' }[s] || s
}

function statusTag(s) {
  return { pending: 'warning', accepted: '', rejected: 'info', takedown: 'danger', restored: 'success' }[s] || ''
}

function typeLabel(t) {
  return { infringement: '侵权', plagiarism: '抄袭', unauthorized: '未授权转载', other: '其他' }[t] || t
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
.cp-page { padding: 20px 24px; }
.cp-page__head { margin-bottom: 14px; }
.cp-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.cp-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.cp-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.cp-pager { margin-top: 16px; display: flex; justify-content: flex-end; }
.cp-muted { color: #9499a0; }
.cp-link { color: #00a1d6; cursor: pointer; }
.cp-link:hover { text-decoration: underline; }
.cp-desc { white-space: pre-wrap; word-break: break-word; line-height: 1.6; }
.cp-section { margin-top: 16px; }
.cp-section__title { margin: 0 0 8px; font-size: 14px; font-weight: 600; color: #18191c; }
.cp-evidence { list-style: none; padding: 0; margin: 0; }
.cp-evidence li { padding: 4px 0; }
.cp-evidence__link { color: #00a1d6; font-size: 13px; word-break: break-all; }
.cp-evidence__link:hover { text-decoration: underline; }
.cp-comment { background: #f6f7f8; border-radius: 6px; padding: 10px 14px; font-size: 13px; color: #61666d; line-height: 1.6; }
.cp-actions { display: flex; gap: 8px; align-items: center; margin-top: 10px; flex-wrap: wrap; }
</style>
