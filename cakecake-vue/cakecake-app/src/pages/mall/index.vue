<template>
  <view class="mall-page">
    <!-- 顶部搜索 -->
    <view class="top-search safe-area-top">
      <view class="search-wrap">
        <text class="search-icon">🔍</text>
        <text class="placeholder">{{ hotWord || '通行证明日方舟52' }}</text>
      </view>
      <view class="search-btn">搜索</view>
    </view>

    <!-- 订单入口 -->
    <scroll-view scroll-x class="entry-row" :show-scrollbar="false">
      <view v-for="e in entries" :key="e.label" class="entry" @tap="goEntry(e)">
        <view class="entry-icon" :style="{ background: e.color }">
          <text>{{ e.icon }}</text>
        </view>
        <text class="entry-label">{{ e.label }}</text>
      </view>
      <view class="entry" @tap="goEntry({ label: '商品' })">
        <view class="entry-icon" style="background: #E1F5EE;"><text>📦</text></view>
        <text class="entry-label">商品</text>
      </view>
    </scroll-view>

    <!-- 标签云 -->
    <scroll-view scroll-x class="tag-row" :show-scrollbar="false">
      <view v-for="t in tags" :key="t" class="tag" :class="{ active: tagActive.includes(t) }" @tap="toggleTag(t)">{{ t }}</view>
      <view class="tag tag-all">
        <text>🎨</text>
        <text>88 全部</text>
      </view>
    </scroll-view>

    <!-- 2x2 活动方块 -->
    <view class="promo-grid">
      <view v-for="p in promos" :key="p.title" class="promo" :style="{ background: p.bg }">
        <view class="promo-top">
          <text class="promo-tag">{{ p.tag }}</text>
        </view>
        <image class="promo-img" :src="p.image" mode="aspectFit" />
        <view class="promo-meta">
          <text class="promo-title">{{ p.title }}</text>
          <text class="promo-cta">立即参与 ›</text>
        </view>
      </view>
    </view>

    <!-- 限时活动 2 张 -->
    <view class="flash-grid">
      <view v-for="f in flashSales" :key="f.id" class="flash-card">
        <view class="flash-cover">
          <image :src="f.cover" mode="aspectFill" />
          <view class="badge">限时抢购</view>
          <view class="discount">立享{{ f.discount }}</view>
          <view class="time">{{ f.time }}</view>
        </view>
        <text class="flash-title">{{ f.title }}</text>
        <text class="flash-price">到手价 ¥<text class="price-num">{{ f.price }}</text></text>
      </view>
    </view>

    <CustomTabBar />
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { useAppStore } from '@/store/app'
import CustomTabBar from '@/custom-tab-bar/index.vue'

onShow(() => {
  useAppStore().currentTab = 2
})

const hotWord = ref('通行证明日方舟52')
const tagActive = ref<string[]>(['手办雕像', '漫展演出', '漫画', '可动'])
const tags = ['手办雕像', '漫展演出', '漫画', '可动']

const entries = [
  { label: '我的订单', icon: '📄', color: '#FFE4EC' },
  { label: '购物车', icon: '🛒', color: '#E1F5EE' },
  { label: '优惠券', icon: '🎟', color: '#E6F1FB' },
  { label: '我的文件', icon: '📁', color: '#FAEEDA' },
  { label: '商品收藏', icon: '⭐', color: '#FBEAF0' }
]

const promos = [
  { tag: '同好圈', title: 'MINISO凡人新品独家开抢限量满赠！', image: '/static/mall/promo1.png', bg: 'linear-gradient(135deg,#FFE4EC,#FFB6C1)' },
  { tag: '今日上新', title: '上线1380', image: '/static/mall/promo2.png', bg: 'linear-gradient(135deg,#E1F5EE,#9FE1CB)' },
  { tag: '我的世界', title: '热门爆款', image: '/static/mall/promo3.png', bg: 'linear-gradient(135deg,#E6F1FB,#85B7EB)' },
  { tag: '欧气盲盒', title: '¥10/抽', image: '/static/mall/promo4.png', bg: 'linear-gradient(135deg,#FAEEDA,#EF9F27)' }
]

const flashSales = [
  { id: 1, cover: '/static/mall/flash1.jpg', title: '自营 Hapitopi & hitomi 独家特典', price: '124.5', discount: '5折', time: '8.14 20:20' },
  { id: 2, cover: '/static/mall/flash2.jpg', title: '经典爆款 废弃工厂', price: '89', discount: '7.5折', time: '8.10 00:00' }
]

