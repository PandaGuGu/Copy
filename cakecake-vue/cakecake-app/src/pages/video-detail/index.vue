<template>
  <view class="detail-page">
    <view v-if="loading" class="loading-mask"><text>加载中…</text></view>

    <!-- 顶部固定返回按钮 -->
    <view class="floating-bar safe-area-top">
      <text class="back" @tap="goBack">‹</text>
      <text class="title text-ellipsis-1">{{ video?.title || '视频详情' }}</text>
      <text class="more" @tap="onMore">⋯</text>
    </view>

    <!-- 视频播放器（B 站风格：自定义控制条） -->
    <view class="player">
      <video
        v-if="video"
        :id="'vd-' + video.id"
        :src="playUrl"
        :poster="video.cover_url"
        :controls="false"
        show-fullscreen-btn
        :show-center-play-btn="false"
        object-fit="contain"
        @play="onPlay"
        @pause="onPause"
        @timeupdate="onTimeUpdate"
        @ended="onEnded"
        @loadedmetadata="onLoadedMeta"
        @fullscreenchange="onFullscreenChange"
        style="width:100%;height:100%;"
      />
      <!-- 封面遮罩（未播放时显示） -->
      <view v-if="!playing && !userStarted" class="player-cover" @tap="onTogglePlay">
        <view class="play-btn"><text>▶</text></view>
        <text class="cover-tip">点击播放</text>
      </view>
      <!-- 控制条 -->
      <view v-if="userStarted" class="controls" @tap.stop>
        <view class="ctrl-row">
          <view class="ctrl-btn" @tap="onTogglePlay">
            <text>{{ playing ? '❚❚' : '▶' }}</text>
          </view>
          <text class="time">{{ formatClock(currentTime) }} / {{ formatClock(duration) }}</text>
          <view class="spacer" />
          <view class="ctrl-btn speed" @tap="onCycleSpeed">
            <text>{{ speed }}x</text>
          </view>
          <view class="ctrl-btn" @tap="onFullscreen">
            <text>⛶</text>
          </view>
        </view>
        <!-- 进度条 -->
        <view class="progress-bar" @tap="onSeekTap">
          <view class="progress-fill" :style="{ width: progressPct + '%' }" />
          <view class="progress-thumb" :style="{ left: progressPct + '%' }" />
        </view>
      </view>
    </view>

    <!-- UP主信息 -->
    <view v-if="video" class="up-card">
      <image class="avatar" :src="video.uploader_avatar_url || '/static/avatar/default.png'" mode="aspectFill" @tap="goSpace" />
      <view class="up-meta" @tap="goSpace">
        <view class="row1">
          <text class="nickname">{{ video.uploader }}</text>
          <text class="fans">{{ formatCount(video.uploader_follower_count || 0) }}粉丝</text>
        </view>
        <text class="intro">共 {{ video.uploader_published_count || 0 }} 个视频</text>
      </view>
      <view class="follow-btn" @tap="onFollow">
        <text>{{ followed ? '已关注' : '+ 关注' }}</text>
      </view>
    </view>

    <!-- 标题区 -->
    <view v-if="video" class="title-section">
      <text class="title">{{ video.title }}</text>
      <text class="meta">▶ {{ formatCount(video.play_count) }}  💬 {{ video.comment_count }}  发布于 {{ formatTime(video.created_at) }}</text>
    </view>

    <!-- 操作按钮组 -->
    <view v-if="video" class="action-row">
      <view class="action-btn" @tap="onLike">
        <text class="icon">{{ video.liked_by_me ? '❤️' : '👍' }}</text>
        <text>{{ formatCount(video.like_count) }}</text>
      </view>
      <view class="action-btn">
        <text class="icon">👎</text>
        <text>不喜欢</text>
      </view>
      <view class="action-btn" @tap="onCoin">
        <text class="icon">🪙</text>
        <text>投币{{ video.my_coin_amount > 0 ? ` (${video.my_coin_amount})` : '' }}</text>
      </view>
      <view class="action-btn" @tap="onFavorite">
        <text class="icon">{{ video.favorited_by_me ? '🌟' : '⭐' }}</text>
        <text>{{ formatCount(video.fav_count) }}</text>
      </view>
      <view class="action-btn" @tap="onShare">
        <text class="icon">↗</text>
        <text>分享</text>
      </view>
    </view>

    <!-- 简介 -->
    <view v-if="video?.description" class="intro-card">
      <text :class="{ expanded }">{{ video.description }}</text>
      <text class="toggle" @tap="expanded = !expanded">{{ expanded ? '收起 ⌃' : '展开 ⌄' }}</text>
    </view>

    <!-- 弹幕发送 -->
    <view class="danmaku-bar">
      <input
        v-model="danmakuText"
        class="danmaku-input"
        placeholder="发个弹幕见证当下"
        confirm-type="send"
        @confirm="sendDanmaku"
      />
      <view class="danmaku-send" @tap="sendDanmaku">
        <text>发送</text>
      </view>
    </view>

    <!-- 评论区 -->
    <view class="comment-section">
      <view class="comment-head">
        <text class="section-title">评论 {{ video?.comment_count || 0 }}</text>
      </view>

      <view v-if="!isLoggedIn" class="login-tip" @tap="goLogin">
        <text>登录后参与评论 ›</text>
      </view>

      <view v-else class="comment-input-row">
        <input
          v-model="commentText"
          class="comment-input"
          placeholder="发一条友善的评论"
          confirm-type="send"
          @confirm="submitComment"
        />
        <view class="comment-send" @tap="submitComment">
          <text>发布</text>
        </view>
      </view>

      <view v-if="comments.length === 0 && !commentsLoading" class="empty-state">
        <text class="icon">💬</text>
        <text>暂无评论，快来抢沙发</text>
      </view>

      <!-- 评论二级树：顶层评论 + 回复列表 -->
      <view v-for="c in topComments" :key="c.id" class="comment-item">
        <image class="avatar" :src="c.avatar_url || '/static/avatar/default.png'" mode="aspectFill" />
        <view class="comment-body">
          <view class="comment-row">
            <text class="username">{{ c.username }}</text>
            <text class="time">{{ formatTime(c.created_at) }}</text>
            <text v-if="c.pinned" class="up-badge pin">📌 置顶</text>
            <text v-if="c.comments_curated" class="up-badge selected">✓ 精选</text>
          </view>
          <text class="content">{{ c.content }}</text>
          <view class="comment-actions">
            <text class="like" @tap="toggleCommentLike(c)">{{ c.liked_by_me ? '❤️' : '👍' }} {{ c.like_count || '' }}</text>
            <text class="reply" @tap="replyTo(c)">回复</text>
            <template v-if="isUploader">
              <text class="up-act" @tap="pinComment(c)">{{ c.pinned ? '取消置顶' : '置顶' }}</text>
              <text class="up-act" @tap="approveComment(c)" v-if="!c.comments_curated">精选</text>
            </template>
          </view>

          <!-- 回复子列表 -->
          <view v-if="repliesOf(c.id).length > 0" class="reply-tree">
            <view v-for="r in (expandedReplies.has(c.id) ? repliesOf(c.id) : repliesOf(c.id).slice(0, 3))" :key="r.id" class="reply-item">
              <image class="reply-avatar" :src="r.avatar_url || '/static/avatar/default.png'" mode="aspectFill" />
              <view class="reply-body">
                <view class="reply-row">
                  <text class="reply-name">{{ r.username }}</text>
                  <text class="reply-time">{{ formatTime(r.created_at) }}</text>
                </view>
                <text class="reply-content">{{ r.content }}</text>
                <view class="reply-actions">
                  <text class="like" @tap="toggleCommentLike(r)">{{ r.liked_by_me ? '❤️' : '👍' }} {{ r.like_count || '' }}</text>
                  <text class="reply" @tap="replyTo(r)">回复</text>
                </view>
              </view>
            </view>
            <text v-if="repliesOf(c.id).length > 3" class="expand-replies" @tap="toggleReplies(c.id)">
              {{ expandedReplies.has(c.id) ? '收起回复' : `展开 ${repliesOf(c.id).length - 3} 条回复` }} ▾
            </text>
          </view>
        </view>
      </view>

      <view v-if="hasMoreComments" class="load-more" @tap="loadMoreComments">
        <text>加载更多评论</text>
      </view>
    </view>

    <!-- 相关推荐 -->
    <view class="related">
      <text class="section-title">相关推荐</text>
      <view v-for="r in related" :key="r.id" class="related-card" @tap="loadDetail(r.id)">
        <image class="cover" :src="r.cover_url || '/static/placeholder.png'" mode="aspectFill" />
        <view class="meta">
          <text class="title text-ellipsis-2">{{ r.title }}</text>
          <text class="up">{{ r.uploader || '匿名' }}</text>
          <text class="stats">▶ {{ formatCount(r.play_count) }} · {{ formatDuration(r.duration) }}</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, getCurrentInstance } from 'vue'
