<template>
  <view class="follow-page">
    <!-- 顶部 -->
    <view class="follow-header safe-area-top">
      <text class="title">关注</text>
      <text class="edit-icon" @tap="goEdit">✎</text>
    </view>

    <view class="type-tabs">
      <view class="type-tab" :class="{ active: tab === 'all' }" @tap="tab = 'all'">全部</view>
      <view class="type-tab" :class="{ active: tab === 'video' }" @tap="tab = 'video'">视频</view>
    </view>

    <template v-if="isLoggedIn">
      <!-- 最常访问（我的关注列表） -->
      <view class="section">
        <view class="section-head">
          <text class="title">最常访问</text>
          <text class="more">{{ liveCount }}人直播中 · 更多 ›</text>
        </view>
        <scroll-view scroll-x class="hot-row" :show-scrollbar="false">
          <view v-for="u in following" :key="u.user_id" class="hot-item">
            <view class="hot-avatar">
              <img :src="u.avatar_url || '/static/avatar/default.png'" class="hot-img" referrerpolicy="no-referrer" />
              <view v-if="isLiving(u.user_id)" class="live-dot" />
            </view>
            <text class="hot-name">{{ u.nickname || u.username }}</text>
          </view>
        </scroll-view>
      </view>

      <!-- 直播中横向 -->
      <scroll-view v-if="liveRooms.length" scroll-x class="live-row" :show-scrollbar="false">
        <view v-for="r in liveRooms" :key="r.id" class="live-card" @tap="goRoom(r)">
          <view class="live-avatar">
            <img :src="r.cover_url || '/static/placeholder.png'" class="live-img" referrerpolicy="no-referrer" />
            <text class="live-badge">⏺ 直播中</text>
          </view>
          <view class="live-mask">
            <text class="live-name">{{ r.host_name || '主播' }} · {{ r.title }}</text>
            <text class="live-viewers">{{ r.viewer_count }}人观看</text>
          </view>
        </view>
      </scroll-view>

      <!-- 动态 Feed -->
      <view class="dynamics">
        <view v-for="d in dynamics" :key="d.id" class="dynamic-card">
          <view class="dynamic-head">
            <img class="avatar" :src="d.author_avatar || '/static/avatar/default.png'" referrerpolicy="no-referrer" />
            <view class="meta">
              <view class="row">
                <text class="name">{{ d.author_name }}</text>
              </view>
              <text class="time">{{ formatTime(d.created_at) }}</text>
            </view>
          </view>
          <view v-if="d.title" class="dynamic-title"><text class="t">{{ d.title }}</text></view>
          <text class="dynamic-content">{{ d.content }}</text>
          <view v-if="d.images && d.images.length" class="dynamic-imgs">
            <img
              v-for="(img, i) in d.images.slice(0, 3)"
              :key="i"
              :src="img"
              class="dyn-img"
              mode="aspectFill"
              referrerpolicy="no-referrer"
            />
          </view>
          <view class="dynamic-actions">
            <text>💬 {{ d.comment_count }}</text>
            <text>👍 {{ d.like_count }}</text>
            <text class="liked" v-if="d.liked_by_me">已赞</text>
          </view>
        </view>

        <view v-if="dynamics.length === 0" class="empty-state">
          <text class="icon">📭</text>
          <text>关注的人还没有动态</text>
        </view>
      </view>
    </template>

    <!-- 未登录 -->
    <view v-else class="empty-state login-tip">
      <text class="icon">👤</text>
      <text>登录后查看关注动态</text>
      <view class="login-btn" @tap="goLogin">
        <text>立即登录</text>
      </view>
    </view>

    <CustomTabBar />
  </view>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { dynamicApi, userApi, liveApi } from '@/api'
import { useUserStore } from '@/store/user'
import { useAppStore } from '@/store/app'
import CustomTabBar from '@/custom-tab-bar/index.vue'
import type { Dynamic, LiveRoom } from '@/utils/types'

interface FollowItem { user_id: number; nickname?: string; username?: string; avatar_url?: string }

const userStore = useUserStore()
const isLoggedIn = computed(() => userStore.isLoggedIn)

const tab = ref<'all' | 'video'>('all')
const following = ref<FollowItem[]>([])
const liveRooms = ref<LiveRoom[]>([])
const dynamics = ref<Dynamic[]>([])
const loading = ref(false)

const liveCount = computed(() => liveRooms.value.filter((r) => r.status === 'live').length)
const liveUserIds = computed(() => new Set(liveRooms.value.filter((r) => r.status === 'live').map((r) => r.user_id)))

onMounted(() => load())
onShow(() => {
  useAppStore().currentTab = 1
  if (isLoggedIn.value) load()
})

async function load() {
  if (!isLoggedIn.value || loading.value) return
  loading.value = true
  try {
    const [fl, rooms, dyn] = await Promise.all([
      userApi.myFollowings(50),
      liveApi.rooms(),
      dynamicApi.following(undefined, 30)
    ])
    following.value = fl.items || []
    liveRooms.value = rooms.rooms || []
    dynamics.value = dyn.items || []
  } catch { /* 已 toast */ }
  finally { loading.value = false }
}

function isLiving(uid: number) {
  return liveUserIds.value.has(uid)
}

