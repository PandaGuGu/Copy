//! 应用外壳：路由、底部导航、异步加载
//!
//! 原则：页面上的每个数字都来自接口；接口没有的东西就不显示。
//! 加载失败只显示错误与重试，不用假数据兜底。

use crate::{
    api::{self, Comment, Danmaku, LiveRoom, Me, Video, VideoHit},
    images::Covers,
    pages, theme, video::Player, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Stroke, Vec2};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Home,
    Dynamic,
    Live,
    Mine,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    Tab(Tab),
    Detail(u64),
    LiveRoom(u64),
    Search,
    /// 全部分区（首页导航栏最右「☰」进）
    Zones,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LoadState {
    #[default]
    Idle,
    Loading,
    Ready,
    Err(String),
}

pub struct DetailData {
    pub video: api::VideoDetail,
    pub danmakus: Vec<Danmaku>,
    pub comments: Vec<Comment>,
    /// 相关推荐（来自 /feed/recommendation，已剔除当前视频）
    pub related: Vec<Video>,
}

/// 首页导航栏当前选中的分类
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HomeCat {
    /// 推荐（/feed/recommendation）
    Recommend,
    /// 热门（/videos?sort=hot）
    Hot,
    /// 某个分区，值为 `/zones/:zone/recommendation` 的 zone（父分区或「父-子」）
    Zone(String),
}

/// 详情页下面的两个 tab
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DetailTab {
    Intro,
    Comments,
}

/// 详情页可执行的操作（都走后端 authd 接口）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Like,
    Coin,
    Favorite,
    Follow,
}

/// 关注页数据：我关注的人 + 文字动态 + 关注的人的投稿
#[derive(Default)]
pub struct DynamicData {
    pub followings: Vec<api::FollowUser>,
    pub dynamics: Vec<api::DynItem>,
    pub videos: Vec<api::SubVideo>,
}

/// 直播间聊天的一条消息
#[derive(Clone)]
pub struct ChatMsg {
    pub user: String,
    pub content: String,
    pub system: bool,
}

/// 直播间聊天状态
#[derive(Default)]
pub struct ChatState {
    pub msgs: Vec<ChatMsg>,
    pub online: u64,
    pub state: LoadState,
    pub input: String,
    conn: Option<crate::livechat::Conn>,
    room_id: u64,
}

enum Msg {
    Videos(Result<Vec<Video>, String>),
    Live(Result<Vec<LiveRoom>, String>),
    Detail(u64, Result<DetailData, String>),
    LiveRoom(u64, Result<LiveRoom, String>),
    Login(Result<(String, Me), String>),
    Search(Result<Vec<VideoHit>, String>),
    Hot(Result<Vec<String>, String>),
    /// 点赞/投币/收藏/关注的回执（成功后静默刷新详情）
    Action(u64, Result<(), String>),
    /// 关注页（动态/投稿/关注列表）
    Dynamic(Result<DynamicData, String>),
}

pub struct App {
    pub client: api::Client,
    pub covers: Covers,

    pub route: Route,
    /// 返回栈：安卓返回键/页面左上角返回都走这里，回到进入本页之前的页面
    history: Vec<Route>,
    pub videos: Vec<Video>,
    pub videos_state: LoadState,
    /// 首页导航栏选中的分类（推荐 / 热门 / 某分区）
    pub home_cat: HomeCat,
    /// 关注页（动态 tab）
    pub dynamic: DynamicData,
    pub dynamic_state: LoadState,
    /// 关注页子 tab：false=全部，true=只看视频
    pub dynamic_video_only: bool,
    pub live: Vec<LiveRoom>,
    pub live_state: LoadState,
    pub detail: Option<DetailData>,
    pub detail_state: LoadState,
    pub detail_tab: DetailTab,
    /// 弹幕输入框内容
    pub danmaku_text: String,
    /// 弹幕开关（详情页右上角「弹」）
    pub danmaku_on: bool,
    /// 操作回执（未登录/失败原因），显示在操作栏下方
    pub action_msg: Option<String>,
    pub room: Option<LiveRoom>,
    pub room_state: LoadState,
    /// 直播间聊天
    pub chat: ChatState,
    pub video: Player,

