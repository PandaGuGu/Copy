import { request } from '@/utils/request'

export interface HistoryItem {
  video_id: number
  article_id?: number
  media_type: 'video' | 'article' | 'live' | 'album'
  title: string
  cover_url: string
  progress_sec: number
  duration_sec: number
  viewed_at: string
  viewed_time: string
  device: string
  category: string
  uploader_id: number
  uploader_name: string
  uploader_avatar_url: string
}

export const historyApi = {
  /** 观看历史 */
  list(cursor?: string, limit = 20): Promise<{ items: HistoryItem[]; paused: boolean; total: number }> {
    return request({ url: '/api/v1/users/me/view-history', method: 'GET', params: { cursor, limit } })
  },
  /** 删除单条历史 */
  remove(videoId: number): Promise<void> {
    return request({ url: `/api/v1/users/me/view-history/${videoId}`, method: 'DELETE' })
  },
  /** 清空历史 */
  clear(): Promise<void> {
    return request({ url: '/api/v1/users/me/view-history', method: 'DELETE' })
  }
}