import { onLoad } from '@dcloudio/uni-app'
import { videoApi } from '@/api/video'
import { useUserStore } from '@/store/user'
import type { Comment, Video } from '@/utils/types'

const userStore = useUserStore()
const isLoggedIn = computed(() => userStore.isLoggedIn)

const video = ref<Video | null>(null)
const related = ref<Video[]>([])
const loading = ref(false)
const expanded = ref(false)
const followed = ref(false)
let currentId = 0

// 播放地址：相对路径拼 baseURL
const baseURL = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'
const playUrl = computed(() => {
  if (!video.value?.video_url) return ''
  const u = video.value.video_url
  return u.startsWith('http') ? u : baseURL + u
})

// 评论
const comments = ref<Comment[]>([])
const commentsLoading = ref(false)
const commentText = ref('')
const hasMoreComments = ref(false)
let commentCursor: string | null = null
let replyParentId: number | null = null

// 弹幕
const danmakuText = ref('')

onLoad((q) => {
  currentId = Number(q?.id) || 0
  loadDetail(currentId)
})

async function loadDetail(id: number) {
  currentId = id
  loading.value = true
  video.value = null
  comments.value = []
  commentCursor = null
  try {
    const [v, rels] = await Promise.all([
      videoApi.detail(id),
      videoApi.list(undefined, 50)
    ])
    video.value = v
    followed.value = v.followed_by_me
    related.value = rels.items.filter((r) => r.id !== id).slice(0, 8)
    loadComments()
  } finally {
    loading.value = false
  }
}

