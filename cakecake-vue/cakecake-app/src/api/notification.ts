import { request } from '@/utils/request'

/** 通知条目（真实 GET /notifications?category=） */
export interface NotificationItem {
  id: number
  type: string                    // reply_received / like_aggregation / system_notice / at_me / my_message
  comment_preview: string
  message: string
  sender_username: string
  total_likes: number
  inbox_kind: string
  is_read: boolean
  created_at: string
}

/** 未读汇总（真实 GET /notifications/unread-summary） */
export interface UnreadSummary {
  at_me: number
  like_aggregation: number
  my_message: number
  reply_received: number
  system_notice: number
}

export const notificationApi = {
  list(category: string, cursor?: string, limit = 30): Promise<{ items: NotificationItem[]; next_cursor: string }> {
    return request({ url: '/api/v1/notifications', method: 'GET', params: { category, cursor, limit } })
  },
  unreadSummary(): Promise<UnreadSummary> {
    return request({ url: '/api/v1/notifications/unread-summary', method: 'GET' })
  }
}