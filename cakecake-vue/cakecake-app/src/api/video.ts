import { request } from '@/utils/request'
import type { Comment, CommentListResp, CreateCommentResp, CursorPageResp, Danmaku, Video } from '@/utils/types'

/** 排行榜视频（真实 GET /leaderboard：Go 默认字段名无 json tag，大写开头） */
export interface LeaderboardVideo {
  ID: number
  UserID: number
  Title: string
  Description: string
  DurationSec: number
  Status: string
  VideoURL: string
  CoverURL: string
  PlayCount: number
  DanmakuCount: number
  CommentCount: number
}

export const videoApi = {
  /** 推荐流（F17 ItemCF / F14 规则，真实：游标分页） */
  recommendation(cursor?: string, limit = 20): Promise<CursorPageResp<Video>> {
    return request({ url: '/api/v1/feed/recommendation', method: 'GET', params: { cursor, limit } })
  },
  /** 视频列表（与 PC 端 getHomeRecommendPool 完全同源：GET /api/v1/videos?limit=&cursor=） */
  list(cursor?: string, limit = 50): Promise<CursorPageResp<Video>> {
    return request({ url: '/api/v1/videos', method: 'GET', params: { limit, cursor } })
  },
  /** 排行榜（真实 GET /leaderboard，返回数组；B站热门榜样式） */
  leaderboard(limit = 50): Promise<LeaderboardVideo[]> {
    return request({ url: '/api/v1/leaderboard', method: 'GET', params: { limit } })
  },
  /** 视频详情 */
  detail(id: number): Promise<Video> {
    return request({ url: `/api/v1/videos/${id}`, method: 'GET' })
  },
  /** 点赞/取消点赞 */
  toggleLike(id: number): Promise<{ liked: boolean; count: number }> {
    return request({ url: `/api/v1/videos/${id}/like`, method: 'POST' })
  },
  /** 投币 */
  coin(id: number): Promise<{ amount: number; coin_balance: number; coin_count: number; coined: boolean; my_coin_amount: number }> {
    return request({ url: `/api/v1/videos/${id}/coin`, method: 'POST' })
  },
  /** 收藏/取消收藏 */
  toggleFavorite(id: number): Promise<{ favorited: boolean; fav_count: number }> {
    return request({ url: `/api/v1/videos/${id}/favorite`, method: 'POST' })
  },
  /** 评论置顶（UP主专属） */
  pinComment(commentId: number): Promise<void> {
    return request({ url: `/api/v1/comments/${commentId}/pin`, method: 'POST' })
  },
  /** 评论精选（UP主专属） */
  approveComment(commentId: number): Promise<void> {
    return request({ url: `/api/v1/comments/${commentId}/approve`, method: 'POST' })
  },
  /** 评论列表 */
  comments(id: number, cursor?: string): Promise<CommentListResp> {
    return request({ url: `/api/v1/videos/${id}/comments`, method: 'GET', params: { cursor } })
  },
  /** 发评论 */
  createComment(id: number, content: string, parentId?: number): Promise<CreateCommentResp> {
    return request({
      url: `/api/v1/videos/${id}/comments`,
      method: 'POST',
      data: parentId ? { content, parent_id: parentId } : { content }
    })
  },
  /** 发弹幕（type 白名单：scroll/top/bottom；color 空默认 #FFFFFF） */
  sendDanmaku(id: number, d: { content: string; video_time: number; type?: 'scroll' | 'top' | 'bottom'; color?: string }): Promise<Danmaku> {
    return request({
      url: `/api/v1/videos/${id}/danmaku`,
      method: 'POST',
      data: {
        content: d.content,
        video_time: d.video_time,
        type: d.type || 'scroll',
        color: d.color || ''
      }
    })
  }
}

export type { Comment }
