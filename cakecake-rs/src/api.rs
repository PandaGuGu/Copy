//! 后端数据层 —— 只封装真实存在的接口，不造数据
//!
//! 信封 `{code,msg,data}`；列表接口用 `{items,...}` 或 `{rooms,total}`。
//! 后端不可达时返回 Err，由 UI 显示空态，绝不回落到假数据。
//!
//! 登录态由 [`Client`] 自己持有：登录后存下 access/refresh 两个 token，
//! 任何鉴权接口返回 `40100`（未登录或 Token 已过期）时自动用 refresh_token
//! 换一对新 token 并重试一次；刷新失败才清空登录态，让 UI 回到登录页。

use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Video {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub danmaku_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub like_count: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LiveRoom {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub stream_key: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub viewer_count: i64,
    #[serde(default)]
    pub host_name: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub started_at: Option<String>,
}

/// 搜索结果里的视频（B 站 aid 语义，和 `Video` 不是同一套字段）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct VideoHit {
    #[serde(default)]
    pub aid: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub pic: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub play: u64,
    #[serde(default)]
    pub video_review: u64,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub mid: u64,
}

/// 视频详情（GET /api/v1/videos/:id）
///
/// 字段对齐后端 `videoDetail()`：`video_url` 是可播放地址（转码后的 mp4 / HLS），
/// 带上 UP 主信息与互动态，够详情页把播放器、UP 主区、操作栏都画出来。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct VideoDetail {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub danmaku_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub fav_count: u64,
    #[serde(default)]
    pub coin_count: u64,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub uploader: String,
    #[serde(default)]
    pub uploader_avatar_url: String,
    #[serde(default)]
    pub uploader_sign: String,
    #[serde(default)]
    pub uploader_follower_count: u64,
    #[serde(default)]
    pub uploader_published_count: u64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub video_url: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub watching_count: i64,
    #[serde(default)]
    pub liked_by_me: bool,
    #[serde(default)]
    pub coined_by_me: bool,
    #[serde(default)]
    pub favorited_by_me: bool,
    #[serde(default)]
    pub followed_by_me: bool,
    #[serde(default)]
    pub in_watch_later: bool,
    #[serde(default, deserialize_with = "null_vec")]
    pub tags: Vec<String>,
}

/// 收藏夹（GET /users/me/favorite-folders）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FavoriteFolder {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_public: bool,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub video_count: u64,
    #[serde(default)]
    pub cover_url: Option<String>,
    /// 详情页收藏弹窗里才有：是否已收藏到该夹
    #[serde(default)]
    pub selected: bool,
    /// 详情页收藏弹窗里才有：“12” 或 “12/999”
    #[serde(default)]
    pub count_label: String,
}

/// 详情页收藏夹选择弹窗（GET /videos/:id/favorite-picker）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FavoritePicker {
    #[serde(default)]
    pub favorited: bool,
    #[serde(default)]
    pub fav_count: u64,
    #[serde(default)]
    pub folder_ids: Vec<u64>,
    #[serde(default)]
    pub items: Vec<FavoriteFolder>,
}

/// 收藏夹里的视频（GET /users/me/favorites）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FavoriteItem {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub danmaku_count: u64,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub uploader: String,
    #[serde(default)]
    pub uploader_id: u64,
    #[serde(default)]
    pub favorited_at: String,
    #[serde(default)]
    pub folder_id: u64,
}

/// 关注页：我关注的 UP 主（/users/me/followings）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FollowUser {
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub sign: String,
}

/// 关注页：文字/图文动态（/dynamics/following）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DynItem {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub liked_by_me: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub author_id: u64,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_avatar: String,
}

/// 关注页：关注的人的投稿（/feed/subscription，直接返回 model.Video，字段是 Go 的驼峰名）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SubVideo {
    #[serde(rename = "ID", default)]
    pub id: u64,
    #[serde(rename = "UserID", default)]
    pub user_id: u64,
    #[serde(rename = "Title", default)]
    pub title: String,
    #[serde(rename = "Description", default)]
    pub description: String,
    #[serde(rename = "CoverURL", default)]
    pub cover_url: String,
    #[serde(rename = "DurationSec", default)]
    pub duration: f64,
    #[serde(rename = "PlayCount", default)]
    pub play_count: u64,
    #[serde(rename = "DanmakuCount", default)]
    pub danmaku_count: u64,
    #[serde(rename = "CommentCount", default)]
    pub comment_count: u64,
    #[serde(rename = "LikeCount", default)]
    pub like_count: u64,
    #[serde(rename = "CreatedAt", default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Danmaku {
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub video_time: f64,
    #[serde(default)]
    pub like_count: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Comment {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub liked_by_me: bool,
    #[serde(default)]
    pub disliked_by_me: bool,
    /// 父评论 id（0 = 一级评论），后端最多三层
    #[serde(default)]
    pub parent_id: u64,
    #[serde(default)]
    pub level: u64,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub user_level: u64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub is_by_uploader: bool,
    /// 评论时所在省份（后端 iplocate 归一化后的展示名，可能为空）
    #[serde(default)]
    pub ip_location: String,
}

/// 稍后再看列表项（GET /users/me/watch-later）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WatchLaterItem {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub danmaku_count: u64,
    /// 后端给的是 DurationSec（浮点秒）
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub uploader: String,
    #[serde(default)]
    pub uploader_id: u64,
    #[serde(default)]
    pub uploader_avatar_url: String,
    #[serde(default)]
    pub watched: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub added_at: String,
}

/// 观看历史列表项（GET /users/me/view-history）
///
/// 同一份列表里混有视频/专栏/直播，客户端只打开视频条目。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ViewHistoryItem {
    #[serde(default)]
    pub media_type: String,
    #[serde(default)]
    pub video_id: u64,
    #[serde(default)]
    pub article_id: u64,
    #[serde(default)]
    pub live_room_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub duration_sec: f64,
    #[serde(default)]
    pub progress_sec: f64,
    #[serde(default)]
    pub device: String,
    #[serde(default)]
    pub viewed_at: String,
    #[serde(default)]
    pub viewed_time: String,
    #[serde(default)]
    pub uploader_id: u64,
    #[serde(default)]
    pub uploader_name: String,
    #[serde(default)]
    pub uploader_avatar_url: String,
    #[serde(default)]
    pub category: String,
}

/// GET /users/me/view-history 的信封：`{items,total,paused}`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HistoryPage {
    #[serde(default)]
    pub items: Vec<ViewHistoryItem>,
    #[serde(default)]
    pub paused: bool,
}

/// 视频章节（GET /videos/:id/chapters，后端 Go 模型无 json tag → 大写键）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Chapter {
    #[serde(rename = "ID", default)]
    pub id: u64,
    #[serde(rename = "Title", default)]
    pub title: String,
    #[serde(rename = "TimeSec", default)]
    pub time_sec: f64,
}

/// 消息中心（GET /notifications?category=…）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Notification {
    #[serde(default)]
    pub id: u64,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub comment_preview: String,
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub total_likes: u64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub sender_names: Vec<String>,
    #[serde(default)]
    pub sender_username: String,
    #[serde(default)]
    pub sender_avatar_urls: Vec<String>,
    #[serde(default)]
    pub reply_content: String,
    #[serde(default)]
    pub video_title: String,
    #[serde(default)]
    pub video_id: u64,
    /// 回复/评论类通知对应的评论 id
    #[serde(default)]
    pub reply_comment_id: u64,
    /// 回复的父评论 id（多级回复时可用于定位）
    #[serde(default)]
    pub parent_comment_id: u64,
    /// 「收到的赞」对应的被点赞评论 id
    #[serde(default)]
    pub liked_comment_id: u64,
    #[serde(default)]
    pub comment_full_text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TokenPair {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LevelInfo {
    #[serde(default)]
    pub current_level: u32,
    #[serde(default)]
    pub current_exp: u64,
    #[serde(default)]
    pub next_exp: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Me {
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub cake_id: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub sign: String,
    #[serde(default)]
    pub coin_balance: i64,
    #[serde(default)]
    pub level_info: LevelInfo,
}

/// 首页轮播图（GET /home-banners）
///
/// 后端按 B 站口径给字段：`name` 是标题、`pic` 是图片、`url` 是点击跳转。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Banner {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub pic: String,
    #[serde(default)]
    pub url: String,
}

/// 首页统计（GET /stats/home）：在线人数 + 已发布视频数
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HomeStats {
    #[serde(default)]
    pub web_online: u64,
    #[serde(default)]
    pub all_count: u64,
}

/// 真实分区（GET /zones，按视频数倒序）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ZoneItem {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub video_count: u64,
}

/// 排行榜条目（GET /leaderboard 直接返回 model.Video → 大写键、时长是浮点秒）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RankVideo {
    #[serde(rename = "ID", default)]
    pub id: u64,
    #[serde(rename = "Title", default)]
    pub title: String,
    #[serde(rename = "CoverURL", default)]
    pub cover_url: String,
    #[serde(rename = "DurationSec", default)]
    pub duration: f64,
    #[serde(rename = "PlayCount", default)]
    pub play_count: u64,
    #[serde(rename = "DanmakuCount", default)]
    pub danmaku_count: u64,
    #[serde(rename = "CommentCount", default)]
    pub comment_count: u64,
    #[serde(rename = "LikeCount", default)]
    pub like_count: u64,
    #[serde(rename = "CoinCount", default)]
    pub coin_count: u64,
    #[serde(rename = "FavCount", default)]
    pub fav_count: u64,
    #[serde(rename = "CreatedAt", default)]
    pub created_at: String,
}

