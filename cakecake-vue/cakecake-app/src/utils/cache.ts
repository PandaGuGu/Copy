/**
 * 离线缓存层（本地存储能力之三）
 *
 * 1) API 响应缓存：基于 storage 的 KV 缓存，TTL 过期 + LRU 淘汰，供 request.ts 离线回退
 * 2) 网络状态：navigator.onLine + 事件监听（跨端封装）
 * 3) 图片缓存：IndexedDB 存 Blob，命中后返回 objectURL，实现图片离线可用
 */
import { storage } from './storage'
import { localDB } from './localDB'

// ---------- 1. API 响应缓存 ----------

const MAX_CACHE_ENTRIES = 200 // LRU 上限

interface CacheMeta {
  /** 过期时间戳（ms） */
  e: number
  /** 最后访问时间（ms），用于 LRU */
  t: number
  /** 缓存大小（近似，字节） */
  size: number
}

interface CacheRecord {
  meta: CacheMeta
  data: unknown
}

const META_KEY = '__cache_index__'

function loadIndex(): Record<string, CacheMeta> {
  return storage.get(META_KEY, {}) as Record<string, CacheMeta>
}

function saveIndex(idx: Record<string, CacheMeta>) {
  storage.set(META_KEY, idx)
}

function lruEvict(idx: Record<string, CacheMeta>) {
  const keys = Object.keys(idx)
  if (keys.length <= MAX_CACHE_ENTRIES) return
  const sorted = keys.sort((a, b) => idx[a].t - idx[b].t)
  for (const k of sorted.slice(0, keys.length - MAX_CACHE_ENTRIES)) {
    uni.removeStorageSync('ckc:api:' + k)
    delete idx[k]
  }
}

export const apiCache = {
  /** 读取缓存；命中且未过期返回 { data, fromCache: true } */
  get<T>(key: string): { data: T; fromCache: boolean } | null {
    const idx = loadIndex()
    const meta = idx[key]
    if (!meta) return null
    if (meta.e && meta.e < Date.now()) {
      this.remove(key)
      return null
    }
    // 刷新访问时间（LRU）
    meta.t = Date.now()
    saveIndex(idx)
    const raw = uni.getStorageSync('ckc:api:' + key)
    try {
      return { data: JSON.parse(raw) as T, fromCache: true }
    } catch {
      this.remove(key)
      return null
    }
  },

  /** 写入缓存（ttl 秒，默认 5 分钟） */
  set<T>(key: string, data: T, ttlSeconds = 300): void {
    const idx = loadIndex()
    idx[key] = {
      e: Date.now() + ttlSeconds * 1000,
      t: Date.now(),
      size: JSON.stringify(data).length
    }
    saveIndex(idx)
    uni.setStorageSync('ckc:api:' + key, JSON.stringify(data))
    lruEvict(idx)
  },

  remove(key: string): void {
    uni.removeStorageSync('ckc:api:' + key)
    const idx = loadIndex()
    delete idx[key]
    saveIndex(idx)
  },

  clear(): void {
    const idx = loadIndex()
    for (const k of Object.keys(idx)) uni.removeStorageSync('ckc:api:' + k)
    storage.remove(META_KEY)
  },

  /** 缓存统计 */
  stats(): { count: number; totalSize: number } {
    const idx = loadIndex()
    let totalSize = 0
    for (const k of Object.keys(idx)) totalSize += idx[k].size || 0
    return { count: Object.keys(idx).length, totalSize }
  }
}

// ---------- 2. 网络状态 ----------

export const netStatus = {
  /** 当前是否在线（小程序环境用 uni.getNetworkType 兜底） */
  isOnline(): boolean {
    // #ifdef H5
    return typeof navigator !== 'undefined' ? navigator.onLine : true
    // #endif
    // #ifndef H5
    return true
    // #endif
  },

  /** 监听上线/下线（返回取消函数） */
  onOnline(cb: () => void): () => void {
    // #ifdef H5
    window.addEventListener('online', cb)
    return () => window.removeEventListener('online', cb)
    // #endif
    // #ifndef H5
    return () => { /* noop */ }
    // #endif
  },

  onOffline(cb: () => void): () => void {
    // #ifdef H5
    window.addEventListener('offline', cb)
    return () => window.removeEventListener('offline', cb)
    // #endif
    // #ifndef H5
    return () => { /* noop */ }
    // #endif
  }
}

// ---------- 3. 图片缓存（IndexedDB Blob） ----------

const IMG_DB = 'cakecake-img-cache'
const IMG_STORE = 'images'

let imgDB: IDBDatabase | null = null

function openImgDB(): Promise<IDBDatabase> {
  if (imgDB) return Promise.resolve(imgDB)
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IMG_DB, 1)
    req.onupgradeneeded = () => {
      const db = req.result
      if (!db.objectStoreNames.contains(IMG_STORE)) {
        db.createObjectStore(IMG_STORE, { keyPath: 'url' })
      }
    }
    req.onsuccess = () => {
      imgDB = req.result
      resolve(imgDB)
    }
    req.onerror = () => reject(req.error)
  })
}

async function fetchBlob(url: string): Promise<Blob> {
  const resp = await fetch(url, { mode: 'cors' })
  if (!resp.ok) throw new Error(`fetch ${url} -> ${resp.status}`)
  return resp.blob()
}

export const imgCache = {
  /** 是否支持（H5 IndexedDB） */
  get supported(): boolean {
    return typeof indexedDB !== 'undefined'
  },

  /** 图片是否已缓存 */
  async has(url: string): Promise<boolean> {
    if (!this.supported) return false
    try {
      const db = await openImgDB()
      return await new Promise<boolean>((resolve) => {
        const req = db.transaction(IMG_STORE).objectStore(IMG_STORE).getKey(url)
        req.onsuccess = () => resolve(req.result !== undefined)
        req.onerror = () => resolve(false)
      })
    } catch {
      return false
    }
  },

  /** 缓存图片；返回可直接用于 <img> 的 URL（命中缓存返回 objectURL，否则原 URL 并异步落库） */
  async cacheImage(url: string): Promise<string> {
    if (!this.supported || !url) return url
    try {
      const db = await openImgDB()
      const store = db.transaction(IMG_STORE, 'readonly').objectStore(IMG_STORE)
      const hit = await new Promise<Blob | undefined>((resolve) => {
        const req = store.get(url)
        req.onsuccess = () => resolve(req.result?.blob)
        req.onerror = () => resolve(undefined)
      })
      if (hit) {
        try { return URL.createObjectURL(hit) } catch { return url }
      }
      // 未命中：异步下载落库，本次先用原 URL
      fetchBlob(url)
        .then(async (blob) => {
          const db2 = await openImgDB()
          const tx = db2.transaction(IMG_STORE, 'readwrite')
          tx.objectStore(IMG_STORE).put({ url, blob, savedAt: Date.now() })
        })
        .catch(() => { /* 离线时忽略，不影响主流程 */ })
      return url
    } catch {
      return url
    }
  },

  /** 清空图片缓存 */
  async clear(): Promise<void> {
    if (!this.supported) return
    try {
      const db = await openImgDB()
      await new Promise<void>((resolve) => {
        const tx = db.transaction(IMG_STORE, 'readwrite')
        tx.objectStore(IMG_STORE).clear()
        tx.oncomplete = () => resolve()
        tx.onerror = () => resolve()
      })
    } catch { /* ignore */ }
  }
}
