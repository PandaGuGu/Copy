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
    /// 稍后再看（「我的」里进）
    WatchLater,
    /// 观看历史（「我的」里进）
    History,
    /// 消息中心（「我的」里进）
    Notifications,
    /// 我的收藏（「我的」里进）
    Favorites,
    /// 他人个人主页（点 UP 主头像/名字进）
    Space(u64),
    /// 排行榜（首页右上角「榜」进）
    Rank,
    /// 评论详情（一级评论 + 它下面的全部回复）
    CommentDetail { video_id: u64, comment_id: u64 },
    /// 账号与设置
    Settings,
    /// 私信列表
    Dm,
    /// 私信会话
    DmChat(u64),
    /// 专栏列表 / 详情 / 我的文章
    ArticleList,
    Article(u64),
    MyArticles,
    /// 创作中心（投稿 / 数据 / 评论 / 弹幕）
    Creator,
    CreatorComments,
    CreatorDanmakus,
    /// 动态详情
    DynamicDetail(u64),
    /// 专题
    Specials,
    /// 硬币账本（含每日任务）
    Ledger,
    /// 治理与客服
    Governance,
    /// 直播开播
    LiveHost,
    /// 稿件编辑（创作中心）
    VideoEdit(u64),
    /// 投稿上传
    Upload,
}

/// 「更多」页面的统一数据槽：一个路由一次只用到其中几项，
/// 用同一套 LoadState + Msg 承载，省掉十几套重复状态。
#[derive(Default)]
pub struct MoreData {
    pub articles: Vec<api::ArticleCard>,
    pub article: Option<api::ArticleDetail>,
    pub article_comments: Vec<api::ArticleComment>,
    pub my_articles: Vec<api::ArticleCard>,
    pub dynamic: Option<api::DynItem>,
    pub dynamic_comments: Vec<api::Comment>,
    pub videos: Vec<api::MyVideo>,
    pub stats: Option<api::CreatorStats>,
    pub creator_comments: Vec<api::Comment>,
    pub danmakus: Vec<api::CreatorDanmaku>,
    pub specials: Vec<api::SpecialPage>,
    pub ledger: Option<api::CoinLedgerPage>,
    pub rewards: Option<api::DailyRewards>,
    pub tickets: Vec<api::Ticket>,
    pub appeals: Vec<api::Appeal>,
    pub cs: Vec<api::CsConversation>,
    /// 当前正在编辑的文章 id（0 = 新建）
    pub editing: u64,
    /// 我的直播间
    pub live: Option<api::LiveRoom>,
    /// 稿件编辑用的视频详情与章节
    pub video: Option<api::VideoDetail>,
    pub chapters: Vec<api::Chapter>,
    /// 拉取失败原因
    pub err: Option<String>,
}

/// 他人主页的子 tab
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpaceTab {
    Videos,
    Articles,
    Dynamics,
    Favorites,
    Following,
    Followers,
}

/// 他人主页一次拉齐的数据（资料 + 六个子资源）
#[derive(Default)]
pub struct SpaceData {
    pub profile: api::UserProfile,
    pub videos: Vec<api::SpaceVideo>,
    pub articles: Vec<api::ArticleCard>,
    pub dynamics: Vec<api::DynItem>,
    pub favorites: Vec<api::FavoriteItem>,
    pub following: Vec<api::FollowUser>,
    pub followers: Vec<api::FollowUser>,
}

/// 首页运营位：轮播图 + 统计 + 真实分区，一次拉齐
#[derive(Default)]
pub struct HomeOps {
    pub banners: Vec<api::Banner>,
    pub stats: Option<api::HomeStats>,
    /// `/zones` 原始分区（父子形式，如「生活-日常」），按视频数倒序
    pub zones: Vec<api::ZoneItem>,
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
    /// 章节（GET /videos/:id/chapters）
    pub chapters: Vec<api::Chapter>,
    /// 多清晰度（GET /videos/:id/bitrates），空 = 只有原画
    pub bitrates: Vec<api::VideoBitrate>,
    /// 字幕轨道（GET /videos/:id/subtitles，含 content）
    pub subtitles: Vec<api::Subtitle>,
    /// 播放源是否真的存在（预检 404），false 时页面直接提示，不去调播放器
    pub source_ready: bool,
    /// 上次看到的位置（秒），来自 /users/me/view-history，用于断点续播
    pub resume_sec: Option<f64>,
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
    WatchLater,
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
    Login(Result<Me, String>),
    /// 注册（后端注册不返回 token，成功后再自动登录并拉 /users/me）
    Register(Result<Me, String>),
    Search(Result<Vec<VideoHit>, String>),
    Hot(Result<Vec<String>, String>),
    /// 点赞/投币/收藏/关注的回执（成功后静默刷新详情）
    Action(u64, Result<(), String>),
    /// 关注页（动态/投稿/关注列表）
    Dynamic(Result<DynamicData, String>),
    /// 稍后再看列表
    WatchLater(Result<Vec<api::WatchLaterItem>, String>),
    /// 稍后再看页的增删操作
    WatchLaterAction(Result<(), String>),
    /// 消息中心列表
    Notifications(Result<Vec<api::Notification>, String>),
    /// 消息页的已读/删除操作
    NotifAction(Result<(), String>),
    /// 观看历史列表
    History(Result<api::HistoryPage, String>),
    /// 观看历史的删除/清空/暂停设置
    HistoryAction(Result<(), String>),
    /// 评论点赞（video_id, 结果）
    CommentLike(u64, Result<bool, String>),
    /// 踩评论
    CommentDislike(u64, Result<bool, String>),
    /// 评论管理操作（删除/置顶/精选/举报）
    CommentAction(u64, Result<(), String>),
    /// 发表评论/回复（video_id, 结果）
    CommentPosted(u64, Result<(), String>),
    /// 详情页收藏夹选择弹窗（video_id, 结果）
    Picker(u64, Result<api::FavoritePicker, String>),
    /// 保存收藏夹选择
    PickerSave(Result<(), String>),
    /// 我的收藏：视频列表
    Favorites(Result<Vec<api::FavoriteItem>, String>),
    /// 我的收藏：收藏夹列表
    Folders(Result<Vec<api::FavoriteFolder>, String>),
    /// 新建 / 删除收藏夹
    FolderAction(Result<(), String>),
    /// 首页运营位（轮播图 / 统计 / 真实分区）
    HomeOps(Result<HomeOps, String>),
    /// 排行榜
    Rank(Result<Vec<api::RankVideo>, String>),
    /// 搜索联想（term, 结果）——term 用于丢弃过期回包
    Suggest(String, Result<Vec<api::SuggestTag>, String>),
    /// 他人主页（user_id, 结果）
    Space(u64, Result<SpaceData, String>),
    /// 他人主页关注按钮（user_id, 结果）
    SpaceFollow(u64, Result<(), String>),
    /// 未读角标
    Unread(Result<api::UnreadSummary, String>),
    /// 设置页写操作（保存资料 / 公告 / 密码 / 注销）
    SettingsDone(Result<(), String>),
    /// 空间隐私
    Privacy(Result<api::SpacePrivacy, String>),
    /// 关注分组
    Groups(Result<Vec<api::FollowGroup>, String>),
    /// 私信会话列表
    DmList(Result<Vec<api::DmConversation>, String>),
    /// 私信消息（会话 id, 结果）
    DmMessages(u64, Result<Vec<api::DmMessage>, String>),
    /// 私信发送完成
    DmSent(u64, Result<(), String>),
    /// 找到/创建私信会话后直接进聊天
    DmCreated(Result<api::DmConversation, String>),
    /// 重新拉取当前登录用户
    Me(Result<api::Me, String>),
    /// 自动切换到可用的后端地址
    Server(String),
    /// 「更多」页面统一回包
    More(Box<MoreData>),
    /// 「更多」页面的写操作结果
    MoreDone(Result<(), String>),
    /// 投稿上传结果（新视频 id）
    Uploaded(Result<u64, String>),
    /// 文章编辑器回填（标题, 正文）
    ArticleEditor(Result<api::ArticleDetail, String>),
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
    /// 评论输入框内容
    pub comment_text: String,
    /// 当前回复目标（评论 id, 昵称）；None = 发一级评论
    pub reply_to: Option<(u64, String)>,
    /// 从消息中心跳转时，需要定位并高亮的评论 id
    pub pending_comment: Option<u64>,
    /// 跳转后保持高亮的评论 id（滚动完成后依然能看见）
    pub highlight_comment: Option<u64>,
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
    /// 未登录页当前是不是「注册」模式
    pub show_register: bool,
    pub reg_user: String,
    pub reg_pass: String,
    pub reg_state: LoadState,
    pub me: Option<Me>,

