<template>
  <view class="search-page">
    <!-- 顶部搜索栏 -->
    <view class="search-header safe-area-top">
      <view class="search-bar">
        <text class="search-icon">🔍</text>
        <input
          v-model="keyword"
          class="search-input"
          placeholder="搜索视频、UP主、用户"
          confirm-type="search"
          @confirm="onSearch"
          @input="onInputChange"
        />
        <text v-if="keyword" class="clear-icon" @tap="keyword = ''">×</text>
      </view>
      <text class="cancel-btn" @tap="goBack">取消</text>
    </view>

    <!-- 未搜索：热搜榜 + 历史 -->
    <scroll-view v-if="!searched" scroll-y class="content">
      <!-- 热搜榜 -->
      <view class="section">
        <view class="section-head">
          <text class="title">🔥 热搜榜</text>
        </view>
        <view v-if="hotKeywords.length" class="hot-list">
          <view
            v-for="h in hotKeywords.slice(0, 10)"
            :key="h.rank"
            class="hot-item"
            @tap="quickSearch(h.title)"
          >
            <text class="rank" :class="{ top: h.rank <= 3 }">{{ h.rank }}</text>
            <text class="word">{{ h.title }}</text>
            <text v-if="h.badge" class="badge">{{ h.badge }}</text>
          </view>
        </view>
        <view v-else class="empty-tip"><text>暂无热搜</text></view>
      </view>

      <!-- 搜索历史 -->
      <view class="section">
        <view class="section-head">
          <text class="title">🕘 搜索历史</text>
          <text v-if="history.length" class="clear-all" @tap="clearHistory">清空</text>
        </view>
        <view v-if="history.length" class="history">
          <view v-for="(h, i) in history" :key="i" class="history-item" @tap="quickSearch(h)">
            <text>{{ h }}</text>
            <text class="del" @tap.stop="removeHistory(i)">×</text>
          </view>
        </view>
        <view v-else class="empty-tip"><text>暂无搜索历史</text></view>
      </view>
    </scroll-view>

    <!-- 搜索中 -->
    <view v-else-if="loading" class="loading"><text>搜索中…</text></view>

    <!-- 搜索结果 -->
    <scroll-view v-else scroll-y class="content">
      <view v-if="results.length === 0" class="empty-result">
        <text class="icon">🔍</text>
        <text>没找到 "{{ lastKeyword }}" 相关结果</text>
      </view>
      <view v-else class="results">
        <view
          v-for="r in results"
          :key="r.aid"
          class="result-card"
          @tap="goVideo(r)"
        >
          <view class="cover-wrap">
            <img :src="r.pic" :alt="r.title" class="cover-img" referrerpolicy="no-referrer" />
            <text class="duration">{{ r.duration }}</text>
          </view>
          <view class="meta">
            <text class="title text-ellipsis-2">{{ r.title }}</text>
            <text class="stats">▶ {{ formatCount(r.play) }}  ·  💬 {{ formatCount(r.video_review || 0) }}</text>
            <text class="up">{{ r.author || '匿名' }}  ·  {{ formatPubdate(r.pubdate) }}</text>
          </view>
        </view>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { searchApi, type HotSearchItem, type SearchVideo } from '@/api/search'
import { useUserStore } from '@/store/user'

const keyword = ref('')
const searched = ref(false)
const lastKeyword = ref('')
const loading = ref(false)
const results = ref<SearchVideo[]>([])

const hotKeywords = ref<HotSearchItem[]>([])
const history = ref<string[]>([])
const userStore = useUserStore()

let debounceTimer: ReturnType<typeof setTimeout> | null = null

onShow(async () => {
  await loadHot()
  await loadHistory()
})

async function loadHot() {
  try {
    const resp = await searchApi.hotSearch()
    hotKeywords.value = resp.items || []
  } catch { /* 已 toast */ }
}

async function loadHistory() {
  try {
    const resp = await searchApi.history()
    history.value = resp.keywords || []
  } catch {
    // 未登录或失败：用本地缓存兜底
    history.value = (uni.getStorageSync('search_history') as string[]) || []
  }
}

function onInputChange() {
  // 输入即搜（防抖 400ms）
  if (debounceTimer) clearTimeout(debounceTimer)
  if (!keyword.value.trim()) {
    searched.value = false
    results.value = []
    return
  }
  debounceTimer = setTimeout(() => onSearch(), 400)
}

