/**
 * 通用 API 类型定义（与后端 R-API-1 信封对齐）
 * 2026-08-16 按真实后端响应修正（联调对齐）
 */

/** 后端统一响应信封 { code, msg, data } */
export interface ApiEnvelope<T = unknown> {
  code: number
  msg: string
  data: T
}

/** 游标分页响应（真实后端：next_cursor，非 page/total） */
export interface CursorPageResp<T> {
  items: T[]
  next_cursor: string | null
}

/** 视频（真实字段，2026-08-16 联调对齐） */
export interface Video {
  id: number
  user_id: number
  title: string
  description: string
  cover_url: string
  video_url: string            // 相对路径，如 /uploads/xxx.mp4，播放时需拼 baseURL
  duration: number             // 秒
  play_count: number
  danmaku_count: number
  comment_count: number
  like_count: number
  coin_count: number
  fav_count: number            // 真实字段名（不是 favorite_count）
  danmaku_closed: boolean
  comments_closed: boolean
  comments_curated: boolean
  liked_by_me: boolean
  coined_by_me: boolean
  favorited_by_me: boolean
  followed_by_me: boolean
  in_watch_later: boolean
  my_coin_amount: number
  category: string
  zone: string
  zone_child: string
  zone_parent: string
  tags: string[]
  status: string               // published / processing / ...
  fail_reason: string
  watching_count: number
  uploader: string             // 真实是字符串用户名（不是对象）
  uploader_avatar_url?: string
  uploader_follower_count?: number
  uploader_published_count?: number
  uploader_sign?: string
  created_at: string
}

/** 评论（真实字段） */
export interface Comment {
  id: number
  user_id: number
  username: string
  avatar_url: string
  content: string
  created_at: string
  like_count: number
  liked_by_me: boolean
  disliked_by_me: boolean
  parent_id: number
  pinned: boolean
  level: number
  user_level: number
  ip_location: string
  is_by_uploader: boolean
}

/** 评论列表响应 */
export interface CommentListResp {
  comments_curated: boolean
  items: Comment[]
}

/** 发评论响应 */
export interface CreateCommentResp {
  id: number
  approved: boolean
  ip_location: string
}

/** 弹幕 */
export interface Danmaku {
  id: number
  content: string
  type: 'scroll' | 'top' | 'bottom'
  color: string               // #RRGGBB
  font_size: 'sm' | 'md' | 'lg'
  video_time: number          // 秒
  user?: string
  created_at?: string
}

/** 用户（真实 users/me 结构） */
export interface User {
  user_id: number
  username: string
  nickname: string
  avatar_url: string
  sign: string
  announcement: string
  gender: string
  birthday: string
  cake_id: string
  coin_balance: number        // 硬币
  creator_up_days: number
  first_published_at?: string
  created_at: string
  level_info: {
    current_level: number
    current_exp: number
    current_min: number
    next_exp: number
  }
  space_privacy: {
    public_favorites: boolean
    public_recent_coins: boolean
    public_following: boolean
    public_fans: boolean
    public_birthday: boolean
  }
  pending_deletion: boolean
}

/** 登录响应（真实：只有 token，无 user 对象） */
export interface LoginResp {
  access_token: string
  refresh_token: string
}

/** 动态（后端 user-dynamics 结构） */
export interface Dynamic {
  id: number
  user_id: number
  type: 'video' | 'article' | 'image' | 'text'
  content?: string
  created_at: string
  like_count: number
  comment_count: number
  forward_count: number
  images?: string[]
}

/** 分类（后端 categories 结构） */
export interface Category {
  id: number
  name: string
  icon_url?: string
  parent_id?: number
  sort: number
  video_count?: number
}

/** 轮播图（真实后端 /api/v1/home-banners 字段：2026-08-16 联调对齐） */
export interface Banner {
  id: number
  name: string                  // 真实字段：title → name
  pic: string                   // 真实字段：image_url → pic
  url?: string                  // 真实字段：link_url → url（站内路径，如 /#/video/BV89）
}

/** 直播间（真实 GET /live/rooms 字段） */
export interface LiveRoom {
  id: number
  user_id: number
  title: string
  cover_url: string
  status: 'live' | 'offline' | 'ended' | 'banned'
  viewer_count: number
  stream_key?: string
  host_name?: string
  avatar_url?: string
  started_at?: string
}

/** 商品（会员购，占位） */
export interface MallItem {
  id: number
  title: string
  cover_url: string
  price: number
  original_price?: number
  tags: string[]
  sale_count: number
  badge?: string
}
