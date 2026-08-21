<template>
  <view class="detail-page">
    <view v-if="loading" class="loading-mask"><text>加载中…</text></view>

    <!-- 顶部固定返回按钮 -->
    <view class="floating-bar safe-area-top">
      <text class="back" @tap="goBack">‹</text>
      <text class="title text-ellipsis-1">{{ video?.title || '视频详情' }}</text>
      <text class="more" @tap="onMore">⋯</text>
    </view>

    <!-- 视频播放器（B 站风格：自定义控制条 + canvas 弹幕层 + 应用内全屏） -->
    <view class="player" :class="{ 'fs-active': fullscreen }">
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
        style="width:100%;height:100%;"
      />
      <!-- 弹幕层（canvas，pointer-events none 不挡控制条） -->
      <canvas
        v-if="video && dmReady"
        canvas-id="dm-layer"
        id="dm-layer"
        class="dm-layer"
        :style="{ width: dmW + 'px', height: dmH + 'px' }"
      />
      <!-- 封面遮罩（未播放时显示；App 端用 cover-view 盖在原生 video 上） -->
      <cover-view v-if="!playing && !userStarted" class="player-cover" @tap="onTogglePlay">
        <cover-view class="play-btn"><cover-view>▶</cover-view></cover-view>
        <cover-view class="cover-tip">点击播放</cover-view>
      </cover-view>
        <!-- 控制条：App 端 video 是原生组件会盖住普通 view，必须用 cover-view 才能显示在 video 上方 -->
        <cover-view v-if="userStarted" class="controls" @tap.stop>
          <cover-view class="ctrl-row">
            <cover-view class="ctrl-btn" @tap="onTogglePlay">
              <cover-view>{{ playing ? '❚❚' : '▶' }}</cover-view>
            </cover-view>
            <cover-view class="time">{{ formatClock(currentTime) }} / {{ formatClock(duration) }}</cover-view>
            <cover-view class="spacer" />
            <!-- 弹幕开关 -->
            <cover-view class="ctrl-btn" @tap="toggleDmVisible">
              <cover-view>{{ dmVisible ? '弹' : '关弹' }}</cover-view>
            </cover-view>
            <!-- 倍速菜单(弹层) -->
            <cover-view class="ctrl-btn speed" @tap="openSpeedMenu">
              <cover-view>{{ speedLabel }}</cover-view>
            </cover-view>
            <!-- 循环 -->
            <cover-view class="ctrl-btn" @tap="toggleLoop">
              <cover-view>{{ loopMode === 'off' ? '↻' : (loopMode === 'one' ? '🔂' : '🔁') }}</cover-view>
            </cover-view>
            <!-- 宽屏 -->
            <cover-view class="ctrl-btn" @tap="toggleWide">
              <cover-view>{{ wideMode ? '▣' : '▢' }}</cover-view>
            </cover-view>
            <!-- 音量 -->
            <cover-view class="ctrl-btn" @tap="toggleMute">
              <cover-view>{{ muted ? '🔇' : '🔊' }}</cover-view>
            </cover-view>
            <!-- 全屏 -->
            <cover-view class="ctrl-btn" @tap="fullscreen ? onExitFullscreen() : onFullscreen()">
              <cover-view>{{ fullscreen ? '✕' : '⛶' }}</cover-view>
            </cover-view>
          </cover-view>
          <!-- 进度条 -->
          <cover-view
            class="progress-bar"
            @tap="onSeekTap"
            @touchstart="onSeekTouchStart"
            @touchmove="onSeekTouchMove"
            @touchend="onSeekTouchEnd"
          >
            <cover-view class="progress-fill" :style="{ width: progressPct + '%' }" />
            <cover-view class="progress-thumb" :style="{ left: progressPct + '%' }" />
          </cover-view>
          <!-- 倍速菜单弹层(cover-view 内只能用 cover-view) -->
          <cover-view v-if="speedMenuOpen" class="speed-menu" @tap.stop>
            <cover-view
              v-for="s in SPEEDS"
              :key="s"
              class="speed-item"
              :class="{ active: s === speed }"
              @tap="pickSpeed(s)"
            >
              <cover-view>{{ s === 1 ? '1x (正常)' : s + 'x' }}</cover-view>
            </cover-view>
          </cover-view>
        </cover-view>
        <!-- 全屏顶部返回条 -->
        <view v-if="fullscreen" class="fs-topbar" @tap.stop>
          <view class="fs-exit" @tap="onExitFullscreen"><text>‹ 退出全屏</text></view>
          <text class="fs-title text-ellipsis-1">{{ video?.title }}</text>
        </view>
      </view>

    <!-- 第1行：简介 / 评论 切换 tab + 弹幕发送 + 弹幕开关 -->
    <view class="detail-tabs">
      <view class="tab-item" :class="{ active: activeTab === 'intro' }" @tap="activeTab = 'intro'"><text>简介</text></view>
      <view class="tab-item" :class="{ active: activeTab === 'comment' }" @tap="activeTab = 'comment'"><text>评论 {{ video?.comment_count ? formatCount(video.comment_count) : '' }}</text></view>
      <view class="tab-spacer" />
      <input
        v-model="danmakuText"
        class="tab-dm-input"
        placeholder="发个弹幕"
        confirm-type="send"
        @confirm="sendDanmaku"
      />
      <view class="tab-dm-send" @tap="sendDanmaku"><text>发送</text></view>
      <view class="dm-toggle" :class="{ off: !dmVisible }" @tap="toggleDmVisible">
        <text>{{ dmVisible ? '✓弹' : '弹' }}</text>
      </view>
    </view>

    <!-- 简介视图（第2-5行 + 简介 + 相关推荐） -->
    <view v-if="activeTab === 'intro'">
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
      <text class="meta">▶ {{ formatCount(video.play_count) }}  💬 {{ formatCount(video.danmaku_count) }}  📅 {{ formatDate(video.created_at) }}  🟢 {{ video.watching_count || 0 }} 人在线</text>
    </view>

    <!-- 操作按钮组 -->
    <view v-if="video" class="action-row">
      <view class="action-btn" @tap="onLike">
        <text class="icon">{{ video.liked_by_me ? '❤️' : '👍' }}</text>
        <text>{{ formatCount(video.like_count) }}</text>
      </view>
      <view class="action-btn" @tap="onDislike">
        <text class="icon">{{ disliked ? '👎' : '🤙' }}</text>
        <text>点踩</text>
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
        <text>转发</text>
      </view>
    </view>

    <!-- 视频合集导航（后端有合集数据时才显示） -->
    <view v-if="video && seriesList && seriesList.length > 0" class="series-bar" @tap="openSeries">
      <text class="series-title">合集 · {{ seriesTitle || '视频合集' }}</text>
      <text class="series-arrow">▾ 点击展开</text>
    </view>

    <!-- 简介 -->
    <view v-if="video?.description" class="intro-card">
      <text :class="{ expanded }">{{ video.description }}</text>
      <text class="toggle" @tap="expanded = !expanded">{{ expanded ? '收起 ⌃' : '展开 ⌄' }}</text>
    </view>

    <!-- 相关推荐（简介视图底部） -->
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
    <!-- /简介视图 -->

    <!-- 评论视图（评论区，点击"评论"tab 后显示，上方视频不动） -->
    <view v-if="activeTab === 'comment'">
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
    </view>
    <!-- /评论视图 -->
  </view>
