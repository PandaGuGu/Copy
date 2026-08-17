<template>
  <view class="fav-page">
    <view class="header">
      <text class="back" @tap="goBack">‹</text>
      <view class="title-wrap">
        <text class="title">我的收藏</text>
        <text class="sub">共 {{ total }} 个内容</text>
      </view>
      <text class="action" @tap="onManage">管理</text>
    </view>

    <view v-if="loading" class="loading"><text>加载中…</text></view>
    <view v-else-if="items.length === 0" class="empty">
      <text class="icon">⭐</text>
      <text class="text">还没有收藏内容</text>
      <text class="tip">在视频页点⭐就能收藏啦</text>
    </view>

    <view v-else class="fav-grid">
      <view v-for="v in items" :key="v.id" class="fav-card" @tap="goVideo(v.id)">
        <view class="cover-wrap">
          <img class="cover" :src="v.cover_url || '/static/placeholder.png'" referrerpolicy="no-referrer" />
          <text class="duration">{{ formatDuration(v.duration) }}</text>
          <view v-if="managing" class="remove-btn" @tap.stop="onRemove(v)">
            <text>✕</text>
          </view>
          <view class="fav-time">
            <text>{{ formatFavTime(v.favorited_at) }}收藏</text>
          </view>
        </view>
        <view class="meta">
          <text class="title text-ellipsis-2">{{ v.title }}</text>
          <view class="stats">
            <text>▶ {{ formatCount(v.play_count) }}</text>
            <text>💬 {{ v.danmaku_count }}</text>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { favoriteApi, type FavoriteVideo } from '@/api/favorite'

const items = ref<FavoriteVideo[]>([])
const total = ref(0)
const loading = ref(false)
const managing = ref(false)

onShow(load)

async function load() {
  loading.value = true
  try {
    const r = await favoriteApi.mine()
    items.value = r.items || []
    total.value = r.total || items.value.length
  } finally {
    loading.value = false
  }
}

async function onRemove(v: FavoriteVideo) {
  try {
    await favoriteApi.toggle(v.id)
    items.value = items.value.filter((x) => x.id !== v.id)
    total.value = Math.max(0, total.value - 1)
    uni.showToast({ title: '已取消收藏', icon: 'success' })
  } catch { /* 已 toast */ }
}

function goVideo(id: number) {
  uni.navigateTo({ url: `/pages/video-detail/index?id=${id}` })
}
function goBack() { uni.navigateBack() }
function onManage() {
  managing.value = !managing.value
  uni.showToast({ title: managing.value ? '点击✕取消收藏' : '管理结束', icon: 'none' })
}

function formatCount(n: number): string {
  if (n >= 10000) return (n / 10000).toFixed(1) + '万'
  return String(n ?? 0)
}
function formatDuration(s: number): string {
  if (!s) return '00:00'
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  return `${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
}
function formatFavTime(t: string): string {
  if (!t) return ''
  const d = new Date(t)
  if (Number.isNaN(d.getTime())) return t
  return `${d.getMonth() + 1}-${d.getDate()}`
}
</script>

<style lang="scss" scoped>
.fav-page {
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

.fav-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  padding: 24rpx;
  gap: 24rpx 0;
}
.fav-card {
  width: 48.7%;
  background: #FFF;
  border-radius: 12rpx;
  overflow: hidden;
  .cover-wrap {
    position: relative;
    width: 100%;
    height: 220rpx;
    background: #E8E8E8;
    .cover {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }
    .duration {
      position: absolute;
      right: 8rpx;
      bottom: 8rpx;
      background: rgba(0,0,0,0.7);
      color: #FFF;
      font-size: 20rpx;
      padding: 2rpx 8rpx;
      border-radius: 4rpx;
    }
    .fav-time {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 0;
      padding: 32rpx 8rpx 6rpx;
      background: linear-gradient(transparent, rgba(0,0,0,0.55));
      color: #FFF;
      font-size: 20rpx;
      text-align: left;
    }
    .remove-btn {
      position: absolute;
      top: 8rpx;
      right: 8rpx;
      width: 44rpx;
      height: 44rpx;
      border-radius: 50%;
      background: rgba(0,0,0,0.65);
      color: #FFF;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 24rpx;
    }
  }
  .meta {
    padding: 12rpx;
    .title {
      font-size: 26rpx;
      color: #181818;
      line-height: 1.4;
      display: -webkit-box;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 2;
      overflow: hidden;
    }
    .stats {
      display: flex;
      gap: 16rpx;
      margin-top: 8rpx;
      font-size: 22rpx;
      color: #999;
    }
  }
}
</style>
