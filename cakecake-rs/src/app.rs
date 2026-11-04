//! 应用外壳：路由、底部导航、异步加载
//!
//! 原则：页面上的每个数字都来自接口；接口没有的东西就不显示。
//! 加载失败只显示错误与重试，不用假数据兜底。

use crate::{
    api::{self, Comment, Danmaku, LiveRoom, Me, Video, VideoHit},
    images::Covers,
    pages, theme, video::Player, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

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
    /// 我的图文收藏
    ArticleFavorites,
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
    /// 申诉详情
    Appeal(u64),
    /// 工单详情
    Ticket(u64),
    /// 客服会话
    CsChat(u64),
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
    pub article_favorites: Vec<api::ArticleCard>,
    pub dynamic: Option<api::DynItem>,
    pub dynamic_comments: Vec<api::Comment>,
    pub videos: Vec<api::MyVideo>,
    pub stats: Option<api::CreatorStats>,
    pub video_stats: Vec<api::CreatorVideoStat>,
    pub creator_comments: Vec<api::Comment>,
    pub danmakus: Vec<api::CreatorDanmaku>,
    pub specials: Vec<api::SpecialPage>,
    pub ledger: Option<api::CoinLedgerPage>,
    pub rewards: Option<api::DailyRewards>,
    pub tickets: Vec<api::Ticket>,
    pub ticket_detail: Option<api::TicketDetail>,
    pub appeals: Vec<api::Appeal>,
    pub appeal: Option<api::Appeal>,
    pub cs: Vec<api::CsConversation>,
    pub cs_detail: Option<api::CsConversationDetail>,
    pub copyrights: Vec<api::CopyrightComplaint>,
    /// 当前正在编辑的文章 id（0 = 新建）
    pub editing: u64,
    /// 我的直播间
    pub live: Option<api::LiveRoom>,
    /// 稿件编辑用的视频详情与章节
    pub video: Option<api::VideoDetail>,
    pub chapters: Vec<api::Chapter>,
    /// 稿件字幕轨道
    pub subtitles: Vec<api::Subtitle>,
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
    RecentCoins,
    FavoriteFolders,
    ArticleFavorites,
    Following,
    Followers,
}

/// 他人主页一次拉齐的数据（资料 + 九个子资源）
#[derive(Default)]
pub struct SpaceData {
    pub profile: api::UserProfile,
    pub videos: Vec<api::SpaceVideo>,
    pub articles: Vec<api::ArticleCard>,
    pub dynamics: Vec<api::DynItem>,
    pub favorites: Vec<api::FavoriteItem>,
    pub recent_coins: Vec<api::CoinVideo>,
    pub favorite_folders: Vec<api::FavoriteFolder>,
    pub article_favorites: Vec<api::ArticleCard>,
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

/// 全屏播放时底部弹出的选项面板
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FsMenu {
    Speed,
    Quality,
    Subtitle,
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
    /// 到达时刻。只用于横屏弹幕的飞行动画计时，**不显示**给用户看
    pub at: Instant,
}

/// 直播间聊天状态
#[derive(Default)]
pub struct ChatState {
    pub msgs: Vec<ChatMsg>,
    pub online: u64,
    pub state: LoadState,
    pub input: String,
    /// 服务端在 `user_info` 里回传的自己的昵称，用来给自己的回显去重
    pub me: String,
    /// 自己刚发出、还没收到服务器回显的消息（内容, 时间）
    pending: Vec<(String, Instant)>,
    /// 最近一次发出的内容，用来挡住原生输入框清空之前又把这句镜像回来
    last_sent: Option<(String, Instant)>,
    /// 断线后下一次自动重连的时间
    retry_at: Option<Instant>,
    /// 本轮进房失败过几次（界面上给个「重连」按钮）
    pub tries: u32,
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
    /// 他人主页收藏夹里的视频（user_id, folder_id, 结果）
    SpaceFolder(u64, u64, Result<Vec<api::FavoriteItem>, String>),
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
    /// 直播媒体源探测结果（room_id, 可播放地址或失败明细）
    LiveMedia(u64, Result<String, String>),
    /// 直播「流畅」变体探测结果
    LiveVariant(Option<String>),
    /// 「更多」页面统一回包
    More(Box<MoreData>),
    /// 「更多」页面的写操作结果
    MoreDone(Result<(), String>),
    /// 投稿上传结果（新视频 id）
    Uploaded(Result<u64, String>),
    /// 文章编辑器回填（标题, 正文）
    ArticleEditor(Result<api::ArticleDetail, String>),
    /// 文章保存结果（文章 id, 是否按发布提交, 后端落地状态）
    ArticlePosted(Result<(u64, bool, String), String>),
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
    /// 评论配图（本地路径，空 = 纯文字评论）
    pub comment_image_path: String,
    /// 是否展开评论配图选择
    pub comment_image_open: bool,
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
    /// 直播间媒体源解析：Ready 后才把地址交给播放器（避免把回环地址丢进去无限缓冲）
    pub room_media: Option<String>,
    pub room_media_state: LoadState,
    /// 解析失败时列出尝试过的地址，方便判断是 IP 变了还是没开播
    pub room_media_err: Option<String>,
    /// 直播「流畅」流地址（SRS 转码产物 `<key>_sd.m3u8`；探不到就是 None）
    pub live_sd_url: Option<String>,
    /// 当前播的是不是「流畅」（false = 原画）
    pub live_quality_sd: bool,
    /// 直播间聊天
    pub chat: ChatState,
    /// 聊天输入是否正由安卓原生输入框接管（和评论/弹幕同一套）
    pub chat_ime: bool,
    pub video: Player,

