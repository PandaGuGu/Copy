<template>
  <view class="msg-page">
    <view class="page-header safe-area-top">
      <text class="back-icon" @tap="goBack">‹</text>
      <text class="title">消息</text>
      <text class="spacer" />
    </view>

    <!-- 未登录提示 -->
    <view v-if="!isLoggedIn" class="login-tip">
      <text>登录后查看消息</text>
      <view class="login-btn" @tap="goLogin"><text>立即登录</text></view>
    </view>

    <template v-else>
      <!-- 分类 Tabs（带未读小红点） -->
      <view class="tabs">
        <view
          v-for="t in tabs"
          :key="t.key"
          class="tab"
          :class="{ active: activeTab === t.key }"
          @tap="switchTab(t.key)"
        >
          <text>{{ t.label }}</text>
          <view v-if="unread[t.key] > 0" class="red-dot">{{ unread[t.key] > 99 ? '99+' : unread[t.key] }}</view>
        </view>
      </view>

      <!-- 列表 -->
      <scroll-view scroll-y class="content">
        <view v-if="loading && items.length === 0" class="loading"><text>加载中…</text></view>
        <view v-else-if="items.length === 0" class="empty">
          <text class="icon">🔔</text>
          <text>暂无{{ currentLabel }}</text>
        </view>
        <view v-else class="list">
          <view v-for="n in items" :key="n.id" class="msg-item">
            <view class="msg-avatar">
              <text>{{ n.sender_username?.slice(0, 1) || '?' }}</text>
            </view>
            <view class="msg-body">
              <view class="msg-row">
                <text class="sender">{{ n.sender_username || '系统' }}</text>
                <text class="time">{{ formatTime(n.created_at) }}</text>
              </view>
              <text class="content" :class="{ unread: !n.is_read }">{{ displayContent(n) }}</text>
              <view v-if="n.total_likes > 0" class="like-row">👍 {{ n.total_likes }} 人赞过</view>
            </view>
            <view v-if="!n.is_read" class="unread-dot" />
          </view>
        </view>
      </scroll-view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { notificationApi, type NotificationItem, type UnreadSummary } from '@/api/notification'
import { useUserStore } from '@/store/user'

const userStore = useUserStore()
const isLoggedIn = computed(() => userStore.isLoggedIn)

const tabs: { key: string; label: string }[] = [
  { key: 'reply_received',  label: '回复' },
  { key: 'like_aggregation', label: '赞' },
  { key: 'system_notice',  label: '系统' },
  { key: 'my_message',     label: '私信' },
  { key: 'at_me',          label: '@我' }
]

const activeTab = ref('reply_received')
const items = ref<NotificationItem[]>([])
const unread = ref<UnreadSummary>({ at_me: 0, like_aggregation: 0, my_message: 0, reply_received: 0, system_notice: 0 })
const loading = ref(false)

const currentLabel = computed(() => tabs.find((t) => t.key === activeTab.value)?.label ?? '')

onShow(async () => {
  if (isLoggedIn.value) {
    await Promise.all([loadUnread(), load()])
  }
})

async function loadUnread() {
  try {
    unread.value = await notificationApi.unreadSummary()
  } catch { /* ignore */ }
}

async function load() {
  if (loading.value) return
  loading.value = true
  try {
    const resp = await notificationApi.list(activeTab.value)
    items.value = resp.items || []
  } finally {
    loading.value = false
  }
}

function switchTab(key: string) {
  activeTab.value = key
  load()
}

function displayContent(n: NotificationItem): string {
  if (n.type === 'system_notice') {
    try {
      const p = JSON.parse(n.message || '{}')
      return p.title || '系统通知'
    } catch { return n.message || '系统通知' }
  }
  if (n.comment_preview) return n.comment_preview
  return n.message || n.type
}

function formatTime(s: string): string {
  if (!s) return ''
  const d = new Date(s)
  const diff = (Date.now() - d.getTime()) / 1000
  if (diff < 60) return '刚刚'
  if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`
  if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`
  return `${d.getMonth() + 1}-${d.getDate()}`
}

function goBack()  { uni.navigateBack() }
function goLogin() { uni.navigateTo({ url: '/pages/login/index' }) }
</script>

<style lang="scss" scoped>
.msg-page { min-height: 100vh; background: #FFF; }

.page-header {
  display: flex;
  align-items: center;
  padding: 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F1F1F1;

  .back-icon { font-size: 48rpx; width: 60rpx; }
  .title { flex: 1; text-align: center; font-size: 32rpx; font-weight: 500; }
  .spacer { width: 60rpx; }
}

.login-tip {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24rpx;
  padding: 160rpx 0;
  color: #999;

  .login-btn {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 16rpx 56rpx;
    border-radius: 32rpx;
    font-size: 26rpx;
  }
}

.tabs {
  display: flex;
  border-bottom: 1rpx solid #F1F1F1;

  .tab {
    flex: 1;
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6rpx;
    padding: 20rpx 0;
    font-size: 26rpx;
    color: #666;

    &.active {
      color: #FB7299;
      font-weight: 500;
      border-bottom: 4rpx solid #FB7299;
    }

    .red-dot {
      min-width: 28rpx;
      height: 28rpx;
      padding: 0 6rpx;
      background: #F56C6C;
      color: #FFF;
      border-radius: 14rpx;
      font-size: 18rpx;
      line-height: 28rpx;
      text-align: center;
      box-sizing: border-box;
    }
  }
}

.content { height: calc(100vh - 200rpx); }

.loading, .empty {
  text-align: center;
  padding: 100rpx 0;
  color: #999;
  .icon { font-size: 80rpx; display: block; margin-bottom: 16rpx; }
}

.list { padding: 8rpx 0; }

.msg-item {
  display: flex;
  gap: 16rpx;
  padding: 24rpx;
  border-bottom: 1rpx solid #F8F8F8;

  &:active { background: #F8F8F8; }
}

.msg-avatar {
  width: 72rpx;
  height: 72rpx;
  border-radius: 50%;
  background: linear-gradient(135deg, #FFE4EC, #FFB6C1);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 30rpx;
  font-weight: 600;
  color: #FB7299;
  flex-shrink: 0;
}

.msg-body {
  flex: 1;

  .msg-row {
    display: flex;
    align-items: center;
    gap: 12rpx;
    .sender { font-size: 26rpx; color: #181818; font-weight: 500; }
    .time { font-size: 20rpx; color: #C0C4CC; }
  }
  .content {
    display: block;
    margin-top: 6rpx;
    font-size: 24rpx;
    color: #999;
    line-height: 1.5;

    &.unread { color: #333; }
  }
  .like-row {
    margin-top: 6rpx;
    font-size: 20rpx;
    color: #999;
  }
}

.unread-dot {
  width: 14rpx;
  height: 14rpx;
  border-radius: 50%;
  background: #F56C6C;
  margin-top: 8rpx;
  flex-shrink: 0;
}
</style>