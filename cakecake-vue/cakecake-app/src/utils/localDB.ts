/**
 * 本地结构化数据库（本地存储能力之二）
 *
 * H5 端基于 IndexedDB（可存大对象、Blob，容量远超 localStorage）；
 * 非 H5（小程序/App）环境自动降级为内存 + KV storage 模拟，接口保持一致。
 *
 * 能力：建表（store）→ 增删改查 → 内存级过滤/排序/分页查询。
 * 用途：收藏、观看历史、离线数据等结构化数据。
 */
import { storage } from './storage'

export interface StoreSchema {
  name: string
  /** 主键字段名，默认 'id' */
  keyPath?: string
  /** 附加索引字段（仅 IndexedDB 模式生效） */
  indexes?: string[]
}

export interface QueryOptions<T = Record<string, any>> {
  /** 过滤条件（内存过滤） */
  filter?: (doc: T) => boolean
  /** 排序字段 */
  sortBy?: string
  /** 排序方向 */
  order?: 'asc' | 'desc'
  /** 页码（从 1 起），与 pageSize 同传才生效 */
  page?: number
  pageSize?: number
}

interface FallbackTable {
  rows: Record<string, any>[]
  seq: number
}

const FALLBACK_KEY = 'ckc:localdb:'

class LocalDBImpl {
  private db: IDBDatabase | null = null
  private dbName = 'cakecake-localdb'
  private stores: StoreSchema[] = []
  /** fallback 模式（非 H5）下保存所有表 */
  private fallback: Record<string, FallbackTable> = {}
  private _initialized = false

  /** 是否已完成初始化 */
  get initialized(): boolean {
    return this._initialized
  }

  /** 当前模式：idb | kv */
  get mode(): 'idb' | 'kv' {
    return this.db ? 'idb' : 'kv'
  }

  /** 是否支持 IndexedDB */
  static get supported(): boolean {
    return typeof indexedDB !== 'undefined'
  }

  async init(name = 'cakecake-localdb', stores: StoreSchema[] = []): Promise<void> {
    if (this._initialized && this.dbName === name) return
    this.dbName = name
    this.stores = stores
    this._initialized = true
    if (LocalDBImpl.supported && stores.length > 0) {
      try {
        await this.openDB()
        return
      } catch {
        this.db = null
      }
    }
    // 降级：从 KV 恢复 fallback 数据
    for (const s of stores) {
      this.fallback[s.name] = (storage.get(s.name, []) as Record<string, any>[]).reduce<FallbackTable>(
        (acc, row) => {
          acc.rows.push(row)
          acc.seq = Math.max(acc.seq, Number(row[s.keyPath || 'id']) || 0)
          return acc
        },
        { rows: [], seq: 0 }
      )
    }
  }

  private openDB(): Promise<void> {
    return new Promise((resolve, reject) => {
      const req = indexedDB.open(this.dbName, 1)
      req.onupgradeneeded = () => {
        const db = req.result
        for (const s of this.stores) {
          if (!db.objectStoreNames.contains(s.name)) {
            // keyPath + autoIncrement：有显式 keyPath 时自动生成主键并回填到该字段，
            // 文档自带主键值时以文档为准（兼容 add/put 两种用法）
            const os = db.createObjectStore(s.name, {
              keyPath: s.keyPath || 'id',
              autoIncrement: true
            })
            for (const idx of s.indexes || []) {
              if (!os.indexNames.contains(idx)) os.createIndex(idx, idx)
            }
          }
        }
      }
      req.onsuccess = () => {
        this.db = req.result
        resolve()
      }
      req.onerror = () => reject(req.error)
    })
  }

  // ---------- 底层事务 ----------

