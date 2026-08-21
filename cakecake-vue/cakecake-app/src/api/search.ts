import { request } from '@/utils/request'

/** 热搜词条（真实 GET /hot-search） */
export interface HotSearchItem {
  badge: string      // "热" /"" / "新"
  rank: number
  title: string
}

/** 搜索建议标签（真实 GET /search/suggest） */
export interface SuggestTag {
  name: string
  value: string
}

/** 搜索结果视频（真实 GET /search：B 站 BV 风格字段） */
export interface SearchVideo {
  aid: number
  title: string
  pic: string
  duration: string      // "MM:SS"
  play: number
  video_review?: number // 弹幕数
  pubdate: number       // unix timestamp
  author: string
  mid: number           // up 主 id
}

export interface SearchResult {
  result: {
    video?: SearchVideo[]
    article?: unknown[]
    bili_user?: unknown[]
    [k: string]: unknown
  }
  top_tlist?: Record<string, number>
  pageinfo?: Record<string, unknown>
}

export const searchApi = {
  /** 真实搜索（GET /search?keyword=&page=&page_size=）
   *  ⚠️ App 端（uni-app axios 适配器）GET params 可能不序列化 → 手动拼 URL 保证 keyword 送达 */
  search(keyword: string, page = 1, pageSize = 20): Promise<SearchResult> {
    const q = `keyword=${encodeURIComponent(keyword)}&page=${page}&page_size=${pageSize}`
    return request({ url: `/api/v1/search?${q}`, method: 'GET' })
  },
  /** 热搜榜（GET /hot-search） */
  hotSearch(): Promise<{ items: HotSearchItem[] }> {
    return request({ url: '/api/v1/hot-search', method: 'GET' })
  },
  /** 搜索建议（GET /search/suggest?keyword=） */
  suggest(keyword: string): Promise<{ tag: SuggestTag[] }> {
    return request({ url: '/api/v1/search/suggest', method: 'GET', params: { keyword } })
  },
  /** 我的搜索历史（GET /users/me/search-history，auth） */
  history(): Promise<{ keywords: string[] }> {
    return request({ url: '/api/v1/users/me/search-history', method: 'GET' })
  },
  /** 记录搜索历史（POST /users/me/search-history，auth） */
  saveHistory(keyword: string): Promise<unknown> {
    return request({ url: '/api/v1/users/me/search-history', method: 'POST', data: { keyword } })
  }
}