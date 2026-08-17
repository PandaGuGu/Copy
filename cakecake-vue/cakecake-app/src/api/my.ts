import { request } from '@/utils/request'

/** 我的视频 */
export interface MyVideoItem {
  id: number
  title: string
  cover_url: string
  duration: number
  play_count: number
  like_count: number
  coin_count: number
  fav_count: number
  comment_count: number
  danmaku_count: number
  status: string
  created_at: string
}

export interface MyVideosResp {
  counts: { draft: number; passed: number; processing: number; rejected: number }
  items: MyVideoItem[]
}

/** 我的视频（真实：GET /api/v1/users/me/videos，需 auth） */
export const myApi = {
  videos(): Promise<MyVideosResp> {
    return request({ url: '/api/v1/users/me/videos', method: 'GET' })
  }
}