function goEdit()   { uni.showToast({ title: '编辑关注分组开发中', icon: 'none' }) }
function goLogin()  { uni.navigateTo({ url: '/pages/login/index' }) }
function goRoom(r: LiveRoom) {
  uni.navigateTo({ url: `/pages/live-room/index?id=${r.id}` })
}

function formatTime(s: string) {
  if (!s) return ''
  const diff = Date.now() - new Date(s).getTime()
  if (diff < 60_000) return '刚刚'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}分钟前`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}小时前`
  const d = new Date(s)
  return `${d.getFullYear()}-${(d.getMonth() + 1).toString().padStart(2, '0')}-${d.getDate().toString().padStart(2, '0')}`
}
</script>

<style lang="scss" scoped>
.follow-page {
  min-height: 100vh;
  background: #FFF;
  padding-bottom: 120rpx;
}

.follow-header {
  display: flex;
  align-items: center;
  padding: 24rpx;
  border-bottom: 1rpx solid #F1F1F1;
  .title { flex: 1; text-align: center; font-size: 32rpx; font-weight: 500; }
  .edit-icon { font-size: 32rpx; }
}

.type-tabs {
  display: flex;
  border-bottom: 1rpx solid #F1F1F1;
  .type-tab {
    flex: 1;
    text-align: center;
    padding: 24rpx 0;
    font-size: 28rpx;
    color: #666;
    &.active { color: #FB7299; font-weight: 500; border-bottom: 4rpx solid #FB7299; }
  }
}

.section { padding: 24rpx 16rpx 0; }
.section-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  .title { font-size: 28rpx; font-weight: 500; }
  .more  { font-size: 24rpx; color: #999; }
}

.hot-row {
  white-space: nowrap;
  margin-top: 16rpx;
  .hot-item {
    display: inline-block;
    width: 120rpx;
    margin-right: 16rpx;
    text-align: center;
  }
  .hot-avatar {
    position: relative;
    width: 90rpx;
    height: 90rpx;
    margin: 0 auto;
    .hot-img {
      width: 100%; height: 100%;
      border-radius: 50%;
      background: #F0F0F0;
    }
    .live-dot {
      position: absolute;
      bottom: 0; right: 0;
      width: 18rpx; height: 18rpx;
      border-radius: 50%;
      background: #FB7299;
      border: 4rpx solid #FFF;
    }
  }
  .hot-name {
    display: block;
    margin-top: 8rpx;
    font-size: 22rpx;
    color: #333;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.live-row {
  white-space: nowrap;
  padding: 24rpx 16rpx;
  .live-card {
    display: inline-block;
    width: 220rpx;
    height: 150rpx;
    border-radius: 16rpx;
    margin-right: 16rpx;
    position: relative;
    overflow: hidden;
    vertical-align: top;
  }
  .live-avatar { width: 100%; height: 100%; }
  .live-img { width: 100%; height: 100%; background: #E8E8E8; }
  .live-badge {
    position: absolute;
    top: 8rpx; left: 8rpx;
    background: #FB7299; color: #FFF;
    padding: 4rpx 12rpx; border-radius: 6rpx;
    font-size: 20rpx;
  }
  .live-mask {
    position: absolute;
    bottom: 0; left: 0; right: 0;
    background: linear-gradient(transparent, rgba(0,0,0,0.6));
    color: #FFF;
    padding: 16rpx;
    .live-name { display: block; font-size: 22rpx; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .live-viewers { display: block; font-size: 18rpx; opacity: 0.85; margin-top: 2rpx; }
  }
}

.dynamics { padding: 0 16rpx 24rpx; }

.dynamic-card {
  border-top: 1rpx solid #F1F1F1;
  padding: 24rpx 0;
}

.dynamic-head {
  display: flex;
  gap: 16rpx;
  align-items: center;
  .avatar {
    width: 72rpx; height: 72rpx;
    border-radius: 50%;
    background: #F0F0F0;
  }
  .meta { flex: 1; }
  .row {
    display: flex;
    align-items: center;
    gap: 8rpx;
    .name { font-size: 26rpx; color: #FB7299; font-weight: 500; }
  }
  .time { font-size: 22rpx; color: #999; margin-top: 4rpx; display: block; }
}

.dynamic-title {
  margin-top: 16rpx;
  .t { font-size: 28rpx; color: #181818; font-weight: 500; }
}

.dynamic-content {
  display: block;
  margin-top: 8rpx;
  font-size: 26rpx;
  color: #333;
  line-height: 1.5;
}

.dynamic-imgs {
  display: flex;
  gap: 8rpx;
  margin-top: 16rpx;
  .dyn-img {
    width: 200rpx;
    height: 150rpx;
    border-radius: 8rpx;
    background: #E8E8E8;
  }
}

.dynamic-actions {
  display: flex;
  justify-content: flex-end;
  gap: 32rpx;
  margin-top: 16rpx;
  font-size: 24rpx;
  color: #666;
  .liked { color: #FB7299; }
}

.login-tip {
  flex-direction: column;
  gap: 24rpx;
  padding-top: 120rpx;
  .login-btn {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 16rpx 56rpx;
    border-radius: 32rpx;
    font-size: 26rpx;
  }
}
</style>