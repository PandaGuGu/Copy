<template>
  <view class="home-page">
    <!-- 顶部：头像 + 搜索栏 + 游戏icon + 消息(信封) -->
    <view class="home-header safe-area-top">
      <view class="header-avatar" @tap="goMine">
        <img :src="avatarUrl" class="avatar-img" alt="avatar" />
      </view>
      <view class="search-bar" @tap="goSearch">
        <text class="search-icon">🔍</text>
        <text class="search-placeholder">美国为何接连发生…</text>
      </view>
      <view class="header-actions">
        <text class="action-icon">🎮</text>
        <view class="msg-btn" @tap="goMessages">
          <!-- B站风格信封（SVG） -->
          <svg viewBox="0 0 24 24" class="envelope-svg">
            <path d="M20 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2zm0 4.5-8 5.5-8-5.5V6l8 5.5L20 6v2.5z" fill="#FB7299"/>
          </svg>
          <view v-if="unreadTotal > 0" class="msg-badge">{{ unreadTotal > 99 ? '99+' : unreadTotal }}</view>
        </view>
      </view>
    </view>

    <!-- Tab 切换 + 更多(三横杆) -->
    <view class="tab-bar-wrap">
      <scroll-view class="home-tabs" scroll-x :scroll-into-view="`tab-${activeTab}`" :show-scrollbar="false">
        <view
          v-for="t in tabs"
          :id="`tab-${t.key}`"
          :key="t.key"
          class="tab-item"
          :class="{ active: activeTab === t.key }"
          @tap="onTabChange(t.key)"
        >
          {{ t.label }}
        </view>
      </scroll-view>
      <view class="tab-more" @tap="goCategories">
        <view class="line" />
        <view class="line" />
        <view class="line" />
      </view>
    </view>

    <!-- 内容区 -->
    <scroll-view scroll-y class="home-content" @scrolltolower="onReachBottom">
      <!-- 顶部大图 banner：自动向右流动（scroll-left 单向累加轮播），原生 img 保证渲染 -->
      <scroll-view
        v-if="activeTab === 'recommend' && banners.length > 0"
        class="banner-swiper"
        scroll-x
        :show-scrollbar="false"
        :scroll-left="bannerScrollLeft"
        :scroll-with-animation="true"
      >
        <view
          v-for="(b, idx) in banners"
          :key="b.id"
          :id="`banner-${idx}`"
          class="banner-item"
          @tap="onBannerTap(b)"
        >
          <img :src="b.pic" :alt="b.name" class="banner-img" referrerpolicy="no-referrer" />
        </view>
      </scroll-view>

      <!-- 直播 tab：直播房间卡片（真实 /live/rooms） -->
      <view v-if="activeTab === 'live'" class="live-grid">
        <view
          v-for="r in liveRooms"
          :key="r.id"
          class="live-card"
          @tap="goRoom(r)"
        >
          <view class="live-cover">
            <img :src="r.cover_url || '/static/placeholder.png'" class="live-img" referrerpolicy="no-referrer" />
            <text class="live-badge">● 直播中</text>
            <text class="live-viewers">{{ r.viewer_count }}人观看</text>
          </view>
          <text class="live-title text-ellipsis-1">{{ r.host_name || '主播' }}</text>
          <text class="live-desc text-ellipsis-2">{{ r.title }}</text>
        </view>
        <view v-if="liveRooms.length === 0 && !loading" class="empty-state">
          <text class="icon">📡</text>
          <text>暂无直播</text>
        </view>
      </view>

      <!-- 热门 tab：排行榜（B站热门榜：排名+封面+标题+播放量） -->
      <view v-else-if="activeTab === 'hot'" class="rank-list">
        <view
          v-for="(v, i) in leaderboard"
          :key="v.ID"
          class="rank-item"
          @tap="goVideoDetail(v.ID)"
        >
          <text class="rank-num" :class="{ top: i < 3 }">{{ i + 1 }}</text>
          <view class="rank-cover">
            <img :src="v.CoverURL" class="rank-img" referrerpolicy="no-referrer" />
            <text class="rank-duration">{{ formatDuration(v.DurationSec) }}</text>
          </view>
          <view class="rank-meta">
            <text class="rank-title text-ellipsis-2">{{ v.Title }}</text>
            <text class="rank-stats">▶ {{ formatCount(v.PlayCount) }} · 💬 {{ formatCount(v.DanmakuCount) }}</text>
          </view>
        </view>
        <view v-if="leaderboard.length === 0 && !loading" class="empty-state">
          <text class="icon">🏆</text>
          <text>暂无排行</text>
        </view>
      </view>

      <!-- 其他 tab：视频卡片瀑布流（推荐/动画/影视/新人） -->
      <template v-else>
        <view class="video-grid">
          <view
            v-for="v in videos"
            :key="v.id"
            class="video-card"
            @tap="goVideoDetail(v.id)"
          >
            <view class="video-cover">
              <img :src="v.cover_url || '/static/placeholder.png'" class="video-cover-img" referrerpolicy="no-referrer" />
              <text class="video-duration">{{ formatDuration(v.duration) }}</text>
            </view>
            <view class="video-meta">
              <text class="title">{{ v.title }}</text>
              <text class="up">{{ v.uploader || '匿名' }}</text>
              <view class="stats">
                <text>▶ {{ formatCount(v.play_count) }}</text>
                <text>💬 {{ v.comment_count }}</text>
              </view>
            </view>
          </view>
        </view>

        <view v-if="loading" class="loading-more">
          <text>加载中...</text>
        </view>
        <view v-else-if="videos.length === 0" class="empty-state">
          <text class="icon">📭</text>
          <text>暂无内容</text>
        </view>
        <view v-else-if="!hasMore" class="loading-more">
          <text>— 已经到底了 —</text>
        </view>
      </template>
    </scroll-view>

    <CustomTabBar />
  </view>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { onShow, onPullDownRefresh } from '@dcloudio/uni-app'
