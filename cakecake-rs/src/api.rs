//! 后端数据层 —— 只封装真实存在的接口，不造数据
//!
//! 信封 `{code,msg,data}`；列表接口用 `{items,...}` 或 `{rooms,total}`。
//! 后端不可达时返回 Err，由 UI 显示空态，绝不回落到假数据。

use serde::Deserialize;
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
    pub tags: Vec<String>,
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
    pub username: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub user_level: u64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub is_by_uploader: bool,
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

#[derive(Deserialize)]
struct Envelope<T> {
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

#[derive(Clone)]
pub struct Client {
    pub base: String,
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
        }
    }

    fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base, path);
        let body = ureq::get(&url)
            .timeout(Duration::from_secs(6))
            .call()
            .map_err(|e| format!("{e}"))?
            .into_string()
            .map_err(|e| format!("读取响应失败: {e}"))?;
        let env: Envelope<T> =
            serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败: {e}"))?;
        env.data.ok_or_else(|| {
            if env.msg.is_empty() {
                "响应缺少 data".to_owned()
            } else {
                env.msg
            }
        })
    }

    fn send<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
    ) -> Result<T, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = if method == "POST" {
            ureq::post(&url)
        } else {
            ureq::get(&url)
        }
        .timeout(Duration::from_secs(6));
        if let Some(t) = token {
            req = req.set("Authorization", &format!("Bearer {t}"));
        }
        let resp = match body {
            Some(b) => req.send_json(b),
            None => req.call(),
        };
        let text = match resp {
            Ok(r) => r.into_string().map_err(|e| e.to_string())?,
            // 4xx 也带信封，读出来才能显示后端给的原因
            Err(ureq::Error::Status(_, r)) => r.into_string().unwrap_or_default(),
            Err(e) => return Err(format!("{e}")),
        };
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

    /// POST /api/v1/auth/login
    pub fn login(&self, username: &str, password: &str) -> Result<TokenPair, String> {
        self.send(
            "POST",
            "/api/v1/auth/login",
            Some(serde_json::json!({ "username": username, "password": password })),
            None,
        )
    }

    /// GET /api/v1/users/me（需要 Authorization）
    pub fn me(&self, token: &str) -> Result<Me, String> {
        self.send("GET", "/api/v1/users/me", None, Some(token))
    }

    /// GET /api/v1/videos?limit=
    pub fn videos(&self, limit: usize) -> Result<Vec<Video>, String> {
        let page: Items<Video> = self.get(&format!("/api/v1/videos?limit={limit}"))?;
        Ok(page.items)
    }

    /// GET /api/v1/videos/:id
    pub fn video(&self, id: u64) -> Result<Video, String> {
        self.get(&format!("/api/v1/videos/{id}"))
    }

    /// GET /api/v1/videos/:id（完整详情）
    pub fn video_detail(&self, id: u64) -> Result<VideoDetail, String> {
        self.get(&format!("/api/v1/videos/{id}"))
    }

    /// GET /api/v1/feed/recommendation（相关推荐，按 id 去重交给调用方）
    pub fn related(&self, limit: usize) -> Result<Vec<Video>, String> {
        self.recommend(limit)
    }

    /// GET /api/v1/feed/recommendation（首页「推荐」）
    pub fn recommend(&self, limit: usize) -> Result<Vec<Video>, String> {
        let page: Items<Video> = self.get(&format!("/api/v1/feed/recommendation?limit={limit}"))?;
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
    pub fn toggle_like(&self, id: u64, token: &str) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/like"),
            Some(serde_json::json!({})),
            Some(token),
        )?;
        Ok(())
    }

    /// POST /api/v1/videos/:id/coin（一次投 1 枚，后端按 authd 校验）
    pub fn post_coin(&self, id: u64, token: &str) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/coin"),
            Some(serde_json::json!({ "amount": 1 })),
            Some(token),
        )?;
        Ok(())
    }

    /// POST /api/v1/videos/:id/favorite
    pub fn toggle_favorite(&self, id: u64, token: &str) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/videos/{id}/favorite"),
            Some(serde_json::json!({})),
            Some(token),
        )?;
        Ok(())
    }

    /// POST /api/v1/users/:id/follow
    pub fn toggle_follow(&self, user_id: u64, token: &str) -> Result<(), String> {
        let _: serde_json::Value = self.send(
            "POST",
            &format!("/api/v1/users/{user_id}/follow"),
            Some(serde_json::json!({})),
            Some(token),
        )?;
        Ok(())
    }

    /// POST /api/v1/videos/:id/danmaku（content/color/type/font_size/video_time）
    pub fn post_danmaku(
        &self,
        id: u64,
        content: &str,
        video_time: f64,
        token: &str,
    ) -> Result<(), String> {
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
            Some(token),
        )?;
        Ok(())
    }

    /// GET /api/v1/videos/:id/danmaku
    pub fn danmakus(&self, id: u64) -> Result<Vec<Danmaku>, String> {
        let page: Items<Danmaku> = self.get(&format!("/api/v1/videos/{id}/danmaku?limit=200"))?;
        Ok(page.items)
    }

    /// GET /api/v1/videos/:id/comments
    pub fn comments(&self, id: u64) -> Result<Vec<Comment>, String> {
        let page: Items<Comment> = self.get(&format!("/api/v1/videos/{id}/comments?limit=20"))?;
        Ok(page.items)
    }

    /// GET /api/v1/live/rooms
    pub fn live_rooms(&self, limit: usize) -> Result<Vec<LiveRoom>, String> {
        let page: Rooms = self.get(&format!("/api/v1/live/rooms?page_size={limit}"))?;
        Ok(page.rooms)
    }

    /// GET /api/v1/live/room/:id
    pub fn live_room(&self, id: u64) -> Result<LiveRoom, String> {
        self.get(&format!("/api/v1/live/room/{id}"))
    }

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

    /// GET /api/v1/users/me/followings（需要登录）
    pub fn followings(&self, token: &str) -> Result<Vec<FollowUser>, String> {
        let page: Items<FollowUser> =
            self.send("GET", "/api/v1/users/me/followings?limit=30", None, Some(token))?;
        Ok(page.items)
    }

    /// GET /api/v1/dynamics/following（关注页动态流，需要登录）
    pub fn following_dynamics(&self, token: &str) -> Result<Vec<DynItem>, String> {
        let page: Items<DynItem> = self.send(
            "GET",
            "/api/v1/dynamics/following?limit=30",
            None,
            Some(token),
        )?;
        Ok(page.items)
    }

    /// GET /api/v1/feed/subscription（关注的人的投稿，需要登录）
    pub fn subscription_feed(&self, token: &str) -> Result<Vec<SubVideo>, String> {
        let page: Items<SubVideo> = self.send(
            "GET",
            "/api/v1/feed/subscription?limit=30",
            None,
            Some(token),
        )?;
        Ok(page.items)
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