async function loadComments() {
  if (!currentId || commentsLoading.value) return
  commentsLoading.value = true
  try {
    const resp = await videoApi.comments(currentId, commentCursor ?? undefined)
    comments.value.push(...resp.items)
    // 游标：取最后一条顶层评论 id（回复不参与翻页游标）
    const tops = resp.items.filter((c: Comment) => !c.parent_id)
    if (tops.length) {
      commentCursor = String(tops[tops.length - 1].id)
    } else if (!resp.items.length) {
      commentCursor = null
    }
    hasMoreComments.value = !!commentCursor
  } catch { /* 已 toast */ }
  finally { commentsLoading.value = false }
}

/** 顶层评论（parent_id 为空/0） */
const topComments = computed(() => comments.value.filter((c) => !c.parent_id || c.parent_id === 0))
/** 某顶层评论下的回复 */
function repliesOf(parentId: number): Comment[] {
  return comments.value.filter((c) => c.parent_id === parentId)
}
/** 展开的回复集合 */
const expandedReplies = ref<Set<number>>(new Set())
function toggleReplies(parentId: number) {
  const s = new Set(expandedReplies.value)
  if (s.has(parentId)) s.delete(parentId)
  else s.add(parentId)
  expandedReplies.value = s
}

async function loadMoreComments() {
  if (!commentCursor) return
  await loadComments()
}

