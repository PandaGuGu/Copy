<template>
  <view class="my-videos-page">
    <view class="page-header safe-area-top">
      <text class="back-icon" @tap="goBack">‹</text>
      <text class="title">我的投稿</text>
      <text class="upload-link" @tap="goPublish">发布 ›</text>
    </view>

    <!-- Tabs：已发布/草稿/审核中/已删除 -->
    <view class="tabs">
      <view
        v-for="t in tabs"
        :key="t.key"
        class="tab"
        :class="{ active: activeTab === t.key }"
        @tap="switchTab(t.key)"
      >
        <text>{{ t.label }}</text>
        <text v-if="counts[t.key] != null" class="tab-count">{{ counts[t.key] }}</text>
      </view>
    </view>

    <scroll-view scroll-y class="content">
      <view v-if="loading && videos.length === 0" class="loading"><text>加载中…</text></view>
      <view v-else-if="videos.length === 0" class="empty">
        <text class="icon">📦</text>
        <text>暂无{{ currentLabel }}的视频</text>
        <view class="empty-btn" @tap="goPublish">
          <text>去发布</text>
        </view>
      </view>
      <view v-else class="list">
        <view v-for="v in videos" :key="v.id" class="video-row" @tap="goVideoDetail(v.id)">
          <view class="cover-wrap">
            <img :src="v.cover_url || '/static/placeholder.png'" class="cover-img" referrerpolicy="no-referrer" />
            <text class="duration">{{ formatDuration(v.duration) }}</text>
          </view>
          <view class="meta">
            <text class="title text-ellipsis-2">{{ v.title }}</text>
            <view class="stats">
              <text>▶ {{ formatCount(v.play_count) }}</text>
              <text>💬 {{ formatCount(v.comment_count) }}</text>
              <text>👍 {{ formatCount(v.like_count) }}</text>
              <text>🪙 {{ formatCount(v.coin_count) }}</text>
            </view>
            <view class="sub">
              <text class="time">{{ formatTime(v.created_at) }}</text>
              <text v-if="v.status !== 'published'" class="status" :class="v.status">{{ statusLabel(v.status) }}</text>
            </view>
          </view>
        </view>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { myApi, type MyVideoItem } from '@/api/my'

type TabKey = 'passed' | 'draft' | 'processing' | 'rejected'

const tabs: { key: TabKey; label: string }[] = [
  { key: 'passed',    label: '已发布' },
  { key: 'processing', label: '审核中' },
  { key: 'draft',     label: '草稿' },
  { key: 'rejected',  label: '未通过' }
]

const activeTab = ref<TabKey>('passed')
const allVideos = ref<MyVideoItem[]>([])
const counts = ref<Record<TabKey, number>>({ passed: 0, draft: 0, processing: 0, rejected: 0 })
const loading = ref(false)

const currentLabel = computed(() => tabs.find((t) => t.key === activeTab.value)?.label ?? '')

/** counts tab key（passed/draft/processing/rejected）→ 真实 item.status 值映射 */
const tabToItemStatus: Record<TabKey, string> = {
  passed: 'published',
  processing: 'processing',
  draft: 'draft',
  rejected: 'rejected'
}

const videos = computed(() => allVideos.value.filter((v) => v.status === tabToItemStatus[activeTab.value]))

onMounted(load)

async function load() {
  if (loading.value) return
  loading.value = true
  try {
    const resp = await myApi.videos()
    counts.value = { ...resp.counts }
    allVideos.value = resp.items || []
  } finally {
    loading.value = false
  }
}

function switchTab(key: TabKey) {
  activeTab.value = key
}

function statusLabel(s: string): string {
  return { draft: '草稿', processing: '审核中', rejected: '未通过', failed: '失败', pending_review: '待审核', published: '已发布' }[s] || s
}
function goBack()        { uni.navigateBack() }
function goVideoDetail(id: number) { uni.navigateTo({ url: `/pages/video-detail/index?id=${id}` }) }
function goPublish()     { uni.navigateTo({ url: '/pages/publish/index' }) }

function formatDuration(s: number): string {
  if (!s) return ''
  const m = Math.floor(s / 60)
  const sec = (s % 60).toString().padStart(2, '0')
  return `${m}:${sec}`
}
function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}万`
  return String(n)
}
function formatTime(s: string): string {
  if (!s) return ''
  return s.slice(0, 10)
}
</script>

<style lang="scss" scoped>
.my-videos-page { min-height: 100vh; background: #FFF; }

.page-header {
  display: flex;
  align-items: center;
  padding: 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F1F1F1;

  .back-icon { font-size: 48rpx; width: 60rpx; }
  .title { flex: 1; text-align: center; font-size: 32rpx; font-weight: 500; }
  .upload-link { font-size: 26rpx; color: #FB7299; }
}

.tabs {
  display: flex;
  background: #FFF;
  border-bottom: 1rpx solid #F1F1F1;

  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4rpx;
    padding: 20rpx 0;
    color: #666;

    &.active {
      color: #FB7299;
      border-bottom: 4rpx solid #FB7299;
      font-weight: 500;
    }

    .tab-count {
      font-size: 20rpx;
      color: #999;
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

.empty-btn {
  display: inline-block;
  margin-top: 24rpx;
  background: linear-gradient(90deg, #FB7299, #FF9DB5);
  color: #FFF;
  padding: 16rpx 48rpx;
  border-radius: 32rpx;
  font-size: 26rpx;
}

.list { padding: 16rpx 0; }

.video-row {
  display: flex;
  gap: 16rpx;
  padding: 16rpx 24rpx;
  border-bottom: 1rpx solid #F8F8F8;

  &:active { background: #F8F8F8; }

  .cover-wrap {
    position: relative;
    width: 240rpx;
    height: 150rpx;
    border-radius: 8rpx;
    overflow: hidden;
    flex-shrink: 0;
    background: #E8E8E8;

    .cover-img { width: 100%; height: 100%; object-fit: cover; display: block; }
    .duration {
      position: absolute;
      right: 6rpx; bottom: 6rpx;
      background: rgba(0,0,0,0.7);
      color: #FFF;
      font-size: 18rpx;
      padding: 2rpx 6rpx;
      border-radius: 4rpx;
    }
  }

  .meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: space-between;

    .title { font-size: 26rpx; color: #181818; line-height: 1.4; }

    .stats {
      display: flex;
      gap: 16rpx;
      font-size: 22rpx;
      color: #999;
    }

    .sub {
      display: flex;
      gap: 12rpx;
      align-items: center;
      font-size: 22rpx;
      color: #999;

      .status {
        font-size: 20rpx;
        padding: 2rpx 8rpx;
        border-radius: 4rpx;
        &.draft { background: #F4F4F4; color: #999; }
        &.processing { background: #E6F1FB; color: #2A7DD1; }
        &.rejected { background: #FCEBEB; color: #E24B4A; }
        &.failed { background: #FCEBEB; color: #E24B4A; }
      }
    }
  }
}
</style>