import { videoApi, bannerApi, categoryApi, liveApi, notificationApi } from '@/api'
import type { Video, Banner, LiveRoom } from '@/utils/types'
import type { LeaderboardVideo } from '@/api/video'
import { useUserStore } from '@/store/user'
import { useAppStore } from '@/store/app'
import CustomTabBar from '@/custom-tab-bar/index.vue'

interface Tab { key: string; label: string }

const userStore = useUserStore()
const appStore = useAppStore()

// 头像：登录显示真实头像，未登录默认头像
const user = computed(() => userStore.user)
const avatarUrl = computed(() => user.value?.avatar_url || '/static/avatar/default.png')

// 消息未读数（B站信封红点）
const unreadTotal = ref(0)

const tabs: Tab[] = [
  { key: 'live',     label: '直播' },
  { key: 'recommend',label: '推荐' },
  { key: 'hot',      label: '热门' },
  { key: 'anime',    label: '动画' },
  { key: 'movie',    label: '影视' },
  { key: 'fresh',    label: '新人' }
]

const activeTab = ref<string>('recommend')
const banners = ref<Banner[]>([])
const videos = ref<Video[]>([])
const leaderboard = ref<LeaderboardVideo[]>([])
const liveRooms = ref<LiveRoom[]>([])
const cursor = ref<string | null>(null)
const hasMore = ref(true)
const loading = ref(false)

// banner 自动向右流动（单向轮播：scroll-left 累加，到末尾回到 0）
const bannerScrollLeft = ref(0)
let bannerTimer: ReturnType<typeof setInterval> | null = null
let bannerPos = 0

function startBannerAutoPlay() {
  stopBannerAutoPlay()
  const n = banners.value.length
  if (n <= 1) return
  const sw = uni.getSystemInfoSync().windowWidth // 单张 banner 宽 = 屏宽
  bannerTimer = setInterval(() => {
    bannerPos += sw
    if (bannerPos >= sw * n) bannerPos = 0
    bannerScrollLeft.value = bannerPos
  }, 4000)
}

function stopBannerAutoPlay() {
  if (bannerTimer) {
    clearInterval(bannerTimer)
    bannerTimer = null
  }
}

onMounted(() => {
  loadBanners()
  loadVideos(true)
})

onUnmounted(() => stopBannerAutoPlay())

onShow(async () => {
  appStore.currentTab = 0
  userStore.refreshMe()
  await loadUnread()
})

// 拉取未读汇总（5 类通知求和），未登录跳过
async function loadUnread() {
  if (!userStore.isLoggedIn) { unreadTotal.value = 0; return }
  try {
    const s = await notificationApi.unreadSummary()
    unreadTotal.value = Object.values(s).reduce((a, b) => a + (b || 0), 0)
  } catch { unreadTotal.value = 0 }
}

