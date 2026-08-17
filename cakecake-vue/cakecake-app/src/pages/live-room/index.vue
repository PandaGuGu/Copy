<template>
  <view class="live-page">
    <view class="header">
      <text class="back" @tap="goBack">‹</text>
      <text class="title">直播</text>
      <text class="action" @tap="onShare">↗</text>
    </view>

    <view v-if="loading" class="loading"><text>加载中…</text></view>
    <view v-else-if="room" class="room">
      <!-- 顶部封面（占位播放区，B站风格） -->
      <view class="cover-wrap">
        <img class="cover" :src="room.cover_url || '/static/demo/live_default.jpg'" referrerpolicy="no-referrer" />
        <view class="live-tag" :class="room.status">
          <view class="dot" />
          <text>{{ room.status === 'live' ? 'LIVE' : room.status === 'ended' ? '已结束' : room.status === 'paused' ? '暂停' : '已封禁' }}</text>
        </view>
        <view class="viewer-badge">
          <text>👁 {{ formatCount(room.viewer_count) }}</text>
        </view>
        <view class="live-time">
          <text>{{ formatStartTime(room.started_at) }}</text>
        </view>
      </view>

      <!-- 主播卡片 -->
      <view class="host-card">
        <view class="avatar-wrap">
          <text class="avatar-text">{{ (room.host_name || 'U').slice(0, 1).toUpperCase() }}</text>
        </view>
        <view class="host-meta">
          <view class="host-name-row">
            <text class="host-name">{{ room.host_name }}</text>
            <text v-if="room.status === 'live'" class="host-tag">主播</text>
          </view>
          <text class="host-id">房间 #{{ room.id }} · stream: {{ room.stream_key.slice(0, 8) }}…</text>
        </view>
        <view class="follow-btn">
          <text>+ 关注</text>
        </view>
      </view>

      <!-- 直播信息 -->
      <view class="info-card">
        <text class="info-title">{{ room.title || '无标题直播' }}</text>
        <view class="info-meta">
          <text>开播 {{ formatStartTime(room.started_at) }}</text>
          <text>·</text>
          <text>{{ room.status === 'live' ? '进行中' : (room.ended_at ? '已结束 ' + formatStartTime(room.ended_at) : '已结束') }}</text>
        </view>
        <view class="info-stats">
          <view class="stat">
            <text class="num">{{ formatCount(room.viewer_count) }}</text>
            <text class="label">在线人数</text>
          </view>
          <view class="stat">
            <text class="num">直播</text>
            <text class="label">分类</text>
          </view>
          <view class="stat">
            <text class="num">Lv 1</text>
            <text class="label">主播等级</text>
          </view>
        </view>
      </view>

      <!-- 互动入口 -->
      <view class="action-grid">
        <view class="act" @tap="onAction('弹幕')">
          <view class="act-icon">💬</view>
          <text>弹幕</text>
        </view>
        <view class="act" @tap="onAction('礼物')">
          <view class="act-icon">🎁</view>
          <text>礼物</text>
        </view>
        <view class="act" @tap="onAction('分享')">
          <view class="act-icon">↗</view>
          <text>分享</text>
        </view>
        <view class="act" @tap="onAction('设置')">
          <view class="act-icon">⚙</view>
          <text>设置</text>
        </view>
      </view>

      <view class="empty-area">
        <text class="empty-text">— 暂时没有更多直播 —</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onLoad as onPageLoad } from '@dcloudio/uni-app'
import { liveApi, type LiveRoom } from '@/api/live-detail'

const room = ref<LiveRoom | null>(null)
const loading = ref(false)

onPageLoad((q: any) => {
  const id = Number(q?.id) || 0
  if (id) loadRoom(id)
})

async function loadRoom(id: number) {
  loading.value = true
  try {
    room.value = await liveApi.detail(id)
  } catch { /* 已 toast */ }
  finally { loading.value = false }
}

function goBack()   { uni.navigateBack() }
function onShare()  { uni.showToast({ title: '分享功能待开发', icon: 'none' }) }
function onAction(name: string) {
  uni.showToast({ title: `${name} - 待开发`, icon: 'none' })
}

function formatCount(n: number): string {
  if (n >= 10000) return (n / 10000).toFixed(1) + '万'
  return String(n ?? 0)
}