    pub search_kw: String,
    pub search_state: LoadState,
    pub search_hits: Vec<VideoHit>,
    pub hot: Vec<String>,
    pub hot_state: LoadState,
    /// 稍后再看
    pub watch_later: Vec<api::WatchLaterItem>,
    pub watch_later_state: LoadState,
    /// 消息中心（按分类拉取）
    pub notifications: Vec<api::Notification>,
    pub notifications_state: LoadState,
    pub notification_cat: String,
    /// 消息页的操作反馈
    pub notify_msg: Option<String>,
    /// 观看历史
    pub view_history: Vec<api::ViewHistoryItem>,
    pub history_state: LoadState,
    /// 后端设置：是否暂停记录观看历史
    pub history_paused: bool,
    /// 上次上报播放进度的时刻（限流用）
    progress_at: Option<Instant>,
    /// 待应用的续播位置（video_id, 秒）
    resume: Option<(u64, f64)>,
    /// 详情页收藏夹选择弹窗的数据（None 表示弹窗关闭）
    pub favorite_picker: Option<api::FavoritePicker>,
    pub picker_state: LoadState,
    /// 弹窗里当前勾选的收藏夹
    pub picker_selected: Vec<u64>,
    /// 弹窗对应的视频
    pub picker_video: u64,
    /// 我的收藏：视频与收藏夹
    pub favorites: Vec<api::FavoriteItem>,
    pub favorites_state: LoadState,
    pub favorite_folders: Vec<api::FavoriteFolder>,
    pub folders_state: LoadState,
    /// 当前筛选的收藏夹（None = 全部）
    pub favorites_folder: Option<u64>,
    /// 新建收藏夹的输入框
    pub new_folder_title: String,
    pub folder_msg: Option<String>,
    /// 首页运营位：轮播图 + 统计 + 真实分区
    pub home_ops: HomeOps,
    pub home_ops_state: LoadState,
    /// 排行榜
    pub rank: Vec<api::RankVideo>,
    pub rank_state: LoadState,
    pub rank_by: String,
    pub rank_period: String,
    /// 搜索联想词（`suggest_for` 是这份结果对应的关键词）
    pub suggest: Vec<api::SuggestTag>,
    pub suggest_for: String,
    /// 他人主页
    pub space: Option<SpaceData>,
    pub space_state: LoadState,
    pub space_tab: SpaceTab,
    /// 消息中心未读角标
    pub unread: api::UnreadSummary,
    /// 评论排序：hot（按热度）/ latest（按时间），本地排，不重新请求
    pub comment_sort: String,
    /// 当前展开「⋯」菜单的评论 id
    pub comment_menu: Option<u64>,
    /// 详情页当前选的清晰度（None = 原画）
    pub quality: Option<u64>,
    /// 详情页当前字幕轨道（None = 关闭）与解析好的时间轴
    pub subtitle_id: Option<u64>,
    pub cues: Vec<crate::subtitle::Cue>,
    // ---- 设置页 ----
    pub set_nickname: String,
    pub set_sign: String,
    pub set_gender: String,
    pub set_birthday: String,
    pub set_announcement: String,
    pub set_old_pw: String,
    pub set_new_pw: String,
    pub set_del_pw: String,
    pub set_server: String,
    pub settings_state: LoadState,
    pub settings_msg: Option<String>,
    pub privacy: Option<api::SpacePrivacy>,
    pub groups: Vec<api::FollowGroup>,
    pub new_group: String,
    pub deletion_pending: bool,
    // ---- 私信 ----
    pub dm_convs: Vec<api::DmConversation>,
    pub dm_state: LoadState,
    pub dm_chat: Vec<api::DmMessage>,
    pub dm_chat_state: LoadState,
    pub dm_text: String,
    pub dm_peer_name: String,
    // ---- 「更多」页面 ----
    pub more: MoreData,
    pub more_state: LoadState,
    pub f_title: String,
    pub f_body: String,
    pub f_text: String,
    pub f_tags: String,
    pub f_schedule: String,
    // ---- 投稿 ----
    pub inbox: Vec<(String, u64)>,
    pub up_path: Option<String>,
    pub up_state: LoadState,
    pub up_msg: Option<String>,
    pub up_zone: String,
    /// 软键盘状态，避免每帧都调 InputMethodManager
    pub ime_on: bool,
    /// 本帧是否有输入框持焦（多个页面各自上报，帧末只下发一次）
    pub ime_wanted: bool,
    /// 本帧底部被系统占掉的高度（导航栏 + 键盘），页面做悬浮层时要用
    pub bottom_inset: f32,
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
        // 启动时优先用设置页保存的服务器地址
        let mut client = api::Client::from_env();
        if let Some(u) = Self::saved_server() {
            client.base = u;
        }
        let covers = Covers::new(client.base.clone());
        Self {
            client,
            covers,
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
            comment_text: String::new(),
            reply_to: None,
            pending_comment: None,
            highlight_comment: None,
            danmaku_on: true,
            action_msg: None,
            room: None,
            room_state: LoadState::Idle,
            chat: ChatState::default(),
            video: Player::new(),
            login_user: String::new(),
            login_pass: String::new(),
            auth_state: LoadState::Idle,
            show_register: false,
            reg_user: String::new(),
            reg_pass: String::new(),
            reg_state: LoadState::Idle,
            me: None,
            search_kw: String::new(),
            search_state: LoadState::Idle,
            search_hits: Vec::new(),
            hot: Vec::new(),
            hot_state: LoadState::Idle,
            watch_later: Vec::new(),
            watch_later_state: LoadState::Idle,
            notifications: Vec::new(),
            notifications_state: LoadState::Idle,
            notification_cat: "reply_received".to_owned(),
            notify_msg: None,
            view_history: Vec::new(),
            history_state: LoadState::Idle,
            history_paused: false,
            progress_at: None,
            resume: None,
            favorite_picker: None,
            picker_state: LoadState::Idle,
            picker_selected: Vec::new(),
            picker_video: 0,
            favorites: Vec::new(),
            favorites_state: LoadState::Idle,
            favorite_folders: Vec::new(),
            folders_state: LoadState::Idle,
            favorites_folder: None,
            new_folder_title: String::new(),
            folder_msg: None,
            home_ops: HomeOps::default(),
            home_ops_state: LoadState::Idle,
            rank: Vec::new(),
            rank_state: LoadState::Idle,
            rank_by: "play".to_owned(),
            rank_period: "all".to_owned(),
            suggest: Vec::new(),
            suggest_for: String::new(),
            space: None,
            space_state: LoadState::Idle,
            space_tab: SpaceTab::Videos,
            unread: api::UnreadSummary::default(),
            comment_sort: "hot".to_owned(),
            comment_menu: None,
            quality: None,
            subtitle_id: None,
            cues: Vec::new(),
            set_nickname: String::new(),
            set_sign: String::new(),
            set_gender: "secret".to_owned(),
            set_birthday: String::new(),
            set_announcement: String::new(),
            set_old_pw: String::new(),
            set_new_pw: String::new(),
            set_del_pw: String::new(),
            set_server: String::new(),
            settings_state: LoadState::Idle,
            settings_msg: None,
            privacy: None,
            groups: Vec::new(),
            new_group: String::new(),
            deletion_pending: false,
            dm_convs: Vec::new(),
            dm_state: LoadState::Idle,
            dm_chat: Vec::new(),
            dm_chat_state: LoadState::Idle,
            dm_text: String::new(),
            dm_peer_name: String::new(),
            more: MoreData::default(),
            more_state: LoadState::Idle,
            f_title: String::new(),
            f_body: String::new(),
            f_text: String::new(),
            f_tags: String::new(),
            f_schedule: String::new(),
            inbox: Vec::new(),
            up_path: None,
            up_state: LoadState::Idle,
            up_msg: None,
            up_zone: String::new(),
            ime_on: false,
            ime_wanted: false,
            bottom_inset: 0.0,
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
        if !self.client.logged_in() {
            self.dynamic_state = LoadState::Idle;
            return;
        }
        if self.dynamic_state == LoadState::Loading {
            return;
        }
        self.dynamic_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                let followings = client.followings()?;
                let dynamics = client.following_dynamics().unwrap_or_default();
                let videos = client.subscription_feed().unwrap_or_default();
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
            self.client.access_token().as_deref(),
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
            let token = self.client.access_token();
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
        self.reply_to = None;
        self.comment_text.clear();
        self.pending_comment = None;
        self.highlight_comment = None;
        // 第一次上报从播放 20 秒后开始，别把刚续播的位置又写成 0
        self.progress_at = Some(Instant::now());
        self.resume = None;
        self.quality = None;
        self.subtitle_id = None;
        self.cues.clear();
        self.spawn_detail(id);
    }

    /// 从消息中心跳到某条评论：直接切到评论 tab，并在列表里定位高亮
    pub fn open_detail_at_comment(&mut self, video_id: u64, comment_id: u64) {
        self.open_detail(video_id);
        self.detail_tab = DetailTab::Comments;
        self.pending_comment = Some(comment_id);
        self.highlight_comment = Some(comment_id);
    }