async function submitComment() {
  const content = commentText.value.trim()
  if (!content) {
    uni.showToast({ title: '请输入评论内容', icon: 'none' })
    return
  }
  try {
    const resp = await videoApi.createComment(currentId, content, replyParentId ?? undefined)
    uni.showToast({ title: resp.approved ? '评论成功' : '评论待审核', icon: 'success' })
    commentText.value = ''
    replyParentId = null
    // 重新拉取评论
    comments.value = []
    commentCursor = null
    await loadComments()
  } catch { /* 已 toast */ }
}

function replyTo(c: Comment) {
  replyParentId = c.parent_id || c.id
  commentText.value = `回复 @${c.username}：`
  uni.pageScrollTo({ scrollTop: 0, duration: 300 })
}

async function toggleCommentLike(c: Comment) {
  c.liked_by_me = !c.liked_by_me
  c.like_count += c.liked_by_me ? 1 : -1
  uni.showToast({ title: c.liked_by_me ? '已点赞' : '已取消', icon: 'none' })
}

// 弹幕
async function sendDanmaku() {
  const content = danmakuText.value.trim()
  if (!content) {
    uni.showToast({ title: '请输入弹幕', icon: 'none' })
    return
  }
  if (!isLoggedIn.value) {
    uni.showToast({ title: '请先登录', icon: 'none' })
    setTimeout(() => goLogin(), 500)
    return
  }
  try {
    await videoApi.sendDanmaku(currentId, { content, video_time: Math.floor(currentTime) || 0 })
    uni.showToast({ title: '弹幕已发送', icon: 'success' })
    danmakuText.value = ''
  } catch { /* 已 toast */ }
}

// 互动
async function onLike() {
  if (!video.value) return
  if (!isLoggedIn.value) { goLogin(); return }
  try {
    const r = await videoApi.toggleLike(video.value.id)
    video.value.liked_by_me = r.liked
    video.value.like_count = r.count
  } catch { /* 已 toast */ }
}

async function onCoin() {
  if (!video.value) return
  if (!isLoggedIn.value) { goLogin(); return }
  try {
    const r = await videoApi.coin(video.value.id)
    video.value.my_coin_amount = r.my_coin_amount
    video.value.coin_count = r.coin_count
    video.value.coined_by_me = r.coined
    uni.showToast({ title: `投币成功！剩余硬币 ${r.coin_balance}`, icon: 'success' })
  } catch { /* 已 toast */ }
}

async function onFavorite() {
  if (!video.value) return
  if (!isLoggedIn.value) { goLogin(); return }
  try {
    const r = await videoApi.toggleFavorite(video.value.id)
    video.value.favorited_by_me = r.favorited
    video.value.fav_count = r.fav_count
    uni.showToast({ title: r.favorited ? '已收藏' : '已取消收藏', icon: 'success' })
  } catch { /* 已 toast */ }
}

function onShare()     { uni.showToast({ title: '分享功能待完善', icon: 'none' }) }

const isUploader = computed(() => !!video.value && isLoggedIn.value && userStore.user?.user_id === video.value.user_id)

async function pinComment(c: Comment) {
  try {
    await videoApi.pinComment(c.id)
    c.pinned = !c.pinned
    uni.showToast({ title: c.pinned ? '已置顶' : '已取消置顶', icon: 'success' })
  } catch (e) {
    uni.showToast({ title: (e as Error).message || '置顶失败（需UP主）', icon: 'none' })
  }
}
async function approveComment(c: Comment) {
  try {
    await videoApi.approveComment(c.id)
    c.comments_curated = true
    uni.showToast({ title: '已设为精选', icon: 'success' })
  } catch (e) {
    uni.showToast({ title: (e as Error).message || '精选失败（需UP主）', icon: 'none' })
  }
}
function onFollow()    { followed.value = !followed.value }
function goSpace() {
  if (video.value) uni.navigateTo({ url: `/pages/space/index?id=${video.value.user_id}` })
}
function onPlay()      { playing.value = true }
function onPause()     { playing.value = false }
function onEnded()     { playing.value = false }