onPullDownRefresh(async () => {
  await loadBanners()
  await loadVideos(true)
  uni.stopPullDownRefresh()
})

async function loadBanners() {
  try {
    banners.value = await bannerApi.active()
    startBannerAutoPlay()
  } catch { /* ignore */ }
}

async function loadVideos(reset = false) {
  if (loading.value) return
  if (reset) { cursor.value = null; hasMore.value = true }
  loading.value = true
  try {
    const tab = activeTab.value
    // 热门 tab：排行榜（B站热门榜）
    if (tab === 'hot') {
      const list = await videoApi.leaderboard(50)
      leaderboard.value = reset ? list : leaderboard.value.concat(list)
      hasMore.value = false
    }
    // 直播 tab：直播房间
    else if (tab === 'live') {
      const resp = await liveApi.rooms()
      liveRooms.value = resp.rooms || []
      hasMore.value = false
    }
    // 分区 tab：动画/影视 走 zones/:zone/recommendation
    else if (tab === 'anime' || tab === 'movie') {
      const zone = tab === 'anime' ? '动画' : '影视'
      const resp = await categoryApi.zoneVideos(zone, cursor.value ?? undefined, 50)
      if (reset) videos.value = resp.items
      else videos.value.push(...resp.items)
      cursor.value = resp.next_cursor
      hasMore.value = !!resp.next_cursor && resp.items.length > 0
    }
    // 推荐/新人：与 PC 端同源 GET /videos
    else {
      const resp = await videoApi.list(cursor.value ?? undefined, 50)
      if (reset) videos.value = resp.items
      else videos.value.push(...resp.items)
      cursor.value = resp.next_cursor
      hasMore.value = !!resp.next_cursor && resp.items.length > 0
    }
  } catch { /* 错误已 toast */ }
  finally { loading.value = false }
}

async function onReachBottom() {
  if (hasMore.value && !loading.value) await loadVideos(false)
}

async function onTabChange(key: string) {
  activeTab.value = key
  await loadVideos(true)
}

function goRoom(r: LiveRoom) {
  uni.navigateTo({ url: `/pages/live-room/index?id=${r.id}` })
}

// 路由跳转
function goSearch()      { uni.navigateTo({ url: '/pages/search/index' }) }
function goMessages()    { uni.navigateTo({ url: '/pages/notifications/index' }) }
function goCategories()  { uni.navigateTo({ url: '/pages/categories/index' }) }
function goMine()        { uni.switchTab({ url: '/pages/mine/index' }) }
function goVideoDetail(id: number) { uni.navigateTo({ url: `/pages/video-detail/index?id=${id}` }) }

function onBannerTap(b: Banner) {
  const target = b.url
  if (!target) return
  const m = target.match(/\/video\/([\w]+)/)
  if (m) {
    uni.showToast({ title: `跳转视频 ${m[1]}`, icon: 'none' })
    uni.setClipboardData({ data: target })
  } else {
    uni.setClipboardData({ data: target })
    uni.showToast({ title: '链接已复制', icon: 'none' })
  }
}

// 工具
function formatDuration(s: number): string {
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
.home-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #FFF;
}

.home-header {
  display: flex;
  align-items: center;
  padding: 12rpx 24rpx;
  gap: 16rpx;
  background-color: #FFF;
  border-bottom: 1rpx solid #F1F1F1;
}

/* 搜索框左侧：账号头像 */
.header-avatar {
  width: 64rpx;
  height: 64rpx;
  border-radius: 50%;
  overflow: hidden;
  flex-shrink: 0;
  background: #F4F4F4;

  .avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
}

.search-bar {
  flex: 1;
  height: 64rpx;
  background-color: #F4F4F4;
  border-radius: 32rpx;
  display: flex;
  align-items: center;
  padding: 0 24rpx;
  gap: 12rpx;
  font-size: 24rpx;
  color: #999;

  .search-placeholder { flex: 1; }
  .search-icon { font-size: 28rpx; }
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 20rpx;

  .action-icon {
    font-size: 40rpx;
  }
}

/* B站风格信封消息按钮 + 未读红点 */
.msg-btn {
  position: relative;
  width: 48rpx;
  height: 48rpx;
  display: flex;
  align-items: center;
  justify-content: center;

  .envelope-svg {
    width: 48rpx;
    height: 48rpx;
    display: block;
  }

  .msg-badge {
    position: absolute;
    top: -6rpx;
    right: -12rpx;
    background: #FB7299;
    color: #FFF;
    font-size: 18rpx;
    min-width: 30rpx;
    height: 30rpx;
    line-height: 30rpx;
    text-align: center;
    border-radius: 15rpx;
    padding: 0 6rpx;
    border: 2rpx solid #FFF;
    box-sizing: border-box;
  }
}