</template>

<script setup lang="ts">
import { computed, ref, getCurrentInstance } from 'vue'
import { onLoad, onReady, onUnload, onBackPress } from '@dcloudio/uni-app'
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
const disliked = ref(false) // 点踩（后端暂无 dislike 接口，先本地态）
const seriesList = ref<unknown[]>([]) // 合集（后端暂无接口，预留条件渲染）
const seriesTitle = ref('')
const activeTab = ref<'intro' | 'comment'>('intro') // 简介/评论 切换
let currentId = 0

// 播放地址 baseURL：与 request.ts 同款条件编译（App 端必须用局域网 IP，不能 127.0.0.1）
// #ifdef APP-PLUS
const baseURL = import.meta.env.VITE_API_BASE_URL_APP || 'http://192.168.1.100:8080'
// #endif
// #ifndef APP-PLUS
// 注意：H5 dev 的 VITE_API_BASE_URL 为空，必须 fallback 127.0.0.1:8080（vite dev 不代理 /uploads）
const baseURL = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'
// #endif
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

// ===== 弹幕引擎（canvas 时间轴回放） =====
interface DmTimelineItem {
  content: string
  color: string
  type: string
  font_size: string
  video_time: number
}
interface ActiveDm {
  text: string
  color: string
  type: string
  fontSize: number
  x: number
  y: number
  speed: number
  born: number
  duration: number
}

