<template>
  <view class="demo-page">
    <!-- 头部 -->
    <view class="header safe-area-top">
      <view class="back" @tap="goBack">‹</view>
      <text class="title">本地存储</text>
      <view class="back" />
    </view>

    <!-- 引擎状态 -->
    <view class="card">
      <view class="card-title">🖥 存储引擎状态</view>
      <view class="status-row">
        <view class="chip" :class="dbMode === 'idb' ? 'on' : ''">
          {{ dbMode === 'idb' ? 'IndexedDB 数据库模式' : 'KV 降级模式' }}
        </view>
        <view class="chip" :class="online ? 'on' : 'off'">
          {{ online ? '● 在线' : '○ 离线' }}
        </view>
      </view>
      <view class="hint">H5 端使用 IndexedDB 做结构化数据库（容量大、可存 Blob）；小程序/App 自动降级 KV。</view>
    </view>

    <!-- 1. KV 持久化 -->
    <view class="card">
      <view class="card-title">🔑 KV 持久化（storage）</view>
      <view class="form-row">
        <input class="input" v-model="kvKey" placeholder="key，如 note" />
        <input class="input" v-model="kvVal" placeholder="value，如 今天看了《XX》" />
      </view>
      <view class="form-row">
        <input class="input small" v-model="kvTTL" type="number" placeholder="TTL 秒（空=永久）" />
      </view>
      <view class="btn-row">
        <view class="btn primary" @tap="kvWrite">写入</view>
        <view class="btn" @tap="kvRead">读取</view>
        <view class="btn" @tap="kvRemove">删除</view>
        <view class="btn danger" @tap="kvClearAll">清空</view>
      </view>
      <view class="result" v-if="kvResult">{{ kvResult }}</view>
      <view class="kv-list" v-if="kvKeys.length">
        <view class="kv-item" v-for="k in kvKeys" :key="k">
          <text class="kv-key">{{ k }}</text>
          <text class="kv-val">{{ storage.get(k) ?? '∅' }}</text>
        </view>
      </view>
    </view>

    <!-- 2. 本地数据库 -->
    <view class="card">
      <view class="card-title">🗄 本地数据库（localDB · 收藏记录）</view>
      <view class="btn-row">
        <view class="btn primary" @tap="dbAdd">添加收藏</view>
        <view class="btn" @tap="dbRefresh">刷新列表</view>
        <view class="btn danger" @tap="dbClear">清空</view>
      </view>
      <view class="db-list" v-if="dbRows.length">
        <view class="db-item" v-for="r in dbRows" :key="r.id" @tap="dbRemove(r.id)">
          <text class="db-title">{{ r.title }}</text>
          <text class="db-time">{{ r.created_at }}</text>
          <text class="db-del">删除 ✕</text>
        </view>
      </view>
      <view class="hint" v-else>点「添加收藏」写入一条记录（主键自增，重启 App 后仍保留）</view>
    </view>

    <!-- 3. API 离线缓存 -->
    <view class="card">
      <view class="card-title">📦 API 离线缓存（推荐流）</view>
      <view class="btn-row">
        <view class="btn primary" @tap="cacheFetch('first')">首次请求</view>
        <view class="btn" @tap="cacheFetch('again')">再次请求（应命中）</view>
        <view class="btn danger" @tap="cacheClear">清空缓存</view>
      </view>
      <view class="result" v-if="cacheResult">{{ cacheResult }}</view>
      <view class="hint">
        缓存条目 {{ cacheStats.count }} 个 / 约 {{ (cacheStats.totalSize / 1024).toFixed(1) }} KB。
        断网时 `request({ cacheable: true })` 会自动回退缓存，不弹报错。
      </view>
    </view>

    <!-- 4. 图片缓存 -->
    <view class="card">
      <view class="card-title">🖼 图片缓存（IndexedDB Blob）</view>
      <view class="btn-row">
        <view class="btn primary" @tap="imgLoad">加载封面图</view>
        <view class="btn" @tap="imgCheck">检查缓存</view>
      </view>
      <image v-if="imgUrl" class="img-preview" :src="imgUrl" mode="aspectFill" />
      <view class="result" v-if="imgResult">{{ imgResult }}</view>
      <view class="hint">命中缓存时返回 objectURL（不依赖网络），首次加载后异步落库。</view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import { storage } from '@/utils/storage'