  private tx<T>(storeName: string, mode: IDBTransactionMode, fn: (os: IDBObjectStore) => IDBRequest): Promise<T> {
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(storeName, mode)
      const req = fn(tx.objectStore(storeName))
      req.onsuccess = () => resolve(req.result as T)
      req.onerror = () => reject(req.error)
    })
  }

  private fallbackGet(storeName: string): FallbackTable {
    if (!this.fallback[storeName]) this.fallback[storeName] = { rows: [], seq: 0 }
    return this.fallback[storeName]
  }

  private fallbackSave(storeName: string) {
    storage.set(storeName, this.fallback[storeName].rows)
  }

  // ---------- CRUD ----------

  /** 新增（自动生成主键时回填 id） */
  async add<T extends Record<string, any>>(storeName: string, doc: T): Promise<T> {
    if (this.db) {
      const key = await this.tx<any>(storeName, 'readwrite', (os) => os.add(doc))
      return { ...doc, [this.keyPathOf(storeName)]: key ?? doc[this.keyPathOf(storeName)] }
    }
    const table = this.fallbackGet(storeName)
    const kp = this.keyPathOf(storeName)
    const rec = doc as Record<string, any>
    if (!rec[kp]) rec[kp] = ++table.seq
    table.rows.push({ ...rec })
    this.fallbackSave(storeName)
    return doc
  }

  /** 新增或覆盖（按主键） */
  async put<T extends Record<string, any>>(storeName: string, doc: T): Promise<T> {
    if (this.db) {
      await this.tx(storeName, 'readwrite', (os) => os.put(doc))
      return doc
    }
    const table = this.fallbackGet(storeName)
    const kp = this.keyPathOf(storeName)
    const rec = doc as Record<string, any>
    if (!rec[kp]) rec[kp] = ++table.seq
    const idx = table.rows.findIndex((r) => r[kp] === rec[kp])
    if (idx >= 0) table.rows[idx] = { ...rec }
    else table.rows.push({ ...rec })
    this.fallbackSave(storeName)
    return doc
  }

  /** 按主键读取 */
  async get<T>(storeName: string, key: any): Promise<T | null> {
    if (this.db) {
      const r = await this.tx<T | undefined>(storeName, 'readonly', (os) => os.get(key))
      return r ?? null
    }
    const table = this.fallbackGet(storeName)
    const row = table.rows.find((r) => r[this.keyPathOf(storeName)] === key)
    return (row as T) ?? null
  }

  /** 按主键删除 */
  async delete(storeName: string, key: any): Promise<void> {
    if (this.db) {
      await this.tx(storeName, 'readwrite', (os) => os.delete(key))
      return
    }
    const table = this.fallbackGet(storeName)
    const kp = this.keyPathOf(storeName)
    table.rows = table.rows.filter((r) => r[kp] !== key)
    this.fallbackSave(storeName)
  }

  /** 全量读取 */
  async all<T>(storeName: string): Promise<T[]> {
    if (this.db) {
      return this.tx<T[]>(storeName, 'readonly', (os) => os.getAll())
    }
    return this.fallbackGet(storeName).rows as T[]
  }

  /** 计数 */
  async count(storeName: string): Promise<number> {
    if (this.db) {
      return this.tx<number>(storeName, 'readonly', (os) => os.count())
    }
    return this.fallbackGet(storeName).rows.length
  }

  /** 清空表 */
  async clear(storeName: string): Promise<void> {
    if (this.db) {
      await this.tx(storeName, 'readwrite', (os) => os.clear())
      return
    }
    this.fallbackGet(storeName).rows = []
    this.fallbackGet(storeName).seq = 0
    this.fallbackSave(storeName)
  }

  /** 结构化查询：过滤 + 排序 + 分页（内存执行，数据量小场景足够） */
  async query<T>(storeName: string, opts: QueryOptions<T> = {}): Promise<T[]> {
    let rows = await this.all<T>(storeName)
    if (opts.filter) rows = rows.filter(opts.filter)
    if (opts.sortBy) {
      const k = opts.sortBy
      const dir = opts.order === 'desc' ? -1 : 1
      rows = [...rows].sort((a, b) => {
        const av = (a as Record<string, any>)[k]
        const bv = (b as Record<string, any>)[k]
        if (av == null) return 1
        if (bv == null) return -1
        return av > bv ? dir : av < bv ? -dir : 0
      })
    }
    if (opts.page && opts.pageSize) {
      const start = (opts.page - 1) * opts.pageSize
      rows = rows.slice(start, start + opts.pageSize)
    }
    return rows
  }

  private keyPathOf(storeName: string): string {
    return this.stores.find((s) => s.name === storeName)?.keyPath || 'id'
  }
}

/** 全局单例（在 main.ts 或页面中 init 后使用） */
export const localDB = new LocalDBImpl()

export const FALLBACK_DB_KEY = FALLBACK_KEY
