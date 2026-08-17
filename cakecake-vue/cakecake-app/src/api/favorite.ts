import { request } from '@/utils/request'

export interface FavoriteVideo {
  id: number
  title: string
  cover_url: string
  duration: number
  play_count: number
  danmaku_count: number
  uploader: string
  uploader_id: number
  uploader_avatar_url: string
  folder_id: number
  created_at: string
  favorited_at: string
}

export const favoriteApi = {
  /** 我的收藏列表 */
  mine(cursor?: string, limit = 20): Promise<{ items: FavoriteVideo[]; total: number }> {
    return request({ url: '/api/v1/users/me/favorites', method: 'GET', params: { cursor, limit } })
  },
  /** 收藏/取消收藏（toggle） */
  toggle(videoId: number): Promise<{ fav_count: number; favorited: boolean }> {
    return request({ url: `/api/v1/videos/${videoId}/favorite`, method: 'POST' })
  }
}
