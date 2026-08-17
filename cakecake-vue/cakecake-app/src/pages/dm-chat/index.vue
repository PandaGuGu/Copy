<template>
  <view class="chat-page">
    <view class="header">
      <text class="back" @tap="goBack">‹</text>
      <view class="title-wrap">
        <text class="title">{{ peerName }}</text>
        <text v-if="kind === 'agent'" class="sub">官方 AI 助手</text>
      </view>
      <text class="action">⋯</text>
    </view>

    <scroll-view scroll-y class="msg-list" :scroll-into-view="scrollIntoView">
      <view v-if="loading" class="loading"><text>加载消息中…</text></view>
      <view v-else-if="messages.length === 0" class="empty">
        <text class="icon">💬</text>
        <text class="text">暂无消息，开始聊聊吧</text>
      </view>
      <view
        v-for="m in messages"
        :key="m.id"
        :id="`msg-${m.id}`"
        class="msg"
        :class="{ mine: m.role === 'user', ai: m.role !== 'user' }"
      >
        <view class="bubble">{{ m.content }}</view>
        <text class="time">{{ formatTime(m.created_at) }}</text>
      </view>
    </scroll-view>

    <view class="input-bar safe-area-bottom">
      <input
          v-model="text"
          class="input"
          placeholder="说点什么吧…"
          confirm-type="send"
          @confirm="onSend"
        />
      <view class="send-btn" :class="{ active: text.trim() }" @tap="onSend">
        <text>发送</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onLoad as onPageLoad } from '@dcloudio/uni-app'
import { dmApi, type DMMessage } from '@/api/dm'
import { useUserStore } from '@/store/user'

const messages = ref<DMMessage[]>([])
const text = ref('')
const loading = ref(false)
const conversationId = ref(0)
const peerName = ref('会话')
const kind = ref<'agent' | 'user'>('user')
const scrollIntoView = ref('')
const userStore = useUserStore()

onPageLoad((q: any) => {
  conversationId.value = Number(q?.id) || 0
  peerName.value = decodeURIComponent(q?.name || '会话')
  // 通过 kind 判定：AI 助手是 agent；普通用户是 user
  // 这里简单以 name 包含 "AI" 推断
  kind.value = /AI|助手/i.test(peerName.value) ? 'agent' : 'user'
  loadMessages()
})

async function loadMessages() {
  if (!conversationId.value) return
  loading.value = true
  try {
    const r = await dmApi.messages(conversationId.value)
    messages.value = r.items || []
    scrollIntoView.value = messages.value.length ? `msg-${messages.value[messages.value.length - 1].id}` : ''
  } finally {
    loading.value = false
  }
}

async function onSend() {
  const c = text.value.trim()
  if (!c) return
  const sent = text.value
  text.value = ''
  try {
    const msg = await dmApi.send(conversationId.value, c)
    messages.value.push(msg)
    scrollIntoView.value = `msg-${msg.id}`
    // AI 助手场景：模拟回执（后端 ws/chat 可能异步推送，这里兜底刷新一次）
    if (kind.value === 'agent') {
      setTimeout(() => loadMessages(), 1500)
    }
  } catch { /* 已 toast */ }
}

function goBack() { uni.navigateBack() }

function formatTime(t: string): string {
  if (!t) return ''
  const d = new Date(t)
  if (Number.isNaN(d.getTime())) return t
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}
</script>

<style lang="scss" scoped>
.chat-page {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #F7F8FA;
}
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 32rpx 24rpx 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F0F1F3;
  flex-shrink: 0;
  .back { font-size: 40rpx; color: #181818; width: 56rpx; }
  .action { font-size: 40rpx; color: #181818; width: 56rpx; text-align: right; }
  .title-wrap { display: flex; flex-direction: column; align-items: center; gap: 4rpx; }
  .title { font-size: 32rpx; font-weight: 600; color: #181818; }
  .sub { font-size: 20rpx; color: #FB7299; }
}
.msg-list {
  flex: 1;
  padding: 24rpx;
  min-height: 0;
}
.loading, .empty {
  text-align: center;
  padding: 120rpx 0;
  color: #999;
  font-size: 28rpx;
  .icon { font-size: 96rpx; display: block; margin-bottom: 16rpx; }
}
.msg {
  display: flex;
  flex-direction: column;
  margin-bottom: 24rpx;
  max-width: 80%;
  .bubble {
    padding: 16rpx 20rpx;
    border-radius: 20rpx;
    font-size: 28rpx;
    line-height: 1.5;
    word-break: break-word;
  }
  .time {
    font-size: 20rpx;
    color: #999;
    margin-top: 8rpx;
  }
  &.ai {
    align-self: flex-start;
    .bubble {
      background: #FFF;
      color: #181818;
      border-top-left-radius: 4rpx;
      box-shadow: 0 1rpx 4rpx rgba(0,0,0,0.06);
    }
    .time { margin-left: 4rpx; }
  }
  &.mine {
    align-self: flex-end;
    align-items: flex-end;
    .bubble {
      background: #FB7299;
      color: #FFF;
      border-top-right-radius: 4rpx;
    }
    .time { margin-right: 4rpx; }
  }
}
.input-bar {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 16rpx 24rpx;
  background: #FFF;
  border-top: 1rpx solid #F0F1F3;
  flex-shrink: 0;
  .input {
    flex: 1;
    height: 72rpx;
    padding: 0 20rpx;
    background: #F7F8FA;
    border-radius: 36rpx;
    font-size: 28rpx;
  }
  .send-btn {
    padding: 0 28rpx;
    height: 72rpx;
    border-radius: 36rpx;
    background: #F7F8FA;
    color: #999;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 26rpx;
    &.active {
      background: #FB7299;
      color: #FFF;
    }
  }
}
</style>