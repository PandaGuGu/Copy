<template>
  <view class="mine-page" v-if="user">
    <!-- 顶部图标 -->
    <view class="top-actions safe-area-top">
      <view class="top-icon" @tap="goService('我的客服')">
        <text>📺</text>
      </view>
      <view class="top-icon" @tap="goService('扫一扫')">
        <text>⌧</text>
      </view>
      <view class="top-icon" @tap="goService('个性装扮')">
        <text>👕</text>
      </view>
      <view class="top-icon" @tap="toggleTheme">
        <text>🌙</text>
      </view>
    </view>

    <!-- 头像 + 用户信息（参考图布局：头像左 + 信息中 + 空间入口右 + 下方 3 列统计） -->
    <view class="user-info">
      <view class="info-top">
        <view class="avatar-wrap">
          <img class="avatar" :src="user.avatar_url || '/static/avatar/default.png'" alt="avatar" referrerpolicy="no-referrer" />
        </view>
        <view class="info-meta">
          <view class="row-1">
            <text class="nickname">{{ user.nickname || '我的主页' }}</text>
            <text class="edit-icon">✎</text>
            <view class="lv-badge">LV{{ user.level_info?.current_level || 1 }}</view>
          </view>
          <view class="vip-tag">正式会员</view>
          <view class="coin-inline">
            <text>B币：0.0</text>
            <text>硬币：{{ user.coin_balance ?? 0 }}</text>
          </view>
        </view>
        <view class="space-link" @tap="goSpace">空间 ›</view>
      </view>

      <view class="stats-row">
        <view class="stat-cell" @tap="goMyDynamics">
          <text class="num">{{ dynamicCount || '—' }}</text>
          <text class="label">动态</text>
        </view>
        <view class="stat-divider" />
        <view class="stat-cell">
          <text class="num">{{ followingCount || '—' }}</text>
          <text class="label">关注</text>
        </view>
        <view class="stat-divider" />
        <view class="stat-cell">
          <text class="num">{{ user.fan_count || '—' }}</text>
          <text class="label">粉丝</text>
        </view>
      </view>
    </view>

    <!-- 快捷入口 4 个 -->
    <view class="quick-row">
      <view class="quick-cell" @tap="goService('离线缓存')">
        <view class="quick-icon-wrap" style="background: #E1F5EE;">
          <text>📥</text>
        </view>
        <text>离线缓存</text>
      </view>
      <view class="quick-cell" @tap="goHistory">
        <view class="quick-icon-wrap" style="background: #FAEEDA;">
          <text>🕐</text>
        </view>
        <text>历史记录</text>
      </view>
      <view class="quick-cell" @tap="goFavorites">
        <view class="quick-icon-wrap" style="background: #E6F1FB;">
          <text>⭐</text>
        </view>
        <text>我的收藏</text>
      </view>
      <view class="quick-cell" @tap="goService('稍后再看')">
        <view class="quick-icon-wrap" style="background: #FBEAF0; position: relative;">
          <text>▶</text>
          <view class="badge-dot">1</view>
        </view>
        <text>稍后再看</text>
      </view>
    </view>

    <!-- 创作中心 -->
    <view class="creator-section">
      <view class="creator-head">
        <text class="title">创作中心</text>
        <view class="publish-btn" @tap="goPublish">
          <text>📤</text>
          <text>发布</text>
        </view>
      </view>
      <view class="grid">
        <view v-for="(item, idx) in creatorItems" :key="idx" class="grid-item" @tap="goService(item)">
          <view class="icon-wrap" :style="{ background: pickColor(idx) }">
            <text>{{ item.icon }}</text>
            <view v-if="item.badge" class="badge-dot">{{ item.badge }}</view>
          </view>
          <text class="label">{{ item.label }}</text>
        </view>
      </view>
    </view>

    <!-- 我的服务（原 mine-services 并入） -->
    <view class="creator-section">
      <view class="creator-head">
        <text class="title">我的服务</text>
      </view>
      <view class="grid">
        <view v-for="(s, idx) in services" :key="s.label" class="grid-item" @tap="goService(s)">
          <view class="icon-wrap" :style="{ background: pickColor(idx) }">
            <text>{{ s.icon }}</text>
          </view>
          <text class="label">{{ s.label }}</text>
        </view>
      </view>
    </view>

    <!-- 游戏中心 -->
    <view class="creator-section">
      <view class="creator-head">
        <text class="title">游戏中心</text>
        <text class="more">拓麻歌子联动开启！›</text>
      </view>
      <view class="grid grid-4">
        <view v-for="g in gameItems" :key="g" class="grid-item" @tap="goService(g)">
          <view class="icon-wrap" style="background: #FFE4EC;">
            <text>🎮</text>
          </view>
          <text class="label">{{ g }}</text>
        </view>
      </view>
    </view>

    <!-- 更多服务（原 mine-services 并入） -->
    <view class="creator-section">
      <view class="creator-head">
        <text class="title">更多服务</text>
      </view>
      <view v-for="m in more" :key="m.label" class="more-row" @tap="goService(m)">
        <view class="more-icon" :style="{ background: m.color }"><text>{{ m.icon }}</text></view>
        <text class="more-label">{{ m.label }}</text>
        <text class="arrow">›</text>
      </view>
    </view>
  </view>

  <!-- 未登录态 -->
  <view v-else class="empty-mine">
    <view class="empty-avatar">👤</view>
    <text class="empty-tip">登录后查看更多</text>
    <view class="login-btn" @tap="goLogin">
      <text>立即登录</text>
    </view>
  </view>

  <CustomTabBar />
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { useUserStore } from '@/store/user'
import { useAppStore } from '@/store/app'
import { dynamicApi, userApi } from '@/api'
import CustomTabBar from '@/custom-tab-bar/index.vue'