function formatStartTime(t?: string): string {
  if (!t) return '—'
  const d = new Date(t)
  if (Number.isNaN(d.getTime())) return t
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  if (diffMs < 60_000) return '刚刚开播'
  if (diffMs < 3_600_000) return `${Math.floor(diffMs / 60_000)}分钟前开播`
  if (diffMs < 86_400_000) return `${Math.floor(diffMs / 3_600_000)}小时前开播`
  return `${d.getMonth() + 1}-${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}
</script>

<style lang="scss" scoped>
.live-page {
  min-height: 100vh;
  background: #FFF;
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
  .action { text-align: right; }
  .title { font-size: 32rpx; font-weight: 600; }
}
.loading {
  text-align: center;
  padding: 120rpx 0;
  color: #999;
  font-size: 28rpx;
}

/* 顶部封面 */
.cover-wrap {
  position: relative;
  width: 100%;
  height: 480rpx;
  background: #000;
  overflow: hidden;
  .cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .live-tag {
    position: absolute;
    top: 24rpx;
    left: 24rpx;
    display: flex;
    align-items: center;
    gap: 6rpx;
    padding: 4rpx 12rpx;
    border-radius: 4rpx;
    font-size: 20rpx;
    color: #FFF;
    font-weight: 700;
    &.live { background: #FF3B30; }
    &.ended { background: rgba(0,0,0,0.6); }
    &.paused { background: #FF9500; }
    &.banned { background: #8E8E93; }
    .dot {
      width: 12rpx;
      height: 12rpx;
      border-radius: 50%;
      background: #FFF;
      animation: pulse 1.5s infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.4; }
    }
  }
  .viewer-badge {
    position: absolute;
    top: 24rpx;
    right: 24rpx;
    padding: 4rpx 12rpx;
    border-radius: 4rpx;
    background: rgba(0,0,0,0.6);
    color: #FFF;
    font-size: 22rpx;
  }
  .live-time {
    position: absolute;
    bottom: 24rpx;
    left: 24rpx;
    color: #FFF;
    font-size: 24rpx;
    background: rgba(0,0,0,0.5);
    padding: 4rpx 12rpx;
    border-radius: 4rpx;
  }
}

/* 主播卡片 */
.host-card {
  display: flex;
  align-items: center;
  gap: 20rpx;
  padding: 24rpx;
  border-bottom: 1rpx solid #F7F8FA;
}
.avatar-wrap {
  width: 88rpx;
  height: 88rpx;
  border-radius: 50%;
  background: linear-gradient(135deg, #FB7299, #FF9CB0);
  color: #FFF;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36rpx;
  flex-shrink: 0;
}
.host-meta {
  flex: 1;
  min-width: 0;
  .host-name-row {
    display: flex;
    align-items: center;
    gap: 8rpx;
    .host-name { font-size: 30rpx; color: #181818; font-weight: 600; }
    .host-tag {
      font-size: 18rpx;
      color: #FB7299;
      border: 1rpx solid #FB7299;
      padding: 1rpx 6rpx;
      border-radius: 4rpx;
    }
  }
  .host-id { font-size: 20rpx; color: #999; margin-top: 4rpx; }
}
.follow-btn {
  padding: 12rpx 24rpx;
  border-radius: 32rpx;
  background: #FB7299;
  color: #FFF;
  font-size: 24rpx;
  flex-shrink: 0;
}

/* 直播信息 */
.info-card {
  padding: 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F7F8FA;
  .info-title {
    font-size: 32rpx;
    color: #181818;
    font-weight: 600;
    line-height: 1.4;
  }
  .info-meta {
    display: flex;
    gap: 8rpx;
    margin-top: 12rpx;
    font-size: 22rpx;
    color: #999;
  }
  .info-stats {
    display: flex;
    margin-top: 24rpx;
    gap: 0;
    .stat {
      flex: 1;
      text-align: center;
      .num {
        display: block;
        font-size: 28rpx;
        color: #181818;
        font-weight: 600;
      }
      .label {
        display: block;
        font-size: 22rpx;
        color: #999;
        margin-top: 4rpx;
      }
    }
  }
}

/* 互动入口 */
.action-grid {
  display: flex;
  padding: 24rpx 0;
  border-bottom: 1rpx solid #F7F8FA;
  .act {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8rpx;
    color: #181818;
    font-size: 22rpx;
    .act-icon {
      width: 80rpx;
      height: 80rpx;
      border-radius: 50%;
      background: #FBEAF0;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 36rpx;
    }
  }
}

.empty-area {
  text-align: center;
  padding: 80rpx 0;
  .empty-text {
    font-size: 24rpx;
    color: #C0C2C5;
  }
}
</style>