/// 他人主页资料（GET /space/:userId）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserProfile {
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub cake_id: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub sign: String,
    #[serde(default)]
    pub announcement: String,
    #[serde(default)]
    pub is_owner: bool,
    #[serde(default)]
    pub following_count: u64,
    #[serde(default)]
    pub follower_count: u64,
    #[serde(default)]
    pub published_count: u64,
    #[serde(default)]
    pub followed_by_me: bool,
    #[serde(default)]
    pub level_info: LevelInfo,
}

/// 主页投稿 / 搜索结果共用：videoCard 的 `duration` 是浮点秒
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SpaceVideo {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub danmaku_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub uploader: String,
    #[serde(default)]
    pub created_at: String,
}

impl SpaceVideo {
    /// 转成首页/推荐在用的卡片模型，好复用 widgets 里的封面卡
    pub fn as_video(&self) -> Video {
        Video {
            id: self.id,
            user_id: self.user_id,
            title: self.title.clone(),
            cover_url: self.cover_url.clone(),
            duration: self.duration.max(0.0) as u64,
            play_count: self.play_count,
            danmaku_count: self.danmaku_count,
            comment_count: self.comment_count,
            like_count: self.like_count,
        }
    }
}

/// 专栏卡片（GET /space/:userId/articles）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArticleCard {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub view_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub coin_count: u64,
    #[serde(default)]
    pub fav_count: u64,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub author_name: String,
}

/// 多清晰度（GET /videos/:id/bitrates）
///
/// 后端 `model.VideoBitrate` 没有 json tag，所以键是 Go 字段名大写。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct VideoBitrate {
    #[serde(rename = "ID", default)]
    pub id: u64,
    #[serde(rename = "Label", default)]
    pub label: String,
    #[serde(rename = "Width", default)]
    pub width: i64,
    #[serde(rename = "Height", default)]
    pub height: i64,
    #[serde(rename = "Kbps", default)]
    pub kbps: i64,
    #[serde(rename = "URL", default)]
    pub url: String,
}

/// 字幕轨道（GET /videos/:id/subtitles）
///
/// 列表接口会把 `content` 一起返回，客户端只在用户选了某条轨道时才用它。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Subtitle {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub video_id: u64,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub auto_gen: bool,
}

/// 未读角标（GET /notifications/unread-summary）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UnreadSummary {
    #[serde(default)]
    pub reply_received: i64,
    #[serde(default)]
    pub at_me: i64,
    #[serde(default)]
    pub like_aggregation: i64,
    #[serde(default)]
    pub system_notice: i64,
    /// 私信未读（按会话 unread_count 求和）
    #[serde(default)]
    pub my_message: i64,
}

impl UnreadSummary {
    pub fn total(&self) -> i64 {
        self.reply_received + self.at_me + self.like_aggregation + self.system_notice + self.my_message
    }

    /// 按消息中心 tab 的 category 取对应未读数
    pub fn of(&self, category: &str) -> i64 {
        match category {
            "reply_received" => self.reply_received,
            "at_me" => self.at_me,
            "like_aggregation" => self.like_aggregation,
            "system_notice" => self.system_notice,
            "my_message" => self.my_message,
            _ => 0,
        }
    }
}

/// 搜索联想词（GET /search/suggest → `{tag:[{name,value}]}`）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SuggestTag {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
}

impl SuggestTag {
    /// 展示与搜索都用同一条词，`name` 为空时退回 `value`
    pub fn term(&self) -> String {
        if self.name.trim().is_empty() {
            self.value.trim().to_owned()
        } else {
            self.name.trim().to_owned()
        }
    }
}

#[derive(Deserialize, Default)]
struct SuggestWrap {
    #[serde(default)]
    tag: Vec<SuggestTag>,
}

// ───────────────────────── 账号 / 设置 / 私信 / 内容 ─────────────────────────

/// 空间隐私开关（GET/PUT /users/me/space-privacy）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SpacePrivacy {
    #[serde(default)]
    pub public_favorites: bool,
    #[serde(default)]
    pub public_recent_coins: bool,
    #[serde(default)]
    pub public_following: bool,
    #[serde(default)]
    pub public_fans: bool,
    #[serde(default)]
    pub public_birthday: bool,
}

/// 关注分组（/users/me/follow-groups）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FollowGroup {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub member_count: u64,
}

/// 私信会话（/dm/conversations）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DmConversation {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub peer_id: u64,
    #[serde(default)]
    pub peer_name: String,
    #[serde(default)]
    pub peer_avatar: String,
    #[serde(default)]
    pub last_preview: String,
    #[serde(default)]
    pub last_message_at: String,
    #[serde(default)]
    pub unread_count: i64,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub is_agent: bool,
}

/// 私信消息（/dm/conversations/:id/messages）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DmMessage {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub sender_id: u64,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub created_at: String,
    /// 是不是我发的（后端带上就不用客户端再比 user_id）
    #[serde(default)]
    pub from_me: bool,
}

/// 专栏详情（/articles/:id）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArticleDetail {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub body_md: String,
    #[serde(default)]
    pub body_html: String,
    #[serde(default, deserialize_with = "null_vec")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub view_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub coin_count: u64,
    #[serde(default)]
    pub fav_count: u64,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_avatar: String,
    #[serde(default)]
    pub favorited_by_me: bool,
    #[serde(default)]
    pub coined_by_me: bool,
    #[serde(default)]
    pub is_author: bool,
    #[serde(default)]
    pub comments_closed: bool,
    #[serde(default)]
    pub comments_curated: bool,
}

/// 专栏评论（/articles/:id/comments）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArticleComment {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub liked_by_me: bool,
    #[serde(default)]
    pub parent_id: u64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub is_by_author: bool,
}

/// 硬币收支（/users/me/coin-ledger）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CoinLedgerEntry {
    #[serde(default)]
    pub id: u64,
    /// 正数=收入，负数=支出
    #[serde(default)]
    pub amount: i64,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub balance_after: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CoinLedgerPage {
    #[serde(default)]
    pub items: Vec<CoinLedgerEntry>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub balance: i64,
}

/// 每日任务的一项
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DailyTask {
    #[serde(default)]
    pub exp: i64,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub progress: i64,
    #[serde(default)]
    pub max: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DailyRewards {
    #[serde(default)]
    pub login: DailyTask,
    #[serde(default)]
    pub watch: DailyTask,
    #[serde(default)]
    pub coin: DailyTask,
    #[serde(default)]
    pub share: DailyTask,
}

/// 创作者数据（/users/me/creator/stats）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreatorStats {
    #[serde(default)]
    pub total_videos: i64,
    #[serde(default)]
    pub total_plays: i64,
    #[serde(default)]
    pub total_coins: i64,
    #[serde(default)]
    pub total_fans: i64,
    #[serde(default)]
    pub total_likes: i64,
    #[serde(default)]
    pub total_danmakus: i64,
    #[serde(default)]
    pub total_comments: i64,
}

/// 我的投稿/草稿（/users/me/videos）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MyVideo {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub danmaku_count: u64,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub coin_count: u64,
    #[serde(default)]
    pub fav_count: u64,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub fail_reason: String,
}

/// 举报（POST /reports 的返回体也用它回显）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReportResult {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub status: String,
}

/// 创作者视角的弹幕（/users/me/creator/danmakus）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreatorDanmaku {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub video_id: u64,
    #[serde(default)]
    pub video_title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub video_time: f64,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub created_at: String,
}

/// 专题页（/specials）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SpecialPage {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub video_count: u64,
    #[serde(default, rename = "Videos")]
    pub videos: Vec<Video>,
}

/// 申诉（/appeals/me）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Appeal {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub target_type: String,
    #[serde(default)]
    pub target_id: u64,
    #[serde(default)]
    pub reason_type: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub reply: String,
    #[serde(default)]
    pub created_at: String,
}

/// 工单（/users/me/tickets）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Ticket {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

/// 客服会话（/users/me/cs/conversations）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CsConversation {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub last_message: String,
    #[serde(default)]
    pub last_message_at: String,
    #[serde(default)]
    pub unread_count: i64,
}

#[derive(Deserialize)]
struct Envelope<T> {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    msg: String,
    data: Option<T>,
}

#[derive(Deserialize)]
struct Items<T> {
    #[serde(default)]
    items: Vec<T>,
}

#[derive(Deserialize)]
struct Rooms {
    #[serde(default)]
    rooms: Vec<LiveRoom>,
}

/// GET /api/v1/search 的信封：`{ result: { video: [...] } }`
#[derive(Deserialize, Default)]
struct SearchAll {
    #[serde(default)]
    result: SearchBuckets,
}

#[derive(Deserialize, Default)]
struct SearchBuckets {
    #[serde(default)]
    video: Vec<VideoHit>,
}

#[derive(Deserialize)]
struct HotWrap {
    #[serde(default)]
    items: Vec<HotItem>,
}

#[derive(Deserialize, Default)]
struct HotItem {
    #[serde(default)]
    title: String,
}

/// 鉴权失败码：未登录或 Token 已过期
const CODE_UNAUTHORIZED: i64 = 40100;
/// 刷新也失败时的统一提示
const EXPIRED_MSG: &str = "登录已过期，请重新登录";

#[derive(Debug, Clone, Default)]
struct AuthState {
    access: Option<String>,
    refresh: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    pub base: String,
    /// 登录态随 Client 克隆共享：后台线程拿着 clone 也能读到最新 token
    auth: Arc<Mutex<AuthState>>,
    /// 串行化刷新，避免并发 40100 把同一个 refresh_token 用两次
    refresh_lock: Arc<Mutex<()>>,
}

