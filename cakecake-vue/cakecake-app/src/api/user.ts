import { request } from '@/utils/request'
import type { CommentListResp, CursorPageResp, Dynamic, LiveRoom, User } from '@/utils/types'

export const dynamicApi = {
  /** 关注的人最新动态（2026-08-16 后端新增：GET /dynamics/following，游标分页） */
  following(cursor?: string, limit = 20): Promise<CursorPageResp<Dynamic>> {
    return request({ url: '/api/v1/dynamics/following', method: 'GET', params: { cursor, limit } })
  },
  /** 我的动态（GET /users/me/dynamics） */
  mine(limit = 20): Promise<{ items: Dynamic[]; next_cursor?: string }> {
    return request({ url: '/api/v1/users/me/dynamics', method: 'GET', params: { limit } })
  },
  /** 动态详情（GET /user-dynamics/:id） */
  detail(id: number): Promise<Dynamic> {
    return request({ url: `/api/v1/user-dynamics/${id}`, method: 'GET' })
  },
  /** 动态点赞/取消（POST /user-dynamics/:id/like） */
  toggleLike(id: number): Promise<{ liked: boolean; like_count_delta: number }> {
    return request({ url: `/api/v1/user-dynamics/${id}/like`, method: 'POST' })
  },
  /** 动态评论列表（GET /user-dynamics/:id/comments） */
  comments(id: number): Promise<CommentListResp> {
    return request({ url: `/api/v1/user-dynamics/${id}/comments`, method: 'GET' })
  },
  /** 发动态评论（POST /user-dynamics/:id/comments） */
  createComment(id: number, content: string): Promise<{ id: number; approved: boolean }> {
    return request({ url: `/api/v1/user-dynamics/${id}/comments`, method: 'POST', data: { content } })
  },
  /**
   * 发图文动态（真实：POST /api/v1/users/me/dynamics，**multipart/form-data**，不是 JSON）
   * 后端字段：title(≤20) + content(≤233) + images[](file)
   * 走原生 fetch + FormData，避开 axios envelope 解封
   */
  async createText(token: string, content: string, title = ''): Promise<Dynamic> {
    const fd = new FormData()
    fd.append('title', title)
    fd.append('content', content)
    // App 端无 vite 代理，必须用局域网 IP（与 request.ts 同款条件编译）
    // #ifdef APP-PLUS
    const apiBase = import.meta.env.VITE_API_BASE_URL_APP || 'http://192.168.1.100:8080'
    // #endif
    // #ifndef APP-PLUS
    const apiBase = import.meta.env.VITE_API_BASE_URL || ''
    // #endif
    const baseURL = apiBase + '/api/v1/users/me/dynamics'
    const resp = await fetch(baseURL, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
      body: fd
    })
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    const data = await resp.json()
    if (data.code !== 0) throw new Error(data.msg || '发布失败')
    return data.data
  }
}

export const userApi = {
  detail(id: number): Promise<User> {
    return request({ url: `/api/v1/users/${id}`, method: 'GET' })
  },
  /** 我的关注列表（后端新增：GET /users/me/followings，无隐私限制） */
  myFollowings(limit = 100): Promise<{ items: User[]; total: number }> {
    return request({ url: '/api/v1/users/me/followings', method: 'GET', params: { limit } })
  },
  follow(id: number): Promise<void> {
    return request({ url: `/api/v1/users/${id}/follow`, method: 'POST' })
  },
  unfollow(id: number): Promise<void> {
    return request({ url: `/api/v1/users/${id}/follow`, method: 'DELETE' })
  }
}

export const liveApi = {
  /** 直播房间列表 */
  rooms(): Promise<{ rooms: LiveRoom[] }> {
    return request({ url: '/api/v1/live/rooms', method: 'GET' })
  }
}