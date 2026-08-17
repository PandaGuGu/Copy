<template>
  <view class="dm-page">
    <view class="header">
      <text class="back" @tap="goBack">‹</text>
      <text class="title">消息</text>
      <text class="action" @tap="onNew">+</text>
    </view>

    <view v-if="loading" class="loading"><text>加载中…</text></view>
    <view v-else-if="conversations.length === 0" class="empty">
      <text class="icon">�</text>
      <text class="text">暂无消息</text>
    </view>
    <view v-else class="conv-list">
      <view
        v-for="c in conversations"
        :key="c.id"
        class="conv-item"
        @tap="goChat(c)"
      >
        <view class="avatar-wrap">
          <text class="avatar-text">{{ c.is_agent ? '🤖' : c.peer_name.slice(0,1) }}</text>
          <view v-if="c.unread_count > 0" class="badge">
            <text>{{ c.unread_count > 99 ? '99+' : c.unread_count }}</text>
          </view>
        </view>
        <view class="conv-body">
          <view class="conv-row1">
            <text class="peer-name">{{ c.peer_name }}</text>
            <text class="time">{{ formatTime(c.last_message_at) }}</text>
          </view>
          <view class="conv-row2">
            <text class="preview text-ellipsis-1">{{ c.last_preview || '……' }}</text>
            <text v-if="c.kind === 'agent'" class="tag">官方助手</text>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { dmApi, type DMConversation } from '@/api/dm'

const conversations = ref<DMConversation[]>([])
const loading = ref(false)

onMounted(load)

async function load() {
  loading.value = true
  try {
    const r = await dmApi.conversations()
    conversations.value = r.items || []
  } finally {
    loading.value = false
  }
}

function goChat(c: DMConversation) {
  uni.navigateTo({ url: `/pages/dm-chat/index?id=${c.id}&name=${encodeURIComponent(c.peer_name)}` })
}
function goBack() { uni.navigateBack() }
function onNew() { uni.showToast({ title: '新建私信待开发', icon: 'none' }) }

function formatTime(t: string): string {
  if (!t) return ''
  const d = new Date(t)
  if (Number.isNaN(d.getTime())) return t
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  if (diff < 60_000) return '刚刚'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}分钟前`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}小时前`
  if (diff < 7 * 86_400_000) return `${Math.floor(diff / 86_400_000)}天前`
  return `${d.getMonth() + 1}-${d.getDate()}`
}
</script>

<style lang="scss" scoped>
.dm-page {
  min-height: 100vh;
  background: #F7F8FA;
}
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 32rpx 24rpx 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F0F1F3;
  .back, .action {
    font-size: 40rpx;
    color: #181818;
    width: 56rpx;
  }
  .title { font-size: 32rpx; font-weight: 600; }
}
.loading, .empty {
  text-align: center;
  padding: 120rpx 0;
  color: #999;
  font-size: 28rpx;
  .icon { font-size: 96rpx; display: block; margin-bottom: 16rpx; }
}
.conv-list {
  background: #FFF;
}
.conv-item {
  display: flex;
  align-items: center;
  padding: 24rpx;
  gap: 20rpx;
  border-bottom: 1rpx solid #F7F8FA;
  &:active { background: #F0F1F3; }
  &:last-child { border-bottom: none; }
}
.avatar-wrap {
  position: relative;
  width: 80rpx;
  height: 80rpx;
  border-radius: 50%;
  background: #FB7299;
  color: #FFF;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36rpx;
  flex-shrink: 0;
  .avatar-text { font-size: 36rpx; }
  .badge {
    position: absolute;
    top: -4rpx;
    right: -4rpx;
    min-width: 32rpx;
    height: 32rpx;
    padding: 0 8rpx;
    border-radius: 16rpx;
    background: #FF3B30;
    color: #FFF;
    font-size: 20rpx;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2rpx solid #FFF;
  }
}
.conv-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8rpx;
}
.conv-row1 {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  .peer-name { font-size: 28rpx; color: #181818; font-weight: 500; }
  .time { font-size: 22rpx; color: #999; }
}
.conv-row2 {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12rpx;
  .preview {
    flex: 1;
    font-size: 24rpx;
    color: #999;
  }
  .tag {
    font-size: 20rpx;
    color: #FB7299;
    padding: 2rpx 8rpx;
    border: 1rpx solid #FB7299;
    border-radius: 4rpx;
    flex-shrink: 0;
  }
}
</style>