const userStore = useUserStore()
const user = computed(() => userStore.user)

const COLORS = ['#FFE4EC', '#E1F5EE', '#E6F1FB', '#FAEEDA', '#FBEAF0', '#EAF3DE', '#FAECE7', '#FBEAF0']

// 真实接口数据（onShow 拉取）
const dynamicCount = ref(0)
const followingCount = ref(0)

const creatorItems: { label: string; icon: string; badge?: string; path?: string }[] = [
  { label: '创作中心',   icon: '💡', path: '/pages/my-videos/index' },
  { label: '稿件管理',   icon: '🎬', path: '/pages/my-videos/index' },
  { label: '数据中心',   icon: '📊' },
  { label: '有奖活动',   icon: '🎁', badge: '1' },
  { label: '开播福利',   icon: '🎉' },
  { label: '主播中心',   icon: '🎙' },
  { label: '直播数据',   icon: '📈' },
  { label: '主播活动',   icon: '🏆' }
]

// 我的服务（原 mine-services 合并而来）
const services: { label: string; icon: string; path?: string }[] = [
  { label: '我的课程',   icon: '📚' },
  { label: '免流量服务', icon: '📶' },
  { label: '个性装扮',   icon: '👕' },
  { label: '我的钱包',   icon: '👛' },
  { label: '会员购订单', icon: '🛍', path: '/pages/mall/index' },
  { label: '我的直播',   icon: '📡' },
  { label: '漫画',       icon: '📖' },
  { label: '必火推广',   icon: '🔥' },
  { label: '社区中心',   icon: '💬' },
  { label: '哔哩哔哩公益', icon: '❤️' },
  { label: '工房',       icon: '🏪' },
  { label: '能量加油站', icon: '⛽' }
]

const gameItems = ['我的游戏', '我的预约', '找游戏', '游戏排行榜']

// 更多服务（原 mine-services 合并而来）
const more: { label: string; icon: string; color: string; path?: string }[] = [
  { label: '联系客服',    icon: '🎧', color: '#FBEAF0' },
  { label: '听视频',      icon: '🎵', color: '#E1F5EE' },
  { label: '未成年人守护', icon: '🌱', color: '#EAF3DE' },
  { label: '设置',        icon: '⚙', color: '#F1EFE8' }
]

onShow(async () => {
  useAppStore().currentTab = 3
  userStore.refreshMe()
  await loadMyStats()
})

async function loadMyStats() {
  if (!userStore.isLoggedIn) return
  // 关注数 / 动态数（并行）
  const [follow, mineDynamics] = await Promise.all([
    userApi.myFollowings(200).catch(() => ({ total: 0, items: [] })),
    dynamicApi.mine().catch(() => ({ items: [] }))
  ])
  followingCount.value = follow.total || 0
  dynamicCount.value = mineDynamics.items?.length || 0
}

function pickColor(i: number) { return COLORS[i % COLORS.length] }

function goService(item: { label: string; path?: string }) {
  if (item.path) uni.navigateTo({ url: item.path })
  else uni.showToast({ title: `${item.label} 功能待开发`, icon: 'none' })
}

function goPublish()  { uni.navigateTo({ url: '/pages/publish/index' }) }
function goLogin()    { uni.navigateTo({ url: '/pages/login/index' }) }
function goFavorites() { uni.navigateTo({ url: '/pages/favorites/index' }) }
function goHistory() { uni.navigateTo({ url: '/pages/view-history/index' }) }
function goMyDynamics() { uni.navigateTo({ url: '/pages/my-dynamics/index' }) }
function goSpace() { uni.navigateTo({ url: `/pages/space/index?id=${user.value?.user_id || ''}` }) }

function toggleTheme() {
  uni.showToast({ title: '主题切换开发中', icon: 'none' })
}
</script>

<style lang="scss" scoped>
.mine-page {
  min-height: 100vh;
  background: #FFF;
  padding-bottom: 140rpx;
}

.top-actions {
  display: flex;
  justify-content: flex-end;
  gap: 16rpx;
  padding: 24rpx;
  .top-icon {
    width: 64rpx; height: 64rpx;
    background: rgba(255,255,255,0.2);
    border-radius: 16rpx;
    display: flex; align-items: center; justify-content: center;
    font-size: 32rpx;
  }
}