    /// 互动之后静默刷新（不清空旧数据，避免整页闪「加载中」）
    fn refresh_detail(&mut self, id: u64) {
        self.spawn_detail(id);
    }

    /// 当前路由是否在看这条视频（详情页或它的评论详情页）
    fn on_video_route(&self, video_id: u64) -> bool {
        match self.route {
            Route::Detail(id) => id == video_id,
            Route::CommentDetail { video_id: id, .. } => id == video_id,
            _ => false,
        }
    }

    fn spawn_detail(&mut self, id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                let v = client.video_detail(id)?;
                let d = client.danmakus(id).unwrap_or_default();
                let c = client.comments(id).unwrap_or_default();
                let chapters = client.chapters(id).unwrap_or_default();
                let bitrates = client.bitrates(id).unwrap_or_default();
                let subtitles = client.subtitles(id).unwrap_or_default();
                // 优先用转码产物（bitrate），没有才回原画；顺便探一下能不能取到
                let primary = bitrates
                    .last()
                    .map(|b| b.url.clone())
                    .unwrap_or_else(|| v.video_url.clone());
                let source_ready = !primary.is_empty() && client.probe_media(&primary);
                // 观看历史里找到这条视频的上次进度（没有就从头播）
                let resume_sec = if client.logged_in() {
                    client.view_history().ok().and_then(|p| {
                        p.items
                            .into_iter()
                            .find(|h| h.video_id == id && h.media_type == "video")
                            .map(|h| h.progress_sec)
                    })
                } else {
                    None
                };
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
                    chapters,
                    bitrates,
                    subtitles,
                    source_ready,
                    resume_sec,
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
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能发弹幕，先去「我的」登录".to_owned());
            return;
        }
        let at = (self.video.position_ms().max(0) as f64) / 1000.0;
        self.action_msg = None;
        self.danmaku_text.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Action(id, client.post_danmaku(id, &text, at)));
        });
    }

    /// 点赞 / 投币 / 收藏 / 关注：都要登录态，未登录直接给提示
    pub fn act(&mut self, id: u64, kind: Action) {
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能操作，先去「我的」登录".to_owned());
            return;
        }
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
                Action::Like => client.toggle_like(id),
                Action::Coin => client.post_coin(id),
                Action::Favorite => client.toggle_favorite(id),
                Action::Follow => client.toggle_follow(uid),
                Action::WatchLater => client.toggle_watch_later(id).map(|_| ()),
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

    /// 登录：Client 存下 token 后立刻拉 /users/me
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
            let r = client.login(&user, &pass).and_then(|_| client.me());
            let _ = tx.send(Msg::Login(r));
        });
    }

    /// 注册：POST /users 成功后直接自动登录并拉 /users/me
    pub fn do_register(&mut self) {
        let user = self.reg_user.trim().to_owned();
        let pass = self.reg_pass.clone();
        if user.is_empty() || pass.is_empty() {
            self.reg_state = LoadState::Err("用户名和密码都要填".to_owned());
            return;
        }
        if pass.chars().count() < 8 {
            self.reg_state = LoadState::Err("密码至少 8 位".to_owned());
            return;
        }
        self.reg_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client
                .register(&user, &pass)
                .and_then(|_| client.login(&user, &pass))
                .and_then(|_| client.me());
            let _ = tx.send(Msg::Register(r));
        });
    }

    pub fn logout(&mut self) {
        self.client.clear_tokens();
        self.me = None;
        self.login_pass.clear();
        self.reg_pass.clear();
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
        // 离开详情前把最后几秒也存下来
        self.report_progress(true);
        self.video.stop();
        self.close_favorite_picker();
        self.close_ime();
        self.route = self.history.pop().unwrap_or(Route::Tab(Tab::Home));
        // 从详情返回这几个页面时重新拉一次，避免列表里的状态还是旧的
        match self.route {
            Route::WatchLater => self.watch_later_state = LoadState::Idle,
            Route::Notifications => self.notifications_state = LoadState::Idle,
            Route::History => self.history_state = LoadState::Idle,
            Route::Favorites => {
                self.favorites_state = LoadState::Idle;
                self.folders_state = LoadState::Idle;
            }
            _ => {}
        }
    }

    /// 输入框每帧上报自己的焦点；多个输入框同时存在时不会互相打架。
    pub fn note_focus(&mut self, focused: bool) {
        if focused {
            self.ime_wanted = true;
        }
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
        // 搜索结果出来后联想词就该收起；下次改词再重新拉
        self.suggest.clear();
        self.suggest_for.clear();
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

    // ---------- 稍后再看 / 消息中心 ----------

    pub fn open_watch_later(&mut self) {
        self.watch_later_state = LoadState::Idle;
        self.navigate(Route::WatchLater);
    }

    pub fn load_watch_later(&mut self) {
        if !self.client.logged_in() {
            self.watch_later_state = LoadState::Idle;
            return;
        }
        if self.watch_later_state == LoadState::Loading {
            return;
        }
        self.watch_later_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::WatchLater(client.watch_later()));
        });
    }

    /// 标记已看 / 清空（watched_only=true 只清已看）
    pub fn watch_later_clear(&mut self, watched_only: bool) {
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能操作，先去「我的」登录".to_owned());
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = if watched_only {
                client.clear_watched_watch_later()
            } else {
                client.clear_watch_later()
            };
            let _ = tx.send(Msg::WatchLaterAction(r));
        });
    }

    pub fn mark_watched(&mut self, id: u64) {
        if !self.client.logged_in() {
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.mark_watch_later_watched(id);
            let _ = tx.send(Msg::WatchLaterAction(r));
        });
    }

    // ---------- 收藏夹 ----------

    /// 打开详情页的收藏夹选择弹窗
    pub fn open_favorite_picker(&mut self, video_id: u64) {
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能收藏，先去「我的」登录".to_owned());
            return;
        }
        self.action_msg = None;
        self.picker_video = video_id;
        self.favorite_picker = None;
        self.picker_selected.clear();
        self.picker_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Picker(video_id, client.favorite_picker(video_id)));
        });
    }

    pub fn close_favorite_picker(&mut self) {
        self.favorite_picker = None;
        self.picker_state = LoadState::Idle;
        self.picker_selected.clear();
    }

    pub fn toggle_picker_folder(&mut self, folder_id: u64) {
        if let Some(pos) = self.picker_selected.iter().position(|x| *x == folder_id) {
            self.picker_selected.remove(pos);
        } else {
            self.picker_selected.push(folder_id);
        }
    }

    /// 保存选择；一个都没选 = 取消收藏
    pub fn save_favorite_picker(&mut self) {
        if !self.client.logged_in() {
            return;
        }
        let id = self.picker_video;
        let ids = self.picker_selected.clone();
        self.action_msg = None;
        self.picker_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::PickerSave(client.set_favorite_folders(id, &ids)));
        });
    }

    // ---------- 我的收藏 ----------

    pub fn open_favorites(&mut self) {
        self.favorites_state = LoadState::Idle;
        self.folders_state = LoadState::Idle;
        self.favorites_folder = None;
        self.folder_msg = None;
        self.new_folder_title.clear();
        self.navigate(Route::Favorites);
    }

    pub fn load_favorites(&mut self) {
        if !self.client.logged_in() {
            self.favorites_state = LoadState::Idle;
            return;
        }
        if self.favorites_state == LoadState::Loading {
            return;
        }
        self.favorites_state = LoadState::Loading;
        let folder = self.favorites_folder;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Favorites(client.my_favorites(folder)));
        });
    }

    pub fn load_folders(&mut self) {
        if !self.client.logged_in() {
            self.folders_state = LoadState::Idle;
            return;
        }
        if self.folders_state == LoadState::Loading {
            return;
        }
        self.folders_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Folders(client.favorite_folders()));
        });
    }

    /// 切换收藏夹筛选（None = 全部）
    pub fn select_favorites_folder(&mut self, folder: Option<u64>) {
        if self.favorites_folder == folder {
            return;
        }
        self.favorites_folder = folder;
        self.favorites_state = LoadState::Idle;
        self.load_favorites();
    }

    /// 新建收藏夹（弹窗与收藏页共用输入框）
    pub fn create_folder(&mut self) {
        let title = self.new_folder_title.trim().to_owned();
        if title.is_empty() {
            self.folder_msg = Some("收藏夹名称不能为空".to_owned());
            return;
        }
        if title.chars().count() > 20 {
            self.folder_msg = Some("名称最多 20 个字".to_owned());
            return;
        }
        if !self.client.logged_in() {
            self.folder_msg = Some("登录后才能新建收藏夹".to_owned());
            return;
        }
        self.folder_msg = None;
        self.new_folder_title.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::FolderAction(client.create_favorite_folder(&title)));
        });
    }

    pub fn delete_folder(&mut self, folder_id: u64) {
        if !self.client.logged_in() {
            return;
        }
        self.folder_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::FolderAction(client.delete_favorite_folder(folder_id)));
        });
    }

    /// App 私有目录（安卓上 adb push / 文件管理器都能访问）
    pub fn files_dir() -> String {
        #[cfg(target_os = "android")]
        {
            "/sdcard/Android/data/com.cakecake.rust/files".to_owned()
        }
        #[cfg(not(target_os = "android"))]
        {
            std::env::var("CAKECAKE_FILES_DIR").unwrap_or_else(|_| ".".to_owned())
        }
    }

    fn server_file() -> String {
        format!("{}/server.txt", Self::files_dir())
    }

    /// 读取保存过的服务器地址（第一行当前，第二行备用）
    pub fn saved_servers() -> Vec<String> {
        let Ok(text) = std::fs::read_to_string(Self::server_file()) else {
            return Vec::new();
        };
        text.lines()
            .map(|l| l.trim().trim_end_matches('/').to_owned())
            .filter(|l| l.starts_with("http://") || l.starts_with("https://"))
            .collect()
    }

    pub fn saved_server() -> Option<String> {
        Self::saved_servers().into_iter().next()
    }

    fn save_servers(&self, list: &[String]) {
        let _ = std::fs::create_dir_all(Self::files_dir());
        let body = list.join("
");
        let _ = std::fs::write(Self::server_file(), body);
    }

    /// 启动时探一遍所有候选地址，主地址不通就自动切到能用的那个
    fn probe_servers(&mut self) {
        let current = self.client.base.clone();
        let mut cands = Self::saved_servers();
        cands.push(current.clone());
        // 没配过也能用：USB 映射 与 模拟器网关 各兜一个
        cands.push("http://127.0.0.1:8080".to_owned());
        cands.push("http://10.0.2.2:8080".to_owned());
        let mut uniq: Vec<String> = Vec::new();
        for c in cands {
            if !uniq.iter().any(|x| x == &c) {
                uniq.push(c);
            }
        }
        uniq.retain(|c| c != &current);
        if uniq.is_empty() {
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            // 主地址先探一次：通了就什么都不用做
            if client.probe_base(&client.base) {
                return;
            }
            for c in uniq {
                if client.probe_base(&c) {
                    let _ = tx.send(Msg::Server(c));
                    return;
                }
            }
        });
    }

    /// 切到另一台后端（自动或手动）
    pub fn switch_server(&mut self, url: String) {
        if url == self.client.base {
            return;
        }
        let old = self.client.base.clone();
        if old != url {
            self.save_servers(&[url.clone(), old]);
        }
        self.save_servers(&[url.clone()]);
        self.client.base = url.clone();
        self.covers = crate::images::Covers::new(url.clone());
        self.videos_state = LoadState::Idle;
        self.home_ops_state = LoadState::Idle;
        self.live_state = LoadState::Idle;
        self.dm_state = LoadState::Idle;
        self.more_state = LoadState::Idle;
        self.client.clear_tokens();
        self.me = None;
        self.settings_msg = Some(format!("已切换到 {url}"));
    }

    /// 保存并立即切换后端地址（图片缓存一并切，避免新旧域名混着用）
    pub fn apply_server(&mut self) {
        let mut url = self.set_server.trim().to_owned();
        if url.is_empty() {
            self.settings_msg = Some("地址不能为空".to_owned());
            return;
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            url = format!("http://{url}");
        }
        let url = url.trim_end_matches('/').to_owned();
        let _ = std::fs::create_dir_all(Self::files_dir());
        if std::fs::write(Self::server_file(), &url).is_err() {
            self.settings_msg = Some("写入配置失败".to_owned());
            return;
        }
        self.save_servers(&[url.clone()]);
        self.client.base = url.clone();
        self.covers = crate::images::Covers::new(url.clone());
        // 换后端后所有缓存都失效，把状态打回 Idle 让各页面重新拉
        self.videos_state = LoadState::Idle;
        self.home_ops_state = LoadState::Idle;
        self.dm_state = LoadState::Idle;
        self.more_state = LoadState::Idle;
        self.me = None;
        self.client.clear_tokens();
        self.settings_msg = Some(format!("已切换到 {url}，请重新登录"));
        self.navigate(Route::Tab(Tab::Mine));
    }

    // ---------- 账号与设置 ----------

    pub fn open_settings(&mut self) {
        if let Some(me) = &self.me {
            self.set_nickname = me.nickname.clone();
            self.set_sign = me.sign.clone();
        }
        self.set_gender = "secret".to_owned();
        self.settings_msg = None;
        self.settings_state = LoadState::Idle;
        self.navigate(Route::Settings);
        self.load_privacy();
        self.load_groups();
    }

    /// 重新拉一次 /users/me（保存资料后用）
    pub fn reload_me(&mut self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Me(client.me()));
        });
    }

    pub fn load_privacy(&mut self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Privacy(client.space_privacy()));
        });
    }

    pub fn load_groups(&mut self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Groups(client.follow_groups()));
        });
    }

    fn settings_action<F>(&mut self, f: F)
    where
        F: FnOnce(&api::Client) -> Result<(), String> + Send + 'static,
    {
        if self.settings_state == LoadState::Loading {
            return;
        }
        self.settings_state = LoadState::Loading;
        self.settings_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = f(&client);
            let _ = tx.send(Msg::SettingsDone(r));
        });
    }

    pub fn save_profile(&mut self) {
        let (nick, sign, gender, birthday) = (
            self.set_nickname.trim().to_owned(),
            self.set_sign.trim().to_owned(),
            self.set_gender.clone(),
            self.set_birthday.trim().to_owned(),
        );
        self.settings_action(move |c| c.update_profile(&nick, &sign, &gender, &birthday));
    }

    pub fn save_announcement(&mut self) {
        let text = self.set_announcement.trim().to_owned();
        self.settings_action(move |c| c.update_announcement(&text));
    }

    pub fn save_password(&mut self) {
        let (old, new) = (self.set_old_pw.clone(), self.set_new_pw.clone());
        if new.len() < 8 {
            self.settings_msg = Some("新密码至少 8 位".to_owned());
            return;
        }
        self.settings_action(move |c| c.update_password(&old, &new));
    }

    pub fn save_privacy(&mut self, p: api::SpacePrivacy) {
        self.privacy = Some(p.clone());
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Privacy(client.set_space_privacy(&p)));
        });
    }

    pub fn create_group(&mut self) {
        let name = self.new_group.trim().to_owned();
        if name.is_empty() {
            return;
        }
        self.new_group.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = client.create_follow_group(&name);
            let _ = tx.send(Msg::Groups(client.follow_groups()));
        });
    }

    pub fn delete_group(&mut self, group_id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = client.delete_follow_group(group_id);
            let _ = tx.send(Msg::Groups(client.follow_groups()));
        });
    }

    pub fn request_deletion(&mut self) {
        let pw = self.set_del_pw.clone();
        if pw.is_empty() {
            self.settings_msg = Some("请输入密码确认".to_owned());
            return;
        }
        self.settings_action(move |c| c.request_deletion(&pw));
    }

    pub fn revoke_deletion(&mut self) {
        self.settings_action(|c| c.revoke_deletion());
    }

    // ---------- 私信 ----------

    pub fn open_dm(&mut self) {
        self.dm_state = LoadState::Idle;
        self.navigate(Route::Dm);
        self.load_dm();
    }

    pub fn load_dm(&mut self) {
        if !self.client.logged_in() || self.dm_state == LoadState::Loading {
            return;
        }
        self.dm_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.dm_conversations();
            let _ = tx.send(Msg::DmList(r));
        });
    }

    pub fn open_dm_chat(&mut self, conv: &api::DmConversation) {
        self.dm_peer_name = conv.peer_name.clone();
        self.dm_chat.clear();
        self.dm_chat_state = LoadState::Loading;
        self.dm_text.clear();
        self.navigate(Route::DmChat(conv.id));
        self.load_dm_chat(conv.id);
    }

    pub fn load_dm_chat(&mut self, conv_id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.dm_messages(conv_id);
            let _ = tx.send(Msg::DmMessages(conv_id, r));
        });
    }

    pub fn send_dm(&mut self, conv_id: u64) {
        let text = self.dm_text.trim().to_owned();
        if text.is_empty() {
            return;
        }
        self.dm_text.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.post_dm(conv_id, &text);
            let _ = tx.send(Msg::DmSent(conv_id, r));
        });
    }

    /// 置顶 / 免打扰
    pub fn dm_patch(&mut self, id: u64, pinned: Option<bool>, muted: Option<bool>) {
        if let Some(c) = self.dm_convs.iter_mut().find(|c| c.id == id) {
            if let Some(v) = pinned {
                c.pinned = v;
            }
            if let Some(v) = muted {
                c.muted = v;
            }
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = client.patch_dm(id, pinned, muted);
            let _ = tx.send(Msg::DmList(client.dm_conversations()));
        });
    }

    pub fn dm_delete(&mut self, id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = client.delete_dm(id);
            let _ = tx.send(Msg::DmList(client.dm_conversations()));
        });
    }

    /// 在他人主页发起私信（找到或创建会话后进入聊天）
    pub fn start_dm_with(&mut self, peer_id: u64) {
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能发私信".to_owned());
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.create_dm(peer_id);
            let _ = tx.send(Msg::DmCreated(r));
        });
    }

    // ---------- 更多页面（文章 / 创作中心 / 动态 / 专题 / 账本 / 治理） ----------

    pub fn open_more(&mut self, route: Route) {
        self.more = MoreData::default();
        self.more_state = LoadState::Loading;
        self.f_title.clear();
        self.f_body.clear();
        self.f_text.clear();
        self.navigate(route);
        self.load_more();
    }

    /// 按当前路由拉这一页需要的数据（线程里跑，回包统一走 Msg::More）
    pub fn load_more(&mut self) {
        let route = self.route;
        self.more_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let mut d = MoreData::default();
            let r = (|| -> Result<(), String> {
                match route {
                    Route::ArticleList => d.articles = client.articles(30)?,
                    Route::Article(id) => {
                        d.article = Some(client.article(id)?);
                        d.article_comments = client.article_comments(id).unwrap_or_default();
                        let _ = client.post_article_view(id);
                        d.editing = id;
                    }
                    Route::MyArticles => d.my_articles = client.my_articles()?,
                    Route::Creator => {
                        d.stats = client.creator_stats().ok();
                        d.videos = client.my_videos()?;
                    }
                    Route::CreatorComments => d.creator_comments = client.creator_comments()?,
                    Route::CreatorDanmakus => d.danmakus = client.creator_danmakus()?,
                    Route::DynamicDetail(id) => {
                        d.dynamic = Some(client.user_dynamic(id)?);
                        d.dynamic_comments = client.dynamic_comments(id).unwrap_or_default();
                    }
                    Route::Specials => d.specials = client.specials()?,
                    Route::Ledger => {
                        d.ledger = Some(client.coin_ledger("month", 50)?);
                        d.rewards = client.daily_rewards().ok();
                    }
                    Route::LiveHost => d.live = Some(client.my_live_room()?),
                    Route::VideoEdit(id) => {
                        d.video = Some(client.video_detail(id)?);
                        d.chapters = client.chapters(id).unwrap_or_default();
                    }
                    Route::Governance => {
                        d.tickets = client.my_tickets().unwrap_or_default();
                        d.appeals = client.my_appeals().unwrap_or_default();
                        d.cs = client.my_cs_conversations().unwrap_or_default();
                    }
                    _ => {}
                }
                Ok(())
            })();
            if let Err(e) = r {
                d.err = Some(e);
            }
            let _ = tx.send(Msg::More(Box::new(d)));
        });
    }

    /// 这些页面的写操作：成功后原地重拉
    pub fn more_action<F>(&mut self, f: F)
    where
        F: FnOnce(&api::Client) -> Result<(), String> + Send + 'static,
    {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = f(&client);
            let _ = tx.send(Msg::MoreDone(r));
        });
    }

    // ---------- 投稿上传 ----------

    /// 稿件收件目录：安卓放在 App 私有外部目录，adb push 或文件管理器都能放进去
    pub fn inbox_dir() -> String {
        std::env::var("CAKECAKE_INBOX").unwrap_or_else(|_| {
            "/sdcard/Android/data/com.cakecake.rust/files/inbox".to_owned()
        })
    }

    pub fn open_upload(&mut self) {
        if self.up_state == LoadState::Loading {
            return;
        }
        self.up_msg = None;
        self.f_title.clear();
        self.f_body.clear();
        self.up_path = None;
        self.scan_inbox();
        self.navigate(Route::Upload);
    }

    /// 扫一遍收件目录里的视频文件
    pub fn scan_inbox(&mut self) {
        let dir = Self::inbox_dir();
        let _ = std::fs::create_dir_all(&dir);
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                let ext = p
                    .extension()
                    .map(|x| x.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if !matches!(ext.as_str(), "mp4" | "mov" | "mkv" | "webm") {
                    continue;
                }
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                out.push((p.to_string_lossy().to_string(), size));
            }
        }
        out.sort();
        self.inbox = out;
    }

    pub fn do_upload(&mut self) {
        let Some(path) = self.up_path.clone() else {
            self.up_msg = Some("先选一个视频文件".to_owned());
            return;
        };
        let title = self.f_title.trim().to_owned();
        if title.is_empty() {
            self.up_msg = Some("标题不能为空".to_owned());
            return;
        }
        if !self.client.logged_in() {
            self.up_msg = Some("登录后才能投稿".to_owned());
            return;
        }
        self.up_state = LoadState::Loading;
        self.up_msg = Some("上传中，别退出…".to_owned());
        let desc = self.f_body.clone();
        let zone = self.up_zone.clone();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.upload_video(&path, &title, &desc, &zone);
            let _ = tx.send(Msg::Uploaded(r));
        });
    }

    /// 打开稿件编辑（只对自己投稿开放，后端 PUT 时还会校验）
    pub fn open_video_edit(&mut self, id: u64) {
        self.f_tags.clear();
        self.f_schedule.clear();
        self.open_more(Route::VideoEdit(id));
    }

    /// 把已发布文章的正文拉回来填进编辑器
    pub fn edit_article(&mut self, id: u64) {
        self.more.editing = id;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.my_article(id);
            let _ = tx.send(Msg::ArticleEditor(r));
        });
    }

    pub fn open_dynamic(&mut self, id: u64) {
        self.open_more(Route::DynamicDetail(id));
    }

    /// 文章列表进详情
    pub fn open_article(&mut self, id: u64) {
        self.open_more(Route::Article(id));
    }

    // ---------- 首页运营位 ----------

    /// 轮播图 + 统计 + 真实分区，一次拉齐（统计/分区拿不到不影响轮播图）
    pub fn load_home_ops(&mut self) {
        if self.home_ops_state == LoadState::Loading {
            return;
        }
        self.home_ops_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                let banners = client.banners()?;
                let stats = client.home_stats().ok();
                let zones = client.zones().unwrap_or_default();
                Ok(HomeOps {
                    banners,
                    stats,
                    zones,
                })
            })();
            let _ = tx.send(Msg::HomeOps(r));
        });
    }

    /// `/zones` 返回的是「父-子」粒度，这里按父分区汇总视频数（顺序按总量倒序）
    pub fn zone_parents(&self) -> Vec<(String, u64)> {
        let mut out: Vec<(String, u64)> = Vec::new();
        for z in &self.home_ops.zones {
            let parent = z.name.split('-').next().unwrap_or(&z.name).trim();
            if parent.is_empty() {
                continue;
            }
            match out.iter_mut().find(|(n, _)| n == parent) {
                Some((_, c)) => *c += z.video_count,
                None => out.push((parent.to_owned(), z.video_count)),
            }
        }
        out.sort_by(|a, b| b.1.cmp(&a.1));
        out
    }

    /// 某个父分区下真实有视频的子分区
    pub fn zone_children(&self, parent: &str) -> Vec<(String, u64)> {
        self.home_ops
            .zones
            .iter()
            .filter_map(|z| {
                let (p, c) = z.name.split_once('-')?;
                if p.trim() != parent {
                    return None;
                }
                Some((c.trim().to_owned(), z.video_count))
            })
            .collect()
    }

    // ---------- 排行榜 ----------

    pub fn open_rank(&mut self) {
        self.rank_state = LoadState::Idle;
        self.navigate(Route::Rank);
    }

    pub fn load_rank(&mut self) {
        if self.rank_state == LoadState::Loading {
            return;
        }
        self.rank_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let by = self.rank_by.clone();
        let period = self.rank_period.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Rank(client.leaderboard(&by, &period)));
        });
    }

    /// 切榜单维度：by = play/coin/like/fav，period = all/week/month
    pub fn set_rank(&mut self, by: Option<&str>, period: Option<&str>) {
        let mut changed = false;
        if let Some(by) = by {
            if self.rank_by != by {
                self.rank_by = by.to_owned();
                changed = true;
            }
        }
        if let Some(period) = period {
            if self.rank_period != period {
                self.rank_period = period.to_owned();
                changed = true;
            }
        }
        if changed {
            self.rank.clear();
            self.rank_state = LoadState::Idle;
            self.load_rank();
        }
    }

    // ---------- 搜索联想 ----------

    /// 输入变化时调用；同一个关键词不重复请求
    pub fn request_suggest(&mut self) {
        let term = self.search_kw.trim().to_owned();
        if term.is_empty() {
            self.suggest.clear();
            self.suggest_for.clear();
            return;
        }
        if self.suggest_for == term {
            return;
        }
        self.suggest_for = term.clone();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.search_suggest(&term, 8);
            let _ = tx.send(Msg::Suggest(term, r));
        });
    }

    // ---------- 他人主页 ----------

    pub fn open_space(&mut self, user_id: u64) {
        if user_id == 0 {
            return;
        }
        if self.route == Route::Space(user_id) {
            return;
        }
        self.navigate(Route::Space(user_id));
        self.space = None;
        self.space_state = LoadState::Loading;
        self.space_tab = SpaceTab::Videos;
        self.spawn_space(user_id);
    }

    fn spawn_space(&mut self, user_id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = (|| {
                // 资料是硬门槛（被拉黑会 403），其余子资源失败就当空列表
                let profile = client.user_profile(user_id)?;
                let videos = client.user_videos(user_id).unwrap_or_default();
                let articles = client.user_articles(user_id).unwrap_or_default();
                let dynamics = client.user_dynamics(user_id).unwrap_or_default();
                let favorites = client.user_favorites(user_id).unwrap_or_default();
                let following = client.user_following(user_id).unwrap_or_default();
                let followers = client.user_followers(user_id).unwrap_or_default();
                Ok(SpaceData {
                    profile,
                    videos,
                    articles,
                    dynamics,
                    favorites,
                    following,
                    followers,
                })
            })();
            let _ = tx.send(Msg::Space(user_id, r));
        });
    }

    /// 失败后点「重试」：强制重拉当前主页
    pub fn reload_space(&mut self) {
        let Route::Space(uid) = self.route else {
            return;
        };
        self.space = None;
        self.space_state = LoadState::Loading;
        self.spawn_space(uid);
    }

    /// 在他人主页点「关注 / 已关注」
    pub fn toggle_space_follow(&mut self) {
        let Route::Space(uid) = self.route else {
            return;
        };
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能关注，先去「我的」登录".to_owned());
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.toggle_follow(uid);
            let _ = tx.send(Msg::SpaceFollow(uid, r));
        });
    }

    // ---------- 未读角标 ----------

    pub fn load_unread(&mut self) {
        if !self.client.logged_in() {
            self.unread = api::UnreadSummary::default();
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Unread(client.unread_summary()));
        });
    }

    // ---------- 播放选项（清晰度 / 字幕） ----------

    /// 切清晰度：换播放地址并从当前进度续播（切完回 0 秒体验很差）
    pub fn select_quality(&mut self, video_id: u64, bitrate: &api::VideoBitrate) {
        let url = self.client.abs(&bitrate.url);
        if url.is_empty() {
            return;
        }
        let pos = self.video.position_ms().max(0);
        self.quality = Some(bitrate.id);
        self.video.play(&url);
        if pos > 500 {
            self.resume = Some((video_id, pos as f64 / 1000.0));
        }
    }

    /// 选字幕轨道（id = None 表示关闭）；列表接口已带 content，不用再请求
    pub fn select_subtitle(&mut self, subs: &[api::Subtitle], id: Option<u64>) {
        self.subtitle_id = id;
        self.cues.clear();
        let Some(id) = id else { return };
        if let Some(s) = subs.iter().find(|s| s.id == id) {
            self.cues = crate::subtitle::parse(&s.content);
        }
    }

    // ---------- 观看历史 ----------

    pub fn open_history(&mut self) {
        self.history_state = LoadState::Idle;
        self.action_msg = None;
        self.navigate(Route::History);
    }

    pub fn load_history(&mut self) {
        if !self.client.logged_in() {
            self.history_state = LoadState::Idle;
            return;
        }
        if self.history_state == LoadState::Loading {
            return;
        }
        self.history_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::History(client.view_history()));
        });
    }

    pub fn delete_history(&mut self, media_type: &str, id: u64) {
        if !self.client.logged_in() {
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let kind = media_type.to_owned();
        std::thread::spawn(move || {
            let r = match kind.as_str() {
                "live" => client.delete_live_history_entry(id),
                "article" => client.delete_article_history_entry(id),
                _ => client.delete_view_history_entry(id),
            };
            let _ = tx.send(Msg::HistoryAction(r));
        });
    }

    pub fn clear_history(&mut self) {
        if !self.client.logged_in() {
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::HistoryAction(client.clear_view_history()));
        });
    }

    /// 开关「记录观看历史」
    pub fn set_history_paused(&mut self, paused: bool) {
        if !self.client.logged_in() {
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::HistoryAction(client.set_view_history_paused(paused)));
        });
    }

    /// 把当前进度报给后端（断点续播用）；force=true 时不看 20 秒限流
    fn report_progress(&mut self, force: bool) {
        let Route::Detail(id) = self.route else {
            self.progress_at = None;
            return;
        };
        if !self.client.logged_in() {
            return;
        }
        if !matches!(
            self.video.phase(),
            crate::video::Phase::Playing | crate::video::Phase::Paused
        ) {
            return;
        }
        let pos = self.video.position_ms();
        let dur = self.video.duration_ms();
        if pos <= 0 || dur <= 0 {
            return;
        }
        if !force {
            if let Some(t) = self.progress_at {
                if t.elapsed().as_secs() < 20 {
                    return;
                }
            }
        }
        self.progress_at = Some(Instant::now());
        let client = self.client.clone();
        std::thread::spawn(move || {
            eprintln!("[history] save progress {id} at {pos} / {dur} ms");
            let _ = client.record_view_history(id, pos as f64 / 1000.0, dur as f64 / 1000.0);
        });
    }

    /// 拿到续播点后，等播放器真正开播再 seek（prepare 前 seek 会被系统忽略）
    fn apply_resume(&mut self) {
        let Some((id, sec)) = self.resume else {
            return;
        };
        if self.route != Route::Detail(id) {
            self.resume = None;
            return;
        }
        if self.video.duration_ms() <= 0 || self.video.phase() != crate::video::Phase::Playing {
            return;
        }
        self.video.seek_to((sec * 1000.0) as i64);
        eprintln!("[history] resume {id} to {sec}s");
        self.resume = None;
    }

    pub fn open_notifications(&mut self) {
        self.notifications_state = LoadState::Idle;
        self.notify_msg = None;
        self.load_unread();
        self.navigate(Route::Notifications);
    }

    pub fn load_notifications(&mut self) {
        if !self.client.logged_in() {
            self.notifications_state = LoadState::Idle;
            return;
        }
        if self.notifications_state == LoadState::Loading {
            return;
        }
        self.notifications_state = LoadState::Loading;
        let client = self.client.clone();
        let cat = self.notification_cat.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Notifications(client.notifications(&cat)));
        });
    }

    pub fn select_notification_cat(&mut self, cat: &str) {
        if self.notification_cat == cat {
            return;
        }
        self.notification_cat = cat.to_owned();
        self.notifications.clear();
        self.notifications_state = LoadState::Idle;
        self.notify_msg = None;
        self.load_notifications();
    }

    pub fn mark_all_notifications_read(&mut self) {
        if !self.client.logged_in() {
            return;
        }
        self.notify_msg = None;
        let client = self.client.clone();
        let cat = self.notification_cat.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.mark_notifications_read(&cat);
            let _ = tx.send(Msg::NotifAction(r));
        });
    }

    pub fn mark_notification_read(&mut self, id: u64) {
        if !self.client.logged_in() {
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.mark_notification_read(id);
            let _ = tx.send(Msg::NotifAction(r));
        });
    }

    pub fn delete_notification(&mut self, id: u64) {
        if !self.client.logged_in() {
            return;
        }
        self.notify_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.delete_notification(id);
            let _ = tx.send(Msg::NotifAction(r));
        });
    }

    /// 评论点赞：成功后静默刷新详情（评论计数由后端算，不在本地造）
    pub fn toggle_comment_like(&mut self, video_id: u64, comment_id: u64) {
        if comment_id == 0 {
            return;
        }
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能点赞，先去「我的」登录".to_owned());
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.toggle_comment_like(comment_id);
            let _ = tx.send(Msg::CommentLike(video_id, r));
        });
    }

    /// 踩评论（和点赞互斥，后端自己保证）
    pub fn toggle_comment_dislike(&mut self, video_id: u64, comment_id: u64) {
        if comment_id == 0 {
            return;
        }
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能点踩，先去「我的」登录".to_owned());
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.toggle_comment_dislike(comment_id);
            let _ = tx.send(Msg::CommentDislike(video_id, r));
        });
    }

    /// 评论管理：delete / pin / approve / ignore / report（后端按身份校验权限）
    pub fn comment_action(&mut self, video_id: u64, comment_id: u64, action: &str) {
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能操作".to_owned());
            return;
        }
        self.comment_menu = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let action = action.to_owned();
        std::thread::spawn(move || {
            let r = match action.as_str() {
                "delete" => client.delete_comment(comment_id),
                "pin" => client.pin_comment(comment_id),
                "approve" => client.approve_comment(comment_id),
                "ignore" => client.ignore_curated_comment(comment_id),
                "report" => client.report_comment(comment_id, "other", "App 内举报"),
                _ => Ok(()),
            };
            let _ = tx.send(Msg::CommentAction(video_id, r));
        });
    }

    /// 进「评论详情」：一级评论 + 它下面的全部回复
    pub fn open_comment_detail(&mut self, video_id: u64, comment_id: u64) {
        if video_id == 0 || comment_id == 0 {
            return;
        }
        self.highlight_comment = None;
        self.navigate(Route::CommentDetail {
            video_id,
            comment_id,
        });
    }

    /// 评论排序：hot（按热度）/ latest（按时间）
    pub fn set_comment_sort(&mut self, sort: &str) {
        if self.comment_sort != sort {
            self.comment_sort = sort.to_owned();
        }
    }

    /// 发表评论：回复目标由 `reply_to` 决定，成功后静默刷新详情（重拉评论树）
    pub fn post_comment(&mut self, video_id: u64) {
        let text = self.comment_text.trim().to_owned();
        if text.is_empty() {
            self.action_msg = Some("评论不能为空".to_owned());
            return;
        }
        if !self.client.logged_in() {
            self.action_msg = Some("登录后才能评论，先去「我的」登录".to_owned());
            return;
        }
        let parent_id = self.reply_to.as_ref().map(|(id, _)| *id).unwrap_or(0);
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.post_comment(video_id, &text, parent_id);
            let _ = tx.send(Msg::CommentPosted(video_id, r));
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
                            urls.extend(data.comments.iter().map(|c| c.avatar_url.clone()));
                            self.covers.request(&urls);
                            // 只在没播完的情况下续播（看到结尾就当从头开始）
                            if let Some(sec) = data.resume_sec {
                                let total = data.video.duration as f64;
                                if sec > 1.0 && (total <= 0.0 || sec + 5.0 < total) {
                                    self.resume = Some((id, sec));
                                }
                            }
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
                    Ok(me) => {
                        self.me = Some(me);
                        self.login_pass.clear();
                        self.auth_state = LoadState::Ready;
                        self.load_unread();
                    }
                    Err(e) => self.auth_state = LoadState::Err(e),
                },
                Msg::Register(r) => match r {
                    Ok(me) => {
                        self.me = Some(me);
                        self.reg_pass.clear();
                        self.login_user = self.reg_user.clone();
                        self.reg_state = LoadState::Ready;
                        self.auth_state = LoadState::Ready;
                        self.show_register = false;
                        self.load_unread();
                    }
                    Err(e) => self.reg_state = LoadState::Err(e),
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
                Msg::WatchLater(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> =
                            list.iter().map(|v| v.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.watch_later = list;
                        self.watch_later_state = LoadState::Ready;
                    }
                    Err(e) => self.watch_later_state = LoadState::Err(e),
                },
                Msg::WatchLaterAction(r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        if matches!(self.route, Route::WatchLater) {
                            self.watch_later_state = LoadState::Idle;
                            self.load_watch_later();
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::Notifications(r) => match r {
                    Ok(list) => {
                        let mut urls = Vec::new();
                        for n in &list {
                            urls.extend(n.sender_avatar_urls.clone());
                        }
                        self.covers.request(&urls);
                        self.notifications = list;
                        self.notifications_state = LoadState::Ready;
                    }
                    Err(e) => self.notifications_state = LoadState::Err(e),
                },
                Msg::NotifAction(r) => match r {
                    Ok(()) => {
                        self.notify_msg = None;
                        self.load_unread();
                        if matches!(self.route, Route::Notifications) {
                            self.notifications_state = LoadState::Idle;
                            self.load_notifications();
                        }
                    }
                    Err(e) => self.notify_msg = Some(e),
                },
                Msg::History(r) => match r {
                    Ok(page) => {
                        let urls: Vec<String> =
                            page.items.iter().map(|v| v.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.history_paused = page.paused;
                        self.view_history = page.items;
                        self.history_state = LoadState::Ready;
                    }
                    Err(e) => self.history_state = LoadState::Err(e),
                },
                Msg::HistoryAction(r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        if matches!(self.route, Route::History) {
                            self.history_state = LoadState::Idle;
                            self.load_history();
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::CommentLike(video_id, r) => match r {
                    Ok(_) => {
                        self.action_msg = None;
                        if self.on_video_route(video_id) {
                            self.refresh_detail(video_id);
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::CommentDislike(video_id, r) => match r {
                    Ok(_) => {
                        self.action_msg = None;
                        if self.on_video_route(video_id) {
                            self.refresh_detail(video_id);
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::CommentAction(video_id, r) => match r {
                    Ok(()) => {
                        self.action_msg = Some("操作成功".to_owned());
                        if self.on_video_route(video_id) {
                            self.refresh_detail(video_id);
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::CommentPosted(video_id, r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        self.comment_text.clear();
                        self.reply_to = None;
                        if self.on_video_route(video_id) {
                            self.refresh_detail(video_id);
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::Picker(id, r) => {
                    if self.picker_video != id || self.picker_state != LoadState::Loading {
                        continue;
                    }
                    match r {
                        Ok(p) => {
                            let urls: Vec<String> = p
                                .items
                                .iter()
                                .filter_map(|f| f.cover_url.clone())
                                .collect();
                            self.covers.request(&urls);
                            self.picker_selected = p.folder_ids.clone();
                            self.favorite_picker = Some(p);
                            self.picker_state = LoadState::Ready;
                        }
                        Err(e) => self.picker_state = LoadState::Err(e),
                    }
                }
                Msg::PickerSave(r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        self.close_favorite_picker();
                        if let Route::Detail(id) = self.route {
                            self.refresh_detail(id);
                        }
                        self.favorites_state = LoadState::Idle;
                    }
                    Err(e) => {
                        self.picker_state = LoadState::Ready;
                        self.action_msg = Some(e);
                    }
                },
                Msg::Favorites(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> = list.iter().map(|v| v.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.favorites = list;
                        self.favorites_state = LoadState::Ready;
                    }
                    Err(e) => self.favorites_state = LoadState::Err(e),
                },
                Msg::Folders(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> =
                            list.iter().filter_map(|f| f.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.favorite_folders = list;
                        self.folders_state = LoadState::Ready;
                    }
                    Err(e) => self.folders_state = LoadState::Err(e),
                },
                Msg::FolderAction(r) => match r {
                    Ok(()) => {
                        self.folder_msg = None;
                        self.folders_state = LoadState::Idle;
                        self.favorites_state = LoadState::Idle;
                        if matches!(self.route, Route::Favorites) {
                            self.load_folders();
                            self.load_favorites();
                        }
                        if self.favorite_picker.is_some() {
                            self.open_favorite_picker(self.picker_video);
                        }
                    }
                    Err(e) => self.folder_msg = Some(e),
                },
                Msg::HomeOps(r) => match r {
                    Ok(ops) => {
                        let urls: Vec<String> = ops.banners.iter().map(|b| b.pic.clone()).collect();
                        self.covers.request(&urls);
                        self.home_ops = ops;
                        self.home_ops_state = LoadState::Ready;
                    }
                    Err(e) => self.home_ops_state = LoadState::Err(e),
                },
                Msg::Rank(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> = list.iter().map(|v| v.cover_url.clone()).collect();
                        self.covers.request(&urls);
                        self.rank = list;
                        self.rank_state = LoadState::Ready;
                    }
                    Err(e) => self.rank_state = LoadState::Err(e),
                },
                Msg::Suggest(term, r) => {
                    // 只收当前关键词的回包，否则打完字会被上一个词的慢回包盖掉
                    if self.suggest_for != term {
                        continue;
                    }
                    match r {
                        Ok(list) => self.suggest = list,
                        // 联想词失败不打扰用户，清空即可
                        Err(_) => self.suggest.clear(),
                    }
                }
                Msg::Space(uid, r) => {
                    if self.route != Route::Space(uid) {
                        continue;
                    }
                    match r {
                        Ok(data) => {
                            let mut urls = vec![data.profile.avatar_url.clone()];
                            urls.extend(data.videos.iter().map(|v| v.cover_url.clone()));
                            urls.extend(data.articles.iter().map(|a| a.cover_url.clone()));
                            urls.extend(data.dynamics.iter().flat_map(|d| d.images.clone()));
                            urls.extend(data.favorites.iter().map(|f| f.cover_url.clone()));
                            urls.extend(data.following.iter().map(|u| u.avatar_url.clone()));
                            urls.extend(data.followers.iter().map(|u| u.avatar_url.clone()));
                            self.covers.request(&urls);
                            self.space = Some(data);
                            self.space_state = LoadState::Ready;
                        }
                        Err(e) => self.space_state = LoadState::Err(e),
                    }
                }
                Msg::SpaceFollow(uid, r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        if let Some(space) = self.space.as_mut() {
                            // 后端是 toggle 语义，本地同步翻转，不用再拉一遍整页
                            if space.profile.user_id == uid {
                                space.profile.followed_by_me = !space.profile.followed_by_me;
                                if space.profile.followed_by_me {
                                    space.profile.follower_count += 1;
                                } else {
                                    space.profile.follower_count =
                                        space.profile.follower_count.saturating_sub(1);
                                }
                            }
                        }
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::Unread(r) => {
                    if let Ok(u) = r {
                        self.unread = u;
                    }
                }
                Msg::Server(url) => {
                    if url != self.client.base {
                        self.switch_server(url);
                    }
                }
                Msg::Me(r) => {
                    if let Ok(me) = r {
                        self.me = Some(me);
                    }
                }
                Msg::SettingsDone(r) => {
                    self.settings_state = LoadState::Ready;
                    match r {
                        Ok(()) => {
                            self.settings_msg = Some("已保存".to_owned());
                            self.set_old_pw.clear();
                            self.set_new_pw.clear();
                            self.set_del_pw.clear();
                            self.reload_me();
                        }
                        Err(e) => self.settings_msg = Some(e),
                    }
                }
                Msg::Privacy(r) => match r {
                    Ok(p) => self.privacy = Some(p),
                    Err(e) => self.settings_msg = Some(e),
                },
                Msg::Groups(r) => match r {
                    Ok(g) => self.groups = g,
                    Err(e) => self.settings_msg = Some(e),
                },
                Msg::DmList(r) => match r {
                    Ok(list) => {
                        let urls: Vec<String> = list.iter().map(|c| c.peer_avatar.clone()).collect();
                        self.covers.request(&urls);
                        self.dm_convs = list;
                        self.dm_state = LoadState::Ready;
                    }
                    Err(e) => self.dm_state = LoadState::Err(e),
                },
                Msg::DmMessages(id, r) => {
                    if self.route != Route::DmChat(id) {
                        continue;
                    }
                    match r {
                        Ok(list) => {
                            self.dm_chat = list;
                            self.dm_chat_state = LoadState::Ready;
                        }
                        Err(e) => self.dm_chat_state = LoadState::Err(e),
                    }
                }
                Msg::DmSent(id, r) => match r {
                    Ok(()) => {
                        self.action_msg = None;
                        self.load_dm_chat(id);
                    }
                    Err(e) => self.action_msg = Some(e),
                },
                Msg::More(d) => {
                    self.more_state = match &d.err {
                        Some(e) => LoadState::Err(e.clone()),
                        None => LoadState::Ready,
                    };
                    self.more = *d;
                }
                Msg::ArticleEditor(r) => match r {
                    Ok(a) => {
                        self.more.editing = a.id;
                        self.f_title = a.title.clone();
                        self.f_body = a.body_md.clone();
                    }
                    Err(e) => self.more.err = Some(e),
                },
                Msg::Uploaded(r) => {
                    self.up_state = LoadState::Ready;
                    match r {
                        Ok(id) => {
                            self.up_msg = Some(format!("上传成功，稿件 id {id}"));
                            self.f_title.clear();
                            self.f_body.clear();
                            self.up_path = None;
                            self.open_more(Route::Creator);
                        }
                        Err(e) => self.up_msg = Some(e),
                    }
                }
                Msg::MoreDone(r) => match r {
                    Ok(()) => {
                        self.more_state = LoadState::Idle;
                        self.load_more();
                    }
                    Err(e) => self.more_state = LoadState::Err(e),
                },
                Msg::DmCreated(r) => match r {
                    Ok(conv) => {
                        self.dm_state = LoadState::Idle;
                        self.open_dm_chat(&conv);
                    }
                    Err(e) => self.action_msg = Some(e),
                },
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frames += 1;
        // 每帧重置：页面在绘制时通过 note_focus 上报，帧末统一下发一次 IMEAllowed
        self.ime_wanted = false;
        self.poll();
        self.covers.poll(ui.ctx());


        if self.me.is_some() && !self.client.logged_in() {
            // token 过期且 refresh 失败：Client 已清空登录态，这里同步回登录页
            self.logout();
            self.auth_state = LoadState::Err("登录已过期，请重新登录".to_owned());
        }

        if self.videos_state == LoadState::Idle && matches!(self.route, Route::Tab(Tab::Home)) {
            self.load_videos();
        }
        if self.home_ops_state == LoadState::Idle && matches!(self.route, Route::Tab(Tab::Home)) {
            self.load_home_ops();
        }
        if self.rank_state == LoadState::Idle && matches!(self.route, Route::Rank) {
            self.load_rank();
        }
        if self.dm_state == LoadState::Idle && matches!(self.route, Route::Dm) {
            self.load_dm();
        }
        if self.live_state == LoadState::Idle && matches!(self.route, Route::Tab(Tab::Live)) {
            self.load_live();
        }
        if self.dynamic_state == LoadState::Idle
            && matches!(self.route, Route::Tab(Tab::Dynamic))
            && self.client.logged_in()
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
        if self.watch_later_state == LoadState::Idle
            && matches!(self.route, Route::WatchLater)
            && self.client.logged_in()
        {
            self.load_watch_later();
        }
        if self.notifications_state == LoadState::Idle
            && matches!(self.route, Route::Notifications)
            && self.client.logged_in()
        {
            self.load_notifications();
        }
        if self.history_state == LoadState::Idle
            && matches!(self.route, Route::History)
            && self.client.logged_in()
        {
            self.load_history();
        }
        if self.favorites_state == LoadState::Idle
            && matches!(self.route, Route::Favorites)
            && self.client.logged_in()
        {
            self.load_favorites();
        }
        if self.folders_state == LoadState::Idle
            && matches!(self.route, Route::Favorites)
            && self.client.logged_in()
        {
            self.load_folders();
        }
        if self.frames == 2 {
            self.probe_servers();
        }
        self.apply_resume();
        self.report_progress(false);

        // 安卓三段式导航栏的返回键（winit 映射成 BrowserBack）与桌面 Esc：
        // 二级页回上一页，根页面退出 App（否则返回键会被 winit 吃掉、什么都不发生）
        let wants_kb = ui.ctx().egui_wants_keyboard_input();
        // Java 壳转过来的系统返回键（三段式导航/手势返回）
        let hw_back = crate::video::take_back_request();
        let raw_back = ui.input(|i| i.key_pressed(egui::Key::BrowserBack)) || hw_back;
        let esc_back = ui.input(|i| i.key_pressed(egui::Key::Escape)) && !wants_kb;
        if raw_back && wants_kb {
            // 正在输入（评论/弹幕/搜索）：返回键先收键盘并取消焦点，不退出页面
            if let Some(id) = ui.ctx().memory(|m| m.focused()) {
                ui.ctx().memory_mut(|m| m.surrender_focus(id));
            }
            self.close_ime();
        } else if raw_back || esc_back {
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
                Some("rank") => self.open_rank(),
                Some("zones") => self.navigate(Route::Zones),
                Some("space") => {
                    let id = std::env::var("APP_SPACE_ID")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(1);
                    self.open_space(id);
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

        self.bottom_inset = bottom_inset;
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
                    Route::WatchLater => pages::watch_later::show(ui, self),
                    Route::History => pages::history::show(ui, self),
                    Route::Notifications => pages::notifications::show(ui, self),
                    Route::Favorites => pages::favorites::show(ui, self),
                    Route::Space(id) => pages::space::show(ui, self, id),
                    Route::Rank => pages::rank::show(ui, self),
                    Route::CommentDetail {
                        video_id,
                        comment_id,
                    } => pages::comment_detail::show(ui, self, video_id, comment_id),
                    Route::Settings => pages::settings::show(ui, self),
                    Route::Dm => pages::dm::show(ui, self),
                    Route::DmChat(id) => pages::dm::chat(ui, self, id),
                    Route::ArticleList
                    | Route::Article(_)
                    | Route::MyArticles
                    | Route::Creator
                    | Route::CreatorComments
                    | Route::CreatorDanmakus
                    | Route::DynamicDetail(_)
                    | Route::Specials
                    | Route::Ledger
                    | Route::Governance => pages::more::show(ui, self),
                    Route::LiveHost => pages::more::live_host(ui, self),
                    Route::VideoEdit(id) => pages::more::video_edit(ui, self, id),
                    Route::Upload => pages::more::upload(ui, self),
                }
            });

        // 页面都画完了、焦点也上报完了，这里统一决定软键盘开合（放帧末才不会读到空值）
        if self.ime_on != self.ime_wanted {
            self.ime_on = self.ime_wanted;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::IMEAllowed(self.ime_on));
        }

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