import { localDB, type StoreSchema } from '@/utils/localDB'
import { apiCache, netStatus, imgCache } from '@/utils/cache'
import { videoApi } from '@/api/video'

// ---------- 引擎状态 ----------
const dbMode = ref<'idb' | 'kv'>('idb')
const online = ref(true)
const dbRows = ref<{ id: number; title: string; created_at: string }[]>([])

// ---------- KV ----------
const kvKey = ref('note')
const kvVal = ref('本地存储演示 ' + new Date().toLocaleTimeString())
const kvTTL = ref('')
const kvResult = ref('')
const kvKeys = ref<string[]>([])

function refreshKvKeys() {
  kvKeys.value = storage.keys()
}

function kvWrite() {
  if (!kvKey.value) return
  const ttl = kvTTL.value ? Number(kvTTL.value) : undefined
  storage.set(kvKey.value, kvVal.value, ttl)
  kvResult.value = `✓ 已写入 ${kvKey.value}${ttl ? `（TTL ${ttl}s）` : '（永久）'}`
  refreshKvKeys()
}

function kvRead() {
  const v = storage.get(kvKey.value)
  kvResult.value = v === undefined ? '✗ 不存在或已过期' : `✓ 读取到：${JSON.stringify(v)}`
}

function kvRemove() {
  storage.remove(kvKey.value)
  kvResult.value = `✗ 已删除 ${kvKey.value}`
  refreshKvKeys()
}

function kvClearAll() {
  storage.clear()
  kvResult.value = '✗ 已清空全部 KV'
  refreshKvKeys()
}

// ---------- 本地数据库 ----------
const SCHEMAS: StoreSchema[] = [
  { name: 'favorites', keyPath: 'id' },
  { name: 'history', keyPath: 'id' }
]

async function ensureDB() {
  if (!localDB.initialized) {
    await localDB.init('cakecake-localdb', SCHEMAS)
  }
}

async function dbAdd() {
  await ensureDB()
  const row = {
    title: `收藏视频 #${Date.now() % 100000}`,
    created_at: new Date().toLocaleString()
  }
  await localDB.add('favorites', row)
  await dbRefresh()
}

async function dbRefresh() {
  await ensureDB()
  dbRows.value = await localDB.query<{ id: number; title: string; created_at: string }>(
    'favorites',
    { sortBy: 'id', order: 'desc', page: 1, pageSize: 10 }
  )
}

async function dbRemove(id: number) {
  await localDB.delete('favorites', id)
  await dbRefresh()
}

async function dbClear() {
  await ensureDB()
  await localDB.clear('favorites')
  await dbRefresh()
}

// ---------- API 缓存 ----------
const cacheResult = ref('')
const cacheStats = ref({ count: 0, totalSize: 0 })

async function cacheFetch(mode: 'first' | 'again') {
  cacheResult.value = '请求中…'
  const t0 = Date.now()
  try {
    const resp = await videoApi.recommendation(undefined, 20)
    const cost = Date.now() - t0
    cacheResult.value = `${mode === 'first' ? '首次' : '再次'}请求：拿到 ${resp.items?.length || 0} 条，耗时 ${cost}ms` +
      (mode === 'again' ? '（应已命中缓存，耗时≈0ms）' : '（已写入缓存）')
  } catch (e: any) {
    cacheResult.value = `✗ 请求失败：${e?.message || e}（若已缓存会自动回退）`
  }
  refreshCacheStats()
}

function cacheClear() {
  apiCache.clear()
  cacheResult.value = '✗ 已清空 API 缓存'
  refreshCacheStats()
}

function refreshCacheStats() {
  cacheStats.value = apiCache.stats()
}

// ---------- 图片缓存 ----------
const imgUrl = ref('')
const imgResult = ref('')