const dmList = ref<DmTimelineItem[]>([])
const dmReady = ref(false)
const dmW = ref(0)
const dmH = ref(0)
let dmIdx = 0
let activeDms: ActiveDm[] = []
let dmTimer: ReturnType<typeof setInterval> | null = null
let dmCtx: any = null
let dmTrackCursor = 0
const DM_FPS = 30
const DM_TRAVEL_SEC = 8

onReady(() => {
  setupDmCanvas()
})

onUnload(() => {
  stopDm()
  // #ifdef H5
  document.removeEventListener('fullscreenchange', onH5FullscreenChange)
  // #endif
})

function setupDmCanvas() {
  refreshDmCanvasSize()
}

/** 查询 .player 实际尺寸并同步 canvas（普通/全屏切换后都要重查）。
 *  全屏容器为 fixed inset 0（不旋转），boundingClientRect 即视口尺寸，无需特判。 */
function refreshDmCanvasSize() {
  uni
    .createSelectorQuery()
    .in(getCurrentInstance()?.proxy)
    .select('.player')
    .boundingClientRect((rect: any) => {
      if (rect && rect.width > 0) {
        dmW.value = Math.round(rect.width)
        dmH.value = Math.round(rect.height)
        dmReady.value = true
      }
    })
    .exec()
}

function getDmCtx(): any {
  if (!dmCtx) dmCtx = uni.createCanvasContext('dm-layer', getCurrentInstance()?.proxy)
  return dmCtx
}

function fontSizePx(fs: string): number {
  const base = Math.max(12, Math.round(dmW.value / 27))
  if (fs === 'sm') return base - 2
  if (fs === 'lg') return base + 4
  return base
}

function dmTrackCount(): number {
  if (!dmH.value) return 4
  return Math.max(2, Math.floor((dmH.value * 0.82) / (fontSizePx('md') + 8)))
}

async function loadDanmakus() {
  dmList.value = []
  dmIdx = 0
  activeDms = []
  if (!currentId) return
  try {
    const resp = await videoApi.danmakus(currentId, 0, 2000)
    dmList.value = resp.items.map((i) => ({
      content: i.content,
      color: i.color,
      type: i.type,
      font_size: i.font_size,
      video_time: i.video_time
    }))
  } catch {
    /* 弹幕拉取失败不阻塞播放 */
  }
}

function spawnDm(item: DmTimelineItem, now: number) {
  const fs = fontSizePx(item.font_size)
  const isScroll = item.type !== 'top' && item.type !== 'bottom'
  if (isScroll) {
    const speed = dmW.value / DM_TRAVEL_SEC
    dmTrackCursor = (dmTrackCursor + 1) % dmTrackCount()
    const y = dmTrackCursor * (fs + 8) + fs
    activeDms.push({
      text: item.content, color: item.color || '#FFFFFF', type: 'scroll',
      fontSize: fs, x: dmW.value, y, speed, born: now, duration: 0
    })
  } else {
    const y = item.type === 'top' ? fs + 4 : dmH.value - fs - 6
    activeDms.push({
      text: item.content, color: item.color || '#FFFFFF', type: item.type,
      fontSize: fs, x: dmW.value / 2, y, speed: 0, born: now, duration: 3.5
    })
  }
}

function dmTextWidth(text: string, fs: number): number {
  // 估算：全角≈fs，半角≈0.55fs
  let w = 0
  for (const ch of text) {
    w += ch.charCodeAt(0) > 255 ? fs : fs * 0.55
  }
  return w
}

function tickDm() {
  const t = currentTime.value
  while (dmIdx < dmList.value.length && dmList.value[dmIdx].video_time <= t) {
    spawnDm(dmList.value[dmIdx], t)
    dmIdx++
  }
  drawDm(t)
}

