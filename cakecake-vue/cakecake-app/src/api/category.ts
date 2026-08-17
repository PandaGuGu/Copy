import { request } from '@/utils/request'
import type { CursorPageResp, Video } from '@/utils/types'

/** 分区（真实 GET /zones） */
export interface Zone {
  name: string
  video_count: number
}

export const categoryApi = {
  /** 全部分区（后端真实：GET /api/v1/zones，按视频数倒序） */
  all(): Promise<{ items: Zone[]; total: number }> {
    return request({ url: '/api/v1/zones', method: 'GET' })
  },
  /** 分区视频流（真实：GET /api/v1/zones/:zone/recommendation，游标分页） */
  zoneVideos(zone: string, cursor?: string, limit = 20): Promise<CursorPageResp<Video>> {
    return request({
      url: `/api/v1/zones/${encodeURIComponent(zone)}/recommendation`,
      method: 'GET',
      params: { cursor, limit }
    })
  }
}