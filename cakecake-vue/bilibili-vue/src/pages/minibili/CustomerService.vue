<template>
  <div class="cs-page" v-loading="ticketsLoading">
    <!-- 头部 -->
    <div class="cs-hero">
      <div class="cs-hero__icon">🎧</div>
      <h1 class="cs-hero__title">需要帮助吗？</h1>
      <p class="cs-hero__sub">选择您遇到的问题类型，我们会尽快处理</p>
      <div class="cs-hero__links">
        <el-button type="primary" size="default" @click="$router.push('/cs-chat')">
          💬 在线客服聊天
        </el-button>
      </div>
    </div>

    <!-- 工单类型卡片：弱引导 -->
    <div class="cs-cards">
      <div
        v-for="card in ticketTypes"
        :key="card.value"
        class="cs-card"
        :class="{ 'cs-card--active': selectedType === card.value }"
        @click="selectedType = card.value"
      >
        <div class="cs-card__icon">{{ card.icon }}</div>
        <h3 class="cs-card__title">{{ card.label }}</h3>
        <p class="cs-card__desc">{{ card.desc }}</p>
        <div class="cs-card__examples">
          <el-tag
            v-for="ex in card.examples"
            :key="ex"
            size="small"
            effect="plain"
            class="cs-card__tag"
          >{{ ex }}</el-tag>
        </div>
      </div>
    </div>

    <!-- 快捷表单 -->
    <div class="cs-form-card" v-if="selectedType">
      <h3 class="cs-form-card__title">
        提交「{{ ticketTypes.find(t => t.value === selectedType)?.label }}」工单
      </h3>

      <el-form ref="formRef" :model="form" :rules="rules" label-position="top" size="default">
        <el-form-item label="问题标题" prop="subject">
          <el-input
            v-model="form.subject"
            placeholder="一句话描述您遇到的问题"
            maxlength="200"
            show-word-limit
          />
        </el-form-item>

        <el-form-item label="详细描述" prop="description">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="5"
            placeholder="请尽量详细描述，包括发生时间、操作步骤等"
            maxlength="5000"
            show-word-limit
          />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" size="large" :loading="submitting" @click="submit" style="width: 100%">
            {{ submitting ? '提交中...' : '提交工单' }}
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <!-- 还没选类型的提示 -->
    <div class="cs-empty-hint" v-if="!selectedType && !ticketsLoading">
      <span>👆 请先选择一种问题类型，然后填写详情提交</span>
    </div>

    <!-- 我的工单列表 -->
    <el-divider />
    <div class="cs-my-tickets">
      <h3 class="cs-my-tickets__title">
        我的工单
        <el-button size="small" text type="primary" @click="fetchMyTickets" :loading="ticketsLoading">刷新</el-button>
      </h3>

      <div v-if="myTickets.length === 0 && !ticketsLoading" class="cs-empty">
        <span class="cs-empty__icon">📭</span>
        <p>还没有提交过工单</p>
      </div>

      <div v-else class="cs-ticket-list">
        <div
          v-for="t in myTickets"
          :key="t.id"
          class="cs-ticket-item"
          :class="'cs-ticket-item--' + t.status"
          @click="openTicketDetail(t)"
        >
          <div class="cs-ticket-item__left">
            <span class="cs-ticket-item__id">#{{ t.id }}</span>
            <span class="cs-ticket-item__subject">{{ t.subject }}</span>
          </div>
          <div class="cs-ticket-item__right">
            <el-tag :type="statusTag(t.status)" size="small">{{ statusLabel(t.status) }}</el-tag>
            <span class="cs-ticket-item__time">{{ fmtTime(t.created_at) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 工单详情弹窗 -->
    <el-dialog
      v-model="detailVisible"
      :title="'工单 #' + (currentTicket?.id || '')"
      width="640px"
      destroy-on-close
    >
      <template v-if="currentTicket">
        <div class="cs-detail-status">
          <span class="cs-detail-status__label">状态：</span>
          <el-tag :type="statusTag(currentTicket.status)" size="default">{{ statusLabel(currentTicket.status) }}</el-tag>
          <span class="cs-detail-status__time">提交于 {{ fmtTime(currentTicket.created_at) }}</span>
        </div>

        <el-descriptions :column="1" border size="small" class="cs-detail-desc">
          <el-descriptions-item label="标题">{{ currentTicket.subject }}</el-descriptions-item>
          <el-descriptions-item label="类型">{{ categoryLabel(currentTicket.category) }}</el-descriptions-item>
          <el-descriptions-item label="详情">
            <div class="cs-detail-desc__body">{{ currentTicket.description }}</div>
          </el-descriptions-item>
        </el-descriptions>

        <!-- 消息线程 -->
        <div class="cs-detail-thread" v-if="currentTicket.messages && currentTicket.messages.length > 0">
          <h4>处理记录</h4>
          <div
            v-for="msg in currentTicket.messages"
            :key="msg.id"
            class="cs-detail-msg"
            :class="{ 'cs-detail-msg--admin': msg.sender_type === 'admin' }"
          >
            <div class="cs-detail-msg__head">
              <span class="cs-detail-msg__role">{{ msg.sender_type === 'admin' ? '💼 客服' : '🧑 我' }}</span>
              <span class="cs-detail-msg__time">{{ fmtTime(msg.created_at) }}</span>
            </div>
            <div class="cs-detail-msg__body">{{ msg.content }}</div>
          </div>
        </div>
        <div v-else class="cs-detail-empty">暂无处理记录</div>

        <!-- 补充消息 -->
        <div class="cs-detail-reply" v-if="currentTicket.status !== 'closed'">
          <el-input
            v-model="replyText"
            type="textarea"
            :rows="2"
            placeholder="补充描述或追问..."
            maxlength="2000"
            show-word-limit
          />
          <el-button type="primary" size="small" @click="sendReply" :loading="replyLoading" :disabled="!replyText.trim()" style="margin-top: 8px">
            发送
          </el-button>
        </div>

        <!-- 满意度评分 -->
        <div class="cs-detail-rating" v-if="currentTicket.status === 'resolved'">
          <el-divider />
          <p class="cs-detail-rating__hint">问题已解决？请为本次服务评分</p>
          <div class="cs-detail-rating__stars">
            <span
              v-for="s in 5"
              :key="s"
              class="cs-detail-rating__star"
              :class="{ 'cs-detail-rating__star--active': s <= ratingScore }"
              @click="submitRating(s)"
            >★</span>
          </div>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue'
import { ElMessage } from 'element-plus'
import http from '@/utils/http'

const ticketTypes = [
  {
    value: 'report',
    icon: '📋',
    label: '举报投诉',
    desc: '举报违规内容、不当行为或侵权视频',
    examples: ['违规视频', '引战评论', '恶意刷屏弹幕']
  },
  {
    value: 'copyright',
    icon: '©️',
    label: '版权申诉',
    desc: '内容被侵权、下架申诉、版权相关',
    examples: ['视频被盗搬', '音乐侵权', '图片侵权']
  },
  {
    value: 'appeal',
    icon: '🔐',
    label: '账号申诉',
    desc: '账号被封禁、功能受限、安全申诉',
    examples: ['封禁申诉', '限制解除', '账号找回']
  },
  {
    value: 'general',
    icon: '💬',
    label: '其他问题',
    desc: '功能建议、技术故障、Bug 反馈',
    examples: ['页面卡顿', '播放异常', '功能建议']
  }
]

const selectedType = ref('')
const submitting = ref(false)
const ticketsLoading = ref(false)
const replyLoading = ref(false)
const formRef = ref(null)
const myTickets = ref([])
const detailVisible = ref(false)
const currentTicket = ref(null)
const replyText = ref('')
const ratingScore = ref(0)

const form = reactive({
  subject: '',
  description: ''
})

const rules = {
  subject: [{ required: true, message: '请输入标题', trigger: 'blur' }],
  description: [{ required: true, message: '请输入详情', trigger: 'blur' }]
}

function statusTag(s) {
  const m = { open: 'danger', assigned: 'warning', processing: '', resolved: 'success', closed: 'info', reopened: 'warning' }
  return m[s] || 'info'
}

function statusLabel(s) {
  const m = { open: '待处理', assigned: '已指派', processing: '处理中', resolved: '已解决', closed: '已关闭', reopened: '已重开' }
  return m[s] || s
}

function categoryLabel(c) {
  const m = { report: '举报投诉', copyright: '版权申诉', appeal: '账号申诉', general: '其他问题' }
  return m[c] || c
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

async function submit() {
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const res = await http.post('/api/v1/tickets', {
      category: selectedType.value,
      subject: form.subject.trim(),
      description: form.description.trim()
    })
    if (res.code !== 0) throw new Error(res.msg || '提交失败')
    ElMessage.success('工单已提交！工单ID: ' + res.data.id)
    form.subject = ''
    form.description = ''
    selectedType.value = ''
    fetchMyTickets()
  } catch (e) {
    ElMessage.error(e.message || '提交失败')
  } finally {
    submitting.value = false
  }
}

async function fetchMyTickets() {
  ticketsLoading.value = true
  try {
    const res = await http.get('/api/v1/users/me/tickets')
    if (res.code !== 0) throw new Error(res.msg)
    myTickets.value = res.data.items || []
  } catch {
    myTickets.value = []
  } finally {
    ticketsLoading.value = false
  }
}

async function openTicketDetail(ticket) {
  try {
    const res = await http.get(`/api/v1/users/me/tickets/${ticket.id}`)
    if (res.code !== 0) throw new Error(res.msg)
    currentTicket.value = res.data.ticket || res.data
    currentTicket.value.messages = res.data.messages || []
    detailVisible.value = true
    replyText.value = ''
    ratingScore.value = 0
  } catch {
    ElMessage.error('获取工单详情失败')
  }
}

async function sendReply() {
  if (!currentTicket.value || !replyText.value.trim()) return
  replyLoading.value = true
  try {
    const res = await http.post(`/api/v1/users/me/tickets/${currentTicket.value.id}/messages`, {
      content: replyText.value.trim()
    })
    if (res.code !== 0) throw new Error(res.msg)
    ElMessage.success('已发送')
    replyText.value = ''
    await openTicketDetail({ id: currentTicket.value.id })
  } catch (e) {
    ElMessage.error(e.message || '发送失败')
  } finally {
    replyLoading.value = false
  }
}

async function submitRating(score) {
  if (!currentTicket.value) return
  ratingScore.value = score
  try {
    const res = await http.post(`/api/v1/tickets/${currentTicket.value.id}/satisfaction`, {
      score: score,
      comment: ''
    })
    if (res.code !== 0) throw new Error(res.msg)
    ElMessage.success('感谢你的评分！工单已关闭')
    detailVisible.value = false
    fetchMyTickets()
  } catch (e) {
    ElMessage.error(e.message || '评分失败')
    ratingScore.value = 0
  }
}

fetchMyTickets()
</script>

<style scoped>
.cs-page {
  max-width: 780px;
  margin: 0 auto;
  padding: 40px 16px 80px;
}

.cs-hero {
  text-align: center;
  margin-bottom: 32px;
}
.cs-hero__icon {
  font-size: 48px;
  margin-bottom: 8px;
}
.cs-hero__title {
  font-size: 26px;
  font-weight: 700;
  color: #18191c;
  margin: 0 0 8px;
}
.cs-hero__sub {
  font-size: 14px;
  color: #9499a0;
  margin: 0;
}
.cs-hero__links {
  margin-top: 14px;
}

/* 类型卡片 */
.cs-cards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}
.cs-card {
  background: #fff;
  border: 2px solid #e3e5e7;
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all .2s;
  position: relative;
}
.cs-card:hover {
  border-color: #00a1d6;
  box-shadow: 0 2px 12px rgba(0,161,214,.1);
  transform: translateY(-1px);
}
.cs-card--active {
  border-color: #00a1d6;
  background: #f0faff;
  box-shadow: 0 2px 12px rgba(0,161,214,.15);
}
.cs-card--active::after {
  content: '✓';
  position: absolute;
  top: 12px;
  right: 14px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: #00a1d6;
  color: #fff;
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.cs-card__icon {
  font-size: 28px;
  margin-bottom: 8px;
}
.cs-card__title {
  font-size: 16px;
  font-weight: 600;
  color: #18191c;
  margin: 0 0 6px;
}
.cs-card__desc {
  font-size: 13px;
  color: #9499a0;
  margin: 0 0 10px;
  line-height: 1.5;
}
.cs-card__examples {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.cs-card__tag {
  font-size: 11px;
}

/* 表单 */
.cs-form-card {
  background: #fff;
  border: 1px solid #e3e5e7;
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
}
.cs-form-card__title {
  font-size: 16px;
  font-weight: 600;
  color: #18191c;
  margin: 0 0 18px;
}

/* 空提示 */
.cs-empty-hint {
  text-align: center;
  padding: 32px;
  color: #9499a0;
  font-size: 14px;
  background: #f6f7f8;
  border-radius: 12px;
  margin-bottom: 24px;
}

/* 我的工单 */
.cs-my-tickets__title {
  font-size: 15px;
  font-weight: 600;
  color: #18191c;
  margin: 0 0 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.cs-empty {
  text-align: center;
  padding: 40px;
  color: #9499a0;
}
.cs-empty__icon {
  font-size: 40px;
  display: block;
  margin-bottom: 8px;
}
.cs-ticket-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.cs-ticket-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #fff;
  border: 1px solid #e3e5e7;
  border-radius: 8px;
  padding: 12px 16px;
  cursor: pointer;
  transition: all .15s;
}
.cs-ticket-item:hover {
  border-color: #00a1d6;
  box-shadow: 0 1px 6px rgba(0,0,0,.04);
}
.cs-ticket-item--open {
  border-left: 3px solid #f56c6c;
}
.cs-ticket-item--assigned,
.cs-ticket-item--processing {
  border-left: 3px solid #409eff;
}
.cs-ticket-item--resolved {
  border-left: 3px solid #67c23a;
}
.cs-ticket-item__left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.cs-ticket-item__id {
  font-size: 12px;
  color: #9499a0;
  font-weight: 600;
  flex-shrink: 0;
}
.cs-ticket-item__subject {
  font-size: 14px;
  color: #18191c;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cs-ticket-item__right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.cs-ticket-item__time {
  font-size: 12px;
  color: #9499a0;
}

/* 工单详情 */
.cs-detail-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}
.cs-detail-status__label {
  font-size: 13px;
  color: #61666d;
}
.cs-detail-status__time {
  font-size: 12px;
  color: #9499a0;
  margin-left: auto;
}
.cs-detail-desc {
  margin-bottom: 16px;
}
.cs-detail-desc__body {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
}
.cs-detail-thread {
  margin-bottom: 16px;
}
.cs-detail-thread h4 {
  font-size: 14px;
  font-weight: 600;
  color: #18191c;
  margin: 0 0 10px;
}
.cs-detail-msg {
  padding: 10px 14px;
  border-radius: 8px;
  background: #f6f7f8;
  margin-bottom: 8px;
}
.cs-detail-msg--admin {
  background: #e6f7ff;
}
.cs-detail-msg__head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}
.cs-detail-msg__role {
  font-size: 12px;
  font-weight: 600;
  color: #18191c;
}
.cs-detail-msg__time {
  font-size: 11px;
  color: #9499a0;
}
.cs-detail-msg__body {
  font-size: 13px;
  color: #61666d;
  line-height: 1.6;
  white-space: pre-wrap;
}
.cs-detail-empty {
  text-align: center;
  padding: 20px;
  color: #9499a0;
  font-size: 13px;
}
.cs-detail-reply {
  margin-top: 12px;
}
.cs-detail-rating {
  text-align: center;
}
.cs-detail-rating__hint {
  font-size: 14px;
  color: #61666d;
  margin: 0 0 12px;
}
.cs-detail-rating__stars {
  display: flex;
  justify-content: center;
  gap: 8px;
}
.cs-detail-rating__star {
  font-size: 36px;
  color: #ddd;
  cursor: pointer;
  transition: color .15s;
  user-select: none;
}
.cs-detail-rating__star:hover,
.cs-detail-rating__star--active {
  color: #f5a623;
}

@media (max-width: 640px) {
  .cs-cards {
    grid-template-columns: 1fr;
  }
  .cs-ticket-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
}
</style>