// ===== 自定义播放器状态 =====
const playing = ref(false)
const userStarted = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const speed = ref(1)
const SPEEDS = [0.5, 1, 1.5, 2]

function onLoadedMeta(e: any) {
  duration.value = e.detail?.duration || e.target?.duration || 0
}

function onTimeUpdate(e: any) {
  currentTime.value = e.detail?.currentTime || e.target?.currentTime || 0
  if (!duration.value && e.detail?.duration) duration.value = e.detail.duration
}

function onFullscreenChange(e: any) {
  // 全屏切换后保留播放状态
  playing.value = !!e.detail?.fullScreen
}

function onTogglePlay() {
  userStarted.value = true
  const ctx = getVideoContext()
  if (!ctx) return
  if (playing.value) ctx.pause()
  else ctx.play()
}

function onSeekTap(e: any) {
  if (!duration.value) return
  // 计算点击位置比例
  const rect = (e.target as any)?.getBoundingClientRect?.()
  const x = e.detail?.x ?? (rect ? e.detail.x - rect.left : 0)
  const ratio = Math.min(1, Math.max(0, x / 300))
  const target = ratio * duration.value
  currentTime.value = target
  getVideoContext()?.seek(target)
}

function onCycleSpeed() {
  const idx = SPEEDS.indexOf(speed.value)
  speed.value = SPEEDS[(idx + 1) % SPEEDS.length]
  getVideoContext()?.playbackRate(speed.value)
}

function onFullscreen() {
  getVideoContext()?.requestFullScreen({ direction: 0 })
}

function getVideoContext(): UniApp.VideoContext | null {
  if (!video.value) return null
  return uni.createVideoContext('vd-' + video.value.id, getCurrentInstance()?.proxy)
}

const progressPct = computed(() => {
  if (!duration.value) return 0
  return Math.min(100, Math.round((currentTime.value / duration.value) * 100))
})

function formatClock(sec: number): string {
  if (!sec || Number.isNaN(sec)) return '00:00'
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}
function onMore()      { uni.showToast({ title: '更多操作待完善', icon: 'none' }) }
function goBack()      { uni.navigateBack() }
function goLogin()     { uni.navigateTo({ url: '/pages/login/index' }) }

