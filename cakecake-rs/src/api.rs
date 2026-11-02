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