.user-info {
  padding: 32rpx 24rpx 24rpx;
}

/* 顶部行：头像 + 信息 + 空间 */
.info-top {
  display: flex;
  align-items: flex-start;
  gap: 24rpx;
}

.avatar-wrap {
  position: relative;
  width: 140rpx;
  height: 140rpx;
  flex-shrink: 0;

  .avatar {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    object-fit: cover;
    display: block;
    border: 2rpx solid #FFF;
    background: #F4F4F4;
  }
}

.info-meta {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12rpx;
  padding-top: 4rpx;

  .row-1 {
    display: flex;
    align-items: center;
    gap: 12rpx;
  }
  .nickname {
    font-size: 36rpx;
    font-weight: 600;
    color: #181818;
  }
  .edit-icon {
    color: #999;
    font-size: 28rpx;
  }
  .lv-badge {
    background: #F25C0F;
    color: #FFF;
    font-size: 20rpx;
    padding: 2rpx 10rpx;
    border-radius: 6rpx;
    font-weight: 500;
    font-family: 'Helvetica', sans-serif;
    letter-spacing: 1rpx;
  }
  .vip-tag {
    display: inline-block;
    width: fit-content;
    color: #FB7299;
    font-size: 22rpx;
    padding: 4rpx 16rpx;
    border: 2rpx solid #FB7299;
    border-radius: 6rpx;
    background: rgba(251, 114, 153, 0.05);
  }
  .coin-inline {
    display: flex;
    gap: 24rpx;
    color: #999;
    font-size: 24rpx;
    margin-top: 4rpx;
  }
}

.space-link {
  width: 96rpx;
  flex-shrink: 0;
  text-align: right;
  font-size: 26rpx;
  color: #999;
  padding-top: 16rpx;
}

/* 下方 3 列统计 + 竖线分隔 */
.stats-row {
  display: flex;
  align-items: center;
  margin-top: 32rpx;
  padding: 24rpx 80rpx;
  border-top: 1rpx solid #F1F1F1;

  .stat-cell {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8rpx;
    .num   { font-size: 36rpx; font-weight: 600; color: #181818; }
    .label { font-size: 22rpx; color: #999; }
  }
  .stat-divider {
    width: 1rpx;
    height: 48rpx;
    background: #E5E5E5;
  }
}

.quick-row {
  display: flex;
  margin: 24rpx 16rpx;
  background: #FFF;
  border-radius: 16rpx;
  padding: 32rpx 0;

  .quick-cell {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12rpx;
    font-size: 24rpx;
    color: #333;

    .quick-icon-wrap {
      position: relative;
      width: 64rpx; height: 64rpx;
      border-radius: 50%;
      display: flex; align-items: center; justify-content: center;
      font-size: 32rpx;
    }
  }
}

.badge-dot {
  position: absolute;
  top: -4rpx; right: -4rpx;
  background: #F56C6C;
  color: #FFF;
  font-size: 18rpx;
  min-width: 28rpx;
  height: 28rpx;
  line-height: 28rpx;
  text-align: center;
  border-radius: 14rpx;
  padding: 0 6rpx;
}

.creator-section {
  margin: 16rpx 16rpx;
  padding: 24rpx;
  background: #FFF;
  border-radius: 16rpx;
}

.creator-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 16rpx;

  .title { font-size: 30rpx; font-weight: 600; }
  .more  { font-size: 22rpx; color: #999; }

  .publish-btn {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 8rpx 24rpx;
    border-radius: 24rpx;
    font-size: 24rpx;
    display: flex; align-items: center; gap: 8rpx;
  }
}

.grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24rpx 8rpx;

  &.grid-4 { margin-top: 8rpx; }

  .grid-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12rpx;

    .icon-wrap {
      position: relative;
      width: 72rpx; height: 72rpx;
      border-radius: 20rpx;
      display: flex; align-items: center; justify-content: center;
      font-size: 36rpx;
    }
    .label { font-size: 22rpx; color: #333; }
  }
}

.more-row {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 20rpx 8rpx;
  border-bottom: 1rpx solid #F1F1F1;

  &:last-child { border-bottom: none; }

  .more-icon {
    width: 56rpx; height: 56rpx;
    border-radius: 16rpx;
    display: flex; align-items: center; justify-content: center;
    font-size: 28rpx;
  }
  .more-label { flex: 1; font-size: 28rpx; }
  .arrow { color: #C0C4CC; }
}

.empty-mine {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: #FFF;
  gap: 24rpx;

  .empty-avatar {
    width: 160rpx; height: 160rpx;
    border-radius: 50%;
    background: #F4F4F4;
    display: flex; align-items: center; justify-content: center;
    font-size: 80rpx;
  }
  .empty-tip { font-size: 30rpx; color: #999; }
  .login-btn {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 20rpx 80rpx;
    border-radius: 40rpx;
    font-size: 30rpx;
  }
}
</style>