async function imgLoad() {
  imgResult.value = '加载中…'
  const url = 'https://picsum.photos/seed/cakecake/400/240'
  try {
    imgUrl.value = await imgCache.cacheImage(url)
    imgResult.value = '✓ 已返回可用图片 URL（命中缓存时为本地 objectURL）'
  } catch (e: any) {
    imgResult.value = `✗ ${e?.message || e}`
  }
}

async function imgCheck() {
  const has = await imgCache.has('https://picsum.photos/seed/cakecake/400/240')
  imgResult.value = has ? '✓ 该图已在本地缓存中，离线可用' : '○ 尚未缓存（先点「加载封面图」）'
}

// ---------- 生命周期 ----------
onShow(async () => {
  online.value = netStatus.isOnline()
  netStatus.onOffline(() => (online.value = false))
  netStatus.onOnline(() => (online.value = true))

  await ensureDB()
  dbMode.value = localDB.mode
  refreshKvKeys()
  refreshCacheStats()
  await dbRefresh()
})

function goBack() {
  uni.navigateBack({ fail: () => uni.reLaunch({ url: '/pages/mine/index' }) })
}
</script>

<style lang="scss" scoped>
.demo-page {
  min-height: 100vh;
  background: #F6F7F9;
  padding: 24rpx;
  padding-bottom: 120rpx;
  box-sizing: border-box;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16rpx 8rpx 24rpx;

  .back {
    width: 64rpx;
    font-size: 48rpx;
    color: #181818;
    text-align: center;
  }
  .title {
    font-size: 34rpx;
    font-weight: 600;
    color: #181818;
  }
}

.card {
  background: #FFF;
  border-radius: 20rpx;
  padding: 28rpx;
  margin-bottom: 24rpx;

  .card-title {
    font-size: 30rpx;
    font-weight: 600;
    color: #181818;
    margin-bottom: 20rpx;
  }
}

.status-row {
  display: flex;
  gap: 16rpx;
  margin-bottom: 16rpx;

  .chip {
    padding: 10rpx 24rpx;
    border-radius: 30rpx;
    font-size: 24rpx;
    background: #F4F4F4;
    color: #999;

    &.on  { background: #E1F5EE; color: #16A34A; }
    &.off { background: #FBEAF0; color: #E5484D; }
  }
}

.hint {
  font-size: 22rpx;
  color: #999;
  line-height: 1.6;
  margin-top: 12rpx;
}

.form-row {
  display: flex;
  gap: 16rpx;
  margin-bottom: 16rpx;

  .input {
    flex: 1;
    background: #F6F7F9;
    border-radius: 12rpx;
    padding: 16rpx 20rpx;
    font-size: 26rpx;

    &.small { flex: 0.6; }
  }
}

.btn-row {
  display: flex;
  gap: 16rpx;
  margin-bottom: 16rpx;

  .btn {
    flex: 1;
    text-align: center;
    padding: 18rpx 0;
    border-radius: 12rpx;
    background: #F4F4F4;
    color: #333;
    font-size: 26rpx;

    &.primary { background: linear-gradient(90deg, #FB7299, #FF9DB5); color: #FFF; }
    &.danger  { background: #FBEAF0; color: #E5484D; }
  }
}

.result {
  background: #F6F7F9;
  border-radius: 12rpx;
  padding: 16rpx 20rpx;
  font-size: 24rpx;
  color: #333;
  margin-bottom: 12rpx;
  word-break: break-all;
}

.kv-list {
  .kv-item {
    display: flex;
    justify-content: space-between;
    padding: 14rpx 0;
    border-bottom: 1rpx solid #F1F1F1;
    font-size: 24rpx;

    .kv-key { color: #FB7299; }
    .kv-val { color: #666; max-width: 400rpx; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  }
}

.db-list {
  .db-item {
    display: flex;
    align-items: center;
    gap: 16rpx;
    padding: 18rpx 0;
    border-bottom: 1rpx solid #F1F1F1;

    .db-title { flex: 1; font-size: 26rpx; color: #333; }
    .db-time  { font-size: 20rpx; color: #999; }
    .db-del   { font-size: 22rpx; color: #E5484D; }
  }
}

.img-preview {
  width: 100%;
  height: 320rpx;
  border-radius: 16rpx;
  margin-bottom: 16rpx;
  background: #F4F4F4;
}
</style>
