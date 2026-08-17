<template>
  <view class="categories-page">
    <view class="page-header safe-area-top">
      <text class="back-icon" @tap="goBack">‹</text>
      <text class="title">分区</text>
    </view>

    <!-- 快捷访问 -->
    <view class="section">
      <view class="section-title">
        <text>快捷访问</text>
      </view>
      <view class="shortcut-bar" @tap="editShortcuts">
        <text class="plus">+</text>
        <text>编辑</text>
      </view>
    </view>

    <!-- 全部分区 -->
    <view class="section">
      <view class="section-title">
        <text>全部分区</text>
      </view>
      <view v-if="loading" class="loading"><text>加载中...</text></view>
      <view v-else class="grid">
        <view
          v-for="(c, i) in categories"
          :key="c.name"
          class="grid-item"
          @tap="goCategory(c)"
        >
          <view class="icon-wrap" :style="{ background: pickColor(i) }">
            <text class="icon-text">{{ c.name.slice(0, 1) }}</text>
          </view>
          <text class="label">{{ c.name }}</text>
          <text class="count">{{ c.video_count }}个视频</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { categoryApi, type Zone } from '@/api/category'

const categories = ref<Zone[]>([])
const loading = ref(false)

const COLORS = ['#FFE4EC', '#E1F5EE', '#E6F1FB', '#FAEEDA', '#FBEAF0', '#EAF3DE', '#FAECE7', '#E1F5EE']

onMounted(loadCategories)

async function loadCategories() {
  loading.value = true
  try {
    const resp = await categoryApi.all()
    categories.value = resp.items || []
  } finally {
    loading.value = false
  }
}

function pickColor(i: number): string {
  return COLORS[i % COLORS.length]
}

function goCategory(c: Zone) {
  uni.navigateTo({ url: `/pages/zone/index?zone=${encodeURIComponent(c.name)}` })
}

function editShortcuts() {
  uni.showToast({ title: '编辑快捷入口', icon: 'none' })
}

function goBack() { uni.navigateBack() }
</script>

<style lang="scss" scoped>
.categories-page {
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
}

.section { padding: 24rpx 16rpx; }

.section-title {
  padding: 16rpx 8rpx;
  font-size: 30rpx;
  font-weight: 500;
}

.shortcut-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12rpx;
  padding: 24rpx;
  background: #F7F7F7;
  border-radius: 12rpx;
  color: #666;
  font-size: 28rpx;

  .plus { font-size: 36rpx; }
}

.grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24rpx 8rpx;
  margin-top: 16rpx;
}

.grid-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12rpx;

  .icon-wrap {
    width: 80rpx;
    height: 80rpx;
    border-radius: 24rpx;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .icon-text { font-size: 32rpx; font-weight: 600; color: #333; }
  .label { font-size: 22rpx; color: #333; }
  .count { font-size: 18rpx; color: #999; margin-top: -6rpx; }
}

.loading {
  text-align: center;
  padding: 60rpx 0;
  color: #999;
}
</style>