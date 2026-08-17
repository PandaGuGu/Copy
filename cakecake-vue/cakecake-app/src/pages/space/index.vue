<template>
  <view class="space-page">
    <view class="page-header safe-area-top">
      <text class="back-icon" @tap="goBack">‹</text>
      <text class="title">{{ info?.nickname || '个人空间' }}</text>
      <text class="spacer" />
    </view>

    <view v-if="loading && !info" class="loading"><text>加载中…</text></view>

    <template v-else-if="info">
      <!-- 头部：头像 + 信息 + 关注按钮 -->
      <view class="space-head">
        <view class="head-top">
          <view class="avatar-wrap">
            <img :src="info.avatar_url || '/static/avatar/default.png'" class="avatar" referrerpolicy="no-referrer" />
            <view class="level-badge">Lv{{ info.level_info?.current_level || 1 }}</view>
          </view>
          <view class="head-meta">
            <view class="row1">
              <text class="nickname">{{ info.nickname }}</text>
            </view>
            <text class="sign">{{ info.sign || '这个家伙很懒，什么都没有写' }}</text>
            <view class="stats">
              <text>投稿 {{ info.published_count }}</text>
              <text>关注 {{ info.following_count }}</text>
              <text>粉丝 {{ info.follower_count }}</text>
            </view>
          </view>
          <view
            v-if="!info.is_owner"
            class="follow-btn"
            :class="{ followed: info.followed_by_me }"
            @tap="toggleFollow"
          >
            <text>{{ info.followed_by_me ? '已关注' : '+ 关注' }}</text>
          </view>
        </view>
      </view>

      <!-- Tabs：投稿 / 动态 -->
      <view class="tabs">
        <view class="tab" :class="{ active: tab === 'video' }" @tap="tab = 'video'">
          <text>投稿</text>
        </view>
        <view class="tab" :class="{ active: tab === 'dynamic' }" @tap="tab = 'dynamic'">
          <text>动态</text>
        </view>
      </view>

      <!-- 投稿网格 -->
      <view v-if="tab === 'video'" class="video-grid">
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
            <text class="stats">▶ {{ formatCount(v.play_count) }} 💬 {{ v.comment_count }}</text>
          </view>
        </view>
        <view v-if="videos.length === 0 && !loading" class="empty">
          <text>暂无投稿</text>
        </view>
      </view>

      <!-- 动态 tab（占位，R9 完整实现） -->
      <view v-else class="dynamic-placeholder">
        <text>该 UP 主动态（R9 实现）</text>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onLoad, onReachBottom } from '@dcloudio/uni-app'
import { spaceApi, type SpaceInfo } from '@/api/space'
import { useUserStore } from '@/store/user'
import type { Video } from '@/utils/types'

const userStore = useUserStore()
const info = ref<SpaceInfo | null>(null)
const videos = ref<Video[]>([])
const loading = ref(false)
const tab = ref<'video' | 'dynamic'>('video')
let userId = 0
let cursor: string | null = null
let hasMore = true

onLoad((q) => {
  userId = Number(q?.id) || 0
  load()
})

async function load() {
  if (!userId) return
  loading.value = true
  try {
    const [sp, vs] = await Promise.all([
      spaceApi.info(userId),
      spaceApi.videos(userId, undefined, 30)
    ])
    info.value = sp
    videos.value = vs.items
    cursor.value = vs.next_cursor
    hasMore.value = !!vs.next_cursor
  } finally {
    loading.value = false
  }
}

onReachBottom(async () => {
  if (!hasMore || !cursor || loading.value) return
  const vs = await spaceApi.videos(userId, cursor, 30)
  videos.value.push(...vs.items)
  cursor.value = vs.next_cursor
  hasMore.value = !!vs.next_cursor
})

async function toggleFollow() {
  if (!info.value) return
  if (!userStore.isLoggedIn) {
    uni.showToast({ title: '请先登录', icon: 'none' })
    setTimeout(() => uni.navigateTo({ url: '/pages/login/index' }), 800)
    return
  }
  try {
    if (info.value.followed_by_me) {
      await spaceApi.unfollow(userId)
      info.value.followed_by_me = false
      info.value.follower_count = Math.max(0, info.value.follower_count - 1)
      uni.showToast({ title: '已取关', icon: 'none' })
    } else {
      await spaceApi.follow(userId)
      info.value.followed_by_me = true
      info.value.follower_count += 1
      uni.showToast({ title: '关注成功', icon: 'success' })
    }
  } catch { /* 已 toast */ }
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
  return String(n)
}
</script>

<style lang="scss" scoped>
.space-page { min-height: 100vh; background: #FFF; padding-bottom: 60rpx; }

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

.loading { text-align: center; padding: 120rpx 0; color: #999; }

.space-head {
  padding: 40rpx 24rpx 24rpx;
}

.head-top {
  display: flex;
  align-items: flex-start;
  gap: 24rpx;
}

.avatar-wrap {
  position: relative;
  width: 120rpx;
  height: 120rpx;

  .avatar {
    width: 100%; height: 100%;
    border-radius: 50%;
    background: #F0F0F0;
  }
  .level-badge {
    position: absolute;
    bottom: -6rpx; left: 50%;
    transform: translateX(-50%);
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    font-size: 18rpx;
    padding: 2rpx 12rpx;
    border-radius: 12rpx;
    white-space: nowrap;
  }
}

.head-meta {
  flex: 1;

  .row1 { display: flex; align-items: center; gap: 8rpx; }
  .nickname { font-size: 32rpx; font-weight: 600; color: #181818; }
  .sign {
    display: block;
    margin-top: 6rpx;
    font-size: 22rpx;
    color: #999;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .stats {
    display: flex;
    gap: 20rpx;
    margin-top: 12rpx;
    font-size: 22rpx;
    color: #666;

    text { display: inline-block; }
  }
}

.follow-btn {
  flex-shrink: 0;
  background: linear-gradient(90deg, #FB7299, #FF9DB5);
  color: #FFF;
  padding: 10rpx 24rpx;
  border-radius: 24rpx;
  font-size: 24rpx;

  &.followed {
    background: #F4F4F4;
    color: #999;
  }
}

.tabs {
  display: flex;
  border-bottom: 1rpx solid #F1F1F1;

  .tab {
    flex: 1;
    text-align: center;
    padding: 20rpx 0;
    font-size: 28rpx;
    color: #666;

    &.active {
      color: #FB7299;
      font-weight: 500;
      border-bottom: 4rpx solid #FB7299;
    }
  }
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
  border-radius: 12rpx;
  overflow: hidden;

  &:active { transform: scale(0.98); }
}

.video-cover {
  position: relative;
  width: 100%;
  height: 220rpx;
  background: #E8E8E8;
  border-radius: 12rpx;
  overflow: hidden;

  .cover-img { width: 100%; height: 100%; object-fit: cover; display: block; }
  .duration {
    position: absolute;
    right: 8rpx; bottom: 8rpx;
    background: rgba(0,0,0,0.7);
    color: #FFF;
    font-size: 20rpx;
    padding: 2rpx 8rpx;
    border-radius: 4rpx;
  }
}

.meta {
  padding: 12rpx 4rpx;
  .title { font-size: 26rpx; color: #181818; line-height: 1.4; }
  .stats { display: block; margin-top: 6rpx; font-size: 22rpx; color: #999; }
}

.empty { width: 100%; text-align: center; padding: 80rpx 0; color: #999; }
.dynamic-placeholder { text-align: center; padding: 80rpx 0; color: #999; }
</style>