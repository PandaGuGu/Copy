<template>
  <view class="custom-tab-bar safe-area-bottom">
    <view class="tab-item" @tap="switchTab('/pages/index/index', 0)">
      <img class="tab-icon" :src="current === 0 ? '/static/tabbar/home_sel.png' : '/static/tabbar/home.png'" referrerpolicy="no-referrer" />
      <text class="tab-label" :class="{ active: current === 0 }">首页</text>
    </view>

    <view class="tab-item" @tap="switchTab('/pages/follow/index', 1)">
      <img class="tab-icon" :src="current === 1 ? '/static/tabbar/follow_sel.png' : '/static/tabbar/follow.png'" referrerpolicy="no-referrer" />
      <text class="tab-label" :class="{ active: current === 1 }">关注</text>
    </view>

    <!-- 中间 + 号发布按钮（B站风格：粉色大按钮突出） -->
    <view class="tab-center" @tap="goPublish">
      <view class="center-btn">
        <text class="center-plus">＋</text>
      </view>
    </view>

    <view class="tab-item" @tap="switchTab('/pages/mall/index', 2)">
      <img class="tab-icon" :src="current === 2 ? '/static/tabbar/mall_sel.png' : '/static/tabbar/mall.png'" referrerpolicy="no-referrer" />
      <text class="tab-label" :class="{ active: current === 2 }">会员购</text>
    </view>

    <view class="tab-item" @tap="switchTab('/pages/mine/index', 3)">
      <img class="tab-icon" :src="current === 3 ? '/static/tabbar/mine_sel.png' : '/static/tabbar/mine.png'" referrerpolicy="no-referrer" />
      <text class="tab-label" :class="{ active: current === 3 }">我的</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/store/app'

const appStore = useAppStore()
const current = computed(() => appStore.currentTab)

function switchTab(url: string, idx: number) {
  appStore.currentTab = idx
  uni.switchTab({ url })
}

function goPublish() {
  uni.navigateTo({ url: '/pages/publish/index' })
}
</script>

<style lang="scss" scoped>
.custom-tab-bar {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 999;
  display: flex;
  align-items: center;
  height: 100rpx;
  background-color: #FFFFFF;
  border-top: 1rpx solid #F1F1F1;
  padding-bottom: env(safe-area-inset-bottom);
}

.tab-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4rpx;
}

.tab-icon {
  width: 48rpx;
  height: 48rpx;
}

.tab-label {
  font-size: 20rpx;
  color: #999999;

  &.active {
    color: #FB7299;
    font-weight: 500;
  }
}

.tab-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.center-btn {
  width: 88rpx;
  height: 88rpx;
  margin-top: -36rpx;              /* 向上凸出，B站风格 */
  border-radius: 28rpx;
  background: linear-gradient(135deg, #FB7299, #FF9DB5);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 6rpx 16rpx rgba(251, 114, 153, 0.35);
}

.center-plus {
  font-size: 56rpx;
  font-weight: 500;
  color: #FFFFFF;
  line-height: 1;
}
</style>