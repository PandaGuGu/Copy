<template>
  <view class="hist-page">
    <view class="header">
      <text class="back" @tap="goBack">‹</text>
      <view class="title-wrap">
        <text class="title">观看历史</text>
        <text class="sub">共 {{ total }} 条记录</text>
      </view>
      <text class="action" @tap="onClear">清空</text>
    </view>

    <view v-if="loading" class="loading"><text>加载中…</text></view>
    <view v-else-if="items.length === 0" class="empty">
      <text class="icon">🕐</text>
      <text class="text">暂无观看历史</text>
      <text class="tip">看过的视频会出现在这里</text>
    </view>

    <view v-else class="hist-list">
      <view v-for="h in items" :key="h.video_id" class="hist-item" @tap="goVideo(h)">
        <view class="cover-wrap">
          <img class="cover" :src="h.cover_url || '/static/placeholder.png'" referrerpolicy="no-referrer" />
          <view class="progress-bar">
            <view class="progress-fill" :style="{ width: progressPct(h) + '%' }" />
          </view>
          <text class="progress-text">{{ formatProgress(h) }}</text>
        </view>
        <view class="hist-body">
          <text class="title text-ellipsis-2">{{ h.title }}</text>
          <view class="meta">
            <text>▶ {{ h.uploader_name || '匿名' }}</text>
            <text>·</text>
            <text>{{ formatCount(h.progress_sec * 0) + formatRelative(h.viewed_at) }}</text>
          </view>
          <text class="time">{{ formatRelative(h.viewed_at) }}</text>
        </view>
        <view class="del-btn" @tap.stop="onRemove(h)">
          <text>✕</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { historyApi, type HistoryItem } from '@/api/history'

const items = ref<HistoryItem[]>([])
const total = ref(0)
const loading = ref(false)

onShow(load)

async function load() {
  loading.value = true
  try {
    const r = await historyApi.list()
    items.value = r.items || []
    total.value = r.total || items.value.length
  } finally {
    loading.value = false
  }
}

async function onRemove(h: HistoryItem) {
  try {
    await historyApi.remove(h.video_id)
    items.value = items.value.filter((x) => x.video_id !== h.video_id)
    total.value = Math.max(0, total.value - 1)
    uni.showToast({ title: '已删除', icon: 'none' })
  } catch { /* 已 toast */ }
}

function onClear() {
  uni.showModal({
    title: '清空观看历史',
    content: '确定要清空全部观看历史吗？',
    success: async (res) => {
      if (!res.confirm) return
      try {
        await historyApi.clear()
        items.value = []
        total.value = 0
        uni.showToast({ title: '已清空', icon: 'success' })
      } catch { /* 已 toast */ }
    }
  })
}

function goVideo(h: HistoryItem) {
  if (h.media_type === 'video' && h.video_id) {
    uni.navigateTo({ url: `/pages/video-detail/index?id=${h.video_id}` })
  } else {
    uni.showToast({ title: `${h.media_type} 类型暂不支持播放`, icon: 'none' })
  }
}

function progressPct(h: HistoryItem): number {
  if (!h.duration_sec) return 0
  const p = Math.min(100, Math.round((h.progress_sec / h.duration_sec) * 100))
  return p
}

function formatProgress(h: HistoryItem): string {
  const p = progressPct(h)
  return p > 95 ? '已看完' : `看到 ${p}%`
}

function formatRelative(t: string): string {
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

function formatCount(_n: number): string { return '' }
</script>

<style lang="scss" scoped>
.hist-page {
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
  .back { font-size: 40rpx; color: #181818; width: 56rpx; }
  .action { font-size: 26rpx; color: #FB7299; width: 80rpx; text-align: right; }
  .title-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rpx;
    .title { font-size: 32rpx; font-weight: 600; color: #181818; }
    .sub { font-size: 20rpx; color: #999; }
  }
}
.loading, .empty {
  text-align: center;
  padding: 120rpx 0;
  color: #999;
  .icon { font-size: 96rpx; display: block; margin-bottom: 16rpx; }
  .text { font-size: 28rpx; display: block; }
  .tip { font-size: 24rpx; color: #C0C2C5; margin-top: 8rpx; display: block; }
}

.hist-list {
  padding: 8rpx 24rpx;
}
.hist-item {
  display: flex;
  align-items: center;
  gap: 20rpx;
  padding: 20rpx 0;
  border-bottom: 1rpx solid #F7F8FA;
  .cover-wrap {
    position: relative;
    width: 200rpx;
    height: 120rpx;
    border-radius: 8rpx;
    overflow: hidden;
    background: #E8E8E8;
    flex-shrink: 0;
    .cover {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }
    .progress-bar {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 0;
      height: 6rpx;
      background: rgba(255,255,255,0.4);
      .progress-fill {
        height: 100%;
        background: #FB7299;
      }
    }
    .progress-text {
      position: absolute;
      right: 6rpx;
      bottom: 8rpx;
      font-size: 18rpx;
      color: #FFF;
      text-shadow: 0 1rpx 2rpx rgba(0,0,0,0.6);
    }
  }
  .hist-body {
    flex: 1;
    min-width: 0;
    .title {
      font-size: 28rpx;
      color: #181818;
      line-height: 1.4;
      display: -webkit-box;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 2;
      overflow: hidden;
    }
    .meta {
      display: flex;
      gap: 8rpx;
      align-items: center;
      margin-top: 8rpx;
      font-size: 22rpx;
      color: #999;
    }
    .time {
      display: block;
      font-size: 22rpx;
      color: #999;
      margin-top: 4rpx;
    }
  }
  .del-btn {
    width: 48rpx;
    height: 48rpx;
    border-radius: 50%;
    background: #F0F1F3;
    color: #999;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24rpx;
    flex-shrink: 0;
  }
}
</style>