function drawDm(now: number) {
  if (!dmReady.value) return
  const ctx = getDmCtx()
  ctx.clearRect(0, 0, dmW.value, dmH.value)
  const keep: ActiveDm[] = []
  for (const d of activeDms) {
    const age = now - d.born
    if (d.type === 'scroll') {
      d.x -= d.speed * (1 / DM_FPS)
      if (d.x + dmTextWidth(d.text, d.fontSize) < 0) continue
    } else if (age > d.duration) {
      continue
    }
    ctx.setFillStyle(d.color)
    ctx.setFontSize(d.fontSize)
    ctx.setTextAlign(d.type === 'scroll' ? 'left' : 'center')
    ctx.fillText(d.text, d.type === 'scroll' ? d.x : d.x, d.y)
    keep.push(d)
  }
  activeDms = keep
  ctx.draw()
}

function startDm() {
  stopDm()
  if (!dmReady.value || !dmList.value.length) return
  dmTimer = setInterval(tickDm, 1000 / DM_FPS)
}

function stopDm() {
  if (dmTimer) {
    clearInterval(dmTimer)
    dmTimer = null
  }
}

function resetDm() {
  activeDms = []
  dmIdx = 0
  if (dmReady.value) {
    getDmCtx().clearRect(0, 0, dmW.value, dmH.value)
    getDmCtx().draw()
  }
}

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
  stopDm()
  try {
    const [v, rels] = await Promise.all([
      videoApi.detail(id),
      videoApi.list(undefined, 50)
    ])
    video.value = v
    followed.value = v.followed_by_me
    // 时长直接取后端字段（避免 loadedmetadata 事件缺失导致 duration=0）
    duration.value = v.duration || 0
    related.value = rels.items.filter((r) => r.id !== id).slice(0, 8)
    loadComments()
    loadDanmakus()
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
    // 本地上屏：立即加入时间轴，播放中则马上滚动出来
    dmList.value.push({
      content,
      color: '#FFFFFF',
      type: 'scroll',
      font_size: 'md',
      video_time: Math.floor(currentTime) || 0
    })
    if (playing.value && dmReady.value) spawnDm(dmList.value[dmList.value.length - 1], currentTime.value)
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

/** 点踩：后端暂无接口，先本地切换 */
function onDislike() {
  disliked.value = !disliked.value
  uni.showToast({ title: disliked.value ? '已点踩' : '已取消点踩', icon: 'none' })
}

/** 合集展开：后端暂无合集接口，预留 */
function openSeries() {
  uni.showToast({ title: '合集功能待接入', icon: 'none' })
}

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
function onPlay()      { playing.value = true; startDm() }
function onPause()     { playing.value = false; stopDm() }
function onEnded() {
  playing.value = false
  stopDm()
  // 单曲循环
  if (loopMode.value === 'one') {
    currentTime.value = 0
    resetDm()
    setTimeout(() => {
      playing.value = true
      getVideoContext()?.seek(0)
      getVideoContext()?.play()
      startDm()
    }, 200)
  } else {
    resetDm()
  }
}

// ===== 自定义播放器状态 =====
const playing = ref(false)
const userStarted = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const speed = ref(1)
const SPEEDS = [0.5, 1, 1.25, 1.5, 2, 3]
const speedLabel = computed(() => (speed.value === 1 ? '1x' : speed.value + 'x'))
const speedMenuOpen = ref(false)
function openSpeedMenu() { speedMenuOpen.value = !speedMenuOpen.value }
function pickSpeed(s: number) {
  speed.value = s
  speedMenuOpen.value = false
  try { getVideoContext()?.playbackRate(s) } catch { /* ignore */ }
}

// PC 端功能:弹幕开关 / 循环 / 宽屏 / 音量
const dmVisible = ref(true)
function toggleDmVisible() {
  dmVisible.value = !dmVisible.value
  if (!dmVisible.value) { stopDm(); activeDms = [] } else if (playing.value) startDm()
}

const loopMode = ref<'off' | 'one' | 'all'>('off')
function toggleLoop() {
  loopMode.value = loopMode.value === 'off' ? 'one' : loopMode.value === 'one' ? 'all' : 'off'
}

const wideMode = ref(false)
function toggleWide() {
  wideMode.value = !wideMode.value
  // 改 video object-fit (H5 直接生效;App 端原生 video 可能不支持运行时改)
  try {
    const v: any = document.querySelector(`#vd-${video.value?.id}`) || document.querySelector('video')
    if (v) v.style.objectFit = wideMode.value ? 'cover' : 'contain'
  } catch { /* ignore */ }
}

const volume = ref(1.0)
const muted = ref(false)
function setVolume(v: number) {
  volume.value = Math.max(0, Math.min(1, v))
  muted.value = volume.value === 0
  try {
    const vEl: any = document.querySelector('video')
    if (vEl) vEl.volume = volume.value
  } catch { /* ignore */ }
}
function toggleMute() {
  if (muted.value) { if (volume.value === 0) volume.value = 1; setVolume(volume.value) }
  else setVolume(0)
}

function onLoadedMeta(e: any) {
  // 兼容不同事件格式
  duration.value = e?.detail?.duration || e?.duration || e?.target?.duration || 0
}

function onTimeUpdate(e: any) {
  // 拖动进度条时忽略，避免 thumb 被拉回
  if (dragging.value) return
  const t = e?.detail?.currentTime ?? e?.currentTime ?? e?.target?.currentTime
  if (typeof t === 'number' && isFinite(t)) currentTime.value = t
  const d = e?.detail?.duration ?? e?.duration ?? e?.target?.duration
  if (d && !duration.value) duration.value = d
}

function onFullscreenChange(e: any) {
  // 不再使用系统全屏（应用内全屏），保留兼容：若外部触发全屏事件则同步状态
  playing.value = !!e.detail?.fullScreen
}

// ===== 应用内全屏（客户端方案：不进系统播放器，UI 撑满 + 锁横屏，弹幕层跟随） =====
const fullscreen = ref(false)

function lockOrientation(lock: boolean) {
  // #ifdef APP-PLUS
  try {
    if (lock) (plus as any).screen.lockOrientation('landscape-primary')
    else (plus as any).screen.unlockOrientation()
  } catch (_) { /* 部分 ROM 不支持，忽略 */ }
  // #endif
}

// H5：orientation.lock 需要浏览器全屏上下文 → requestFullscreen 后锁横屏
// #ifdef H5
function onH5FullscreenChange() {
  if (document.fullscreenElement) {
    // 进入浏览器全屏成功，再锁横屏（真机生效；桌面/headless 忽略失败）
    try { (screen.orientation as any)?.lock?.('landscape') } catch (_) { /* 忽略 */ }
    setTimeout(() => { refreshDmCanvasSize(); resetDm() }, 150)
  } else if (fullscreen.value) {
    // 用户通过浏览器手势退出全屏 → 同步应用状态
    fullscreen.value = false
    lockOrientation(false)
    setTimeout(() => { refreshDmCanvasSize(); resetDm() }, 150)
  }
}
document.addEventListener('fullscreenchange', onH5FullscreenChange)
// #endif

function onFullscreen() {
  fullscreen.value = true
  lockOrientation(true)
  // #ifdef H5
  try {
    const el = document.documentElement as any
    if (el.requestFullscreen) el.requestFullscreen().catch(() => {})
  } catch (_) { /* 不支持则保持 fixed 全屏 */ }
  setTimeout(() => { refreshDmCanvasSize(); resetDm() }, 150)
  // #endif
  // #ifndef H5
  setTimeout(() => { refreshDmCanvasSize(); resetDm() }, 150)
  // #endif
}

function onExitFullscreen() {
  fullscreen.value = false
  lockOrientation(false)
  // #ifdef H5
  try { if (document.fullscreenElement) document.exitFullscreen() } catch (_) { /* 忽略 */ }
  // #endif
  setTimeout(() => { refreshDmCanvasSize(); resetDm() }, 150)
}

// 物理返回键：全屏时先退出全屏
onBackPress(() => {
  if (fullscreen.value) {
    onExitFullscreen()
    return true
  }
  return false
})

function onTogglePlay() {
  userStarted.value = true
  const ctx = getVideoContext()
  if (!ctx) return
  if (playing.value) ctx.pause()
  else ctx.play()
}

// 进度条拖动状态（cover-view 触摸事件，App 端原生）
const dragging = ref(false)

/** 从 touch 事件算比例：cover-view 无 getBoundingClientRect，用屏幕宽估算（左右各 32rpx 内缩） */
function ratioFromTouch(e: any): number {
  const t = (e.touches && e.touches[0]) || (e.changedTouches && e.changedTouches[0])
  if (!t) return -1
  const x = t.clientX ?? t.pageX
  if (typeof x !== 'number') return -1
  const sys = uni.getSystemInfoSync()
  const screenWpx = (sys.windowWidth || 390) * (sys.pixelRatio || 1)
  const padPx = (32 * screenWpx) / 750
  const w = screenWpx - padPx * 2
  if (w <= 0) return -1
  return Math.min(1, Math.max(0, (x - padPx) / w))
}

function onSeekTouchStart(e: any) {
  if (!duration.value) return
  dragging.value = true
  const ratio = ratioFromTouch(e)
  if (ratio >= 0) currentTime.value = ratio * duration.value
}

function onSeekTouchMove(e: any) {
  if (!dragging.value) return
  const ratio = ratioFromTouch(e)
  if (ratio >= 0) currentTime.value = ratio * duration.value
}

function onSeekTouchEnd(e: any) {
  if (!dragging.value) return
  dragging.value = false
  const ratio = ratioFromTouch(e)
  if (ratio >= 0) {
    const target = ratio * duration.value
    currentTime.value = target
    console.log('[seek] dragEnd ratio=', ratio, 'target=', target, 'ctx=', !!getVideoContext())
    getVideoContext()?.seek(target)
    resetDm() // 拖动进度条后弹幕按新进度重放
  }
}

function onSeekTap(e: any) {
  if (!duration.value) return
  // cover-view 无 rect，用与拖动相同的估算比例
  const ratio = ratioFromTouch(e)
  const target = Math.max(0, ratio) * duration.value
  currentTime.value = target
  console.log('[seek] tap ratio=', ratio, 'target=', target, 'dur=', duration.value, 'ctx=', !!getVideoContext())
  getVideoContext()?.seek(target)
  resetDm() // 拖动进度条后弹幕按新进度重放
}

function onCycleSpeed() {
  const idx = SPEEDS.indexOf(speed.value)
  speed.value = SPEEDS[(idx + 1) % SPEEDS.length]
  getVideoContext()?.playbackRate(speed.value)
}

let vctxCache: UniApp.VideoContext | null = null
function getVideoContext(): UniApp.VideoContext | null {
  if (!video.value) return null
  if (!vctxCache) {
    vctxCache = uni.createVideoContext('vd-' + video.value.id, getCurrentInstance()?.proxy)
  }
  return vctxCache
}

const progressPct = computed(() => {
  // 兜底：duration/currentTime 非法时返回 0，避免 NaN%
  const d = duration.value
  const t = currentTime.value
  if (!d || !isFinite(d) || d <= 0 || !isFinite(t) || t < 0) return 0
  const pct = (t / d) * 100
  if (!isFinite(pct)) return 0
  return Math.min(100, Math.round(pct))
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
/** 发布日期（数据行用，固定 yyyy-MM-dd） */
function formatDate(s: string): string {
  if (!s) return ''
  const d = new Date(s)
  if (isNaN(d.getTime())) return ''
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

/* 应用内全屏：fixed 填满屏幕。横屏由锁屏方向实现（App: plus.screen.lockOrientation；
 * H5: screen.orientation.lock），不旋转容器——避免 canvas 弹幕文字跟着转 90°。 */
.player.fs-active {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 999;
  aspect-ratio: auto;
  background: #000;
}

/* 全屏顶部返回条 */
.fs-topbar {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 16rpx 24rpx;
  background: linear-gradient(rgba(0, 0, 0, 0.6), transparent);
  z-index: 4;
  .fs-exit {
    color: #FFF;
    font-size: 26rpx;
    padding: 8rpx 0;
    flex-shrink: 0;
  }
  .fs-title {
    color: rgba(255, 255, 255, 0.9);
    font-size: 24rpx;
    flex: 1;
  }
}

/* 全屏控制条：字号放大 */
.player.fs-active .controls {
  padding: 24rpx 32rpx 20rpx;
  .ctrl-btn { min-width: 64rpx; height: 64rpx; font-size: 32rpx; }
  .time { font-size: 26rpx; }
  .progress-bar { height: 40rpx; }
}

/* 弹幕层：盖在视频上、控制条下，点击穿透 */
.dm-layer {
  position: absolute;
  left: 0;
  top: 0;
  z-index: 1;
  pointer-events: none;
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
  height: 40rpx;
  display: flex;
  align-items: center;
  margin-top: 4rpx;
  /* 左右留出圆点半径空间：0% 进度时圆点也在屏内可见 */
  padding-left: 16rpx;
  padding-right: 16rpx;
  &::before {
    content: '';
    position: absolute;
    left: 16rpx;
    right: 16rpx;
    height: 6rpx;
    border-radius: 3rpx;
    background: rgba(255, 255, 255, 0.3);
  }
  .progress-fill {
    position: absolute;
    left: 16rpx;
    height: 6rpx;
    border-radius: 3rpx;
    background: #FB7299;
  }
  .progress-thumb {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 28rpx;
    height: 28rpx;
    border-radius: 50%;
    background: #FFF;
    border: 3rpx solid #FB7299;
    box-shadow: 0 0 6rpx rgba(0, 0, 0, 0.4);
  }
}

/* 倍速菜单弹层（cover-view 内只能用 cover-view,所有子元素必须用 cover-view） */
.speed-menu {
  position: absolute;
  right: 16rpx;
  bottom: 130rpx;
  background: rgba(0, 0, 0, 0.78);
  border-radius: 12rpx;
  padding: 8rpx 0;
  min-width: 180rpx;
  z-index: 5;
}
.speed-item {
  padding: 16rpx 28rpx;
  color: #FFF;
  font-size: 26rpx;
  text-align: center;
}
.speed-item.active {
  color: #FB7299;
  font-weight: 600;
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

/* 第1行：简介/评论 tab + 弹幕发送 + 弹幕开关 */
.detail-tabs {
  display: flex;
  align-items: center;
  padding: 0 24rpx;
  height: 88rpx;
  border-bottom: 1rpx solid #F1F1F1;
  .tab-item { padding: 0 24rpx 0 0; font-size: 30rpx; color: #999; }
  .tab-item.active { color: #181818; font-weight: 600; }
  .tab-spacer { flex: 1; }
  .tab-dm-input {
    width: 200rpx;
    height: 56rpx;
    background: #F4F4F4;
    border-radius: 28rpx;
    padding: 0 20rpx;
    font-size: 24rpx;
  }
  .tab-dm-send {
    background: #FB7299;
    color: #FFF;
    padding: 10rpx 20rpx;
    border-radius: 28rpx;
    font-size: 24rpx;
    margin-left: 12rpx;
  }
  .dm-toggle {
    width: 64rpx;
    height: 56rpx;
    margin-left: 12rpx;
    border-radius: 12rpx;
    background: #FB7299;
    color: #FFF;
    font-size: 24rpx;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .dm-toggle.off { background: #999; color: #FFF; }
}

.action-row {
  display: flex;
  border-top: 1rpx solid #F1F1F1;
  border-bottom: 1rpx solid #F1F1F1;

  .action-btn {
    flex: 1;
    display: flex;
    flex-direction: row; /* 图标与文字横排 */
    align-items: center;
    justify-content: center;
    gap: 8rpx;
    padding: 20rpx 0;
    font-size: 22rpx;
    color: #666;

    .icon { font-size: 34rpx; }
  }
}

.series-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 0 24rpx 16rpx;
  padding: 20rpx 24rpx;
  background: #F8F8F8;
  border-radius: 12rpx;
  border-left: 6rpx solid #FB7299;
  .series-title { font-size: 26rpx; color: #181818; font-weight: 500; }
  .series-arrow { font-size: 22rpx; color: #FB7299; }
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