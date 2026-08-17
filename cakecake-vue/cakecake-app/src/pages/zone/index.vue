<template>
  <view class="zone-page">
    <!-- 顶栏 -->
    <view class="page-header safe-area-top">
      <text class="back-icon" @tap="goBack">‹</text>
      <text class="title">{{ zone || '分区' }}</text>
      <text class="spacer" />
    </view>

    <view class="tabs">
      <text class="tab active">综合</text>
      <text class="tab">最新</text>
      <text class="tab">热门</text>
    </view>

    <!-- 视频列表（B站风格双列） -->
    <scroll-view scroll-y class="content" @scrolltolower="loadMore">
      <view v-if="loading && videos.length === 0" class="loading"><text>加载中…</text></view>
      <view v-else class="video-grid">
        <view
          v-for="v in videos"
          :key="v.id"
          class="video-card"
          @tap="goVideoDetail(v.id)"
        >
          <view class="video-cover">
            <img :src="v.cover_url" :alt="v.title" class="cover-img" referrerpolicy="no-referrer" />
            <text class="duration">{{ formatDuration(v.duration) }}</text>
          </view>
          <view class="meta">
            <text class="title text-ellipsis-2">{{ v.title }}</text>
            <text class="up">{{ v.uploader }}</text>
            <text class="stats">▶ {{ formatCount(v.play_count) }} 💬 {{ v.comment_count }}</text>
          </view>
        </view>
      </view>
      <view v-if="videos.length === 0 && !loading" class="empty">
        <text>该分区暂无内容</text>
      </view>
      <view v-if="!hasMore && videos.length > 0" class="end"><text>— 已经到底了 —</text></view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onLoad, onPullDownRefresh } from '@dcloudio/uni-app'
import { categoryApi } from '@/api/category'
import type { Video } from '@/utils/types'

const zone = ref('')
const videos = ref<Video[]>([])
const cursor = ref<string | null>(null)
const hasMore = ref(true)
const loading = ref(false)

onLoad((q) => {
  zone.value = decodeURIComponent((q?.zone as string) || '')
  load(true)
})

onPullDownRefresh(async () => {
  await load(true)
  uni.stopPullDownRefresh()
})

async function load(reset = false) {
  if (loading.value) return
  if (reset) { cursor.value = null; hasMore.value = true }
  loading.value = true
  try {
    const resp = await categoryApi.zoneVideos(zone.value, cursor.value ?? undefined, 20)
    if (reset) videos.value = resp.items
    else videos.value.push(...resp.items)
    cursor.value = resp.next_cursor
    hasMore.value = !!resp.next_cursor && resp.items.length > 0
  } catch { /* 已 toast */ }
  finally { loading.value = false }
}

function loadMore() {
  if (hasMore.value && !loading.value) load(false)
}

function goBack() { uni.navigateBack() }
function goVideoDetail(id: number) { uni.navigateTo({ url: `/pages/video-detail/index?id=${id}` }) }

function formatDuration(s: number): string {
  if (!s) return ''
  const m = Math.floor(s / 60)
  const sec = (s % 60).toString().padStart(2, '0')
  return `${m}:${sec}`
}
function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}万`
  return n.toString()
}
</script>

<style lang="scss" scoped>
.zone-page {
  min-height: 100vh;
  background: #FFF;
}

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

.tabs {
  display: flex;
  gap: 48rpx;
  padding: 20rpx 32rpx;
  border-bottom: 1rpx solid #F1F1F1;

  .tab {
    font-size: 28rpx;
    color: #666;

    &.active { color: #FB7299; font-weight: 500; border-bottom: 4rpx solid #FB7299; padding-bottom: 8rpx; }
  }
}

.content {
  height: calc(100vh - 220rpx);
}

.video-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  padding: 16rpx 24rpx;
  gap: 16rpx 0;
}

.video-card {
  width: 48.7%;
  background: #FFF;
  border-radius: 12rpx;
  overflow: hidden;

  &:active { transform: scale(0.98); }
}

.video-cover {
  position: relative;
  width: 100%;
  height: 220rpx;
  background: #E8E8E8;
  overflow: hidden;

  .cover-img { width: 100%; height: 100%; object-fit: cover; display: block; }
  .duration {
    position: absolute;
    right: 8rpx;
    bottom: 8rpx;
    background: rgba(0,0,0,0.7);
    color: #FFF;
    font-size: 20rpx;
    padding: 2rpx 8rpx;
    border-radius: 4rpx;
    z-index: 2;
  }
}

.meta {
  padding: 12rpx 14rpx;

  .title { font-size: 26rpx; color: #181818; line-height: 1.4; }
  .up { display: block; margin-top: 6rpx; font-size: 22rpx; color: #999; }
  .stats { display: block; margin-top: 6rpx; font-size: 22rpx; color: #999; }
}

.loading { text-align: center; padding: 80rpx 0; color: #999; }
.empty { text-align: center; padding: 80rpx 0; color: #999; }
.end { text-align: center; padding: 40rpx 0; color: #999; font-size: 24rpx; }
</style>