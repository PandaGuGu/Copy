<template>
  <div class="cs-chat-page">
    <!-- 左侧会话列表 -->
    <div class="cs-chat-sidebar" :class="{ 'cs-chat-sidebar--hidden': mobileConvOpen }">
      <div class="cs-chat-sidebar__head">
        <h3>客服消息</h3>
        <el-button size="small" type="primary" @click="startNewConv">新会话</el-button>
      </div>
      <div class="cs-chat-sidebar__list" v-loading="listLoading">
        <div
          v-for="conv in conversations"
          :key="conv.id"
          class="cs-chat-conv"
          :class="{ 'cs-chat-conv--active': activeConv?.id === conv.id }"
          @click="openConv(conv)"
        >
          <div class="cs-chat-conv__head">
            <span class="cs-chat-conv__status" :class="'cs-chat-conv__status--' + conv.status">{{ statusLabel(conv.status) }}</span>
            <span class="cs-chat-conv__time">{{ shortTime(conv.updated_at || conv.created_at) }}</span>
          </div>
          <div class="cs-chat-conv__preview">
            {{ conv.last_msg_preview || '点击查看对话' }}
          </div>
        </div>
        <div v-if="!listLoading && conversations.length === 0" class="cs-chat-sidebar__empty">
          <p>暂无客服会话</p>
          <p class="cs-chat-sidebar__empty-hint">点击「新会话」开始咨询</p>
        </div>
      </div>
    </div>

    <!-- 右侧聊天区 -->
    <div class="cs-chat-main" :class="{ 'cs-chat-main--empty': !activeConv }">
      <template v-if="activeConv">
        <div class="cs-chat-header">
          <el-button size="small" text @click="activeConv = null" class="cs-chat-back">← 返回</el-button>
          <span class="cs-chat-header__status">
            <el-tag :type="activeConv.status === 'active' ? 'success' : 'info'" size="small">{{ statusLabel(activeConv.status) }}</el-tag>
          </span>
        </div>

        <div class="cs-chat-body" ref="chatBody">
          <div
            v-for="msg in messages"
            :key="msg.id"
            class="cs-chat-msg"
            :class="{ 'cs-chat-msg--me': msg.sender_type === 'user' }"
          >
            <div class="cs-chat-msg__bubble">
              <div class="cs-chat-msg__text">{{ msg.content }}</div>
              <div class="cs-chat-msg__time">{{ shortTime(msg.created_at) }}</div>
            </div>
          </div>
          <div v-if="messages.length === 0" class="cs-chat-empty">发送第一条消息开始对话</div>
        </div>

        <div class="cs-chat-input" v-if="activeConv.status !== 'closed'">
          <el-input
            v-model="inputText"
            type="textarea"
            :rows="2"
            placeholder="输入消息..."
            maxlength="2000"
            show-word-limit
            @keydown.enter.exact="sendMsg"
          />
          <el-button type="primary" size="small" @click="sendMsg" :loading="sending" :disabled="!inputText.trim()">发送</el-button>
        </div>
        <div class="cs-chat-closed" v-else>
          <el-tag type="info">会话已结束</el-tag>
        </div>
      </template>
      <div v-else class="cs-chat-placeholder">
        <span class="cs-chat-placeholder__icon">💬</span>
        <p>选择左侧会话或开始新对话</p>
      </div>
    </div>

    <!-- 新建会话弹窗 -->
    <el-dialog v-model="newConvVisible" title="发起客服会话" width="420px" destroy-on-close>
      <el-form :model="newConvForm" label-position="top">
        <el-form-item label="问题描述">
          <el-input
            v-model="newConvForm.message"
            type="textarea"
            :rows="4"
            placeholder="简要描述您遇到的问题..."
            maxlength="500"
            show-word-limit
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="newConvVisible = false">取消</el-button>
        <el-button type="primary" :loading="creating" @click="createConv">发起会话</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import http from '@/utils/http'

const listLoading = ref(false)
const conversations = ref([])
const activeConv = ref(null)
const messages = ref([])
const inputText = ref('')
const sending = ref(false)
const chatBody = ref(null)
const newConvVisible = ref(false)
const creating = ref(false)
const mobileConvOpen = ref(false)
let pollTimer = null

const newConvForm = reactive({ message: '' })

function statusLabel(s) {
  return { waiting: '等待分配', active: '进行中', closed: '已结束' }[s] || s
}

function shortTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = n => String(n).padStart(2, '0')
  return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

async function fetchList() {
  listLoading.value = true
  try {
    const res = await http.get('/api/v1/users/me/cs/conversations')
    if (res.code !== 0) throw new Error(res.msg)
    const items = res.data.items || []
    items.forEach(it => {
      if (it.last_message) it.last_msg_preview = it.last_message.content
    })
    conversations.value = items
  } catch { /* ignore */ } finally {
    listLoading.value = false
  }
}

async function openConv(conv) {
  activeConv.value = conv
  messages.value = []
  try {
    const res = await http.get(`/api/v1/users/me/cs/conversations/${conv.id}`)
    if (res.code !== 0) throw new Error(res.msg)
    messages.value = res.data.messages || []
    conv.status = res.data.status
    scrollBottom()
  } catch {
    ElMessage.error('加载消息失败')
  }
  startPoll()
}

function scrollBottom() {
  nextTick(() => {
    if (chatBody.value) chatBody.value.scrollTop = chatBody.value.scrollHeight
  })
}