    pub login_user: String,
    pub login_pass: String,
    /// 需要登录的操作一律弹这个窗，不再在页面角落塞一句「先去登录」
    pub login_prompt: bool,
    /// 正在放大查看的图片 URL（点评论配图 / 文章插图时设置）
    pub image_view: Option<String>,
    /// 轻提示（发布成功这类），3 秒自动消失
    pub toast: Option<(String, Instant)>,
    /// 下一帧把「我的文章」页滚到顶部（点列表里的文章进入编辑时用）
    pub scroll_top: bool,
    /// 当前由原生输入法接管的自绘输入框（"art-title" / "art-body" …）
    pub focus_field: Option<String>,
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
    /// 他人主页里正在查看的收藏夹（None = 收起）
    pub space_folder: Option<u64>,
    pub space_folder_items: Vec<api::FavoriteItem>,
    pub space_folder_state: LoadState,
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
    /// 视频是否处于全屏（横屏沉浸）播放
    pub player_fullscreen: bool,
    /// 全屏播放时蒙层控件的自动隐藏截止时刻
    controls_hide_at: Option<Instant>,
    /// 进入全屏时的窗口尺寸：转屏后尺寸变化要重新隐藏系统栏
    fs_view: Option<egui::Vec2>,
    /// 正在拖动进度条时的预览比例（None = 没在拖）
    pub seek_preview: Option<f32>,
    /// 松手后短暂保留的进度显示，等 MediaPlayer 的 seek 真正生效
    seek_hold: Option<(f32, Instant)>,
    /// 播放倍速（1.0 = 原速）
    pub speed: f32,
    /// 评论输入是否已交给安卓原生输入框（原生输入框才能收到中文）
    pub comment_ime: bool,
    /// 全屏弹幕输入是否交给原生输入框
    pub danmaku_ime: bool,
    /// 全屏底部正在展开的选项面板
    pub fs_menu: Option<FsMenu>,
    /// 竖屏详情页是否展开「清晰度/字幕/倍速」设置条
    pub player_settings: bool,
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
    /// 实时弹幕 WS（video_id, 事件流）
    danmaku_ws: Option<(u64, crate::wsclient::Client)>,
    danmaku_ws_retry: Option<Instant>,
    /// 私信推送 WS
    chat_ws: Option<crate::wsclient::Client>,
    chat_ws_retry: Option<Instant>,
    // ---- 「更多」页面 ----
    pub more: MoreData,
    pub more_state: LoadState,
    pub f_title: String,
    pub f_body: String,
    pub f_text: String,
    pub f_tags: String,
    pub f_schedule: String,
    /// 本地图片列表（头像 / 封面 / 替换封面都从这里选）
    pub image_files: Vec<String>,
    pub image_path: String,
    /// 正在重命名的收藏夹 id（0 = 不在改名）
    pub rename_folder: u64,
    pub rename_title: String,
    /// 批量移除收藏时勾选的视频
    pub fav_selected: Vec<u64>,
    /// 批量移动收藏的目标收藏夹
    pub move_to_folder: u64,
    /// 工单补话 / 客服消息输入
    pub ticket_text: String,
    pub cs_text: String,
    /// 版权投诉目标类型（video / article）
    pub copyright_type: String,
    /// 字幕上传：本地文件、语言、标题
    pub subtitle_files: Vec<(String, u64)>,
    pub subtitle_path: String,
    pub subtitle_lang: String,
    pub subtitle_title: String,
    // ---- 投稿 ----
    pub inbox: Vec<(String, u64)>,
    pub up_path: Option<String>,
    /// 投稿页选中视频的首帧缩略图（路径, 纹理, 时长 ms）
    pub up_thumb: Option<(String, egui::TextureHandle, i64)>,
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
            comment_image_path: String::new(),
            comment_image_open: false,
            reply_to: None,
            pending_comment: None,
            highlight_comment: None,
            danmaku_on: true,
            action_msg: None,
            room: None,
            room_state: LoadState::Idle,
            room_media: None,
            room_media_state: LoadState::Idle,
            room_media_err: None,
            live_sd_url: None,
            live_quality_sd: false,
            chat: ChatState::default(),
            chat_ime: false,
            video: Player::new(),
            login_user: String::new(),
            login_pass: String::new(),
            login_prompt: false,
            image_view: None,
            toast: None,
            scroll_top: false,
            focus_field: None,
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
            space_folder: None,
            space_folder_items: Vec::new(),
            space_folder_state: LoadState::Idle,
            unread: api::UnreadSummary::default(),
            comment_sort: "hot".to_owned(),
            comment_menu: None,
            quality: None,
            subtitle_id: None,
            cues: Vec::new(),
            player_fullscreen: false,
            controls_hide_at: None,
            fs_view: None,
            seek_preview: None,
            seek_hold: None,
            speed: 1.0,
            comment_ime: false,
            danmaku_ime: false,
            fs_menu: None,
            player_settings: false,
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
            danmaku_ws: None,
            danmaku_ws_retry: None,
            chat_ws: None,
            chat_ws_retry: None,
            more: MoreData::default(),
            more_state: LoadState::Idle,
            f_title: String::new(),
            f_body: String::new(),
            f_text: String::new(),
            f_tags: String::new(),
            f_schedule: String::new(),
            image_files: Vec::new(),
            image_path: String::new(),
            rename_folder: 0,
            rename_title: String::new(),
            fav_selected: Vec::new(),
            move_to_folder: 0,
            ticket_text: String::new(),
            cs_text: String::new(),
            copyright_type: "video".to_owned(),
            subtitle_files: Vec::new(),
            subtitle_path: String::new(),
            subtitle_lang: "zh-CN".to_owned(),
            subtitle_title: String::new(),
            inbox: Vec::new(),
            up_path: None,
            up_thumb: None,
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
        self.chat.pending.clear();
        self.chat.online = 0;
        self.chat.input.clear();
        self.chat.me.clear();
        self.chat.room_id = room_id;
        self.chat.state = LoadState::Loading;
        self.chat.tries = 0;
        self.chat.retry_at = None;
        self.chat.conn = Some(self.new_chat_conn(room_id));
    }

    fn new_chat_conn(&self, room_id: u64) -> crate::livechat::Conn {
        crate::livechat::connect(
            &self.client.base,
            room_id,
            self.client.access_token().as_deref(),
        )
    }

    pub fn chat_close(&mut self) {
        self.chat.conn = None;
        self.chat.room_id = 0;
        self.chat.state = LoadState::Idle;
        self.chat.retry_at = None;
        if self.chat_ime {
            self.chat_ime = false;
            crate::video::ime_close();
        }
    }

    /// 手动重连聊天（页面上的「重连」按钮）
    pub fn chat_retry(&mut self) {
        if self.chat.room_id == 0 {
            return;
        }
        self.chat.retry_at = None;
        self.chat.tries = 0;
        self.chat.state = LoadState::Loading;
        self.chat.conn = Some(self.new_chat_conn(self.chat.room_id));
    }

    /// 安卓：每帧把原生输入框里的内容镜像到 `chat.input`，并处理键盘「发送」键。
    ///
    /// NativeActivity 的窗口没有输入连接，egui 的 TextEdit 收不到中文，
    /// 所以聊天输入和评论/弹幕一样交给 Java 壳里那个隐藏 EditText。
    pub fn chat_sync_ime(&mut self) {
        if !self.chat_ime {
            return;
        }
        if let Some(t) = crate::video::ime_text() {
            if t != self.chat.input {
                // 刚发出去的那句在原生输入框里可能还没清掉，别又抄回来
                let stale = self
                    .chat
                    .last_sent
                    .as_ref()
                    .map(|(sent, at)| *sent == t && at.elapsed() < Duration::from_millis(900))
                    .unwrap_or(false);
                if !stale {
                    self.chat.input = t;
                }
            }
        }
        if crate::video::ime_action() {
            self.chat_send();
        }
    }

    /// 把 WS 线程攒下的事件搬到 UI 状态里（每帧调一次）
    pub fn chat_poll(&mut self) {
        // 断线自动重连：还停在直播间里就退避重试，别让用户只能重进房间
        if self.chat.conn.is_none() && self.chat.room_id != 0 {
            let due = self
                .chat
                .retry_at
                .map(|t| Instant::now() >= t)
                .unwrap_or(false);
            if due {
                self.chat.retry_at = None;
                self.chat.state = LoadState::Loading;
                self.chat.conn = Some(self.new_chat_conn(self.chat.room_id));
            }
        }
        let Some(conn) = self.chat.conn.as_ref() else {
            return;
        };
        let evts = conn.drain();
        for e in evts {
            match e {
                crate::livechat::Evt::Me(name) => {
                    self.chat.me = name;
                    if self.chat.state == LoadState::Loading {
                        self.chat.state = LoadState::Ready;
                    }
                }
                crate::livechat::Evt::Msg { user, content } => {
                    self.chat.state = LoadState::Ready;
                    // 自己发的消息服务器会广播回来：本地已经回显过，别重复
                    let mine = !self.chat.me.is_empty() && user == self.chat.me;
                    if let Some(pos) = self
                        .chat
                        .pending
                        .iter()
                        .position(|(t, _)| *t == content)
                    {
                        if mine {
                            self.chat.pending.remove(pos);
                            continue;
                        }
                    }
                    self.chat.msgs.push(ChatMsg {
                        user,
                        content,
                        system: false,
                        at: Instant::now(),
                    });
                }
                crate::livechat::Evt::System(text) => {
                    self.chat.msgs.push(ChatMsg {
                        user: String::new(),
                        content: text,
                        system: true,
                        at: Instant::now(),
                    });
                    if self.chat.state == LoadState::Loading {
                        self.chat.state = LoadState::Ready;
                    }
                }
                crate::livechat::Evt::Count(n) => self.chat.online = n,
                crate::livechat::Evt::Closed(e) => {
                    self.chat.state = LoadState::Err(e);
                    self.chat.conn = None;
                    self.chat.tries = self.chat.tries.saturating_add(1);
                    // 1s、2s、4s…最多 8s，避免服务端刚重启时把连接打满
                    let backoff = 1u64 << self.chat.tries.min(3);
                    self.chat.retry_at = Some(Instant::now() + Duration::from_secs(backoff));
                    break;
                }
            }
        }
        // 迟迟没等到服务器回显的本地消息不再参与去重
        self.chat
            .pending
            .retain(|(_, at)| at.elapsed() < Duration::from_secs(20));
        // 只留最近 200 条，别无限涨
        let len = self.chat.msgs.len();
        if len > 200 {
            self.chat.msgs.drain(..len - 200);
        }
    }

