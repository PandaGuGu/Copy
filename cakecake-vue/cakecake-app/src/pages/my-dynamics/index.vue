<template>
  <view class="my-dyn-page">
    <view class="page-header safe-area-top">
      <text class="back-icon" @tap="goBack">‹</text>
      <text class="title">我的动态</text>
      <text class="publish-btn" @tap="goPublish">发布 ›</text>
    </view>

    <scroll-view scroll-y class="content">
      <view v-if="loading && items.length === 0" class="loading"><text>加载中…</text></view>
      <view v-else-if="items.length === 0" class="empty">
        <text class="icon">📝</text>
        <text>还没有发过动态</text>
        <view class="empty-btn" @tap="goPublish"><text>去发一条</text></view>
      </view>

      <view v-else class="list">
        <view v-for="d in items" :key="d.id" class="dyn-card">
          <view class="dyn-head">
            <img class="avatar" :src="d.author_avatar || '/static/avatar/default.png'" referrerpolicy="no-referrer" />
            <view class="meta">
              <text class="name">{{ d.author_name || '我' }}</text>
              <text class="time">{{ formatTime(d.created_at) }}</text>
            </view>
          </view>

          <view v-if="d.title" class="dyn-title">{{ d.title }}</view>
          <text class="dyn-content">{{ d.content }}</text>

          <view v-if="d.images && d.images.length" class="dyn-imgs">
            <img
              v-for="(img, i) in d.images.slice(0, 3)"
              :key="i"
              :src="img"
              class="dyn-img"
              referrerpolicy="no-referrer"
            />
          </view>

          <view class="dyn-actions">
            <text class="act" :class="{ liked: d.liked_by_me }" @tap="toggleLike(d)">
              {{ d.liked_by_me ? '❤️' : '👍' }} {{ d.like_count }}
            </text>
            <text class="act" @tap="toggleCommentBox(d)">💬 {{ d.comment_count }}</text>
          </view>

          <!-- 评论弹层 -->
          <view v-if="commentBoxId === d.id" class="comment-box">
            <view class="comment-list">
              <view v-for="c in commentMap[d.id] || []" :key="c.id" class="comment-row">
                <text class="c-user">{{ c.username }}：</text>
                <text class="c-content">{{ c.content }}</text>
              </view>
              <view v-if="!(commentMap[d.id] || []).length" class="c-empty">暂无评论</view>
            </view>
            <view class="comment-input-row">
              <input
                v-model="commentText"
                class="comment-input"
                placeholder="说点什么…"
                confirm-type="send"
                @confirm="submitComment(d)"
              />
              <view class="comment-send" @tap="submitComment(d)"><text>发送</text></view>
            </view>
          </view>
        </view>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { dynamicApi } from '@/api'
import type { Comment, Dynamic } from '@/utils/types'
import { useUserStore } from '@/store/user'

const userStore = useUserStore()
const items = ref<Dynamic[]>([])
const loading = ref(false)
const commentBoxId = ref<number | null>(null)
const commentText = ref('')
const commentMap = ref<Record<number, Comment[]>>({})

onShow(load)

async function load() {
  if (loading.value) return
  loading.value = true
  try {
    const resp = await dynamicApi.mine(30)
    items.value = resp.items || []
  } finally {
    loading.value = false
  }
}

async function toggleLike(d: Dynamic) {
  if (!userStore.isLoggedIn) { goLogin(); return }
  try {
    const r = await dynamicApi.toggleLike(d.id)
    d.liked_by_me = r.liked
    d.like_count = Math.max(0, d.like_count + r.like_count_delta)
  } catch { /* 已 toast */ }
}

async function toggleCommentBox(d: Dynamic) {
  if (commentBoxId.value === d.id) {
    commentBoxId.value = null
    return
  }
  commentBoxId.value = d.id
  commentText.value = ''
  // 拉评论
  if (!commentMap.value[d.id]) {
    try {
      const resp = await dynamicApi.comments(d.id)
      commentMap.value[d.id] = resp.items || []
    } catch { commentMap.value[d.id] = [] }
  }
}

async function submitComment(d: Dynamic) {
  const c = commentText.value.trim()
  if (!c) { uni.showToast({ title: '请输入内容', icon: 'none' }); return }
  try {
    await dynamicApi.createComment(d.id, c)
    uni.showToast({ title: '评论成功', icon: 'success' })
    commentText.value = ''
    d.comment_count += 1
    const resp = await dynamicApi.comments(d.id)
    commentMap.value[d.id] = resp.items || []
  } catch { /* 已 toast */ }
}

function formatTime(s: string): string {
  if (!s) return ''
  const diff = Date.now() - new Date(s).getTime()
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  return s.slice(0, 10)
}

function goBack()    { uni.navigateBack() }
function goPublish() { uni.navigateTo({ url: '/pages/publish/index' }) }
function goLogin()   { uni.navigateTo({ url: '/pages/login/index' }) }
</script>

<style lang="scss" scoped>
.my-dyn-page { min-height: 100vh; background: #FFF; }

.page-header {
  display: flex;
  align-items: center;
  padding: 24rpx;
  background: #FFF;
  border-bottom: 1rpx solid #F1F1F1;

  .back-icon { font-size: 48rpx; width: 60rpx; }
  .title { flex: 1; text-align: center; font-size: 32rpx; font-weight: 500; }
  .publish-btn { font-size: 26rpx; color: #FB7299; }
}

.content { height: calc(100vh - 100rpx); }

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

.list { padding: 0 24rpx; }

.dyn-card {
  padding: 24rpx 0;
  border-bottom: 1rpx solid #F1F1F1;

  &:active { background: #FAFAFA; }
}

.dyn-head {
  display: flex;
  align-items: center;
  gap: 16rpx;

  .avatar {
    width: 72rpx; height: 72rpx;
    border-radius: 50%;
    background: #F0F0F0;
  }
  .meta {
    flex: 1;
    .name { display: block; font-size: 26rpx; color: #FB7299; font-weight: 500; }
    .time { display: block; font-size: 22rpx; color: #999; margin-top: 2rpx; }
  }
}

.dyn-title {
  margin-top: 16rpx;
  font-size: 28rpx;
  color: #181818;
  font-weight: 500;
}

.dyn-content {
  display: block;
  margin-top: 8rpx;
  font-size: 26rpx;
  color: #333;
  line-height: 1.5;
  word-break: break-all;
}

.dyn-imgs {
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

.dyn-actions {
  display: flex;
  justify-content: flex-end;
  gap: 40rpx;
  margin-top: 16rpx;
  font-size: 24rpx;
  color: #666;

  .act.liked { color: #FB7299; }
}

.comment-box {
  margin-top: 16rpx;
  background: #F8F8F8;
  border-radius: 12rpx;
  padding: 16rpx;
}

.comment-list {
  .comment-row {
    padding: 8rpx 0;
    font-size: 24rpx;
    color: #333;
    .c-user { color: #FB7299; }
  }
  .c-empty { text-align: center; color: #999; padding: 16rpx 0; font-size: 22rpx; }
}

.comment-input-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
  margin-top: 12rpx;

  .comment-input {
    flex: 1;
    height: 60rpx;
    background: #FFF;
    border-radius: 30rpx;
    padding: 0 20rpx;
    font-size: 24rpx;
  }
  .comment-send {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 10rpx 24rpx;
    border-radius: 30rpx;
    font-size: 24rpx;
  }
}
</style>