import { request } from '@/utils/request'

export interface DMConversation {
  id: number
  peer_id: number
  peer_name: string
  peer_avatar: string
  kind: 'agent' | 'user'
  is_agent: boolean
  last_preview: string
  last_message_at: string
  unread_count: number
  pinned: boolean
  muted: boolean
  agent_profile_id?: number
}

export interface DMMessage {
  id: number
  conversation_id: number
  sender_id: number
  sender_name: string
  sender_avatar: string
  role: 'user' | 'assistant' | 'system'
  content: string
  created_at: string
}

export interface DMMessageListResp {
  items: DMMessage[]
  next_cursor: string
  peer_id: number
  peer_name: string
  peer_avatar: string
}

export const dmApi = {
  /** 会话列表 */
  conversations(): Promise<{ items: DMConversation[] }> {
    return request({ url: '/api/v1/dm/conversations', method: 'GET' })
  },
  /** 会话详情消息 */
  messages(conversationId: number, cursor?: string, limit = 30): Promise<DMMessageListResp> {
    return request({ url: `/api/v1/dm/conversations/${conversationId}/messages`, method: 'GET', params: { cursor, limit } })
  },
  /** 发送消息 */
  send(conversationId: number, content: string): Promise<DMMessage> {
    return request({ url: `/api/v1/dm/conversations/${conversationId}/messages`, method: 'POST', data: { content } })
  }
}