    /// 往聊天里插一条系统提示（进房提示、直播间标题这类，不算谁的发言）
    pub fn chat_system(&mut self, text: impl Into<String>) {
        let content = text.into();
        if content.trim().is_empty() {
            return;
        }
        self.chat.msgs.push(ChatMsg {
            user: String::new(),
            content,
            system: true,
            at: Instant::now(),
        });
    }

    pub fn chat_send(&mut self) {
        let text = self.chat.input.trim().to_owned();
        if text.is_empty() {
            return;
        }
        if self.chat.room_id == 0 {
            return;
        }
        // 断了就先重连，这句话排进发送队列（WS 线程握手完成后立刻发出），
        // 别让用户白打一段话
        if self.chat.conn.is_none() {
            self.chat.state = LoadState::Loading;
            self.chat.conn = Some(self.new_chat_conn(self.chat.room_id));
        }
        let Some(conn) = self.chat.conn.as_ref() else {
            return;
        };
        conn.send(&text);

        // 本地立即回显：发送在很多人眼里「没反应」就是因为要等服务器广播回来，
        // 弱网下那一下能慢好几秒。服务器回显同一句时按 pending 去重。
        let me = if self.chat.me.is_empty() {
            "我".to_owned()
        } else {
            self.chat.me.clone()
        };
        self.chat.msgs.push(ChatMsg {
            user: me,
            content: text.clone(),
            system: false,
            at: Instant::now(),
        });
        self.chat.pending.push((text.clone(), Instant::now()));
        if self.chat.pending.len() > 20 {
            self.chat.pending.remove(0);
        }
        self.chat.last_sent = Some((text, Instant::now()));
        self.chat.input.clear();
        // 原生输入框里的草稿一起清掉，否则下一句会接着上一句
        crate::video::ime_clear();
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
        self.close_ime();
        crate::video::ime_clear();
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
            self.require_login();
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

    /// 点赞 / 投币 / 收藏 / 关注：都要登录态，未登录直接弹登录窗。
    ///
    /// `uid` 是这条视频的 UP 主 id，必须由页面传进来：详情页渲染时
    /// `self.detail` 已经被 `take()` 走了（见 detail::show），再从 App 里取
    /// 只会拿到 0，关注按钮就会报「参数错误」。
    pub fn act(&mut self, id: u64, uid: u64, kind: Action) {
        if !self.client.logged_in() {
            self.require_login();
            return;
        }
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
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
        // 进房间先把播放器控件亮出来（3 秒后自动隐藏），否则用户看不到返回和全屏键
        self.poke_controls();
        self.room = None;
        self.room_state = LoadState::Loading;
        self.room_media = None;
        self.room_media_state = LoadState::Idle;
        self.room_media_err = None;
        self.live_sd_url = None;
        self.live_quality_sd = false;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::LiveRoom(id, client.live_room(id)));
        });
    }

    /// 后台探一遍直播流：只有真的能拿到 m3u8 才起播放器。
    /// 否则界面直接给出「试过哪些地址」，而不是一个永远转圈的「缓冲中…」。
    fn resolve_live_media(&mut self, room_id: u64, stream_key: String) {
        if stream_key.is_empty() {
            self.room_media_state = LoadState::Idle;
            return;
        }
        self.room_media_state = LoadState::Loading;
        self.room_media_err = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.resolve_media(&stream_key);
            let _ = tx.send(Msg::LiveMedia(room_id, r));
        });
    }

    /// 探一遍「流畅」变体：SRS 把 `<key>_sd` 这个转码流也按 HLS 落地，
    /// 真能拿到播放列表才在播放器里露出清晰度切换，探不到就当作只有原画。
    fn probe_live_variant(&mut self, master: String) {
        let Some(base) = master.strip_suffix(".m3u8") else {
            return;
        };
        let url = format!("{base}_sd.m3u8");
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let ok = client.probe_media(&url);
            let _ = tx.send(Msg::LiveVariant(ok.then_some(url)));
        });
    }

    /// 手动重试媒体源（页面上的「重连」按钮）
    pub fn retry_live_media(&mut self) {
        let route = self.route;
        let Route::LiveRoom(id) = route else { return };
        let Some(key) = self.room.as_ref().map(|r| r.stream_key.clone()) else {
            return;
        };
        self.video.stop();
        self.client.set_media_origin(None);
        self.resolve_live_media(id, key);
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
        // 换页时先退出全屏，避免横屏状态带到别的页面
        self.exit_fullscreen();
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
        // 全屏播放时，返回键先退出全屏而不是离开详情页
        if self.player_fullscreen {
            self.exit_fullscreen();
            return;
        }
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

    /// 需要登录的操作统一走这里：直接弹登录窗。
    /// 不在页面角落塞「登录后才能…」的提示——用户要的是能继续操作，不是一句说明。
    pub fn require_login(&mut self) {
        if self.client.logged_in() {
            return;
        }
        self.action_msg = None;
        self.auth_state = LoadState::Idle;
        self.login_prompt = true;
    }

    /// 搜索页离开/失焦时收起软键盘
    pub fn close_ime(&mut self) {
        if self.comment_ime || self.danmaku_ime || self.chat_ime {
            self.comment_ime = false;
            self.danmaku_ime = false;
            self.chat_ime = false;
            crate::video::ime_close();
        }
        if self.ime_on {
            self.ime_on = false;
            if let Some(ctx) = &self.ime_ctx {
                ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(false));
            }
        }
    }

    // ---------- 播放器：全屏 / 倍速 / 进度拖动 ----------

    /// 进入全屏（横屏）播放：安卓改 Activity 朝向并隐藏系统栏
    pub fn enter_fullscreen(&mut self) {
        if self.player_fullscreen {
            return;
        }
        self.player_fullscreen = true;
        self.poke_controls();
        self.fs_view = None;
        self.fs_menu = None;
        self.player_settings = false;
        crate::video::set_screen_landscape(true);
    }

    pub fn exit_fullscreen(&mut self) {
        if !self.player_fullscreen {
            return;
        }
        self.player_fullscreen = false;
        self.controls_hide_at = None;
        self.fs_view = None;
        self.seek_preview = None;
        self.seek_hold = None;
        self.fs_menu = None;
        if self.danmaku_ime {
            self.danmaku_ime = false;
            crate::video::ime_close();
        }
        // 横屏聊天浮层的键盘也要收掉，否则回到竖屏输入法还盖在页面上
        if self.chat_ime {
            self.chat_ime = false;
            crate::video::ime_close();
        }
        crate::video::set_screen_landscape(false);
    }

    /// 播放器蒙层控件是否可见（点画面才出现，3 秒后自动隐藏；展开设置/菜单时保持显示）
    pub fn fs_controls_visible(&self) -> bool {
        self.player_settings
            || self.fs_menu.is_some()
            || matches!(self.controls_hide_at, Some(t) if Instant::now() < t)
    }

    /// 有操作时把蒙层再续 3 秒
    pub fn poke_controls(&mut self) {
        self.controls_hide_at = Some(Instant::now() + Duration::from_secs(3));
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
        self.video.set_speed(speed);
    }

    /// 开始拖动进度条
    pub fn begin_seek(&mut self, ratio: f32) {
        self.seek_preview = Some(ratio.clamp(0.0, 1.0));
        self.seek_hold = None;
        self.poke_controls();
    }

    /// 拖动中：只更新预览，不动播放器
    pub fn update_seek(&mut self, ratio: f32) {
        if self.seek_preview.is_some() {
            self.seek_preview = Some(ratio.clamp(0.0, 1.0));
        }
    }

    /// 直接跳到某个比例（点击进度条）
    pub fn seek_ratio(&mut self, ratio: f32) {
        let dur = self.video.duration_ms().max(0);
        if dur > 0 {
            let r = ratio.clamp(0.0, 1.0);
            self.video.seek_to((dur as f32 * r) as i64);
            self.seek_hold = Some((r, Instant::now() + Duration::from_millis(700)));
        }
        self.poke_controls();
    }

    /// 立刻隐藏全屏蒙层（点画面切换显示/隐藏用）
    pub fn hide_controls(&mut self) {
        self.controls_hide_at = None;
        self.fs_menu = None;
    }

    /// 松手：真正 seek，并暂留显示值避免指针回弹
    pub fn finish_seek(&mut self) {
        if let Some(ratio) = self.seek_preview.take() {
            let dur = self.video.duration_ms().max(0);
            if dur > 0 {
                self.video.seek_to((dur as f32 * ratio) as i64);
                self.seek_hold = Some((ratio, Instant::now() + Duration::from_millis(700)));
            }
        }
        self.poke_controls();
    }

    /// 当前该显示的进度比例（拖动预览 / 松手暂留 / 真实进度）
    pub fn display_ratio(&mut self) -> f32 {
        if let Some(r) = self.seek_preview {
            return r;
        }
        if let Some((r, until)) = self.seek_hold {
            if Instant::now() < until {
                return r;
            }
            self.seek_hold = None;
        }
        let dur = self.video.duration_ms().max(0);
        if dur > 0 {
            (self.video.position_ms().max(0) as f32 / dur as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// 当前该显示的进度（毫秒）：拖动时按预览比例算，其余用真实位置
    pub fn display_position_ms(&mut self) -> i64 {
        let dur = self.video.duration_ms().max(0);
        let ratio = self.display_ratio();
        if self.seek_preview.is_some() || self.seek_hold.is_some() {
            (dur as f32 * ratio) as i64
        } else {
            self.video.position_ms().max(0)
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
            self.require_login();
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
            self.require_login();
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
            self.require_login();
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

    /// 启动时探一遍所有候选地址，主地址不通就自动切到能用的那个。
    /// 候选顺序即优先级：局域网（真机同 Wi-Fi 直连电脑，延迟最低、
    /// 媒体流也走它）> 手动保存过的 > 当前 > USB 映射 > 模拟器网关。
    /// 全都不通时再在手机当前 /24 网段里自动找一遍，电脑换了 IP 也能连上。
    fn probe_servers(&mut self) {
        let current = self.client.base.clone();
        let mut cands: Vec<String> = Vec::new();
        if let Some(lan) = crate::api::lan_base() {
            cands.push(lan);
        }
        cands.extend(Self::saved_servers());
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
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            use std::sync::atomic::{AtomicUsize, Ordering};
            use std::sync::Arc;

            // 并发探，选「优先级最高且真的通」的那个：串行探一个坏 IP 会白等一两秒
            let total = uniq.len();
            let (done_tx, done_rx) = std::sync::mpsc::channel::<(usize, String, bool)>();
            let finished = Arc::new(AtomicUsize::new(0));
            for (i, c) in uniq.into_iter().enumerate() {
                let client = client.clone();
                let done_tx = done_tx.clone();
                let finished = finished.clone();
                std::thread::spawn(move || {
                    let ok = client.probe_base(&c);
                    let _ = done_tx.send((i, c, ok));
                    finished.fetch_add(1, Ordering::SeqCst);
                });
            }
            let mut best: Option<(usize, String)> = None;
            let deadline = Instant::now() + Duration::from_millis(2200);
            while finished.load(Ordering::SeqCst) < total && Instant::now() < deadline {
                match done_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok((i, c, true)) => {
                        if best.as_ref().map(|(bi, _)| i < *bi).unwrap_or(true) {
                            best = Some((i, c));
                        }
                        // 已经拿到优先级最高的可用地址，不用再等别的
                        if i == 0 {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {}
                }
            }

            // 选中的是回环（USB 映射）时，局域网其实更优：能扫到就直接换成电脑的 IP
            let loopback = best
                .as_ref()
                .map(|(_, c)| crate::api::is_loopback_host(crate::api::host_of(c)))
                .unwrap_or(true);
            if loopback {
                if let Some(base) = crate::api::scan_lan_base(8080) {
                    best = Some((0, base));
                }
            }

            if let Some((_, base)) = best {
                if base != client.base {
                    let _ = tx.send(Msg::Server(base));
                }
            }
        });
    }

    /// 设置页的「自动查找电脑」：在手机当前网段里扫一遍后端，找到就切过去
    pub fn scan_for_lan(&mut self) {
        self.settings_msg = Some("正在扫描局域网…".to_owned());
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let base = crate::api::scan_lan_base(8080).unwrap_or_default();
            let _ = tx.send(Msg::Server(base));
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
        // 换后端地址后媒体源也要重新探，别沿用旧 IP 的 SRS
        self.client.set_media_origin(None);
        self.covers = crate::images::Covers::new(url.clone());
        self.videos_state = LoadState::Idle;
        self.home_ops_state = LoadState::Idle;
        self.live_state = LoadState::Idle;
        self.dm_state = LoadState::Idle;
        self.more_state = LoadState::Idle;
        // 自动切地址（回环 → 局域网）不该把登录态丢掉：token 存在内存里，
        // 清掉就得重新登录。换的若是另一台后端，接口会返回 40100 走刷新失败流程。
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
        self.client.set_media_origin(None);
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
            self.require_login();
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
                    Route::ArticleFavorites => d.article_favorites = client.my_article_favorites()?,
                    Route::Creator => {
                        d.stats = client.creator_stats().ok();
                        d.video_stats = client.creator_video_stats().unwrap_or_default();
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
                        d.subtitles = client.subtitles(id).unwrap_or_default();
                    }
                    Route::Governance => {
                        d.tickets = client.my_tickets().unwrap_or_default();
                        d.appeals = client.my_appeals().unwrap_or_default();
                        d.cs = client.my_cs_conversations().unwrap_or_default();
                        d.copyrights = client.my_copyright_complaints().unwrap_or_default();
                    }
                    Route::Ticket(id) => d.ticket_detail = Some(client.ticket_detail(id)?),
                    Route::CsChat(id) => d.cs_detail = Some(client.cs_conversation(id)?),
                    Route::Appeal(id) => d.appeal = Some(client.appeal(id)?),
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

    /// 打开我的图文收藏
    pub fn open_article_favorites(&mut self) {
        self.open_more(Route::ArticleFavorites);
    }

    /// 打开工单详情
    pub fn open_ticket(&mut self, id: u64) {
        self.ticket_text.clear();
        self.open_more(Route::Ticket(id));
    }

    /// 打开申诉详情
    pub fn open_appeal(&mut self, id: u64) {
        self.open_more(Route::Appeal(id));
    }

    /// 打开客服会话
    pub fn open_cs(&mut self, id: u64) {
        self.cs_text.clear();
        self.open_more(Route::CsChat(id));
    }

    /// 扫描收件目录里的字幕文件（srt / vtt）
    pub fn scan_subtitle_files(&mut self) {
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
                if matches!(ext.as_str(), "srt" | "vtt") {
                    let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                    out.push((p.to_string_lossy().to_string(), size));
                }
            }
        }
        out.sort();
        self.subtitle_files = out;
    }

    /// 扫描收件目录里的图片（头像 / 封面用）
    pub fn scan_images(&mut self) {
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
                if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp") {
                    out.push(p.to_string_lossy().to_string());
                }
            }
        }
        out.sort();
        self.image_files = out;
    }

    /// 上传头像（走设置页的写操作通道，成功后重拉 /users/me）
    pub fn upload_avatar(&mut self) {
        let path = self.image_path.clone();
        if path.is_empty() {
            self.settings_msg = Some("先在下方选一张图片".to_owned());
            return;
        }
        self.settings_action(move |c| c.upload_avatar(&path).map(|_| ()));
    }

    /// 上传视频封面
    pub fn upload_video_cover(&mut self, id: u64) {
        let path = self.image_path.clone();
        if path.is_empty() {
            self.more.err = Some("先在下方选一张图片".to_owned());
            return;
        }
        self.more_action(move |c| c.upload_video_cover(id, &path));
    }

    /// 上传文章封面
    pub fn upload_article_cover(&mut self, id: u64) {
        let path = self.image_path.clone();
        if path.is_empty() {
            self.more.err = Some("先在下方选一张图片".to_owned());
            return;
        }
        self.more_action(move |c| c.upload_article_cover(id, &path));
    }

    /// 上传直播封面
    pub fn upload_live_cover(&mut self, id: u64) {
        let path = self.image_path.clone();
        if path.is_empty() {
            self.more.err = Some("先在下方选一张图片".to_owned());
            return;
        }
        self.more_action(move |c| c.upload_live_cover(id, &path));
    }

    /// 替换稿件源文件（上传后进入转码队列）
    pub fn replace_video_media(&mut self, id: u64, path: &str) {
        if path.is_empty() {
            self.more.err = Some("先选择一个视频文件".to_owned());
            return;
        }
        let path = path.to_owned();
        let (title, desc) = (self.f_title.trim().to_owned(), self.f_body.clone());
        self.more_action(move |c| c.replace_video_media(id, &path, &title, &desc));
    }

    /// 下载草稿源到 inbox，方便本地预览
    pub fn download_draft_preview(&mut self, id: u64) {
        let dir = Self::inbox_dir();
        let _ = std::fs::create_dir_all(&dir);
        let dest = format!("{dir}/draft_{id}.mp4");
        self.more_action(move |c| c.download_draft_source(id, &dest));
    }

    /// 新建直播间
    pub fn create_live_room(&mut self) {
        let title = self.f_title.trim().to_owned();
        if title.is_empty() {
            self.more.err = Some("先填直播间标题".to_owned());
            return;
        }
        self.more_action(move |c| c.create_live_room(&title, "").map(|_| ()));
    }

    /// 收藏夹改名
    pub fn rename_favorite_folder(&mut self, id: u64) {
        let title = self.rename_title.trim().to_owned();
        if title.is_empty() {
            self.folder_msg = Some("新名称不能为空".to_owned());
            return;
        }
        let keep_public = self
            .favorite_folders
            .iter()
            .find(|f| f.id == id)
            .map(|f| f.is_public)
            .unwrap_or(true);
        self.rename_folder = 0;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.update_favorite_folder(id, &title, "", keep_public);
            let _ = tx.send(Msg::FolderAction(r));
        });
    }

    /// 清理收藏夹里的失效视频
    pub fn clear_invalid_favorites(&mut self, id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.clear_invalid_favorites(id);
            let _ = tx.send(Msg::FolderAction(r));
        });
    }

    /// 批量移出收藏夹
    pub fn batch_remove_favorites(&mut self, folder_id: u64) {
        let ids = std::mem::take(&mut self.fav_selected);
        if ids.is_empty() {
            self.folder_msg = Some("先勾选要移出的视频".to_owned());
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.batch_remove_favorites(folder_id, &ids);
            let _ = tx.send(Msg::FolderAction(r));
        });
    }

    /// 批量把勾选的视频移动到另一个收藏夹
    pub fn move_selected_favorites(&mut self, from_folder: u64) {
        let to = self.move_to_folder;
        if to == 0 || to == from_folder {
            self.folder_msg = Some("先选择目标收藏夹".to_owned());
            return;
        }
        let ids = std::mem::take(&mut self.fav_selected);
        if ids.is_empty() {
            self.folder_msg = Some("先勾选要移动的视频".to_owned());
            return;
        }
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let mut err = None;
            for id in ids {
                if let Err(e) = client.move_video_favorite_folder(id, from_folder, to) {
                    err = Some(e);
                    break;
                }
            }
            let _ = tx.send(Msg::FolderAction(match err {
                Some(e) => Err(e),
                None => Ok(()),
            }));
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
            self.require_login();
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

    /// 把选中的文件存为草稿（不进入转码）
    pub fn do_save_draft(&mut self) {
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
            self.require_login();
            return;
        }
        self.up_state = LoadState::Loading;
        self.up_msg = Some("保存草稿中，别退出…".to_owned());
        let desc = self.f_body.clone();
        let zone = self.up_zone.clone();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.save_draft(&path, &title, &desc, &zone);
            let _ = tx.send(Msg::Uploaded(r));
        });
    }

    /// 打开稿件编辑（只对自己投稿开放，后端 PUT 时还会校验）
    pub fn open_video_edit(&mut self, id: u64) {
        self.f_tags.clear();
        self.f_schedule.clear();
        self.open_more(Route::VideoEdit(id));
    }

    /// 把原生输入框里的内容镜像回对应的自绘输入框（安卓专用）。
    /// 每帧调一次，谁被点开了就同步谁。
    pub fn sync_field_ime(&mut self) {
        #[cfg(target_os = "android")]
        {
            let Some(field) = self.focus_field.clone() else {
                return;
            };
            // 这些框没有「发送」语义，把键盘上的发送键标记吃掉，
            // 否则留着会被聊天页当成「发出去」的指令
            let _ = crate::video::ime_action();
            let Some(t) = crate::video::ime_text() else {
                return;
            };
            let buf = match field.as_str() {
                "f-title" => &mut self.f_title,
                "f-body" => &mut self.f_body,
                "f-text" => &mut self.f_text,
                "f-ticket" => &mut self.ticket_text,
                _ => return,
            };
            if *buf != t {
                *buf = t;
            }
        }
    }

    /// 轻提示：底部飘一条，3 秒后自己消失
    pub fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    /// 发布 / 保存文章。成功后直接进文章详情页——既当「发布成功」的确认，
    /// 也顺手给了预览（之前发完只把输入框清空，用户不知道到底发没发出去）。
    pub fn save_article(&mut self, editing: u64, publish: bool) {
        let title = self.f_title.trim().to_owned();
        let body = self.f_body.clone();
        // 和后端 validateArticleContent 对齐：发布要求标题+正文都非空，
        // 草稿只要有一项就行。不先校验，用户只会看到一句「参数错误」。
        if publish {
            if title.is_empty() {
                self.toast("发布需要标题");
                return;
            }
            if body.trim().is_empty() {
                self.toast("发布需要正文，可先存草稿");
                return;
            }
        } else if title.is_empty() && body.trim().is_empty() {
            self.toast("标题和正文至少填一项");
            return;
        }
        if !self.client.logged_in() {
            self.require_login();
            return;
        }
        self.more_state = LoadState::Loading;
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = if editing > 0 {
                client
                    .put_article(editing, &title, &body, publish)
                    .map(|st| (editing, publish, st))
            } else {
                client
                    .post_article(&title, &body, publish)
                    .map(|(id, st)| (id, publish, st))
            };
            let _ = tx.send(Msg::ArticlePosted(r));
        });
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
        self.space_folder = None;
        self.space_folder_items.clear();
        self.space_folder_state = LoadState::Idle;
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
                let recent_coins = client.user_recent_coins(user_id).unwrap_or_default();
                let favorite_folders = client.user_favorite_folders(user_id).unwrap_or_default();
                let article_favorites = client.user_article_favorites(user_id).unwrap_or_default();
                let following = client.user_following(user_id).unwrap_or_default();
                let followers = client.user_followers(user_id).unwrap_or_default();
                Ok(SpaceData {
                    profile,
                    videos,
                    articles,
                    dynamics,
                    favorites,
                    recent_coins,
                    favorite_folders,
                    article_favorites,
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

    /// 打开他人主页里的某个收藏夹
    pub fn open_space_folder(&mut self, folder_id: u64) {
        let Route::Space(uid) = self.route else {
            return;
        };
        if self.space_folder == Some(folder_id) {
            self.space_folder = None;
            return;
        }
        self.space_folder = Some(folder_id);
        self.space_folder_state = LoadState::Loading;
        self.space_folder_items.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let r = client.user_favorites_in_folder(uid, folder_id);
            let _ = tx.send(Msg::SpaceFolder(uid, folder_id, r));
        });
    }

    /// 在他人主页点「关注 / 已关注」
    pub fn toggle_space_follow(&mut self) {
        let Route::Space(uid) = self.route else {
            return;
        };
        if !self.client.logged_in() {
            self.require_login();
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
            // 列表本身带 paused，再用设置接口校正一次，避免两个接口口径不一致
            let paused = client
                .view_history_settings()
                .ok()
                .map(|s| s.paused);
            let mut page = client.view_history();
            if let (Ok(p), Some(paused)) = (&mut page, paused) {
                p.paused = paused;
            }
            let _ = tx.send(Msg::History(page));
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
            self.require_login();
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
            self.require_login();
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
            self.require_login();
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
            self.require_login();
            return;
        }
        let parent_id = self.reply_to.as_ref().map(|(id, _)| *id).unwrap_or(0);
        self.action_msg = None;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let image = self.comment_image_path.clone();
        std::thread::spawn(move || {
            let r = if !image.is_empty() && parent_id == 0 {
                client.post_comment_with_image(video_id, &text, &image)
            } else {
                client.post_comment(video_id, &text, parent_id)
            };
            let _ = tx.send(Msg::CommentPosted(video_id, r));
        });
    }

    /// 按当前页面/登录态维护两条实时通道
    fn ensure_ws(&mut self) {
        let want_video = match self.route {
            Route::Detail(id) => Some(id),
            Route::CommentDetail { video_id, .. } => Some(video_id),
            _ => None,
        };
        match want_video {
            Some(vid) => {
                let need = match self.danmaku_ws.as_ref() {
                    Some((v, ws)) if *v == vid => {
                        if ws.alive() {
                            false
                        } else {
                            match self.danmaku_ws_retry {
                                Some(t) if Instant::now() < t => false,
                                _ => true,
                            }
                        }
                    }
                    _ => true,
                };
                if need {
                    if self
                        .danmaku_ws
                        .as_ref()
                        .map(|(v, ws)| *v == vid && !ws.alive())
                        .unwrap_or(false)
                    {
                        self.danmaku_ws_retry = Some(Instant::now() + std::time::Duration::from_secs(5));
                    }
                    let path = match self.client.access_token() {
                        Some(t) => format!("/api/v1/ws/danmaku?video_id={vid}&token={t}"),
                        None => format!("/api/v1/ws/danmaku?video_id={vid}"),
                    };
                    let url = self.client.ws_url(&path);
                    self.danmaku_ws = Some((vid, crate::wsclient::Client::connect(&url)));
                }
            }
            None => {
                self.danmaku_ws = None;
                self.danmaku_ws_retry = None;
            }
        }

        if !self.client.logged_in() {
            self.chat_ws = None;
            self.chat_ws_retry = None;
            return;
        }
        let need_chat = match self.chat_ws.as_ref() {
            Some(ws) if ws.alive() => false,
            Some(_) => match self.chat_ws_retry {
                Some(t) if Instant::now() < t => false,
                _ => true,
            },
            None => true,
        };
        if need_chat {
            if self.chat_ws.as_ref().map(|ws| !ws.alive()).unwrap_or(false) {
                self.chat_ws_retry = Some(Instant::now() + std::time::Duration::from_secs(5));
            }
            if let Some(t) = self.client.access_token() {
                let url = self.client.ws_url(&format!("/api/v1/ws/chat?token={t}"));
                self.chat_ws = Some(crate::wsclient::Client::connect(&url));
            }
        }
    }

    /// 把 WS 事件并入现有页面状态（弹幕合并、私信追加/会话更新）
    fn poll_ws(&mut self) {
        if let Some((vid, ws)) = self.danmaku_ws.take() {
            let events = ws.drain();
            for v in events {
                self.apply_danmaku_event(vid, v);
            }
            self.danmaku_ws = Some((vid, ws));
        }
        if let Some(ws) = self.chat_ws.take() {
            let events = ws.drain();
            for v in events {
                self.apply_chat_event(v);
            }
            self.chat_ws = Some(ws);
        }
    }

    fn apply_danmaku_event(&mut self, vid: u64, v: serde_json::Value) {
        match v.get("type").and_then(|t| t.as_str()).unwrap_or("") {
            "history" => {
                if let Some(items) = v.get("items").and_then(|x| x.as_array()) {
                    for it in items {
                        if let Ok(d) = serde_json::from_value::<api::Danmaku>(it.clone()) {
                            self.merge_danmaku(vid, d);
                        }
                    }
                }
            }
            "danmaku" => {
                if let Some(data) = v.get("data") {
                    if let Ok(d) = serde_json::from_value::<api::Danmaku>(data.clone()) {
                        self.merge_danmaku(vid, d);
                    }
                }
            }
            "watching" => {
                if let Some(n) = v.get("count").and_then(|x| x.as_i64()) {
                    if let Some(detail) = self.detail.as_mut() {
                        if detail.video.id == vid {
                            detail.video.watching_count = n;
                        }
                    }
                }
            }
            "auth_failed" => {
                self.action_msg = Some(
                    v.get("msg")
                        .and_then(|m| m.as_str())
                        .unwrap_or("弹幕连接鉴权失败")
                        .to_owned(),
                );
            }
            _ => {}
        }
    }

    fn merge_danmaku(&mut self, vid: u64, d: api::Danmaku) {
        let on_video = match self.route {
            Route::Detail(id) => id == vid,
            Route::CommentDetail { video_id, .. } => video_id == vid,
            _ => false,
        };
        if !on_video {
            return;
        }
        let Some(detail) = self.detail.as_mut() else {
            return;
        };
        if detail.video.id != vid {
            return;
        }
        let dup = if d.id != 0 {
            detail.danmakus.iter().any(|x| x.id == d.id)
        } else {
            detail
                .danmakus
                .iter()
                .any(|x| x.content == d.content && (x.video_time - d.video_time).abs() < 0.01)
        };
        if !dup {
            detail.danmakus.push(d);
        }
    }

    fn apply_chat_event(&mut self, v: serde_json::Value) {
        match v.get("type").and_then(|t| t.as_str()).unwrap_or("") {
            "dm_message" => {
                let Some(raw) = v.get("message") else { return };
                let Ok(mut m) = serde_json::from_value::<api::DmMessage>(raw.clone()) else {
                    return;
                };
                let my_id = self.me.as_ref().map(|u| u.user_id).unwrap_or(0);
                m.from_me = m.sender_id == my_id;
                m.conversation_id = m
                    .conversation_id
                    .max(raw.get("conversation_id").and_then(|x| x.as_u64()).unwrap_or(0));
                let conv_id = m.conversation_id;
                if conv_id == 0 {
                    return;
                }
                if let Route::DmChat(active) = self.route {
                    if active == conv_id
                        && m.id != 0
                        && !self.dm_chat.iter().any(|x| x.id == m.id)
                    {
                        self.dm_chat.push(m.clone());
                    }
                }
                if let Some(c) = self.dm_convs.iter_mut().find(|c| c.id == conv_id) {
                    c.last_preview = m.content.clone();
                    c.last_message_at = m.created_at.clone();
                    if !m.from_me && self.route != Route::DmChat(conv_id) {
                        c.unread_count += 1;
                    }
                }
            }
            "dm_conversation" => {
                let Some(raw) = v.get("conversation") else { return };
                if let Ok(c) = serde_json::from_value::<api::DmConversation>(raw.clone()) {
                    if let Some(pos) = self.dm_convs.iter().position(|x| x.id == c.id) {
                        self.dm_convs[pos] = c;
                    } else {
                        self.dm_convs.insert(0, c);
                    }
                    self.dm_convs.sort_by(|a, b| {
                        b.pinned.cmp(&a.pinned).then_with(|| {
                            b.last_message_at.cmp(&a.last_message_at)
                        })
                    });
                }
            }
            "auth_failed" => {
                self.action_msg = Some(
                    v.get("msg")
                        .and_then(|m| m.as_str())
                        .unwrap_or("私信连接鉴权失败")
                        .to_owned(),
                );
            }
            _ => {}
        }
    }

    fn poll(&mut self) {
        self.ensure_ws();
        self.poll_ws();
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
                    // 视频详情页和评论详情页都读这份数据，两者都要接受刷新结果
                    if !self.on_video_route(id) {
                        continue;
                    }
                    match r {
                        Ok(data) => {
                            let mut urls = vec![data.video.cover_url.clone()];
                            urls.push(data.video.uploader_avatar_url.clone());
                            urls.extend(data.related.iter().map(|v| v.cover_url.clone()));
                            urls.extend(data.comments.iter().map(|c| c.avatar_url.clone()));
                            // 评论配图也要预加载，列表里直接贴图
                            urls.extend(
                                data.comments.iter().flat_map(|c| c.images.iter().cloned()),
                            );
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
                        // 弹窗登录成功后收起，并把依赖登录态的数据打回 Idle 让当前页重拉
                        self.login_prompt = false;
                        self.watch_later_state = LoadState::Idle;
                        self.favorites_state = LoadState::Idle;
                        self.folders_state = LoadState::Idle;
                        self.notifications_state = LoadState::Idle;
                        self.history_state = LoadState::Idle;
                        if matches!(self.route, Route::Tab(Tab::Dynamic)) {
                            self.dynamic_state = LoadState::Idle;
                        }
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
                            let live = room.status == "live" && !room.stream_key.is_empty();
                            let key = room.stream_key.clone();
                            // 标题不再占页面上的一整块，改成一条系统提示放进聊天里
                            if !room.title.is_empty() {
                                self.chat_system(format!("直播间：{}", room.title));
                            }
                            self.room = Some(room);
                            self.room_state = LoadState::Ready;
                            if live {
                                // 先确认媒体地址真的能出 m3u8，再起播放器
                                self.resolve_live_media(id, key);
                            }
                        }
                        Err(e) => self.room_state = LoadState::Err(e),
                    }
                }
                Msg::LiveVariant(u) => self.live_sd_url = u,
                Msg::LiveMedia(id, r) => {
                    if self.route != Route::LiveRoom(id) {
                        continue;
                    }
                    match r {
                        Ok(url) => {
                            self.room_media = Some(url.clone());
                            self.room_media_state = LoadState::Ready;
                            self.room_media_err = None;
                            // 顺带探一下 SRS 转码出来的「流畅」流，有就能在播放器里切
                            self.probe_live_variant(url);
                        }
                        Err(e) => {
                            self.room_media = None;
                            self.room_media_state = LoadState::Err(e.clone());
                            self.room_media_err = Some(e);
                        }
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
                        self.comment_image_path.clear();
                        self.comment_image_open = false;
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
                            urls.extend(data.recent_coins.iter().map(|f| f.cover_url.clone()));
                            urls.extend(
                                data.favorite_folders
                                    .iter()
                                    .flat_map(|f| f.cover_url.clone()),
                            );
                            urls.extend(data.article_favorites.iter().map(|a| a.cover_url.clone()));
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
                Msg::SpaceFolder(uid, folder_id, r) => {
                    if self.route != Route::Space(uid) || self.space_folder != Some(folder_id) {
                        continue;
                    }
                    match r {
                        Ok(items) => {
                            let urls: Vec<String> =
                                items.iter().map(|f| f.cover_url.clone()).collect();
                            self.covers.request(&urls);
                            self.space_folder_items = items;
                            self.space_folder_state = LoadState::Ready;
                        }
                        Err(e) => self.space_folder_state = LoadState::Err(e),
                    }
                }
                Msg::Unread(r) => {
                    if let Ok(u) = r {
                        self.unread = u;
                    }
                }
                Msg::Server(url) => {
                    if url.is_empty() {
                        self.settings_msg =
                            Some("没在当前网段找到后端：确认电脑和手机连的是同一个 Wi-Fi".to_owned());
                    } else if url != self.client.base {
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
                            let my_id = self.me.as_ref().map(|u| u.user_id).unwrap_or(0);
                            let mut merged: Vec<api::DmMessage> = list
                                .into_iter()
                                .map(|mut m| {
                                    m.from_me = m.sender_id == my_id;
                                    m
                                })
                                .collect();
                            // 拉取期间 WS 可能已经推了新消息，按 id 合并回去
                            for old in std::mem::take(&mut self.dm_chat) {
                                if !merged.iter().any(|x| x.id == old.id) {
                                    merged.push(old);
                                }
                            }
                            merged.sort_by(|a, b| {
                                a.created_at.cmp(&b.created_at).then_with(|| a.id.cmp(&b.id))
                            });
                            self.dm_chat = merged;
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
                Msg::ArticlePosted(r) => match r {
                    Ok((id, publish, status)) => {
                        self.more_state = LoadState::Idle;
                        if publish {
                            self.f_title.clear();
                            self.f_body.clear();
                            self.more.editing = 0;
                            if status == "pending_review" {
                                self.toast("已提交，审核通过后公开");
                            } else {
                                self.toast("已发布，正在打开预览");
                            }
                            if id > 0 {
                                self.open_more(Route::Article(id));
                            }
                        } else {
                            // 存草稿：留在编辑器里继续写，再点「发布」就是改这一篇
                            self.more.editing = id;
                            self.toast("已存草稿");
                            self.load_more();
                        }
                    }
                    Err(e) => {
                        self.more_state = LoadState::Ready;
                        let hint = if e.contains("参数错误") {
                            "发布失败：内容未通过校验（正文为空或含敏感词）".to_owned()
                        } else {
                            format!("发布失败：{e}")
                        };
                        self.more.err = Some(hint.clone());
                        self.toast(hint);
                    }
                },
                Msg::ArticleEditor(r) => match r {
                    Ok(a) => {
                        self.more.editing = a.id;
                        self.f_title = a.title.clone();
                        self.f_body = a.body_md.clone();
                        // 编辑器在这页最上面：滚回去 + 说一声，否则点了没反应
                        self.scroll_top = true;
                        self.toast("已载入正文，可继续编辑");
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
        // 全屏播放：转屏会重建窗口属性，尺寸一变就把系统栏再藏一次。
        // 不要每帧/每秒重申：反复 setSystemUiVisibility 会抢走窗口焦点，
        // 原生输入框（聊天/弹幕）的输入连接会被系统解绑，键盘就敲不进字了。
        if self.player_fullscreen {
            let vp = ui.ctx().viewport_rect();
            let size = egui::Vec2::new(vp.width(), vp.height());
            if self.fs_view != Some(size) {
                self.fs_view = Some(size);
                crate::video::set_immersive(true);
            }
        }
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
        if (raw_back || esc_back) && self.login_prompt {
            // 登录弹窗开着：返回键先关弹窗
            self.login_prompt = false;
            self.login_pass.clear();
            self.close_ime();
        } else if (raw_back || esc_back) && self.player_fullscreen {
            // 全屏播放：返回键/ Esc 先退出全屏
            self.exit_fullscreen();
        } else if raw_back && (wants_kb || self.comment_ime || self.chat_ime) {
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
        if self.ime_on || self.comment_ime || self.chat_ime {
            let ime = crate::video::ime_points(ui.ctx().pixels_per_point());
            if ime > bottom_inset {
                bottom_inset = ime;
            }
        }
        if self.player_fullscreen {
            // 沉浸式全屏：系统栏已隐藏，不要再按竖屏尺寸补内衬
            top_inset = 0.0;
            bottom_inset = 0.0;
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
                    | Route::ArticleFavorites
                    | Route::Creator
                    | Route::CreatorComments
                    | Route::CreatorDanmakus
                    | Route::DynamicDetail(_)
                    | Route::Specials
                    | Route::Ledger
                    | Route::Governance
                    | Route::Appeal(_)
                    | Route::Ticket(_)
                    | Route::CsChat(_) => pages::more::show(ui, self),
                    Route::LiveHost => pages::more::live_host(ui, self),
                    Route::VideoEdit(id) => pages::more::video_edit(ui, self, id),
                    Route::Upload => pages::more::upload(ui, self),
                }
            });

        // 输入框镜像（安卓原生输入法 → 自绘输入框）
        self.sync_field_ime();

        // 需要登录的操作直接弹窗（盖在整页之上，登录成功后自动收起）
        self.login_modal(ui.ctx());
        self.image_viewer(ui.ctx());
        self.toast_ui(ui.ctx());

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

    /// 轻提示
    fn toast_ui(&mut self, ctx: &egui::Context) {
        let Some((msg, at)) = self.toast.clone() else {
            return;
        };
        if at.elapsed() > Duration::from_secs(3) {
            self.toast = None;
            return;
        }
        let screen = ctx.viewport_rect();
        let center = Pos2::new(screen.center().x, screen.bottom() - self.bottom_inset - 96.0);
        egui::Area::new(egui::Id::new("toast"))
            .order(egui::Order::Tooltip)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let g = ui.painter().layout_no_wrap(
                    msg.clone(),
                    FontId::proportional(12.0),
                    Color32::WHITE,
                );
                let rect = Rect::from_center_size(center, g.size() + Vec2::new(26.0, 18.0));
                ui.painter()
                    .rect_filled(rect, rect.height() / 2.0, Color32::from_black_alpha(200));
                ui.painter()
                    .galley(center - g.size() / 2.0, g, Color32::WHITE);
            });
    }

    /// 图片放大查看：点评论配图后铺满整屏，点任意位置关闭
    fn image_viewer(&mut self, ctx: &egui::Context) {
        let Some(url) = self.image_view.clone() else {
            return;
        };
        let screen = ctx.viewport_rect();
        let tex_size = self.covers.get(&url).map(|t| t.size_vec2());
        let mut close = false;
        egui::Area::new(egui::Id::new("image-viewer"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let mask = ui.interact(screen, egui::Id::new("image-viewer-mask"), Sense::click());
                ui.painter()
                    .rect_filled(screen, 0.0, Color32::from_black_alpha(238));
                let mut rect = screen.shrink(14.0);
                if let Some(sz) = tex_size {
                    if sz.x > 0.0 && sz.y > 0.0 {
                        let k = (rect.width() / sz.x).min(rect.height() / sz.y);
                        rect = egui::Rect::from_center_size(rect.center(), sz * k);
                    }
                }
                widgets::draw_cover(ui, rect, &url, &self.covers, 2.0);
                if mask.clicked() {
                    close = true;
                }
            });
        if close {
            self.image_view = None;
        }
    }

    /// 登录弹窗：盖在整页之上，登录成功后自己关掉
    fn login_modal(&mut self, ctx: &egui::Context) {
        if !self.login_prompt {
            return;
        }
        // 已经登上了（登录页/自动续期）就直接收起
        if self.me.is_some() || self.client.logged_in() {
            self.login_prompt = false;
            return;
        }

        let screen = ctx.viewport_rect();
        let cw = (screen.width() - 60.0).clamp(240.0, 340.0);
        let ch = 232.0;
        // 软键盘弹起时把卡片放到「键盘上方」的中间，别让按钮被盖住
        let ime_h = if self.ime_on || self.ime_wanted {
            crate::video::ime_points(ctx.pixels_per_point())
        } else {
            0.0
        };
        let area = egui::Rect::from_min_max(
            screen.min,
            Pos2::new(screen.right(), (screen.bottom() - ime_h).max(screen.top() + ch)),
        );
        let card = egui::Rect::from_center_size(area.center(), Vec2::new(cw, ch));

        egui::Area::new(egui::Id::new("login-modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                // 遮罩：点卡片外面关掉
                let mask = ui.interact(screen, egui::Id::new("login-mask"), Sense::click());
                ui.painter()
                    .rect_filled(screen, 0.0, Color32::from_black_alpha(120));
                ui.painter().rect_filled(card, 12.0, theme::BG);

                let inner = card.shrink(16.0);
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(inner)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        ui.label(
                            egui::RichText::new("登录后即可继续")
                                .color(theme::TEXT)
                                .size(15.0)
                                .strong(),
                        );
                        ui.add_space(12.0);
                        ui.add_sized(
                            Vec2::new(inner.width(), 32.0),
                            egui::TextEdit::singleline(&mut self.login_user)
                                .id(egui::Id::new("login-modal-user"))
                                .hint_text("用户名")
                                .margin(egui::Margin::symmetric(8, 6)),
                        );
                        ui.add_space(6.0);
                        let pw = ui.add_sized(
                            Vec2::new(inner.width(), 32.0),
                            egui::TextEdit::singleline(&mut self.login_pass)
                                .id(egui::Id::new("login-modal-pass"))
                                .password(true)
                                .hint_text("密码")
                                .margin(egui::Margin::symmetric(8, 6)),
                        );
                        // 软键盘交给 App 每帧汇总（输入法要弹在密码框上）
                        self.note_focus(pw.has_focus());

                        let busy = self.auth_state == LoadState::Loading;
                        if let LoadState::Err(e) = &self.auth_state {
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(e.clone())
                                    .color(theme::UP_TAG)
                                    .size(11.0),
                            );
                        }
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let btn = egui::Button::new(
                                egui::RichText::new(if busy { "登录中…" } else { "登录" })
                                    .color(Color32::WHITE)
                                    .size(12.0),
                            )
                            .fill(theme::PINK)
                            .corner_radius(15.0);
                            if ui
                                .add_enabled(!busy, btn.min_size(Vec2::new(92.0, 30.0)))
                                .clicked()
                            {
                                self.do_login();
                            }
                            if ui
                                .add_sized(Vec2::new(72.0, 30.0), egui::Button::new("取消"))
                                .clicked()
                            {
                                self.login_prompt = false;
                                self.login_pass.clear();
                            }
                        });
                    },
                );

                // 点遮罩空白处 = 取消
                if mask.clicked() {
                    if let Some(q) = mask.interact_pointer_pos() {
                        if !card.contains(q) {
                            self.login_prompt = false;
                            self.login_pass.clear();
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