    pub login_user: String,
    pub login_pass: String,
    pub auth_state: LoadState,
    pub token: Option<String>,
    pub me: Option<Me>,

    pub search_kw: String,
    pub search_state: LoadState,
    pub search_hits: Vec<VideoHit>,
    pub hot: Vec<String>,
    pub hot_state: LoadState,
    /// 软键盘状态，避免每帧都调 InputMethodManager
    pub ime_on: bool,
    /// 刚进搜索页时自动聚焦输入框（拉起键盘）
    pub search_autofocus: bool,
    /// 离开搜索页时要在页面回调外收起键盘，这里留一份 Context
    ime_ctx: Option<egui::Context>,

    tx: Sender<Msg>,
    rx: Receiver<Msg>,

    auto_stop: Option<f32>,
    started: Instant,
    frames: u64,
    shot: Option<String>,
    shot_done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        Self {
            client: api::Client::from_env(),
            covers: Covers::new(api::Client::from_env().base),
            route: Route::Tab(Tab::Home),
            history: Vec::new(),
            videos: Vec::new(),
            videos_state: LoadState::Idle,
            home_cat: HomeCat::Recommend,
            dynamic: DynamicData::default(),
            dynamic_state: LoadState::Idle,
            dynamic_video_only: false,
            live: Vec::new(),
            live_state: LoadState::Idle,
            detail: None,
            detail_state: LoadState::Idle,
            detail_tab: DetailTab::Intro,
            danmaku_text: String::new(),
            danmaku_on: true,
            action_msg: None,
            room: None,
            room_state: LoadState::Idle,
            chat: ChatState::default(),
            video: Player::new(),
            login_user: String::new(),
            login_pass: String::new(),
            auth_state: LoadState::Idle,
            token: None,
            me: None,
            search_kw: String::new(),
            search_state: LoadState::Idle,
            search_hits: Vec::new(),
            hot: Vec::new(),
            hot_state: LoadState::Idle,
            ime_on: false,
            search_autofocus: false,
            ime_ctx: Some(cc.egui_ctx.clone()),
            tx,
            rx,
            auto_stop: std::env::var("APP_SECONDS").ok().and_then(|v| v.parse().ok()),
            started: Instant::now(),
            frames: 0,
            shot: std::env::var("APP_SHOT").ok(),
            shot_done: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    // ---------- 加载 ----------

    pub fn load_videos(&mut self) {
        if self.videos_state == LoadState::Loading {
            return;
        }
        self.videos_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let cat = self.home_cat.clone();
        std::thread::spawn(move || {
            let r = match cat {
                HomeCat::Recommend => client.recommend(30),
                HomeCat::Hot => client.videos(30),
                HomeCat::Zone(z) => client.zone_feed(&z, 30),
            };
            let _ = tx.send(Msg::Videos(r));
        });
    }

    /// 首页切分类：只有分区变化才重新拉；直播交给底部 tab
    pub fn select_cat(&mut self, cat: HomeCat) {
        if self.home_cat == cat {
            return;
        }
        self.home_cat = cat;
        self.videos.clear();
        self.videos_state = LoadState::Idle;
        self.load_videos();
    }

    /// 从「全部分区」页挑了一个分区：切回首页并按该分区加载
    pub fn open_zone(&mut self, zone: String) {
        self.home_cat = HomeCat::Zone(zone);
        self.videos.clear();
        self.videos_state = LoadState::Idle;
        self.navigate(Route::Tab(Tab::Home));
        self.load_videos();
    }

    /// 关注页：我关注的人 + 动态流 + 关注的人的投稿，一次拉齐
    pub fn load_dynamic(&mut self) {
        let Some(token) = self.token.clone() else {
            self.dynamic_state = LoadState::Idle;
            return;
        };
        if self.dynamic_state == LoadState::Loading {
            return;
        }
        self.dynamic_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                let followings = client.followings(&token)?;
                let dynamics = client.following_dynamics(&token).unwrap_or_default();
                let videos = client.subscription_feed(&token).unwrap_or_default();
                Ok(DynamicData {
                    followings,
                    dynamics,
                    videos,
                })
            })();
            let _ = tx.send(Msg::Dynamic(r));
        });
    }

    pub fn load_live(&mut self) {
        if self.live_state == LoadState::Loading {
            return;
        }
        self.live_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Live(client.live_rooms(20)));
        });
    }

    /// 进直播间就连 WS；换房间会先断开旧的
    pub fn chat_open(&mut self, room_id: u64) {
        if self.chat.room_id == room_id && self.chat.conn.is_some() {
            return;
        }
        self.chat_close();
        self.chat.msgs.clear();
        self.chat.online = 0;
        self.chat.input.clear();
        self.chat.room_id = room_id;
        self.chat.state = LoadState::Loading;
        self.chat.conn = Some(crate::livechat::connect(
            &self.client.base,
            room_id,
            self.token.as_deref(),
        ));
    }

    pub fn chat_close(&mut self) {
        self.chat.conn = None;
        self.chat.room_id = 0;
        self.chat.state = LoadState::Idle;
    }

    /// 把 WS 线程攒下的事件搬到 UI 状态里（每帧调一次）
    pub fn chat_poll(&mut self) {
        let Some(conn) = self.chat.conn.as_ref() else {
            return;
        };
        let evts = conn.drain();
        for e in evts {
            match e {
                crate::livechat::Evt::Msg { user, content } => {
                    self.chat.msgs.push(ChatMsg {
                        user,
                        content,
                        system: false,
                    });
                    self.chat.state = LoadState::Ready;
                }
                crate::livechat::Evt::System(text) => {
                    self.chat.msgs.push(ChatMsg {
                        user: String::new(),
                        content: text,
                        system: true,
                    });
                    if self.chat.state == LoadState::Loading {
                        self.chat.state = LoadState::Ready;
                    }
                }
                crate::livechat::Evt::Count(n) => self.chat.online = n,
                crate::livechat::Evt::Closed(e) => {
                    self.chat.state = LoadState::Err(e);
                    self.chat.conn = None;
                    break;
                }
            }
        }
        // 只留最近 200 条，别无限涨
        let len = self.chat.msgs.len();
        if len > 200 {
            self.chat.msgs.drain(..len - 200);
        }
    }

    pub fn chat_send(&mut self) {
        let text = self.chat.input.trim().to_owned();
        if text.is_empty() {
            return;
        }
        if let Some(conn) = self.chat.conn.as_ref() {
            conn.send(&text);
            self.chat.input.clear();
        } else {
            // 断了就重连一次，别让用户对着输入框干等
            let room = self.chat.room_id;
            let base = self.client.base.clone();
            let token = self.token.clone();
            self.chat.conn = Some(crate::livechat::connect(&base, room, token.as_deref()));
            self.chat.state = LoadState::Loading;
        }
    }

    pub fn open_detail(&mut self, id: u64) {
        // 换视频先掐掉上一个的解码器，否则会串音
        if self.route != Route::Detail(id) {
            self.video.stop();
        }
        self.navigate(Route::Detail(id));
        self.detail = None;
        self.detail_tab = DetailTab::Intro;
        self.action_msg = None;
        self.detail_state = LoadState::Loading;
        self.spawn_detail(id);
    }

    /// 互动之后静默刷新（不清空旧数据，避免整页闪「加载中」）
    fn refresh_detail(&mut self, id: u64) {
        self.spawn_detail(id);
    }

    fn spawn_detail(&mut self, id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                let v = client.video_detail(id)?;
                let d = client.danmakus(id).unwrap_or_default();
                let c = client.comments(id).unwrap_or_default();
                let related = client
                    .related(12)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|x| x.id != id)
                    .take(8)
                    .collect();
                Ok(DetailData {
                    video: v,
                    danmakus: d,
                    comments: c,
                    related,
                })
            })();
            let _ = tx.send(Msg::Detail(id, r));
        });
    }

    /// 发弹幕：带当前播放进度作为 video_time
    pub fn send_danmaku(&mut self, id: u64) {
        let text = self.danmaku_text.trim().to_owned();
        if text.is_empty() {
            self.action_msg = Some("弹幕不能为空".to_owned());
            return;
        }
        let Some(token) = self.token.clone() else {
            self.action_msg = Some("登录后才能发弹幕，先去「我的」登录".to_owned());
            return;
        };
        let at = (self.video.position_ms().max(0) as f64) / 1000.0;
        self.action_msg = None;
        self.danmaku_text.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Action(id, client.post_danmaku(id, &text, at, &token)));
        });
    }

    /// 点赞 / 投币 / 收藏 / 关注：都要登录态，未登录直接给提示
    pub fn act(&mut self, id: u64, kind: Action) {
        let Some(token) = self.token.clone() else {
            self.action_msg = Some("登录后才能操作，先去「我的」登录".to_owned());
            return;
        };
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let uid = self
            .detail
            .as_ref()
            .map(|d| d.video.user_id)
            .unwrap_or_default();
        std::thread::spawn(move || {
            let r = match kind {
                Action::Like => client.toggle_like(id, &token),
                Action::Coin => client.post_coin(id, &token),
                Action::Favorite => client.toggle_favorite(id, &token),
                Action::Follow => client.toggle_follow(uid, &token),
            };
            let _ = tx.send(Msg::Action(id, r));
        });
    }

    pub fn open_live(&mut self, id: u64) {
        self.video.stop();
        self.navigate(Route::LiveRoom(id));
        self.chat_open(id);
        self.room = None;
        self.room_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::LiveRoom(id, client.live_room(id)));
        });
    }

    /// 登录：拿 token 后立刻拉 /users/me
    pub fn do_login(&mut self) {
        let user = self.login_user.trim().to_owned();
        let pass = self.login_pass.clone();
        if user.is_empty() || pass.is_empty() {
            self.auth_state = LoadState::Err("用户名和密码都要填".to_owned());
            return;
        }
        self.auth_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.login(&user, &pass).and_then(|pair| {
                let me = client.me(&pair.access_token)?;
                Ok((pair.access_token, me))
            });
            let _ = tx.send(Msg::Login(r));
        });
    }

    pub fn logout(&mut self) {
        self.token = None;
        self.me = None;
        self.login_pass.clear();
        self.auth_state = LoadState::Idle;
    }

    /// 页面跳转：tab 之间切换清栈，二级页记录来源，供返回键回退
    pub fn navigate(&mut self, route: Route) {
        match route {
            Route::Tab(_) => self.history.clear(),
            _ => {
                if self.route != route {
                    self.history.push(self.route);
                    // 只留最近 8 层，够用且不至于无限增长
                    if self.history.len() > 8 {
                        self.history.remove(0);
                    }
                }
            }
        }
        if route != Route::Search {
            self.close_ime();
        } else {
            self.search_autofocus = true;
        }
        self.route = route;
    }

    /// 返回上一页；栈空时回到首页
    pub fn back(&mut self) {
        self.video.stop();
        self.close_ime();
        self.route = self.history.pop().unwrap_or(Route::Tab(Tab::Home));
    }

    /// 搜索页离开/失焦时收起软键盘
    pub fn close_ime(&mut self) {
        if self.ime_on {
            self.ime_on = false;
            if let Some(ctx) = &self.ime_ctx {
                ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(false));
            }
        }
    }

    /// 发一次搜索请求
    pub fn do_search(&mut self) {
        let kw = self.search_kw.trim().to_owned();
        if kw.is_empty() {
            self.search_state = LoadState::Idle;
            self.search_hits.clear();
            return;
        }
        self.search_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Search(client.search(&kw)));
        });
    }

    pub fn load_hot(&mut self) {
        if self.hot_state == LoadState::Loading {
            return;
        }
        self.hot_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Hot(client.hot_search(10)));
        });
    }

    fn poll(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Videos(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> = list.iter().map(|v| v.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.videos = list;
                        self.videos_state = LoadState::Ready;
                    }
                    Err(e) => self.videos_state = LoadState::Err(e),
                },
                Msg::Live(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> = list.iter().map(|v| v.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.live = list;
                        self.live_state = LoadState::Ready;
                        if std::env::var("APP_PAGE").ok().as_deref() == Some("live-room") {
                            if let Some(r) = self.live.first() {
                                let id = r.id;
                                self.open_live(id);
                            }
                        }
                    }
                    Err(e) => self.live_state = LoadState::Err(e),
                },
                Msg::Detail(id, r) => {
                    if self.route != Route::Detail(id) {
                        continue;
                    }
                    match r {
                        Ok(data) => {
                            let mut urls = vec![data.video.cover_url.clone()];
                            urls.push(data.video.uploader_avatar_url.clone());
                            urls.extend(data.related.iter().map(|v| v.cover_url.clone()));
                            self.covers.request(&urls);
                            self.detail = Some(data);
                            self.detail_state = LoadState::Ready;
                        }
                        Err(e) => self.detail_state = LoadState::Err(e),
                    }
                }
                Msg::Action(id, r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        if self.route == Route::Detail(id) {
                            self.refresh_detail(id);
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::Login(r) => match r {
                    Ok((token, me)) => {
                        self.token = Some(token);
                        self.me = Some(me);
                        self.login_pass.clear();
                        self.auth_state = LoadState::Ready;
                    }
                    Err(e) => self.auth_state = LoadState::Err(e),
                },
                Msg::LiveRoom(id, r) => {
                    if self.route != Route::LiveRoom(id) {
                        continue;
                    }
                    match r {
                        Ok(room) => {
                            self.covers.request(&[room.cover_url.clone()]);
                            self.room = Some(room);
                            self.room_state = LoadState::Ready;
                        }
                        Err(e) => self.room_state = LoadState::Err(e),
                    }
                }
                Msg::Search(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> = list.iter().map(|v| v.pic.clone()).collect();
                        self.covers.request(&urls);
                        self.search_hits = list;
                        self.search_state = LoadState::Ready;
                    }
                    Err(e) => self.search_state = LoadState::Err(e),
                },
                Msg::Hot(r) => match r {
                    Ok(items) => {
                        self.hot = items;
                        self.hot_state = LoadState::Ready;
                    }
                    Err(e) => self.hot_state = LoadState::Err(e),
                },
                Msg::Dynamic(r) => match r {
                    Ok(data) => {
                        let mut urls: Vec<String> = data
                            .followings
                            .iter()
                            .map(|u| u.avatar_url.clone())
                            .collect();
                        urls.extend(data.dynamics.iter().map(|d| d.author_avatar.clone()));
                        urls.extend(data.dynamics.iter().flat_map(|d| d.images.clone()));
                        urls.extend(data.videos.iter().map(|v| v.cover_url.clone()));
                        self.covers.request(&urls);
                        self.dynamic = data;
                        self.dynamic_state = LoadState::Ready;
                    }
                    Err(e) => self.dynamic_state = LoadState::Err(e),
                },
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frames += 1;
        self.poll();
        self.covers.poll(ui.ctx());

        if self.videos_state == LoadState::Idle && matches!(self.route, Route::Tab(Tab::Home)) {
            self.load_videos();
        }
        if self.live_state == LoadState::Idle && matches!(self.route, Route::Tab(Tab::Live)) {
            self.load_live();
        }
        if self.dynamic_state == LoadState::Idle
            && matches!(self.route, Route::Tab(Tab::Dynamic))
            && self.token.is_some()
        {
            self.load_dynamic();
        }
        // 离开直播间就断开聊天，别让 WS 在后台挂着
        if !matches!(self.route, Route::LiveRoom(_)) && self.chat.conn.is_some() {
            self.chat_close();
        }
        self.chat_poll();
        if self.hot_state == LoadState::Idle && matches!(self.route, Route::Search) {
            self.load_hot();
        }

        // 安卓三段式导航栏的返回键（winit 映射成 BrowserBack）与桌面 Esc：
        // 二级页回上一页，根页面退出 App（否则返回键会被 winit 吃掉、什么都不发生）
        let raw_back = ui.input(|i| i.key_pressed(egui::Key::BrowserBack));
        let esc_back = ui.input(|i| i.key_pressed(egui::Key::Escape))
            && !ui.ctx().egui_wants_keyboard_input();
        if raw_back || esc_back {
            if matches!(self.route, Route::Tab(_)) {
                // 根页面：退到后台而不是 finish Activity（finish 后同进程重建会让 winit 起不来）
                eprintln!("[app] back on root -> move task to back");
                self.video.stop();
                crate::video::move_task_to_back();
            } else {
                eprintln!("[app] back -> pop {:?}", self.route);
                self.back();
            }
        }

        if self.frames == 10 {
            match std::env::var("APP_PAGE").ok().as_deref() {
                Some("live") | Some("live-room") => self.route = Route::Tab(Tab::Live),
                Some("mine") => self.route = Route::Tab(Tab::Mine),
                Some("mine-login") => {
                    self.route = Route::Tab(Tab::Mine);
                    if let Ok(v) = std::env::var("APP_LOGIN") {
                        if let Some((u, p)) = v.split_once(':') {
                            self.login_user = u.to_owned();
                            self.login_pass = p.to_owned();
                            self.do_login();
                        }
                    }
                }
                Some("detail") if !self.videos.is_empty() => {
                    let id = self.videos[0].id;
                    self.open_detail(id);
                }
                _ => {}
            }
        }

        if self.shot.is_some() {
            use std::sync::atomic::Ordering;
            if self.frames == 20 {
                let path = self.shot.clone().unwrap_or_default();
                let done = self.shot_done.clone();
                let cb = egui::ScreenshotCallback::new(
                    move |img: std::sync::Arc<egui::ColorImage>| {
                        let [w, h] = img.size;
                        let mut bytes = Vec::with_capacity(w * h * 4);
                        for px in &img.pixels {
                            bytes.extend_from_slice(&px.to_array());
                        }
                        if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                            let _ = buf.save(&path);
                            eprintln!("[app] screenshot -> {path} ({w}x{h})");
                        }
                        done.store(true, Ordering::SeqCst);
                    },
                );
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Screenshot(cb));
            }
            if self.shot_done.load(Ordering::SeqCst) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // 系统状态栏/导航栏内缩（安卓真机会压住内容）
        let vp = ui.ctx().viewport_rect();
        let content = ui.ctx().content_rect();
        let mut top_inset = (content.top() - vp.top()).max(0.0);
        if top_inset < 1.0 {
            top_inset = crate::video::status_bar_points(ui.ctx().pixels_per_point());
        }
        let mut bottom_inset = (vp.bottom() - content.bottom()).max(0.0);
        if bottom_inset < 1.0 {
            // 安卓原生窗口是全屏的，egui 拿不到系统导航栏高度，
            // 不补这一截底部导航会被手势条盖住、点不到。
            bottom_inset = crate::video::nav_bar_points(ui.ctx().pixels_per_point());
        }
        // 软键盘是盖在画布上的（Android 15 edge-to-edge 下 adjustResize 不生效），
        // 输入框聚焦时把内容整体顶到键盘上面，否则用户看不见自己在打什么
        if self.ime_on {
            let ime = crate::video::ime_points(ui.ctx().pixels_per_point());
            if ime > bottom_inset {
                bottom_inset = ime;
            }
        }
        if self.frames == 40 {
            eprintln!(
                "[app] insets top={top_inset:.1} bottom={bottom_inset:.1} vp={:?} content={:?}",
                vp, content
            );
        }
        if top_inset > 0.5 {
            ui.add_space(top_inset);
        }

        if matches!(self.route, Route::Tab(_)) {
            self.bottom_nav(ui, bottom_inset);
        } else if bottom_inset > 0.5 {
            // 二级页没有底部导航，也要把系统导航栏那一条让出来，
            // 否则直播间聊天输入框会被三键导航压住点不到
            egui::Panel::bottom("sys-nav-inset")
                .exact_size(bottom_inset)
                .frame(egui::Frame::NONE.fill(theme::BG))
                .show(ui, |_ui| {});
        }

        egui::CentralPanel::no_frame()
            .frame(egui::Frame::NONE.fill(theme::BG))
            .show(ui, |ui| {
                let route = self.route;
                match route {
                    Route::Tab(Tab::Home) => pages::home::show(ui, self),
                    Route::Tab(Tab::Dynamic) => pages::dynamic::show(ui, self),
                    Route::Tab(Tab::Live) => pages::live::list(ui, self),
                    Route::Tab(Tab::Mine) => pages::mine::show(ui, self),
                    Route::Detail(id) => pages::detail::show(ui, self, id),
                    Route::LiveRoom(id) => pages::live::room(ui, self, id),
                    Route::Search => pages::search::show(ui, self),
                    Route::Zones => pages::zones::show(ui, self),
                }
            });

        ui.ctx().request_repaint();

        if let Some(limit) = self.auto_stop {
            if self.started.elapsed().as_secs_f32() >= limit {
                eprintln!(
                    "[app] auto-stop frames={} videos={} live={}",
                    self.frames,
                    self.videos.len(),
                    self.live.len()
                );
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl App {
    fn bottom_nav(&mut self, ui: &mut egui::Ui, bottom_inset: f32) {
        egui::Panel::bottom("nav")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0, theme::LINE))
                    .inner_margin(egui::Margin {
                        left: 0,
                        right: 0,
                        top: 0,
                        bottom: bottom_inset.round() as i8,
                    }),
            )
            .show(ui, |ui| {
                let w = ui.available_width();
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 52.0), Sense::click());
                let items = [
                    (Tab::Home, "首页", widgets::NavIcon::Home),
                    (Tab::Dynamic, "动态", widgets::NavIcon::Star),
                    (Tab::Live, "直播", widgets::NavIcon::Bag),
                    (Tab::Mine, "我的", widgets::NavIcon::Person),
                ];
                let slot = w / items.len() as f32;
                let p = ui.painter();
                for (i, (tab, label, icon)) in items.iter().enumerate() {
                    let cx = rect.left() + slot * (i as f32 + 0.5);
                    let active = self.route == Route::Tab(*tab);
                    let color = if active { theme::PINK } else { theme::TEXT2 };
                    widgets::nav_icon(p, Pos2::new(cx, rect.top() + 17.0), *icon, color);
                    p.text(
                        Pos2::new(cx, rect.top() + 30.0),
                        Align2::CENTER_TOP,
                        *label,
                        FontId::proportional(10.0),
                        color,
                    );
                }
                if resp.clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        for (i, (tab, _, _)) in items.iter().enumerate() {
                            let l = rect.left() + slot * i as f32;
                            if pos.x >= l && pos.x < l + slot {
                                self.navigate(Route::Tab(*tab));
                            }
                        }
                    }
                }
            });
    }

    /// 通用空态：错误信息 + 重试按钮
    pub fn empty_state(ui: &mut egui::Ui, text: &str, hint: &str) -> bool {
        let mut retry = false;
        ui.vertical_centered(|ui| {
            ui.add_space(120.0);
            ui.label(egui::RichText::new(text).color(theme::TEXT).size(14.0));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(hint).color(theme::TEXT3).size(11.0));
            ui.add_space(12.0);
            if ui.button("重试").clicked() {
                retry = true;
            }
        });
        retry
    }
}