/* Tab 栏容器：左侧可滚动 tabs + 右侧固定三横杆 */
.tab-bar-wrap {
  display: flex;
  align-items: center;
  background-color: #FFF;
}

.home-tabs {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  height: 80rpx;

  .tab-item {
    display: inline-block;
    padding: 0 24rpx;
    line-height: 80rpx;
    font-size: 28rpx;
    color: #666;
    transition: all 0.2s;

    &.active {
      color: #181818;
      font-weight: 600;
      font-size: 32rpx;
    }
  }
}

/* 三横杆（更多）按钮 */
.tab-more {
  width: 72rpx;
  height: 80rpx;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8rpx;

  .line {
    width: 34rpx;
    height: 4rpx;
    border-radius: 2rpx;
    background: #181818;
  }
}

.home-content {
  flex: 1;
}

/* 顶部大图 banner：全宽 + 自动向右流动 */
.banner-swiper {
  width: 100%;
  white-space: nowrap;
  margin: 16rpx 0;
}

.banner-item {
  display: inline-block;
  width: 100%;
  height: 280rpx;
  padding: 0 24rpx;
  box-sizing: border-box;
  vertical-align: top;
}

.banner-img {
  width: 100%;
  height: 100%;
  border-radius: 16rpx;
  object-fit: cover;
  display: block;
  background: #E8E8E8;
}

.loading-more {
  text-align: center;
  padding: 40rpx 0;
  color: #999;
  font-size: 24rpx;
}

/* 直播 tab 卡片 */
.live-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  padding: 16rpx 24rpx;
  gap: 16rpx 0;
}

.live-card {
  width: 48.7%;
  border-radius: 12rpx;
  overflow: hidden;

  .live-cover {
    position: relative;
    width: 100%;
    height: 260rpx;
    background: #E8E8E8;
    border-radius: 12rpx;
    overflow: hidden;
  }
  .live-img { width: 100%; height: 100%; object-fit: cover; display: block; }
  .live-badge {
    position: absolute;
    top: 8rpx; left: 8rpx;
    background: rgba(251, 114, 153, 0.9);
    color: #FFF;
    font-size: 20rpx;
    padding: 4rpx 12rpx;
    border-radius: 6rpx;
  }
  .live-viewers {
    position: absolute;
    bottom: 8rpx; right: 8rpx;
    background: rgba(0,0,0,0.6);
    color: #FFF;
    font-size: 20rpx;
    padding: 2rpx 8rpx;
    border-radius: 4rpx;
  }
  .live-title { display: block; margin-top: 8rpx; font-size: 26rpx; color: #181818; font-weight: 500; }
  .live-desc { display: block; margin-top: 4rpx; font-size: 22rpx; color: #999; }
}

/* 热门排行榜（B站热门榜：左排名 + 封面 + 标题 + 播放量） */
.rank-list {
  padding: 8rpx 24rpx;
}

.rank-item {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 16rpx 0;
  border-bottom: 1rpx solid #F8F8F8;

  &:active { opacity: 0.7; }

  .rank-num {
    width: 48rpx;
    text-align: center;
    font-size: 32rpx;
    font-weight: 600;
    color: #999;

    &.top { color: #FB7299; }
  }
  .rank-cover {
    position: relative;
    width: 200rpx;
    height: 125rpx;
    border-radius: 8rpx;
    overflow: hidden;
    flex-shrink: 0;
    background: #E8E8E8;

    .rank-img { width: 100%; height: 100%; object-fit: cover; display: block; }
    .rank-duration {
      position: absolute;
      right: 6rpx; bottom: 6rpx;
      background: rgba(0,0,0,0.7);
      color: #FFF;
      font-size: 18rpx;
      padding: 2rpx 6rpx;
      border-radius: 4rpx;
    }
  }
  .rank-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8rpx;
    .rank-title { font-size: 26rpx; color: #181818; line-height: 1.4; }
    .rank-stats { font-size: 22rpx; color: #999; }
  }
}

/* 视频封面 img（替代 uni-image，保证渲染） */
.video-cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
</style>