async function onSearch() {
  const kw = keyword.value.trim()
  if (!kw) return
  lastKeyword.value = kw
  searched.value = true
  loading.value = true
  // 仅登录用户保存历史（避免未登录触发 401 跳登录页）
  if (userStore.isLoggedIn) {
    try { await searchApi.saveHistory(kw) } catch { /* ignore */ }
  }
  try {
    const resp = await searchApi.search(kw, 1, 20)
    results.value = resp.result?.video || []
  } finally {
    loading.value = false
  }
  if (userStore.isLoggedIn) await loadHistory()
}

function quickSearch(kw: string) {
  keyword.value = kw
  onSearch()
}

function removeHistory(i: number) {
  history.value.splice(i, 1)
  uni.setStorageSync('search_history', history.value)
}

function clearHistory() {
  history.value = []
  uni.removeStorageSync('search_history')
}

function goBack() { uni.navigateBack() }
function goVideo(r: SearchVideo) {
  // aid → 跳详情页（项目用数字 id，aid 可能相同）
  uni.navigateTo({ url: `/pages/video-detail/index?id=${r.aid}` })
}

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}万`
  return String(n)
}
function formatPubdate(ts: number): string {
  if (!ts) return ''
  const d = new Date(ts * 1000)
  return `${d.getFullYear()}-${(d.getMonth() + 1).toString().padStart(2, '0')}-${d.getDate().toString().padStart(2, '0')}`
}
</script>

<style lang="scss" scoped>
.search-page { min-height: 100vh; background: #FFF; }

.search-header {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 16rpx 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F1F1F1;
}

.search-bar {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8rpx;
  height: 64rpx;
  background: #F4F4F4;
  border-radius: 32rpx;
  padding: 0 20rpx;

  .search-icon { color: #999; font-size: 28rpx; }
  .search-input {
    flex: 1;
    font-size: 26rpx;
    color: #181818;
  }
  .clear-icon {
    color: #999;
    font-size: 32rpx;
    padding: 0 8rpx;
  }
}

.cancel-btn {
  font-size: 28rpx;
  color: #FB7299;
}

.content { height: calc(100vh - 100rpx); }

.section { padding: 24rpx 16rpx 8rpx; }

.section-head {
  display: flex;
  align-items: center;
  padding: 8rpx 8rpx 16rpx;
  .title { flex: 1; font-size: 28rpx; font-weight: 600; color: #181818; }
  .clear-all { font-size: 24rpx; color: #999; }
}

.hot-list {
  background: #FFF;
  border-radius: 12rpx;
  overflow: hidden;
}
.hot-item {
  display: flex;
  align-items: center;
  gap: 24rpx;
  padding: 20rpx 16rpx;
  border-bottom: 1rpx solid #F8F8F8;

  &:active { background: #F8F8F8; }
  &:last-child { border-bottom: none; }

  .rank {
    width: 48rpx;
    text-align: center;
    font-size: 28rpx;
    font-weight: 600;
    color: #999;

    &.top { color: #FB7299; }
  }
  .word { flex: 1; font-size: 28rpx; color: #181818; }
  .badge {
    font-size: 20rpx;
    background: linear-gradient(135deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 2rpx 8rpx;
    border-radius: 4rpx;
  }
}

.history {
  display: flex;
  flex-wrap: wrap;
  gap: 16rpx;
  padding: 0 8rpx;
  .history-item {
    display: flex;
    align-items: center;
    gap: 8rpx;
    background: #F4F4F4;
    color: #333;
    padding: 12rpx 20rpx;
    border-radius: 24rpx;
    font-size: 24rpx;
    .del { color: #999; font-size: 24rpx; }
  }
}

.empty-tip {
  text-align: center;
  color: #999;
  padding: 32rpx 0;
}

.loading {
  text-align: center;
  padding: 100rpx 0;
  color: #999;
}

.empty-result {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 120rpx 0;
  color: #999;
  .icon { font-size: 80rpx; margin-bottom: 16rpx; }
}

.results { padding: 16rpx; }

.result-card {
  display: flex;
  gap: 16rpx;
  padding: 16rpx 0;
  border-bottom: 1rpx solid #F8F8F8;

  &:active { opacity: 0.7; }

  .cover-wrap {
    position: relative;
    flex-shrink: 0;
    width: 240rpx;
    height: 150rpx;
    border-radius: 8rpx;
    overflow: hidden;
    background: #E8E8E8;

    .cover-img { width: 100%; height: 100%; object-fit: cover; display: block; }
    .duration {
      position: absolute;
      right: 6rpx;
      bottom: 6rpx;
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
    gap: 8rpx;
    .title { font-size: 26rpx; color: #181818; line-height: 1.4; }
    .stats { font-size: 22rpx; color: #999; margin-top: auto; }
    .up { font-size: 22rpx; color: #999; }
  }
}
</style>