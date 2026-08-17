<template>
  <view class="publish-page">
    <view class="top-bar safe-area-top">
      <text class="close">×</text>
      <text class="title">最近项目</text>
      <text class="drafts">草稿箱</text>
    </view>

    <view class="tabs">
      <text :class="{ active: tab === 'all' }" @tap="tab = 'all'">全部</text>
      <text :class="{ active: tab === 'video' }" @tap="tab = 'video'">视频</text>
      <text :class="{ active: tab === 'photo' }" @tap="tab = 'photo'">照片</text>
    </view>
    <view class="tab-underline">
      <view class="underline-bar" :style="{ transform: `translateX(${tabIndex * 100}%)` }" />
    </view>

    <view class="grid">
      <view v-for="(item, i) in items" :key="i" class="grid-item" @tap="onItem(i)">
        <image :src="item.thumb" mode="aspectFill" />
        <view class="duration">{{ item.duration }}</view>
        <view v-if="i === 0" class="plus-circle">+</view>
      </view>
      <view class="grid-item more" @tap="onMore">
        <image :src="items[1].thumb" mode="aspectFill" />
        <text class="more-count">+ 更多项目</text>
      </view>
    </view>

    <view class="bottom-bar safe-area-bottom">
      <view class="action" @tap="onShoot">
        <view class="action-icon"><text>📷</text></view>
        <text>拍摄</text>
      </view>
      <view class="action action-primary" @tap="onUpload">
        <text :class="{ disabled: uploading }">{{ uploading ? '上传中…' : '上传' }}</text>
        <view class="underline-active" />
      </view>
      <view class="action" @tap="onLive">
        <view class="action-icon"><text>📡</text></view>
        <text>开直播</text>
      </view>
      <view class="action" @tap="onTextDraft">
        <view class="action-icon"><text>📝</text></view>
        <text>发图文</text>
      </view>
    </view>

    <!-- 发图文弹窗 -->
    <view v-if="textDialog" class="dialog-mask" @tap="textDialog = false">
      <view class="dialog" @tap.stop>
        <view class="dialog-title">发动态</view>
        <textarea
          v-model="textContent"
          class="dialog-textarea"
          placeholder="说点什么…（最长 233 字）"
          maxlength="233"
          auto-height
        />
        <view class="dialog-actions">
          <text class="btn-cancel" @tap="textDialog = false">取消</text>
          <text class="btn-submit" :class="{ disabled: submitting }" @tap="submitText">{{ submitting ? '发布中…' : '发布' }}</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useUserStore } from '@/store/user'
import { useAppStore } from '@/store/app'
import { uploadApi } from '@/api/upload'
import { dynamicApi } from '@/api'
import { onShow } from '@dcloudio/uni-app'

type Tab = 'all' | 'video' | 'photo'
const tab = ref<Tab>('all')
const tabIndex = computed(() => ['all', 'video', 'photo'].indexOf(tab.value))

const items = [
  { thumb: '/static/demo/video1.jpg', duration: '00:35' },
  { thumb: '/static/demo/video2.jpg', duration: '00:05' },
  { thumb: '/static/demo/video3.jpg', duration: '00:36' }
]

const userStore = useUserStore()
const appStore = useAppStore()

// 上传相关
const selectedVideo = ref<{ path: string; duration: number } | null>(null)
const uploading = ref(false)

// 发图文弹窗
const textDialog = ref(false)
const textContent = ref('')
const submitting = ref(false)

onShow(() => {
  appStore.currentTab = -1  // publish 是 midButton，不在 4 tab 内
  userStore.refreshMe()
})

function onItem(i: number) {
  uni.showToast({ title: `选择项目 ${i + 1}（草稿上传待完善）`, icon: 'none' })
}
function onMore() { uni.showToast({ title: '草稿箱开发中', icon: 'none' }) }

function requireLogin(action: string): boolean {
  if (!userStore.isLoggedIn) {
    uni.showToast({ title: '请先登录后再' + action, icon: 'none' })
    setTimeout(() => uni.navigateTo({ url: '/pages/login/index' }), 800)
    return false
  }
  return true
}

function onShoot() {
  uni.showToast({ title: '拍摄功能需在 App 端使用（H5 无相机权限）', icon: 'none' })
}

async function onUpload() {
  if (!requireLogin('上传视频')) return
  // 选视频
  const res: any = await new Promise((resolve) => {
    uni.chooseVideo({
      sourceType: ['album'],
      maxDuration: 60,
      success: (r) => resolve(r),
      fail: () => resolve(null)
    })
  })
  if (!res || !res.tempFilePath) {
    uni.showToast({ title: '未选择视频', icon: 'none' })
    return
  }
  selectedVideo.value = { path: res.tempFilePath, duration: res.duration || 0 }
  // 输入标题
  const titleRes: any = await new Promise((resolve) => {
    uni.showModal({
      title: '上传视频',
      editable: true,
      placeholderText: '请输入视频标题（1-80字）',
      success: (r) => resolve(r)
    })
  })
  if (!titleRes.confirm) return
  const title = (titleRes.content || '').trim()
  if (title.length < 1 || title.length > 80) {
    uni.showToast({ title: '标题须 1-80 字', icon: 'none' })
    return
  }
  // 上传
  uploading.value = true
  uni.showLoading({ title: '上传中…', mask: true })
  try {
    const result = await uploadApi.uploadVideo(userStore.token, selectedVideo.value.path, title)
    uni.hideLoading()
    uni.showToast({ title: `上传成功 #${result.id}（${result.status}）`, icon: 'success' })
    selectedVideo.value = null
  } catch (err) {
    uni.hideLoading()
    uni.showToast({ title: (err as Error).message || '上传失败', icon: 'none' })
  } finally {
    uploading.value = false
  }
}

