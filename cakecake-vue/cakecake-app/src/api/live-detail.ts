import { request } from '@/utils/request'

export interface LiveRoom {
  id: number
  user_id: number
  title: string
  cover_url: string
  stream_key: string
  status: 'live' | 'ended' | 'banned' | 'paused'
  viewer_count: number
  host_name: string
  avatar_url: string
  started_at?: string
  ended_at?: string
  created_at: string
  updated_at: string
}

export const liveApi = {
  /** 直播房间详情 */
  detail(roomId: number): Promise<LiveRoom> {
    return request({ url: `/api/v1/live/room/${roomId}`, method: 'GET' })
  },
  /** 当前用户自己的直播房间（用于开播场景） */
  mine(): Promise<LiveRoom | null> {
    return request({ url: '/api/v1/live/room/my', method: 'GET' })
  }
}