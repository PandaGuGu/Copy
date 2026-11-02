//! 应用外壳：路由、底部导航、异步加载
//!
//! 原则：页面上的每个数字都来自接口；接口没有的东西就不显示。
//! 加载失败只显示错误与重试，不用假数据兜底。

use crate::{
    api::{self, Comment, Danmaku, LiveRoom, Me, Video},
    images::Covers,
    pages, theme, video::Player, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Stroke, Vec2};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Live,
    Mine,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Tab(Tab),
    Detail(u64),
    LiveRoom(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState {
    Idle,
    Loading,
    Ready,
    Err(String),
}

pub struct DetailData {
    pub video: Video,
    pub danmakus: Vec<Danmaku>,
    pub comments: Vec<Comment>,
}

enum Msg {
    Videos(Result<Vec<Video>, String>),
    Live(Result<Vec<LiveRoom>, String>),
    Detail(u64, Result<(Video, Vec<Danmaku>, Vec<Comment>), String>),
    LiveRoom(u64, Result<LiveRoom, String>),
    Login(Result<(String, Me), String>),
}

pub struct App {
    pub client: api::Client,
    pub covers: Covers,

    pub route: Route,
    pub videos: Vec<Video>,
    pub videos_state: LoadState,
    pub live: Vec<LiveRoom>,
    pub live_state: LoadState,
    pub detail: Option<DetailData>,
    pub detail_state: LoadState,
    pub room: Option<LiveRoom>,
    pub room_state: LoadState,
    pub video: Player,

    pub login_user: String,
    pub login_pass: String,
    pub auth_state: LoadState,
    pub token: Option<String>,
    pub me: Option<Me>,

    tx: Sender<Msg>,
    rx: Receiver<Msg>,

    auto_stop: Option<f32>,
    started: Instant,
    frames: u64,
    shot: Option<String>,
    shot_done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        Self {
            client: api::Client::from_env(),
            covers: Covers::new(api::Client::from_env().base),
            route: Route::Tab(Tab::Home),
            videos: Vec::new(),
            videos_state: LoadState::Idle,
            live: Vec::new(),
            live_state: LoadState::Idle,
            detail: None,
            detail_state: LoadState::Idle,
            room: None,
            room_state: LoadState::Idle,
            video: Player::new(),
            login_user: String::new(),
            login_pass: String::new(),
            auth_state: LoadState::Idle,
            token: None,
            me: None,
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
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Videos(client.videos(30)));
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

    pub fn open_detail(&mut self, id: u64) {
        self.route = Route::Detail(id);
        self.detail = None;
        self.detail_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                let v = client.video(id)?;
                let d = client.danmakus(id).unwrap_or_default();
                let c = client.comments(id).unwrap_or_default();
                Ok((v, d, c))
            })();
            let _ = tx.send(Msg::Detail(id, r));
        });
    }

    pub fn open_live(&mut self, id: u64) {
        self.video.stop();
        self.route = Route::LiveRoom(id);
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

    pub fn back(&mut self) {
        self.video.stop();
        self.route = Route::Tab(Tab::Home);
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
                        Ok((video, danmakus, comments)) => {
                            self.covers.request(&[video.cover_url.clone()]);
                            self.detail = Some(DetailData {
                                video,
                                danmakus,
                                comments,
                            });
                            self.detail_state = LoadState::Ready;
                        }
                        Err(e) => self.detail_state = LoadState::Err(e),
                    }
                }
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
        }

        egui::CentralPanel::no_frame()
            .frame(egui::Frame::NONE.fill(theme::BG))
            .show(ui, |ui| {
                let route = self.route;
                match route {
                    Route::Tab(Tab::Home) => pages::home::show(ui, self),
                    Route::Tab(Tab::Live) => pages::live::list(ui, self),
                    Route::Tab(Tab::Mine) => pages::mine::show(ui, self),
                    Route::Detail(id) => pages::detail::show(ui, self, id),
                    Route::LiveRoom(id) => pages::live::room(ui, self, id),
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
                    (Tab::Live, "直播", widgets::NavIcon::Bag),
                    (Tab::Mine, "我的", widgets::NavIcon::Person),
                ];
                let slot = w / 3.0;
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
                                self.route = Route::Tab(*tab);
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