function onLive() {
  uni.showToast({ title: '开直播需在 App 端（推流 SRS）', icon: 'none' })
}

function onTextDraft() {
  if (!requireLogin('发图文')) return
  textContent.value = ''
  textDialog.value = true
}

async function submitText() {
  const c = textContent.value.trim()
  if (c.length < 1) {
    uni.showToast({ title: '内容不能为空', icon: 'none' })
    return
  }
  submitting.value = true
  try {
    const d = await dynamicApi.createText(userStore.token, c)
    uni.showToast({ title: `动态已发布 #${d.id}`, icon: 'success' })
    textDialog.value = false
  } catch {
    /* 已 toast */
  } finally {
    submitting.value = false
  }
}
</script>

<style lang="scss" scoped>
.publish-page {
  min-height: 100vh;
  background: #000;
  color: #FFF;
}

.top-bar {
  display: flex;
  align-items: center;
  padding: 24rpx;
  .close { width: 60rpx; font-size: 40rpx; }
  .title { flex: 1; text-align: center; font-size: 32rpx; font-weight: 500; }
  .drafts { font-size: 26rpx; color: #FB7299; }
}

.tabs {
  display: flex;
  justify-content: center;
  gap: 80rpx;
  padding: 24rpx 0;
  font-size: 30rpx;
  color: #999;
  .active { color: #FFF; font-weight: 600; }
}
.tab-underline {
  position: relative;
  height: 4rpx;
  width: 100%;
  .underline-bar {
    position: absolute;
    left: 33.33%;
    width: 33.33%;
    bottom: 0;
    height: 4rpx;
    background: #FB7299;
    transition: transform 0.2s;
  }
}

.grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8rpx;
  padding: 16rpx;
}

.grid-item {
  position: relative;
  aspect-ratio: 9 / 16;
  border-radius: 8rpx;
  overflow: hidden;
  background: #222;

  image { width: 100%; height: 100%; }

  .duration {
    position: absolute;
    bottom: 8rpx; right: 8rpx;
    font-size: 20rpx;
    background: rgba(0,0,0,0.6);
    padding: 2rpx 8rpx;
    border-radius: 4rpx;
  }

  .plus-circle {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 80rpx;
    height: 80rpx;
    background: rgba(255,255,255,0.2);
    color: #FFF;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 60rpx;
  }
}

.more {
  .more-count {
    position: absolute;
    inset: 0;
    margin: auto;
    text-align: center;
    line-height: 80rpx;
    color: #FFF;
    font-size: 24rpx;
  }
}

.bottom-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-around;
  padding: 32rpx 0;
  background: rgba(0,0,0,0.95);
  border-top: 1rpx solid #333;

  .action {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8rpx;
    font-size: 24rpx;
    color: #FFF;
    position: relative;

    .action-icon {
      width: 80rpx;
      height: 80rpx;
      border-radius: 50%;
      background: #333;
      display: flex; align-items: center; justify-content: center;
      font-size: 40rpx;
    }
    .underline-active {
      position: absolute;
      bottom: -10rpx;
      width: 60rpx;
      height: 6rpx;
      background: #FB7299;
      border-radius: 3rpx;
    }
  }
  .action-primary {
    color: #FB7299;

    .disabled { color: #999; }
  }
}

.dialog-mask {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.dialog {
  width: 80%;
  max-width: 600rpx;
  background: #1C1C1C;
  border-radius: 16rpx;
  padding: 32rpx;
  color: #FFF;

  .dialog-title {
    font-size: 32rpx;
    font-weight: 500;
    margin-bottom: 24rpx;
  }
  .dialog-textarea {
    width: 100%;
    min-height: 200rpx;
    background: #2A2A2A;
    color: #FFF;
    font-size: 28rpx;
    padding: 16rpx;
    border-radius: 8rpx;
    box-sizing: border-box;
  }
  .dialog-actions {
    margin-top: 24rpx;
    display: flex;
    justify-content: flex-end;
    gap: 32rpx;
    font-size: 28rpx;
    .btn-cancel { color: #999; padding: 8rpx 16rpx; }
    .btn-submit { color: #FB7299; padding: 8rpx 32rpx; font-weight: 500; }
    .btn-submit.disabled { color: #999; }
  }
}
</style>