impl Client {
    pub fn from_env() -> Self {
        // 运行期环境变量优先；安卓上没有环境变量，用编译期烘焙进去的地址
        let base = std::env::var("CAKECAKE_API")
            .ok()
            .or_else(|| option_env!("CAKECAKE_API").map(str::to_owned))
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        Self {
            base: base.trim_end_matches('/').to_owned(),
            auth: Arc::new(Mutex::new(AuthState::default())),
            refresh_lock: Arc::new(Mutex::new(())),
        }
    }

    // ---------- 登录态 ----------

    pub fn set_tokens(&self, pair: &TokenPair) {
        if let Ok(mut auth) = self.auth.lock() {
            auth.access = Some(pair.access_token.clone());
            auth.refresh = Some(pair.refresh_token.clone());
        }
    }

    pub fn clear_tokens(&self) {
        if let Ok(mut auth) = self.auth.lock() {
            auth.access = None;
            auth.refresh = None;
        }
    }

    pub fn access_token(&self) -> Option<String> {
        self.auth.lock().ok().and_then(|a| a.access.clone())
    }

    pub fn logged_in(&self) -> bool {
        self.auth
            .lock()
            .map(|a| a.access.is_some())
            .unwrap_or(false)
    }

    /// 用 refresh_token 换一对新 token（后端会轮换 refresh_token）
    ///
    /// 只有刷新请求串行化，网络调用期间不持有登录态锁，
    /// 避免后台刷新卡住主线程读取 token。
    fn refresh(&self) -> Result<(), String> {
        let stale = self.access_token();
        let _guard = self
            .refresh_lock
            .lock()
            .map_err(|_| "登录状态异常".to_owned())?;
        // 等锁期间别的线程可能已经刷新成功，直接用新 token 重试即可
        if self.access_token() != stale {
            return if self.logged_in() {
                Ok(())
            } else {
                Err(EXPIRED_MSG.to_owned())
            };
        }
        let refresh_token = self
            .auth
            .lock()
            .map_err(|_| "登录状态异常".to_owned())?
            .refresh
            .clone();
        let Some(refresh_token) = refresh_token else {
            return Err("缺少 refresh_token".to_owned());
        };
        let text = self.raw(
            "POST",
            "/api/v1/auth/refresh",
            Some(&serde_json::json!({ "refresh_token": refresh_token })),
            None,
        )?;
        let env: Envelope<TokenPair> =
            serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))?;
        if env.code != 0 {
            return Err(if env.msg.is_empty() {
                "刷新登录态失败".to_owned()
            } else {
                env.msg
            });
        }
        let pair = env.data.ok_or_else(|| "刷新响应缺少 data".to_owned())?;
        if pair.access_token.is_empty() || pair.refresh_token.is_empty() {
            return Err("刷新响应缺少 token".to_owned());
        }
        let mut auth = self.auth.lock().map_err(|_| "登录状态异常".to_owned())?;
        auth.access = Some(pair.access_token);
        auth.refresh = Some(pair.refresh_token);
        Ok(())
    }

    // ---------- 底层请求 ----------

    fn raw(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
        token: Option<&str>,
    ) -> Result<String, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = match method {
            "POST" => ureq::post(&url),
            "PUT" => ureq::put(&url),
            "PATCH" => ureq::patch(&url),
            "DELETE" => ureq::delete(&url),
            _ => ureq::get(&url),
        }
        .timeout(Duration::from_secs(6));
        if let Some(t) = token {
            req = req.set("Authorization", &format!("Bearer {t}"));
        }
        let resp = match body {
            Some(b) => req.send_json(b),
            None => req.call(),
        };
        match resp {
            Ok(r) => r.into_string().map_err(|e| e.to_string()),
            Err(ureq::Error::Status(_, r)) => Ok(r.into_string().unwrap_or_default()),
            Err(e) => Err(format!("{e}")),
        }
    }

    /// 公开接口（不带 Authorization，也不做刷新）
    fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let text = self.raw("GET", path, None, None)?;
        let env: Envelope<T> =
            serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))?;
        env.data.ok_or_else(|| {
            if env.msg.is_empty() {
                "响应缺少 data".to_owned()
            } else {
                env.msg
            }
        })
    }

    /// 不需要登录态的写请求（登录/注册）
    fn send_anon<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T, String> {
        let text = self.raw(method, path, body.as_ref(), None)?;
        let env: Envelope<T> =
            serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))?;
        if env.code != 0 {
            return Err(if env.msg.is_empty() {
                "操作失败".to_owned()
            } else {
                env.msg
            });
        }
        env.data.ok_or_else(|| {
            if env.msg.is_empty() {
                "响应缺少 data".to_owned()
            } else {
                env.msg
            }
        })
    }

    /// 鉴权请求：自动带 token；遇到 40100 先刷新并重试一次
    fn send<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T, String> {
        let mut retried = false;
        loop {
            let token = self.access_token();
            let text = self.raw(method, path, body.as_ref(), token.as_deref())?;
            let env: Envelope<T> =
                serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))?;
            if !retried && env.code == CODE_UNAUTHORIZED {
                if self.refresh().is_ok() {
                    retried = true;
                    continue;
                }
                self.clear_tokens();
                return Err(EXPIRED_MSG.to_owned());
            }
            return env.data.ok_or_else(|| {
                if env.msg.is_empty() {
                    "响应缺少 data".to_owned()
                } else {
                    env.msg
                }
            });
        }
    }

    /// 只关心成没成功（后端这类接口的 data 是 null）；同样自动刷新
    fn send_action(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let mut retried = false;
        loop {
            let token = self.access_token();
            let text = self.raw(method, path, body.as_ref(), token.as_deref())?;
            let env: Envelope<serde_json::Value> =
                serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))?;
            if !retried && env.code == CODE_UNAUTHORIZED {
                if self.refresh().is_ok() {
                    retried = true;
                    continue;
                }
                self.clear_tokens();
                return Err(EXPIRED_MSG.to_owned());
            }
            if env.code != 0 {
                return Err(if env.msg.is_empty() {
                    "操作失败".to_owned()
                } else {
                    env.msg
                });
            }
            return Ok(());
        }
    }

    // ---------- 账号 ----------

    /// POST /api/v1/auth/login（成功后自动存下 access/refresh）
    pub fn login(&self, username: &str, password: &str) -> Result<TokenPair, String> {
        let pair: TokenPair = self.send_anon(
            "POST",
            "/api/v1/auth/login",
            Some(serde_json::json!({ "username": username, "password": password })),
        )?;
        self.set_tokens(&pair);
        Ok(pair)
    }

    /// POST /api/v1/users（注册；密码至少 8 位，后端校验）
    pub fn register(&self, username: &str, password: &str) -> Result<(), String> {
        let _: serde_json::Value = self.send_anon(
            "POST",
            "/api/v1/users",
            Some(serde_json::json!({ "username": username, "password": password })),
        )?;
        Ok(())
    }

    /// GET /api/v1/users/me（需要 Authorization）
    pub fn me(&self) -> Result<Me, String> {
        self.send("GET", "/api/v1/users/me", None)
    }

    // ---------- 首页运营位 ----------

    /// GET /api/v1/home-banners（公开轮播图，已按后端 sort_order 排好）
    pub fn banners(&self) -> Result<Vec<Banner>, String> {
        let page: Items<Banner> = self.get("/api/v1/home-banners")?;
        Ok(page.items)
    }

    /// GET /api/v1/stats/home
    pub fn home_stats(&self) -> Result<HomeStats, String> {
        self.get("/api/v1/stats/home")
    }

    /// GET /api/v1/zones（真实分区 + 视频数，按视频数倒序）
    pub fn zones(&self) -> Result<Vec<ZoneItem>, String> {
        let page: Items<ZoneItem> = self.get("/api/v1/zones")?;
        Ok(page.items)
    }

    /// GET /api/v1/leaderboard?by=&period=（by: play/coin/like/fav，period: all/week/month）
    pub fn leaderboard(&self, by: &str, period: &str) -> Result<Vec<RankVideo>, String> {
        let by = percent_encode(by.trim());
        let period = percent_encode(period.trim());
        self.get(&format!(
            "/api/v1/leaderboard?by={by}&period={period}&limit=50"
        ))
    }

    /// GET /api/v1/search/suggest?term=（登录后才会带上搜索历史里的词）
    pub fn search_suggest(&self, term: &str, limit: usize) -> Result<Vec<SuggestTag>, String> {
        let q = percent_encode(term.trim());
        let wrap: SuggestWrap = self.send(
            "GET",
            &format!("/api/v1/search/suggest?term={q}&limit={limit}"),
            None,
        )?;
        Ok(wrap.tag)
    }

    // ---------- 他人主页 ----------

    /// GET /api/v1/space/:userId
    pub fn user_profile(&self, user_id: u64) -> Result<UserProfile, String> {
        self.send("GET", &format!("/api/v1/space/{user_id}"), None)
    }

    /// GET /api/v1/space/:userId/videos
    pub fn user_videos(&self, user_id: u64) -> Result<Vec<SpaceVideo>, String> {
        let page: Items<SpaceVideo> = self.send(
            "GET",
            &format!("/api/v1/space/{user_id}/videos?limit=30"),
            None,
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/space/:userId/articles
    pub fn user_articles(&self, user_id: u64) -> Result<Vec<ArticleCard>, String> {
        let page: Items<ArticleCard> = self.send(
            "GET",
            &format!("/api/v1/space/{user_id}/articles?limit=30"),
            None,
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/space/:userId/dynamics
    pub fn user_dynamics(&self, user_id: u64) -> Result<Vec<DynItem>, String> {
        let page: Items<DynItem> = self.send(
            "GET",
            &format!("/api/v1/space/{user_id}/dynamics?limit=30"),
            None,
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/space/:userId/favorites（隐私关闭时后端返回空列表）
    pub fn user_favorites(&self, user_id: u64) -> Result<Vec<FavoriteItem>, String> {
        let page: Items<FavoriteItem> = self.send(
            "GET",
            &format!("/api/v1/space/{user_id}/favorites?limit=60"),
            None,
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/space/:userId/following
    pub fn user_following(&self, user_id: u64) -> Result<Vec<FollowUser>, String> {
        let page: Items<FollowUser> = self.send(
            "GET",
            &format!("/api/v1/space/{user_id}/following?limit=100"),
            None,
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/space/:userId/followers
    pub fn user_followers(&self, user_id: u64) -> Result<Vec<FollowUser>, String> {
        let page: Items<FollowUser> = self.send(
            "GET",
            &format!("/api/v1/space/{user_id}/followers?limit=100"),
            None,
        )?;
        Ok(page.items)
    }

    // ---------- 视频 ----------

    /// GET /api/v1/videos?limit=
    pub fn videos(&self, limit: usize) -> Result<Vec<Video>, String> {
        let page: Items<Video> = self.get(&format!("/api/v1/videos?limit={limit}"))?;
        Ok(page.items)
    }

    /// GET /api/v1/videos/:id
    pub fn video(&self, id: u64) -> Result<Video, String> {
        self.get(&format!("/api/v1/videos/{id}"))
    }

    /// GET /api/v1/videos/:id（完整详情；登录后才有 liked_by_me / in_watch_later 这类互动态）
    pub fn video_detail(&self, id: u64) -> Result<VideoDetail, String> {
        self.send("GET", &format!("/api/v1/videos/{id}"), None)
    }

    /// GET /api/v1/feed/recommendation（相关推荐，按 id 去重交给调用方）
    pub fn related(&self, limit: usize) -> Result<Vec<Video>, String> {
        self.recommend(limit)
    }

    /// GET /api/v1/feed/recommendation（首页「推荐」）
    pub fn recommend(&self, limit: usize) -> Result<Vec<Video>, String> {
        let page: Items<Video> =
            self.get(&format!("/api/v1/feed/recommendation?limit={limit}"))?;
        Ok(page.items)
    }

    /// GET /api/v1/zones/:zone/recommendation
    ///
    /// `zone` 可以是父分区（动画）或完整分区（生活-日常），后端两种都认。
    pub fn zone_feed(&self, zone: &str, limit: usize) -> Result<Vec<Video>, String> {
        let z = percent_encode(zone.trim());
        let page: Items<Video> =
            self.get(&format!("/api/v1/zones/{z}/recommendation?limit={limit}"))?;
        Ok(page.items)
    }

    /// POST /api/v1/videos/:id/like
    pub fn toggle_like(&self, id: u64) -> Result<(), String> {
        let _: serde_json::Value =
            self.send("POST", &format!("/api/v1/videos/{id}/like"), Some(serde_json::json!({})))?;
        Ok(())
    }

    /// POST /api/v1/videos/:id/coin（一次投 1 枚，后端按 authd 校验）
    pub fn post_coin(&self, id: u64) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/coin"),
            Some(serde_json::json!({ "amount": 1 })),
        )?;
        Ok(())
    }

    /// POST /api/v1/videos/:id/favorite
    pub fn toggle_favorite(&self, id: u64) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/favorite"),
            Some(serde_json::json!({})),
        )?;
        Ok(())
    }

    /// GET /api/v1/videos/:id/favorite-picker（收藏夹列表 + 已选中的夹）
    pub fn favorite_picker(&self, id: u64) -> Result<FavoritePicker, String> {
        self.send("GET", &format!("/api/v1/videos/{id}/favorite-picker"), None)
    }

    /// PUT /api/v1/videos/:id/favorite-folders（folder_ids 为空 = 取消收藏）
    pub fn set_favorite_folders(&self, id: u64, folder_ids: &[u64]) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "PUT",
            &format!("/api/v1/videos/{id}/favorite-folders"),
            Some(serde_json::json!({ "folder_ids": folder_ids })),
        )?;
        Ok(())
    }

    /// GET /api/v1/users/me/favorite-folders
    pub fn favorite_folders(&self) -> Result<Vec<FavoriteFolder>, String> {
        let page: Items<FavoriteFolder> =
            self.send("GET", "/api/v1/users/me/favorite-folders", None)?;
        Ok(page.items)
    }

    /// GET /api/v1/users/me/favorites（可选按收藏夹过滤）
    pub fn my_favorites(&self, folder_id: Option<u64>) -> Result<Vec<FavoriteItem>, String> {
        let path = match folder_id {
            Some(fid) => format!("/api/v1/users/me/favorites?folder_id={fid}"),
            None => "/api/v1/users/me/favorites".to_owned(),
        };
        let page: Items<FavoriteItem> = self.send("GET", &path, None)?;
        Ok(page.items)
    }

    /// POST /api/v1/users/me/favorite-folders
    pub fn create_favorite_folder(&self, title: &str) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            "/api/v1/users/me/favorite-folders",
            Some(serde_json::json!({ "title": title, "is_public": true })),
        )?;
        Ok(())
    }

    /// DELETE /api/v1/users/me/favorite-folders/:folderId（默认夹不能删）
    pub fn delete_favorite_folder(&self, folder_id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/favorite-folders/{folder_id}"),
            None,
        )
    }

    /// POST /api/v1/users/:id/follow
    pub fn toggle_follow(&self, user_id: u64) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/users/{user_id}/follow"),
            Some(serde_json::json!({})),
        )?;
        Ok(())
    }

    /// POST /api/v1/videos/:id/danmaku（content/color/type/font_size/video_time）
    pub fn post_danmaku(&self, id: u64, content: &str, video_time: f64) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/danmaku"),
            Some(serde_json::json!({
                "content": content,
                "color": "#FFFFFF",
                "type": "scroll",
                "font_size": "md",
                "video_time": video_time,
            })),
        )?;
        Ok(())
    }

    /// GET /api/v1/videos/:id/danmaku
    pub fn danmakus(&self, id: u64) -> Result<Vec<Danmaku>, String> {
        let page: Items<Danmaku> = self.get(&format!("/api/v1/videos/{id}/danmaku?limit=200"))?;
        Ok(page.items)
    }

    /// GET /api/v1/videos/:id/comments（评论树靠全量列表在本地拼，limit 给大一些）
    pub fn comments(&self, id: u64) -> Result<Vec<Comment>, String> {
        let page: Items<Comment> =
            self.send("GET", &format!("/api/v1/videos/{id}/comments?limit=200"), None)?;
        Ok(page.items)
    }

    /// POST /api/v1/videos/:id/comments（parent_id=0 为一级评论，否则回复到该层）
    pub fn post_comment(&self, id: u64, content: &str, parent_id: u64) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/comments"),
            Some(serde_json::json!({ "content": content, "parent_id": parent_id })),
        )?;
        Ok(())
    }

    // ---------- 直播 ----------

    /// GET /api/v1/live/rooms
    pub fn live_rooms(&self, limit: usize) -> Result<Vec<LiveRoom>, String> {
        let page: Rooms = self.get(&format!("/api/v1/live/rooms?page_size={limit}"))?;
        Ok(page.rooms)
    }

    /// GET /api/v1/live/room/:id
    pub fn live_room(&self, id: u64) -> Result<LiveRoom, String> {
        self.get(&format!("/api/v1/live/room/{id}"))
    }

    // ---------- 搜索 ----------

    /// GET /api/v1/search?keyword=…
    pub fn search(&self, keyword: &str) -> Result<Vec<VideoHit>, String> {
        let q = percent_encode(keyword.trim());
        let page: SearchAll = self.get(&format!("/api/v1/search?keyword={q}&page_size=20"))?;
        Ok(page.result.video)
    }

    /// GET /api/v1/hot-search?limit=
    pub fn hot_search(&self, limit: usize) -> Result<Vec<String>, String> {
        let page: HotWrap = self.get(&format!("/api/v1/hot-search?limit={limit}"))?;
        Ok(page
            .items
            .into_iter()
            .map(|i| i.title)
            .filter(|t| !t.is_empty())
            .collect())
    }

    // ---------- 关注 ----------

    /// GET /api/v1/users/me/followings（需要登录）
    pub fn followings(&self) -> Result<Vec<FollowUser>, String> {
        let page: Items<FollowUser> =
            self.send("GET", "/api/v1/users/me/followings?limit=30", None)?;
        Ok(page.items)
    }

    /// GET /api/v1/dynamics/following（关注页动态流，需要登录）
    pub fn following_dynamics(&self) -> Result<Vec<DynItem>, String> {
        let page: Items<DynItem> =
            self.send("GET", "/api/v1/dynamics/following?limit=30", None)?;
        Ok(page.items)
    }

    /// GET /api/v1/feed/subscription（关注的人的投稿，需要登录）
    pub fn subscription_feed(&self) -> Result<Vec<SubVideo>, String> {
        let page: Items<SubVideo> =
            self.send("GET", "/api/v1/feed/subscription?limit=30", None)?;
        Ok(page.items)
    }

    // ---------- 观看历史 ----------

    /// GET /api/v1/users/me/view-history
    pub fn view_history(&self) -> Result<HistoryPage, String> {
        self.send("GET", "/api/v1/users/me/view-history", None)
    }

    /// POST /api/v1/videos/:id/view-history（上报进度，用于断点续播）
    pub fn record_view_history(
        &self,
        id: u64,
        progress_sec: f64,
        duration_sec: f64,
    ) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/videos/{id}/view-history"),
            Some(serde_json::json!({
                "progress_sec": progress_sec,
                "duration_sec": duration_sec,
                "device": "mobile",
            })),
        )
    }

    /// DELETE /api/v1/users/me/view-history/:videoId
    pub fn delete_view_history_entry(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/view-history/{id}"),
            None,
        )
    }

    /// DELETE /api/v1/users/me/view-history/live/:liveRoomId
    pub fn delete_live_history_entry(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/view-history/live/{id}"),
            None,
        )
    }

    /// DELETE /api/v1/users/me/view-history/articles/:articleId
    pub fn delete_article_history_entry(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/view-history/articles/{id}"),
            None,
        )
    }

    /// DELETE /api/v1/users/me/view-history
    pub fn clear_view_history(&self) -> Result<(), String> {
        self.send_action("DELETE", "/api/v1/users/me/view-history", None)
    }

    /// PUT /api/v1/users/me/view-history/settings
    pub fn set_view_history_paused(&self, paused: bool) -> Result<(), String> {
        self.send(
            "PUT",
            "/api/v1/users/me/view-history/settings",
            Some(serde_json::json!({ "paused": paused })),
        )
        .map(|_: serde_json::Value| ())
    }

    /// GET /api/v1/videos/:id/chapters（无需登录）
    pub fn chapters(&self, id: u64) -> Result<Vec<Chapter>, String> {
        self.get(&format!("/api/v1/videos/{id}/chapters"))
    }

    // ---------- 多清晰度 / 字幕 ----------

    /// GET /api/v1/videos/:id/bitrates（按 kbps 升序，没转码过就是空列表）
    pub fn bitrates(&self, id: u64) -> Result<Vec<VideoBitrate>, String> {
        self.get(&format!("/api/v1/videos/{id}/bitrates"))
    }

    /// GET /api/v1/videos/:id/subtitles（列表里已含 content）
    pub fn subtitles(&self, id: u64) -> Result<Vec<Subtitle>, String> {
        self.get(&format!("/api/v1/videos/{id}/subtitles"))
    }

    /// GET /api/v1/videos/:id/subtitles/:subtitleId
    pub fn subtitle(&self, video_id: u64, subtitle_id: u64) -> Result<Subtitle, String> {
        self.get(&format!(
            "/api/v1/videos/{video_id}/subtitles/{subtitle_id}"
        ))
    }

    // ---------- 稍后再看 ----------

    /// GET /api/v1/users/me/watch-later（需要登录）
    pub fn watch_later(&self) -> Result<Vec<WatchLaterItem>, String> {
        let page: Items<WatchLaterItem> =
            self.send("GET", "/api/v1/users/me/watch-later?limit=100", None)?;
        Ok(page.items)
    }

    /// POST /api/v1/videos/:id/watch-later（切换，返回切换后的状态）
    pub fn toggle_watch_later(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/watch-later"),
            Some(serde_json::json!({})),
        )?;
        Ok(v
            .get("in_watch_later")
            .and_then(|x| x.as_bool())
            .unwrap_or(false))
    }

    /// POST /api/v1/users/me/watch-later/:id/watched
    pub fn mark_watch_later_watched(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/users/me/watch-later/{id}/watched"),
            None,
        )
    }

    /// DELETE /api/v1/users/me/watch-later（清空）
    pub fn clear_watch_later(&self) -> Result<(), String> {
        self.send_action("DELETE", "/api/v1/users/me/watch-later", None)
    }

    /// DELETE /api/v1/users/me/watch-later/watched（清除已看）
    pub fn clear_watched_watch_later(&self) -> Result<(), String> {
        self.send_action("DELETE", "/api/v1/users/me/watch-later/watched", None)
    }

    // ---------- 评论 ----------

    /// POST /api/v1/comments/:id/like（切换，返回 liked）
    pub fn toggle_comment_like(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/comments/{id}/like"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("liked").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// POST /api/v1/comments/:id/dislike（切换，返回 disliked）
    pub fn toggle_comment_dislike(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/comments/{id}/dislike"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("disliked").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    // ---------- 消息中心 ----------

    /// GET /api/v1/notifications?category=…（需要登录）
    pub fn notifications(&self, category: &str) -> Result<Vec<Notification>, String> {
        let cat = percent_encode(category);
        let page: Items<Notification> = self.send(
            "GET",
            &format!("/api/v1/notifications?category={cat}&limit=30"),
            None,
        )?;
        Ok(page.items)
    }

    /// PATCH /api/v1/notifications/read-by-category?category=…
    pub fn mark_notifications_read(&self, category: &str) -> Result<(), String> {
        let cat = percent_encode(category);
        self.send_action(
            "PATCH",
            &format!("/api/v1/notifications/read-by-category?category={cat}"),
            None,
        )
    }

    /// PATCH /api/v1/notifications/:id/read
    pub fn mark_notification_read(&self, id: u64) -> Result<(), String> {
        self.send_action("PATCH", &format!("/api/v1/notifications/{id}/read"), None)
    }

    /// DELETE /api/v1/notifications/:id
    pub fn delete_notification(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/notifications/{id}"), None)
    }

    /// GET /api/v1/notifications/unread-summary（各分类未读数，用于角标）
    pub fn unread_summary(&self) -> Result<UnreadSummary, String> {
        self.send("GET", "/api/v1/notifications/unread-summary", None)
    }

    // ---------- 账号与设置 ----------

    /// PUT /api/v1/users/me（改登录名）
    pub fn update_username(&self, username: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            "/api/v1/users/me",
            Some(serde_json::json!({ "username": username })),
        )
    }

    /// PUT /api/v1/users/me/profile（昵称 / 签名 / 性别 / 生日）
    pub fn update_profile(
        &self,
        nickname: &str,
        sign: &str,
        gender: &str,
        birthday: &str,
    ) -> Result<(), String> {
        self.send_action(
            "PUT",
            "/api/v1/users/me/profile",
            Some(serde_json::json!({
                "nickname": nickname,
                "sign": sign,
                "gender": gender,
                "birthday": birthday,
            })),
        )
    }

    /// PUT /api/v1/users/me/announcement（个人空间公告）
    pub fn update_announcement(&self, text: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            "/api/v1/users/me/announcement",
            Some(serde_json::json!({ "announcement": text })),
        )
    }

    /// PUT /api/v1/users/me/password
    pub fn update_password(&self, old: &str, new: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            "/api/v1/users/me/password",
            Some(serde_json::json!({ "old_password": old, "new_password": new })),
        )
    }

    /// GET /api/v1/users/me/space-privacy
    pub fn space_privacy(&self) -> Result<SpacePrivacy, String> {
        self.send("GET", "/api/v1/users/me/space-privacy", None)
    }

    /// PUT /api/v1/users/me/space-privacy
    pub fn set_space_privacy(&self, p: &SpacePrivacy) -> Result<SpacePrivacy, String> {
        self.send(
            "PUT",
            "/api/v1/users/me/space-privacy",
            Some(serde_json::json!({
                "public_favorites": p.public_favorites,
                "public_recent_coins": p.public_recent_coins,
                "public_following": p.public_following,
                "public_fans": p.public_fans,
                "public_birthday": p.public_birthday,
            })),
        )
    }

    /// POST /api/v1/users/me/deletion/request（需要密码确认）
    pub fn request_deletion(&self, password: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/users/me/deletion/request",
            Some(serde_json::json!({ "password": password })),
        )
    }

    /// POST /api/v1/users/me/deletion/revoke
    pub fn revoke_deletion(&self) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/users/me/deletion/revoke",
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/users/:userId/block（切换＝拉黑/取消拉黑，后端返回最新状态）
    pub fn block_user(&self, user_id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/users/{user_id}/block"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("blocked").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// GET /api/v1/users/me/follow-groups
    pub fn follow_groups(&self) -> Result<Vec<FollowGroup>, String> {
        let page: Items<FollowGroup> =
            self.send("GET", "/api/v1/users/me/follow-groups", None)?;
        Ok(page.items)
    }

    /// POST /api/v1/users/me/follow-groups
    pub fn create_follow_group(&self, name: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/users/me/follow-groups",
            Some(serde_json::json!({ "name": name })),
        )
    }

    /// PUT /api/v1/users/me/follow-groups/:groupId
    pub fn rename_follow_group(&self, group_id: u64, name: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            &format!("/api/v1/users/me/follow-groups/{group_id}"),
            Some(serde_json::json!({ "name": name })),
        )
    }

    /// DELETE /api/v1/users/me/follow-groups/:groupId
    pub fn delete_follow_group(&self, group_id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/follow-groups/{group_id}"),
            None,
        )
    }

    /// POST /api/v1/users/me/follow-groups/:groupId/members
    pub fn add_follow_group_member(&self, group_id: u64, followee_id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/users/me/follow-groups/{group_id}/members"),
            Some(serde_json::json!({ "followee_id": followee_id })),
        )
    }

    /// DELETE /api/v1/users/me/follow-groups/:groupId/members/:followeeId
    pub fn remove_follow_group_member(
        &self,
        group_id: u64,
        followee_id: u64,
    ) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/follow-groups/{group_id}/members/{followee_id}"),
            None,
        )
    }

    /// GET /api/v1/users/me/following/:followeeId/groups
    pub fn followee_groups(&self, followee_id: u64) -> Result<Vec<u64>, String> {
        let v: serde_json::Value = self.send(
            "GET",
            &format!("/api/v1/users/me/following/{followee_id}/groups"),
            None,
        )?;
        let mut out = Vec::new();
        if let Some(arr) = v.get("group_ids").and_then(|x| x.as_array()) {
            for it in arr {
                if let Some(n) = it.as_u64() {
                    out.push(n);
                }
            }
        }
        Ok(out)
    }

    // ---------- 搜索历史 / 硬币 / 每日任务 ----------

    /// GET /api/v1/users/me/search-history
    pub fn search_history(&self) -> Result<Vec<String>, String> {
        let v: serde_json::Value = self.send("GET", "/api/v1/users/me/search-history", None)?;
        let mut out = Vec::new();
        if let Some(arr) = v.get("keywords").and_then(|x| x.as_array()) {
            for it in arr {
                if let Some(s) = it.as_str() {
                    out.push(s.to_owned());
                }
            }
        }
        Ok(out)
    }

    /// PUT /api/v1/users/me/search-history（整体覆盖）
    pub fn put_search_history(&self, keywords: &[String]) -> Result<(), String> {
        self.send_action(
            "PUT",
            "/api/v1/users/me/search-history",
            Some(serde_json::json!({ "keywords": keywords })),
        )
    }

    /// POST /api/v1/users/me/search-history（记一条）
    pub fn post_search_history(&self, keyword: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/users/me/search-history",
            Some(serde_json::json!({ "keyword": keyword })),
        )
    }

    /// GET /api/v1/users/me/coin-ledger?range=week|month
    pub fn coin_ledger(&self, range: &str, limit: usize) -> Result<CoinLedgerPage, String> {
        self.send(
            "GET",
            &format!("/api/v1/users/me/coin-ledger?range={range}&limit={limit}"),
            None,
        )
    }

    /// GET /api/v1/users/me/daily-rewards
    pub fn daily_rewards(&self) -> Result<DailyRewards, String> {
        self.send("GET", "/api/v1/users/me/daily-rewards", None)
    }

    /// POST /api/v1/users/me/daily-rewards/watch
    pub fn post_daily_watch(&self) -> Result<DailyRewards, String> {
        self.send(
            "POST",
            "/api/v1/users/me/daily-rewards/watch",
            Some(serde_json::json!({})),
        )
    }

    // ---------- 消息中心深化 ----------

    /// PATCH /api/v1/notifications/read-batch
    pub fn read_notifications_batch(&self, ids: &[u64]) -> Result<(), String> {
        self.send_action(
            "PATCH",
            "/api/v1/notifications/read-batch",
            Some(serde_json::json!({ "ids": ids })),
        )
    }

    /// GET /api/v1/notifications/:id/like-likers（聚合点赞的通知里，谁赞的）
    pub fn notification_like_likers(&self, id: u64) -> Result<Vec<FollowUser>, String> {
        let page: Items<FollowUser> = self.send(
            "GET",
            &format!("/api/v1/notifications/{id}/like-likers"),
            None,
        )?;
        Ok(page.items)
    }

    /// POST /api/v1/notifications/:id/mute-likes（不再收这条的点赞通知）
    pub fn mute_like_notification(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/notifications/{id}/mute-likes"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/notifications/:id/comment-like（在通知里直接赞回去）
    pub fn notification_comment_like(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/notifications/{id}/comment-like"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("liked").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// POST /api/v1/notifications/:id/comment-reply（在通知里直接回一句）
    pub fn notification_comment_reply(&self, id: u64, content: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/notifications/{id}/comment-reply"),
            Some(serde_json::json!({ "content": content })),
        )
    }

    // ---------- 私信 ----------

    pub fn dm_conversations(&self) -> Result<Vec<DmConversation>, String> {
        let page: Items<DmConversation> = self.send("GET", "/api/v1/dm/conversations", None)?;
        Ok(page.items)
    }

    /// POST /api/v1/dm/conversations（找到或创建与 peer 的会话）
    pub fn create_dm(&self, peer_id: u64) -> Result<DmConversation, String> {
        self.send(
            "POST",
            "/api/v1/dm/conversations",
            Some(serde_json::json!({ "peer_id": peer_id })),
        )
    }

    pub fn delete_dm(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/dm/conversations/{id}"), None)
    }

    /// POST /api/v1/dm/conversations/:id/reset（清空会话历史）
    pub fn reset_dm(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/dm/conversations/{id}/reset"),
            Some(serde_json::json!({})),
        )
    }

    /// PATCH /api/v1/dm/conversations/:id/settings（置顶 / 免打扰）
    pub fn patch_dm(
        &self,
        id: u64,
        pinned: Option<bool>,
        muted: Option<bool>,
    ) -> Result<(), String> {
        let mut body = serde_json::Map::new();
        if let Some(v) = pinned {
            body.insert("pinned".into(), serde_json::json!(v));
        }
        if let Some(v) = muted {
            body.insert("muted".into(), serde_json::json!(v));
        }
        self.send_action(
            "PATCH",
            &format!("/api/v1/dm/conversations/{id}/settings"),
            Some(serde_json::Value::Object(body)),
        )
    }

    pub fn dm_messages(&self, id: u64) -> Result<Vec<DmMessage>, String> {
        let page: Items<DmMessage> = self.send(
            "GET",
            &format!("/api/v1/dm/conversations/{id}/messages?limit=50"),
            None,
        )?;
        Ok(page.items)
    }

    pub fn post_dm(&self, id: u64, content: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/dm/conversations/{id}/messages"),
            Some(serde_json::json!({ "content": content })),
        )
    }

    // ---------- 专栏 ----------

    /// GET /api/v1/articles
    pub fn articles(&self, limit: usize) -> Result<Vec<ArticleCard>, String> {
        let page: Items<ArticleCard> =
            self.get(&format!("/api/v1/articles?limit={limit}"))?;
        Ok(page.items)
    }

    /// GET /api/v1/articles/:id
    pub fn article(&self, id: u64) -> Result<ArticleDetail, String> {
        self.send("GET", &format!("/api/v1/articles/{id}"), None)
    }

    /// POST /api/v1/articles/:id/view（浏览上报，返回最新文章体）
    pub fn post_article_view(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/articles/{id}/view"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/articles/:id/favorite
    pub fn toggle_article_favorite(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/articles/{id}/favorite"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("favorited")
            .and_then(|x| x.as_bool())
            .unwrap_or(false))
    }

    /// POST /api/v1/articles/:id/coin
    pub fn post_article_coin(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/articles/{id}/coin"),
            Some(serde_json::json!({ "amount": 1 })),
        )
    }

    /// GET /api/v1/articles/:id/comments
    pub fn article_comments(&self, id: u64) -> Result<Vec<ArticleComment>, String> {
        let page: Items<ArticleComment> = self.send(
            "GET",
            &format!("/api/v1/articles/{id}/comments?limit=100"),
            None,
        )?;
        Ok(page.items)
    }

    /// POST /api/v1/articles/:id/comments
    pub fn post_article_comment(
        &self,
        id: u64,
        content: &str,
        parent_id: u64,
    ) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/articles/{id}/comments"),
            Some(serde_json::json!({ "content": content, "parent_id": parent_id })),
        )
    }

    /// GET /api/v1/users/me/articles（我写的，含草稿/审核中）
    pub fn my_articles(&self) -> Result<Vec<ArticleCard>, String> {
        let page: Items<ArticleCard> =
            self.send("GET", "/api/v1/users/me/articles?limit=50", None)?;
        Ok(page.items)
    }

    /// GET /api/v1/users/me/articles/:id（编辑器用，带正文）
    pub fn my_article(&self, id: u64) -> Result<ArticleDetail, String> {
        self.send("GET", &format!("/api/v1/users/me/articles/{id}"), None)
    }

    /// POST /api/v1/articles（新建，返回 id）
    pub fn post_article(&self, title: &str, body_md: &str) -> Result<u64, String> {
        let v: serde_json::Value = self.send(
            "POST",
            "/api/v1/articles",
            Some(serde_json::json!({ "title": title, "body_md": body_md })),
        )?;
        Ok(v.get("id").and_then(|x| x.as_u64()).unwrap_or(0))
    }

    /// PUT /api/v1/users/me/articles/:id
    pub fn put_article(&self, id: u64, title: &str, body_md: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            &format!("/api/v1/users/me/articles/{id}"),
            Some(serde_json::json!({ "title": title, "body_md": body_md })),
        )
    }

    /// DELETE /api/v1/users/me/articles/:id
    pub fn delete_article(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/users/me/articles/{id}"),
            None,
        )
    }

    /// PATCH /api/v1/users/me/articles/:id/playback（评论区开关）
    pub fn patch_article_playback(
        &self,
        id: u64,
        comments_closed: Option<bool>,
        comments_curated: Option<bool>,
    ) -> Result<(), String> {
        let mut body = serde_json::Map::new();
        if let Some(v) = comments_closed {
            body.insert("comments_closed".into(), serde_json::json!(v));
        }
        if let Some(v) = comments_curated {
            body.insert("comments_curated".into(), serde_json::json!(v));
        }
        self.send_action(
            "PATCH",
            &format!("/api/v1/users/me/articles/{id}/playback"),
            Some(serde_json::Value::Object(body)),
        )
    }

    // ---------- 动态 ----------

    /// GET /api/v1/user-dynamics/:id
    pub fn user_dynamic(&self, id: u64) -> Result<DynItem, String> {
        self.send("GET", &format!("/api/v1/user-dynamics/{id}"), None)
    }

    /// GET /api/v1/user-dynamics/:id/comments
    pub fn dynamic_comments(&self, id: u64) -> Result<Vec<Comment>, String> {
        let page: Items<Comment> = self.send(
            "GET",
            &format!("/api/v1/user-dynamics/{id}/comments?limit=100"),
            None,
        )?;
        Ok(page.items)
    }

    /// POST /api/v1/user-dynamics/:id/like
    pub fn toggle_dynamic_like(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/user-dynamics/{id}/like"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("liked").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// POST /api/v1/user-dynamics/:id/comments
    pub fn post_dynamic_comment(
        &self,
        id: u64,
        content: &str,
        parent_id: u64,
    ) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/user-dynamics/{id}/comments"),
            Some(serde_json::json!({ "content": content, "parent_id": parent_id })),
        )
    }

    /// GET /api/v1/users/me/dynamics
    pub fn my_dynamics(&self) -> Result<Vec<DynItem>, String> {
        let page: Items<DynItem> = self.send("GET", "/api/v1/users/me/dynamics?limit=50", None)?;
        Ok(page.items)
    }

    /// POST /api/v1/users/me/dynamics
    pub fn post_dynamic(&self, content: &str) -> Result<u64, String> {
        let v: serde_json::Value = self.send(
            "POST",
            "/api/v1/users/me/dynamics",
            Some(serde_json::json!({ "content": content, "type": "text" })),
        )?;
        Ok(v.get("id").and_then(|x| x.as_u64()).unwrap_or(0))
    }

    /// PUT /api/v1/users/me/dynamics/:id
    pub fn put_dynamic(&self, id: u64, content: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            &format!("/api/v1/users/me/dynamics/{id}"),
            Some(serde_json::json!({ "content": content })),
        )
    }

    /// DELETE /api/v1/users/me/dynamics/:id
    pub fn delete_dynamic(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/users/me/dynamics/{id}"), None)
    }

    /// PATCH /api/v1/users/me/dynamics/:id/playback
    pub fn patch_dynamic_playback(
        &self,
        id: u64,
        comments_closed: Option<bool>,
        comments_curated: Option<bool>,
    ) -> Result<(), String> {
        let mut body = serde_json::Map::new();
        if let Some(v) = comments_closed {
            body.insert("comments_closed".into(), serde_json::json!(v));
        }
        if let Some(v) = comments_curated {
            body.insert("comments_curated".into(), serde_json::json!(v));
        }
        self.send_action(
            "PATCH",
            &format!("/api/v1/users/me/dynamics/{id}/playback"),
            Some(serde_json::Value::Object(body)),
        )
    }

    /// 动态评论的赞 / 踩 / 精选 / 忽略 / 删除，action 直接拼到路径上
    pub fn dynamic_comment_action(&self, id: u64, action: &str) -> Result<(), String> {
        let method = if action == "delete" { "DELETE" } else { "POST" };
        let path = if action == "delete" {
            format!("/api/v1/dynamic-comments/{id}")
        } else {
            format!("/api/v1/dynamic-comments/{id}/{action}")
        };
        let body = if method == "POST" {
            Some(serde_json::json!({}))
        } else {
            None
        };
        self.send_action(method, &path, body)
    }

    // ---------- 创作中心 ----------

    /// GET /api/v1/users/me/videos（含草稿 / 审核中）
    pub fn my_videos(&self) -> Result<Vec<MyVideo>, String> {
        let page: Items<MyVideo> =
            self.send("GET", "/api/v1/users/me/videos?limit=100", None)?;
        Ok(page.items)
    }

    /// GET /api/v1/users/me/creator/stats
    pub fn creator_stats(&self) -> Result<CreatorStats, String> {
        self.send("GET", "/api/v1/users/me/creator/stats", None)
    }

    /// POST /api/v1/videos/:id/schedule（定时发布，publish_at="2006-01-02 15:04"）
    pub fn schedule_publish(&self, id: u64, publish_at: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/videos/{id}/schedule"),
            Some(serde_json::json!({ "publish_at": publish_at })),
        )
    }

    /// DELETE /api/v1/videos/:id/schedule
    pub fn cancel_schedule(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/videos/{id}/schedule"), None)
    }

    /// PUT /api/v1/videos/:id（改稿：标题 / 简介 / 标签）
    pub fn update_my_video(
        &self,
        id: u64,
        title: &str,
        description: &str,
        tags: &[String],
    ) -> Result<(), String> {
        self.send_action(
            "PUT",
            &format!("/api/v1/videos/{id}"),
            Some(serde_json::json!({
                "title": title,
                "description": description,
                "tags": tags,
            })),
        )
    }

    /// DELETE /api/v1/videos/:id
    pub fn delete_my_video(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/videos/{id}"), None)
    }

    /// POST /api/v1/videos/:id/publish（草稿发布）
    pub fn publish_draft(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/videos/{id}/publish"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/videos/:id/draft（保存草稿）
    pub fn save_draft(&self, title: &str, description: &str) -> Result<u64, String> {
        let v: serde_json::Value = self.send(
            "POST",
            "/api/v1/videos/draft",
            Some(serde_json::json!({ "title": title, "description": description })),
        )?;
        Ok(v.get("id").and_then(|x| x.as_u64()).unwrap_or(0))
    }

    /// POST /api/v1/videos/:id/chapters（创作者加章节）
    pub fn creator_add_chapter(&self, id: u64, title: &str, time_sec: f64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/videos/{id}/chapters"),
            Some(serde_json::json!({ "title": title, "time_sec": time_sec })),
        )
    }

    /// DELETE /api/v1/videos/:id/chapters/:chapterId
    pub fn creator_delete_chapter(&self, id: u64, chapter_id: u64) -> Result<(), String> {
        self.send_action(
            "DELETE",
            &format!("/api/v1/videos/{id}/chapters/{chapter_id}"),
            None,
        )
    }

    /// GET /api/v1/users/me/creator/comments（自己视频下的评论打理）
    pub fn creator_comments(&self) -> Result<Vec<Comment>, String> {
        let page: Items<Comment> = self.send(
            "GET",
            "/api/v1/users/me/creator/comments?limit=100",
            None,
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/users/me/creator/danmakus（自己视频收到的弹幕）
    pub fn creator_danmakus(&self) -> Result<Vec<CreatorDanmaku>, String> {
        let page: Items<CreatorDanmaku> = self.send(
            "GET",
            "/api/v1/users/me/creator/danmakus?limit=100",
            None,
        )?;
        Ok(page.items)
    }

    // ---------- 评论：权限相关操作 ----------

    /// DELETE /api/v1/comments/:id（本人或 UP 主）
    pub fn delete_comment(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/comments/{id}"), None)
    }

    /// POST /api/v1/comments/:id/pin（仅 UP 主）
    pub fn pin_comment(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/comments/{id}/pin"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/comments/:id/approve（精选通过，仅 UP 主）
    pub fn approve_comment(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/comments/{id}/approve"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/comments/:id/ignore-curated（精选忽略，仅 UP 主）
    pub fn ignore_curated_comment(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/comments/{id}/ignore-curated"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/comments/:id/report
    pub fn report_comment(&self, id: u64, reason: &str, detail: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/comments/{id}/report"),
            Some(serde_json::json!({ "reason": reason, "category": detail })),
        )
    }

    /// GET /api/v1/comments/:id/images
    pub fn comment_images(&self, id: u64) -> Result<Vec<String>, String> {
        let v: serde_json::Value = self.send(
            "GET",
            &format!("/api/v1/comments/{id}/images"),
            None,
        )?;
        let mut out = Vec::new();
        if let Some(arr) = v.get("items").and_then(|x| x.as_array()) {
            for it in arr {
                let url = it
                    .get("image_url")
                    .or_else(|| it.get("url"))
                    .and_then(|x| x.as_str())
                    .unwrap_or_default();
                if !url.is_empty() {
                    out.push(url.to_owned());
                }
            }
        }
        Ok(out)
    }

    // ---------- 弹幕 ----------

    /// DELETE /api/v1/danmakus/:id（发送者或 UP 主）
    pub fn delete_danmaku(&self, id: u64) -> Result<(), String> {
        self.send_action("DELETE", &format!("/api/v1/danmakus/{id}"), None)
    }

    /// POST /api/v1/danmakus/:id/like
    pub fn like_danmaku(&self, id: u64) -> Result<bool, String> {
        let v: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/danmakus/{id}/like"),
            Some(serde_json::json!({})),
        )?;
        Ok(v.get("liked").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    // ---------- 站点 / 专题 / 直播 ----------

    /// GET /api/v1/online
    pub fn online(&self) -> Result<HomeStats, String> {
        self.get("/api/v1/online")
    }

    /// GET /api/v1/specials
    pub fn specials(&self) -> Result<Vec<SpecialPage>, String> {
        let v: serde_json::Value = self.get("/api/v1/specials")?;
        let mut out = Vec::new();
        if let Some(arr) = v.as_array() {
            for it in arr {
                if let Ok(p) = serde_json::from_value::<SpecialPage>(it.clone()) {
                    out.push(p);
                }
            }
        } else if let Some(arr) = v.get("items").and_then(|x| x.as_array()) {
            for it in arr {
                if let Ok(p) = serde_json::from_value::<SpecialPage>(it.clone()) {
                    out.push(p);
                }
            }
        }
        Ok(out)
    }

    /// GET /api/v1/specials/:slug
    pub fn special(&self, slug: &str) -> Result<SpecialPage, String> {
        self.get(&format!("/api/v1/specials/{}", percent_encode(slug)))
    }

    /// GET /api/v1/live/featured
    pub fn live_featured(&self) -> Result<Vec<LiveRoom>, String> {
        let v: serde_json::Value = self.get("/api/v1/live/featured")?;
        let mut out = Vec::new();
        let arr = v
            .get("items")
            .and_then(|x| x.as_array())
            .or_else(|| v.get("rooms").and_then(|x| x.as_array()));
        if let Some(arr) = arr {
            for it in arr {
                if let Ok(r) = serde_json::from_value::<LiveRoom>(it.clone()) {
                    out.push(r);
                }
            }
        }
        Ok(out)
    }

    /// GET /api/v1/live/room/my（我的直播间，没有就建一个）
    pub fn my_live_room(&self) -> Result<LiveRoom, String> {
        self.send("GET", "/api/v1/live/room/my", None)
    }

    /// POST /api/v1/live/room/create
    pub fn create_live_room(&self, title: &str, cover: &str) -> Result<LiveRoom, String> {
        self.send(
            "POST",
            "/api/v1/live/room/create",
            Some(serde_json::json!({ "title": title, "cover_url": cover })),
        )
    }

    /// PUT /api/v1/live/room/:id
    pub fn update_live_room(&self, id: u64, title: &str, cover: &str) -> Result<(), String> {
        self.send_action(
            "PUT",
            &format!("/api/v1/live/room/{id}"),
            Some(serde_json::json!({ "title": title, "cover_url": cover })),
        )
    }

    /// POST /api/v1/live/room/:id/regenerate-key
    pub fn regenerate_stream_key(&self, id: u64) -> Result<LiveRoom, String> {
        self.send(
            "POST",
            &format!("/api/v1/live/room/{id}/regenerate-key"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/live/room/:id/start（开播）
    pub fn start_live(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/live/room/{id}/start"),
            Some(serde_json::json!({})),
        )
    }

    /// POST /api/v1/live/room/:id/end（下播）
    pub fn end_live(&self, id: u64) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/live/room/{id}/end"),
            Some(serde_json::json!({})),
        )
    }

    // ---------- 治理 / 客服 ----------

    /// POST /api/v1/reports（举报用户/视频/评论/动态）
    pub fn post_report(
        &self,
        target_type: &str,
        target_id: u64,
        reason_type: &str,
        detail: &str,
    ) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/reports",
            Some(serde_json::json!({
                "target_type": target_type,
                "target_id": target_id,
                "reason_type": reason_type,
                "reason_detail": detail,
            })),
        )
    }

    /// POST /api/v1/appeals（对处罚/下架申诉）
    pub fn post_appeal(
        &self,
        target_type: &str,
        target_id: u64,
        reason_type: &str,
        content: &str,
    ) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/appeals",
            Some(serde_json::json!({
                "target_type": target_type,
                "target_id": target_id,
                "reason_type": reason_type,
                "content": content,
            })),
        )
    }

    /// GET /api/v1/appeals/me
    pub fn my_appeals(&self) -> Result<Vec<Appeal>, String> {
        let page: Items<Appeal> = self.send("GET", "/api/v1/appeals/me", None)?;
        Ok(page.items)
    }

    /// POST /api/v1/tickets
    pub fn post_ticket(&self, category: &str, subject: &str, desc: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/tickets",
            Some(serde_json::json!({
                "category": category,
                "subject": subject,
                "description": desc,
            })),
        )
    }

    /// GET /api/v1/users/me/tickets
    pub fn my_tickets(&self) -> Result<Vec<Ticket>, String> {
        let page: Items<Ticket> = self.send("GET", "/api/v1/users/me/tickets", None)?;
        Ok(page.items)
    }

    /// POST /api/v1/cs/conversations（发起客服会话）
    pub fn post_cs_conversation(&self, message: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            "/api/v1/cs/conversations",
            Some(serde_json::json!({ "message": message })),
        )
    }

    /// GET /api/v1/users/me/cs/conversations
    pub fn my_cs_conversations(&self) -> Result<Vec<CsConversation>, String> {
        let page: Items<CsConversation> = self.send(
            "GET",
            "/api/v1/users/me/cs/conversations",
            None,
        )?;
        Ok(page.items)
    }

    /// POST /api/v1/users/me/cs/conversations/:id/messages
    pub fn send_cs_message(&self, id: u64, message: &str) -> Result<(), String> {
        self.send_action(
            "POST",
            &format!("/api/v1/users/me/cs/conversations/{id}/messages"),
            Some(serde_json::json!({ "message": message })),
        )
    }


    /// 探测媒体地址是否真的能播：取 1 字节，2xx/206 视为可用。
    /// 后端对缺失文件返回 404，这样就能提前给出「源文件不存在」而不是播放器异常。
    pub fn probe_media(&self, url: &str) -> bool {
        let u = self.abs(url);
        if u.is_empty() {
            return false;
        }
        match ureq::get(&u)
            .set("Range", "bytes=0-0")
            .timeout(Duration::from_secs(4))
            .call()
        {
            Ok(r) => matches!(r.status(), 200 | 206),
            Err(ureq::Error::Status(code, _)) => code == 416,
            Err(_) => false,
        }
    }

    /// 探测某台后端是否在线（换网段/断映射时用来自动找可用地址）
    pub fn probe_base(&self, base: &str) -> bool {
        let url = format!("{}/api/v1/health", base.trim_end_matches('/'));
        matches!(
            ureq::get(&url).timeout(Duration::from_millis(1200)).call(),
            Ok(resp) if resp.status() == 200
        )
    }

    // ---------- 投稿上传 ----------

    /// POST /api/v1/videos（multipart：file + title + description + zone）
    ///
    /// 文件按流式上传，不整个读进内存；超过 512MB 直接拒绝，避免手机上 OOM。
    pub fn upload_video(
        &self,
        path: &str,
        title: &str,
        description: &str,
        zone: &str,
    ) -> Result<u64, String> {
        let meta = std::fs::metadata(path).map_err(|e| format!("读取文件失败：{e}"))?;
        if meta.len() == 0 {
            return Err("文件是空的".to_owned());
        }
        if meta.len() > 512 * 1024 * 1024 {
            return Err("文件超过 512MB".to_owned());
        }
        let file = std::fs::File::open(path).map_err(|e| format!("打开文件失败：{e}"))?;
        let name = std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "upload.mp4".to_owned());
        let ext = std::path::Path::new(path)
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "mp4".to_owned());
        let mime = match ext.as_str() {
            "mov" => "video/quicktime",
            "mkv" => "video/x-matroska",
            "webm" => "video/webm",
            _ => "video/mp4",
        };
        let b = "----cakecakeboundary7d1f3a";
        let mut head = Vec::new();
        head.extend_from_slice(
            format!(
                "--{b}
Content-Disposition: form-data; name=\"file\"; filename=\"{name}\"
Content-Type: {mime}

"
            )
            .as_bytes(),
        );
        let mut tail = Vec::new();
        for (k, v) in [
            ("title", title),
            ("description", description),
            ("zone", zone),
        ] {
            if v.is_empty() {
                continue;
            }
            tail.extend_from_slice(
                format!("
--{b}
Content-Disposition: form-data; name=\"{k}\"

{v}")
                    .as_bytes(),
            );
        }
        tail.extend_from_slice(format!("
--{b}--
").as_bytes());

        let mut retried = false;
        loop {
            let file = std::fs::File::open(path).map_err(|e| format!("打开文件失败：{e}"))?;
            let reader = UploadReader {
                head: std::io::Cursor::new(head.clone()),
                file,
                tail: std::io::Cursor::new(tail.clone()),
            };
            let token = self.access_token();
            let mut req = ureq::post(&format!("{}/api/v1/videos", self.base))
                .timeout(Duration::from_secs(300))
                .set(
                    "Content-Type",
                    &format!("multipart/form-data; boundary={b}"),
                );
            if let Some(t) = token.as_deref() {
                req = req.set("Authorization", &format!("Bearer {t}"));
            }
            let resp = match req.send(reader) {
                Ok(r) => r,
                Err(ureq::Error::Status(_, r)) => r,
                Err(e) => return Err(format!("上传失败：{e}")),
            };
            let text = resp.into_string().map_err(|e| e.to_string())?;
            let env: Envelope<serde_json::Value> = serde_json::from_str(&text)
                .map_err(|e| format!("JSON 解析失败：{e}"))?;
            if !retried && env.code == CODE_UNAUTHORIZED {
                if self.refresh().is_ok() {
                    retried = true;
                    continue;
                }
                self.clear_tokens();
                return Err(EXPIRED_MSG.to_owned());
            }
            if env.code != 0 {
                return Err(if env.msg.is_empty() {
                    "上传失败".to_owned()
                } else {
                    env.msg
                });
            }
            return Ok(env
                .data
                .and_then(|d| d.get("id").and_then(|x| x.as_u64()))
                .unwrap_or(0));
        }
    }

    /// 相对路径补成绝对地址（视频文件 / 头像这类后端直出路径）
    pub fn abs(&self, path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_owned();
        }
        format!("{}{}", self.base.trim_end_matches('/'), path)
    }

    /// 直播播放地址：SRS 的 HLS 输出。
    ///
    /// API 在 8080，媒体（HTTP-FLV/HLS）在 8000，两者同一个主机；
    /// 真机开发用 `adb reverse tcp:8000 tcp:8000` 即可直连。
    pub fn hls_url(&self, stream_key: &str) -> String {
        let host = self.base.split("://").nth(1).unwrap_or("127.0.0.1:8080");
        let hostname = host.split(':').next().unwrap_or("127.0.0.1");
        let port = std::env::var("CAKECAKE_MEDIA_PORT")
            .ok()
            .or_else(|| option_env!("CAKECAKE_MEDIA_PORT").map(str::to_owned))
            .unwrap_or_else(|| "8000".to_owned());
        format!("http://{hostname}:{port}/live/{stream_key}.m3u8")
    }
}


/// 上传用的流式 reader：head（multipart 头）+ 文件 + tail（其余字段与结束边界）
struct UploadReader {
    head: std::io::Cursor<Vec<u8>>,
    file: std::fs::File,
    tail: std::io::Cursor<Vec<u8>>,
}

impl std::io::Read for UploadReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.head.read(buf)?;
        if n > 0 {
            return Ok(n);
        }
        let n = self.file.read(buf)?;
        if n > 0 {
            return Ok(n);
        }
        self.tail.read(buf)
    }
}


/// 后端对「没有标签」返回 null，Vec 解析会直接报错，这里统一把 null 当空数组
fn null_vec<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(d)?.unwrap_or_default())
}

/// 查询参数转义：中文关键词必须按 UTF-8 字节百分号编码，否则服务端收到乱码
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