// 工具
function formatDuration(s: number): string {
  if (!s) return ''
  const m = Math.floor(s / 60)
  const sec = (s % 60).toString().padStart(2, '0')
  return `${m}:${sec}`
}
function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}万`
  return n.toString()
}
function formatTime(s: string): string {
  if (!s) return ''
  const d = new Date(s)
  const now = Date.now()
  const diff = (now - d.getTime()) / 1000
  if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`
  if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`
  return `${d.getFullYear()}-${(d.getMonth() + 1).toString().padStart(2, '0')}-${d.getDate().toString().padStart(2, '0')}`
}
</script>

<style lang="scss" scoped>
.detail-page { min-height: 100vh; background: #FFF; padding-bottom: 60rpx; }

.floating-bar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  padding: 16rpx 24rpx;
  background: rgba(255,255,255,0.95);
  backdrop-filter: blur(8px);

  .back { width: 60rpx; font-size: 48rpx; }
  .title { flex: 1; font-size: 30rpx; font-weight: 500; padding: 0 16rpx; }
  .more { width: 60rpx; font-size: 32rpx; text-align: right; }
}

.player {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: #000;
}

/* 未播放封面遮罩 */
.player-cover {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16rpx;
  z-index: 2;
  .play-btn {
    width: 120rpx;
    height: 120rpx;
    border-radius: 50%;
    background: rgba(251, 114, 153, 0.9);
    color: #FFF;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 48rpx;
    padding-left: 8rpx;
    box-shadow: 0 8rpx 24rpx rgba(251, 114, 153, 0.4);
  }
  .cover-tip {
    color: rgba(255, 255, 255, 0.85);
    font-size: 24rpx;
  }
}

/* 播放器控制条 */
.controls {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 12rpx 16rpx 8rpx;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.75));
  z-index: 3;
}
.ctrl-row {
  display: flex;
  align-items: center;
  gap: 20rpx;
  color: #FFF;
  .ctrl-btn {
    min-width: 56rpx;
    height: 56rpx;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 28rpx;
    &.speed {
      font-size: 22rpx;
      padding: 0 8rpx;
      border: 1rpx solid rgba(255, 255, 255, 0.5);
      border-radius: 4rpx;
      min-width: 64rpx;
    }
  }
  .time {
    font-size: 22rpx;
    color: rgba(255, 255, 255, 0.9);
  }
  .spacer { flex: 1; }
}
.progress-bar {
  position: relative;
  height: 32rpx;
  display: flex;
  align-items: center;
  margin-top: 4rpx;
  &::before {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    height: 6rpx;
    border-radius: 3rpx;
    background: rgba(255, 255, 255, 0.3);
  }
  .progress-fill {
    position: absolute;
    left: 0;
    height: 6rpx;
    border-radius: 3rpx;
    background: #FB7299;
  }
  .progress-thumb {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 20rpx;
    height: 20rpx;
    border-radius: 50%;
    background: #FFF;
    box-shadow: 0 0 6rpx rgba(0, 0, 0, 0.4);
  }
}

.up-card {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 24rpx;
  border-bottom: 1rpx solid #F1F1F1;

  .avatar { width: 80rpx; height: 80rpx; border-radius: 50%; }
  .up-meta { flex: 1; }
  .row1 {
    display: flex;
    align-items: center;
    gap: 8rpx;
    .nickname { font-size: 28rpx; font-weight: 500; }
    .fans { font-size: 20rpx; color: #999; }
  }
  .intro { font-size: 22rpx; color: #999; margin-top: 4rpx; }

  .follow-btn {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 12rpx 32rpx;
    border-radius: 24rpx;
    font-size: 24rpx;
  }
}

.title-section {
  padding: 24rpx;
  .title { display: block; font-size: 32rpx; color: #181818; line-height: 1.4; }
  .meta { display: block; margin-top: 12rpx; font-size: 22rpx; color: #999; }
}

.action-row {
  display: flex;
  border-top: 1rpx solid #F1F1F1;
  border-bottom: 1rpx solid #F1F1F1;

  .action-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8rpx;
    padding: 20rpx 0;
    font-size: 22rpx;
    color: #666;

    .icon { font-size: 36rpx; }
  }
}

.intro-card {
  margin: 16rpx;
  padding: 24rpx;
  background: #F8F8F8;
  border-radius: 12rpx;
  font-size: 26rpx;
  color: #333;
  line-height: 1.6;

  .toggle {
    display: block;
    margin-top: 12rpx;
    color: #FB7299;
    font-size: 22rpx;
  }
}

.danmaku-bar {
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 16rpx 24rpx;
  border-bottom: 1rpx solid #F1F1F1;

  .danmaku-input {
    flex: 1;
    height: 64rpx;
    background: #F4F4F4;
    border-radius: 32rpx;
    padding: 0 24rpx;
    font-size: 26rpx;
  }
  .danmaku-send {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 12rpx 28rpx;
    border-radius: 32rpx;
    font-size: 26rpx;
  }
}

.comment-section {
  padding: 24rpx;
}

.comment-head {
  padding-bottom: 16rpx;
  border-bottom: 1rpx solid #F1F1F1;
}

.section-title {
  font-size: 28rpx;
  font-weight: 500;
}

.login-tip {
  margin-top: 24rpx;
  text-align: center;
  padding: 24rpx;
  background: #FFF5F8;
  border-radius: 12rpx;
  color: #FB7299;
  font-size: 26rpx;
}

.comment-input-row {
  display: flex;
  align-items: center;
  gap: 16rpx;
  margin-top: 24rpx;

  .comment-input {
    flex: 1;
    height: 64rpx;
    background: #F4F4F4;
    border-radius: 32rpx;
    padding: 0 24rpx;
    font-size: 26rpx;
  }
  .comment-send {
    background: linear-gradient(90deg, #FB7299, #FF9DB5);
    color: #FFF;
    padding: 12rpx 28rpx;
    border-radius: 32rpx;
    font-size: 26rpx;
  }
}

.comment-item {
  display: flex;
  gap: 16rpx;
  padding: 24rpx 0;
  border-bottom: 1rpx solid #F8F8F8;

  .avatar {
    width: 64rpx;
    height: 64rpx;
    border-radius: 50%;
    background: #F0F0F0;
  }
  .comment-body { flex: 1; }
  .comment-row {
    display: flex;
    align-items: center;
    gap: 12rpx;
    .username { font-size: 24rpx; color: #757575; }
    .time { font-size: 20rpx; color: #C0C4CC; }
  }
  .content {
    display: block;
    margin-top: 8rpx;
    font-size: 28rpx;
    color: #181818;
    line-height: 1.5;
    word-break: break-all;
  }
  .comment-actions {
    display: flex;
    gap: 24rpx;
    margin-top: 12rpx;
    font-size: 22rpx;
    color: #999;
    flex-wrap: wrap;

    .like, .reply, .up-act { padding: 4rpx 0; }
    .up-act { color: #FB7299; }
  }
}

/* 回复二级树（B 站风格：灰底缩进 + 小头像） */
.reply-tree {
  margin-top: 16rpx;
  padding: 16rpx 20rpx;
  background: #F7F8FA;
  border-radius: 12rpx;

  .reply-item {
    display: flex;
    gap: 12rpx;
    padding: 12rpx 0;
    border-bottom: 1rpx solid #F0F1F3;
    &:last-child { border-bottom: none; }
  }
  .reply-avatar {
    width: 48rpx;
    height: 48rpx;
    border-radius: 50%;
    flex-shrink: 0;
    background: #E8E8E8;
  }
  .reply-body { flex: 1; min-width: 0; }
  .reply-row {
    display: flex;
    align-items: baseline;
    gap: 12rpx;
  }
  .reply-name { font-size: 22rpx; color: #61666D; font-weight: 500; }
  .reply-time { font-size: 20rpx; color: #999; }
  .reply-content {
    font-size: 24rpx;
    color: #181818;
    line-height: 1.5;
    margin-top: 4rpx;
    word-break: break-all;
  }
  .reply-actions {
    display: flex;
    gap: 24rpx;
    margin-top: 8rpx;
    font-size: 20rpx;
    color: #999;
  }
  .expand-replies {
    display: block;
    text-align: center;
    font-size: 22rpx;
    color: #FB7299;
    padding: 12rpx 0 4rpx;
  }
}

.up-badge {
  font-size: 18rpx;
  padding: 2rpx 8rpx;
  border-radius: 4rpx;
  &.pin { background: #FFE4EC; color: #FB7299; }
  &.selected { background: #E6F1FB; color: #2A7DD1; }
}

.load-more {
  text-align: center;
  padding: 24rpx 0;
  color: #FB7299;
  font-size: 24rpx;
}

.related {
  padding: 24rpx;
  .section-title {
    display: block;
    padding: 16rpx 0;
    border-bottom: 1rpx solid #F1F1F1;
  }
  .related-card {
    display: flex;
    gap: 16rpx;
    margin: 24rpx 0;
    .cover {
      width: 240rpx;
      aspect-ratio: 16 / 10;
      border-radius: 8rpx;
      background: #EEE;
    }
    .meta {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 8rpx;
      .title { font-size: 26rpx; color: #181818; }
      .up { font-size: 22rpx; color: #999; }
      .stats { font-size: 22rpx; color: #999; }
    }
  }
}
</style>