async function sendMsg() {
  if (!inputText.value.trim() || !activeConv.value) return
  sending.value = true
  try {
    const res = await http.post(`/api/v1/users/me/cs/conversations/${activeConv.value.id}/messages`, {
      content: inputText.value.trim()
    })
    if (res.code !== 0) throw new Error(res.msg)
    messages.value.push({
      id: res.data.id,
      sender_type: 'user',
      content: inputText.value.trim(),
      created_at: new Date().toISOString()
    })
    inputText.value = ''
    scrollBottom()
  } catch (e) {
    ElMessage.error(e.message || '发送失败')
  } finally {
    sending.value = false
  }
}

async function startNewConv() {
  newConvForm.message = ''
  newConvVisible.value = true
}

async function createConv() {
  if (!newConvForm.message.trim()) {
    ElMessage.warning('请输入问题描述')
    return
  }
  creating.value = true
  try {
    const res = await http.post('/api/v1/cs/conversations', {
      message: newConvForm.message.trim()
    })
    if (res.code !== 0) throw new Error(res.msg)
    ElMessage.success('会话已发起')
    newConvVisible.value = false
    await fetchList()
    const conv = conversations.value.find(c => c.id === res.data.id)
    if (conv) openConv(conv)
  } catch (e) {
    ElMessage.error(e.message || '发起失败')
  } finally {
    creating.value = false
  }
}

function startPoll() {
  stopPoll()
  pollTimer = setInterval(async () => {
    if (!activeConv.value || activeConv.value.status === 'closed') return
    try {
      const res = await http.get(`/api/v1/users/me/cs/conversations/${activeConv.value.id}`)
      if (res.code !== 0) return
      const newMsgs = res.data.messages || []
      if (newMsgs.length !== messages.value.length) {
        messages.value = newMsgs
        if (activeConv.value) activeConv.value.status = res.data.status
        scrollBottom()
      }
    } catch { /* ignore */ }
  }, 3000)
}

function stopPoll() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

onMounted(() => fetchList())
onBeforeUnmount(() => stopPoll())
</script>

<style scoped>
.cs-chat-page {
  display: flex;
  height: calc(100vh - 56px);
  max-width: 900px;
  margin: 0 auto;
  border-left: 1px solid #e3e5e7;
  border-right: 1px solid #e3e5e7;
  background: #fff;
}

.cs-chat-sidebar {
  width: 260px;
  flex-shrink: 0;
  border-right: 1px solid #e3e5e7;
  display: flex;
  flex-direction: column;
  background: #fafafa;
}
.cs-chat-sidebar__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 12px;
  border-bottom: 1px solid #e3e5e7;
}
.cs-chat-sidebar__head h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #18191c;
}
.cs-chat-sidebar__list {
  flex: 1;
  overflow-y: auto;
}
.cs-chat-sidebar__empty {
  padding: 40px 16px;
  text-align: center;
  color: #9499a0;
  font-size: 13px;
}
.cs-chat-sidebar__empty-hint { font-size: 12px; margin-top: 4px; }

.cs-chat-conv {
  padding: 12px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  transition: background .1s;
}
.cs-chat-conv:hover { background: #f0f7ff; }
.cs-chat-conv--active { background: #e6f7ff; }
.cs-chat-conv__head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}
.cs-chat-conv__status { font-size: 11px; padding: 1px 6px; border-radius: 3px; }
.cs-chat-conv__status--waiting { background: #fff3e0; color: #e6a23c; }
.cs-chat-conv__status--active { background: #e8f5e9; color: #67c23a; }
.cs-chat-conv__status--closed { background: #eee; color: #999; }
.cs-chat-conv__time { font-size: 11px; color: #9499a0; }
.cs-chat-conv__preview { font-size: 12px; color: #61666d; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.cs-chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.cs-chat-main--empty {
  align-items: center;
  justify-content: center;
}

.cs-chat-header {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid #e3e5e7;
  gap: 10px;
}
.cs-chat-back { display: none; }
.cs-chat-header__status { margin-left: auto; }

.cs-chat-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.cs-chat-msg { display: flex; }
.cs-chat-msg--me { justify-content: flex-end; }
.cs-chat-msg__bubble {
  max-width: 70%;
  padding: 10px 14px;
  border-radius: 12px;
  background: #f6f7f8;
}
.cs-chat-msg--me .cs-chat-msg__bubble { background: #00a1d6; color: #fff; }
.cs-chat-msg__text { font-size: 14px; line-height: 1.6; white-space: pre-wrap; word-break: break-word; }
.cs-chat-msg__time { font-size: 11px; color: #9499a0; margin-top: 4px; }
.cs-chat-msg--me .cs-chat-msg__time { color: rgba(255,255,255,.7); text-align: right; }

.cs-chat-input {
  display: flex;
  gap: 8px;
  padding: 10px 14px;
  border-top: 1px solid #e3e5e7;
  align-items: flex-end;
}
.cs-chat-input .el-textarea { flex: 1; }

.cs-chat-closed { padding: 14px; text-align: center; border-top: 1px solid #e3e5e7; }
.cs-chat-empty { text-align: center; color: #9499a0; padding: 40px; }
.cs-chat-placeholder { text-align: center; color: #9499a0; }
.cs-chat-placeholder__icon { font-size: 48px; display: block; margin-bottom: 8px; }

@media (max-width: 640px) {
  .cs-chat-sidebar--hidden { display: none; }
  .cs-chat-sidebar { width: 100%; }
  .cs-chat-back { display: inline-flex; }
}
</style>