function toggleTag(t: string) {
  const i = tagActive.value.indexOf(t)
  if (i >= 0) tagActive.value.splice(i, 1)
  else tagActive.value.push(t)
}
function goEntry(e: { label: string }) {
  uni.showToast({ title: `${e.label} - 待开发`, icon: 'none' })
}
</script>

<style lang="scss" scoped>
.mall-page {
  min-height: 100vh;
  background: #FFF;
  padding-bottom: 140rpx;
}

.top-search {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 16rpx 24rpx;

  .search-wrap {
    flex: 1;
    height: 64rpx;
    background: #F4F4F4;
    border: 2rpx solid #FB7299;
    border-radius: 32rpx;
    display: flex;
    align-items: center;
    padding: 0 24rpx;
    gap: 12rpx;
    font-size: 26rpx;
    color: #666;
  }
  .search-btn {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 12rpx 28rpx;
    border-radius: 32rpx;
    font-size: 26rpx;
  }
}

.entry-row {
  white-space: nowrap;
  padding: 16rpx 8rpx;
  .entry {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    width: 140rpx;
    gap: 8rpx;
    .entry-icon {
      width: 80rpx; height: 80rpx;
      border-radius: 24rpx;
      display: flex; align-items: center; justify-content: center;
      font-size: 36rpx;
    }
    .entry-label { font-size: 22rpx; color: #333; }
  }
}

.tag-row {
  white-space: nowrap;
  padding: 16rpx 8rpx;
  border-bottom: 1rpx solid #F1F1F1;

  .tag {
    display: inline-block;
    padding: 12rpx 28rpx;
    margin-right: 16rpx;
    background: #F4F4F4;
    border-radius: 28rpx;
    font-size: 26rpx;
    color: #333;

    &.active {
      background: linear-gradient(90deg, #FFE4EC, #FFB6C1);
      color: #FB7299;
    }
    &.tag-all { background: #FFF; display: inline-flex; align-items: center; gap: 4rpx; }
  }
}

.promo-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16rpx;
  padding: 24rpx 16rpx;

  .promo {
    border-radius: 16rpx;
    padding: 16rpx;
    position: relative;
    min-height: 240rpx;
  }
  .promo-top { margin-bottom: 8rpx; }
  .promo-tag {
    font-size: 22rpx;
    color: #FFF;
    background: rgba(0,0,0,0.2);
    padding: 4rpx 12rpx;
    border-radius: 16rpx;
  }
  .promo-img {
    width: 100%;
    height: 160rpx;
  }
  .promo-meta {
    margin-top: 12rpx;
  }
  .promo-title {
    display: block;
    font-size: 24rpx;
    color: #181818;
    font-weight: 500;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .promo-cta {
    display: block;
    margin-top: 8rpx;
    font-size: 22rpx;
    color: #FB7299;
  }
}

.flash-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16rpx;
  padding: 0 16rpx 32rpx;

  .flash-card {
    .flash-cover {
      position: relative;
      width: 100%;
      aspect-ratio: 1;
      border-radius: 16rpx;
      overflow: hidden;
      image { width: 100%; height: 100%; }
      .badge {
        position: absolute; top: 8rpx; left: 8rpx;
        background: rgba(0,0,0,0.6);
        color: #FFF; font-size: 20rpx;
        padding: 4rpx 12rpx; border-radius: 4rpx;
      }
      .discount {
        position: absolute; top: 70rpx; left: 8rpx;
        background: linear-gradient(90deg, #FB7299, #FF9DB5);
        color: #FFF; font-size: 24rpx;
        padding: 4rpx 12rpx; border-radius: 4rpx;
      }
      .time {
        position: absolute; bottom: 8rpx; left: 8rpx;
        color: #FFD700; font-size: 20rpx;
      }
    }
    .flash-title {
      display: block;
      margin-top: 8rpx;
      font-size: 24rpx;
      color: #181818;
      line-height: 1.3;
      overflow: hidden;
      text-overflow: ellipsis;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
    }
    .flash-price {
      margin-top: 8rpx;
      font-size: 22rpx;
      color: #666;
      .price-num { font-size: 32rpx; color: #FB7299; font-weight: 600; }
    }
  }
}
</style>