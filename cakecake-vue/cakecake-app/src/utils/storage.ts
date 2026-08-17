/**
 * KV 持久化层（本地存储能力之一）
 *
 * 基于 uni.setStorageSync / getStorageSync 封装，跨端可用（H5 / 小程序 / App）。
 * 特性：
 *  - 统一命名空间前缀 `ckc:`，避免污染全局 storage
 *  - 自动 JSON 序列化（支持对象/数组/基本类型）
 *  - 可选 TTL 过期（秒），读取时惰性过期
 *
 * 用途：登录态、用户偏好、草稿、设置项等 KV 数据。
 */
const NS = 'ckc:'

interface Envelope<T> {
  v: T
  /** 过期时间戳（毫秒），0 表示永不过期 */
  e: number
}

function wrapKey(key: string): string {
  return NS + key
}

export const storage = {
  /** 读取；不存在或已过期返回 def */
  get<T>(key: string, def?: T): T | undefined {
    try {
      const raw = uni.getStorageSync(wrapKey(key)) as string | undefined
      if (!raw) return def
      const env = JSON.parse(raw) as Envelope<T>
      if (env.e && env.e < Date.now()) {
        uni.removeStorageSync(wrapKey(key))
        return def
      }
      return env.v
    } catch {
      return def
    }
  },

  /** 写入；ttlSeconds 为过期秒数（缺省永不过期） */
  set<T>(key: string, value: T, ttlSeconds?: number): void {
    const env: Envelope<T> = {
      v: value,
      e: ttlSeconds ? Date.now() + ttlSeconds * 1000 : 0
    }
    uni.setStorageSync(wrapKey(key), JSON.stringify(env))
  },

  /** 删除 */
  remove(key: string): void {
    uni.removeStorageSync(wrapKey(key))
  },

  /** 是否存在且未过期 */
  has(key: string): boolean {
    return this.get(key) !== undefined
  },

  /** 当前命名空间下所有 key（不含前缀） */
  keys(): string[] {
    const keys: string[] = []
    try {
      const info = uni.getStorageInfoSync()
      for (const k of info.keys || []) {
        if (k.startsWith(NS)) keys.push(k.slice(NS.length))
      }
    } catch { /* ignore */ }
    return keys
  },

  /** 当前命名空间条目数 */
  size(): number {
    return this.keys().length
  },

  /** 清空本命名空间（不动其他 App 的 storage） */
  clear(): void {
    for (const k of this.keys()) uni.removeStorageSync(wrapKey(k))
  }
}
