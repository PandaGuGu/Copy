import { request } from '@/utils/request'
import type { CursorPageResp, Video } from '@/utils/types'

/** 个人空间信息（真实 GET /space/:user） */
export interface SpaceInfo {
  user_id: number
  nickname: string
  avatar_url: string
  sign: string
  announcement: string
  gender: string
  birthday: string
  cake_id: string
  follower_count: number
  following_count: number
  published_count: number
  followed_by_me: boolean
  is_owner: boolean
  level_info: {
    current_level: number
    current_exp: number
    current_min: number
    next_exp: number
  }
  privacy: {
    public_favorites: boolean
    public_recent_coins: boolean
    public_following: boolean
    public_fans: boolean
    public_birthday: boolean
  }
}

export const spaceApi = {
  /** 个人空间主页（GET /space/:userId） */
  info(userId: number): Promise<SpaceInfo> {
    return request({ url: `/api/v1/space/${userId}`, method: 'GET' })
  },
  /** 该用户投稿列表（GET /space/:userId/videos，游标分页） */
  videos(userId: number, cursor?: string, limit = 20): Promise<CursorPageResp<Video>> {
    return request({ url: `/api/v1/space/${userId}/videos`, method: 'GET', params: { cursor, limit } })
  },
  /** 关注 / 取关 */
  follow(userId: number): Promise<void> {
    return request({ url: `/api/v1/users/${userId}/follow`, method: 'POST' })
  },
  unfollow(userId: number): Promise<void> {
    return request({ url: `/api/v1/users/${userId}/follow`, method: 'DELETE' })
  }
}