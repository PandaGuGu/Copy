//! 他人个人主页（点 UP 主头像 / 名字进）
//!
//! 数据全部来自 space 系列公开接口：
//!   GET /api/v1/space/:userId           资料（关注数 / 粉丝数 / 投稿数 / 是否已关注）
//!   GET /api/v1/space/:userId/videos    投稿
//!   GET /api/v1/space/:userId/articles  专栏
//!   GET /api/v1/space/:userId/dynamics  动态
//!   GET /api/v1/space/:userId/favorites 收藏（隐私关闭时后端直接给空列表）
//!   GET /api/v1/space/:userId/following · /followers 关注 / 粉丝
//!
//! 子资源在进页面时一次拉齐，切 tab 不再发请求。

use crate::{
    api::{ArticleCard, CoinVideo, FavoriteItem, FollowUser, SpaceVideo, UserProfile},
    app::{App, LoadState, SpaceData, SpaceTab},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App, user_id: u64) {
    if widgets::back_bar(ui, "", "space-back") {
        app.back();
        return;
    }

    match app.space_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            center(ui, "正在加载主页 …");
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "主页加载失败", &e) {
                app.reload_space();
            }
        }
        LoadState::Ready => {
            // 数据是按 user_id 缓存的一份，路由已经切到别人时等新回包
            let Some(data) = app.space.take() else { return };
            if data.profile.user_id != user_id {
                app.space = Some(data);
                center(ui, "正在加载主页 …");
                return;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("space-scroll")
                .show(ui, |ui| {
                    header(ui, app, &data.profile);
                    let tab = app.space_tab;
                    tabs(ui, app, &data);
                    ui.add_space(4.0);
                    match tab {
                        SpaceTab::Videos => videos(ui, app, &data),
                        SpaceTab::Articles => articles(ui, app, &data.articles),
                        SpaceTab::Dynamics => dynamics(ui, &data),
                        SpaceTab::Favorites => favorites(ui, app, &data.favorites),
                        SpaceTab::RecentCoins => recent_coins(ui, app, &data.recent_coins),
                        SpaceTab::FavoriteFolders => favorite_folders(ui, app, &data),
                        SpaceTab::ArticleFavorites => {
                            articles(ui, app, &data.article_favorites)
                        }
                        SpaceTab::Following => users(ui, app, &data.following, "还没有关注任何人"),
                        SpaceTab::Followers => users(ui, app, &data.followers, "还没有粉丝"),
                    }
                    if let Some(msg) = app.action_msg.clone() {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
                        });
                    }
                    ui.add_space(16.0);
                });
            app.space = Some(data);
        }
    }
}

fn center(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(140.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}

/// 资料头部：头像 + 昵称 + 等级 + 公告 + 三个计数 + 关注按钮
fn header(ui: &mut egui::Ui, app: &mut App, p: &UserProfile) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 128.0), Sense::hover());

    let av_c = Pos2::new(rect.left() + 38.0, rect.top() + 34.0);
    if p.avatar_url.is_empty() {
        widgets::avatar(ui.painter(), av_c, 24.0, &p.nickname, p.user_id);
    } else {
        widgets::avatar_tex(
            ui,
            av_c,
            24.0,
            &p.nickname,
            &p.avatar_url,
            &app.covers,
            p.user_id,
        );
    }

    let p2 = ui.painter();
    let name_x = rect.left() + 74.0;
    p2.text(
        Pos2::new(name_x, rect.top() + 14.0),
        Align2::LEFT_TOP,
        &p.nickname,
        FontId::proportional(17.0),
        theme::TEXT,
    );
    // 等级牌
    let lv = Rect::from_min_size(
        Pos2::new(name_x, rect.top() + 40.0),
        Vec2::new(34.0, 15.0),
    );
    p2.rect_filled(lv, 3.0, theme::ORANGE);
    p2.text(
        lv.center(),
        Align2::CENTER_CENTER,
        format!("LV{}", p.level_info.current_level),
        FontId::proportional(9.0),
        Color32::WHITE,
    );
    if !p.cake_id.is_empty() {
        p2.text(
            Pos2::new(lv.right() + 8.0, rect.top() + 42.0),
            Align2::LEFT_TOP,
            &p.cake_id,
            FontId::proportional(10.0),
            theme::TEXT3,
        );
    }
    if !p.sign.is_empty() {
        p2.text(
            Pos2::new(name_x, rect.top() + 60.0),
            Align2::LEFT_TOP,
            &p.sign,
            FontId::proportional(11.0),
            theme::TEXT2,
        );
    }

    // 关注 / 粉丝 / 投稿
    let stats = format!(
        "{} 关注   {} 粉丝   {} 投稿",
        theme::human(p.following_count),
        theme::human(p.follower_count),
        theme::human(p.published_count)
    );
    p2.text(
        Pos2::new(rect.left() + 14.0, rect.top() + 78.0),
        Align2::LEFT_TOP,
        stats,
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    if !p.announcement.is_empty() {
        let lines = widgets::wrap_lines(p2, &format!("公告：{}", p.announcement), 11.0, w - 28.0, 1);
        if let Some(l) = lines.first() {
            p2.text(
                Pos2::new(rect.left() + 14.0, rect.top() + 98.0),
                Align2::LEFT_TOP,
                l,
                FontId::proportional(11.0),
                theme::TEXT2,
            );
        }
    }

    // 关注按钮（自己的主页不显示）
    if !p.is_owner {
        let btn = Rect::from_min_size(
            Pos2::new(rect.right() - 78.0, rect.top() + 18.0),
            Vec2::new(64.0, 26.0),
        );
        if p.followed_by_me {
            p2.rect_stroke(btn, 13.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
            p2.text(
                btn.center(),
                Align2::CENTER_CENTER,
                "已关注",
                FontId::proportional(11.0),
                theme::TEXT3,
            );
        } else {
            p2.rect_filled(btn, 13.0, theme::PINK);
            p2.text(
                btn.center(),
                Align2::CENTER_CENTER,
                "+ 关注",
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
        // painter 用完再交互
        if ui
            .interact(btn, egui::Id::new("space-follow"), Sense::click())
            .clicked()
        {
            app.toggle_space_follow();
        }
    }
}

fn tabs(ui: &mut egui::Ui, app: &mut App, data: &SpaceData) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::hover());
    let count = |n: usize| if n > 0 { format!(" {n}") } else { String::new() };
    let items: [(SpaceTab, String); 9] = [
        (SpaceTab::Videos, format!("投稿{}", count(data.videos.len()))),
        (SpaceTab::Articles, format!("文章{}", count(data.articles.len()))),
        (SpaceTab::Dynamics, format!("动态{}", count(data.dynamics.len()))),
        (SpaceTab::Favorites, format!("收藏{}", count(data.favorites.len()))),
        (SpaceTab::RecentCoins, format!("投币{}", count(data.recent_coins.len()))),
        (SpaceTab::FavoriteFolders, "收藏夹".to_owned()),
        (SpaceTab::ArticleFavorites, "图文".to_owned()),
        (SpaceTab::Following, "关注".to_owned()),
        (SpaceTab::Followers, "粉丝".to_owned()),
    ];
    let slot = w / items.len() as f32;
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
    for (i, (tab, label)) in items.iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let active = app.space_tab == *tab;
        p.text(
            Pos2::new(cx, rect.center().y - 2.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(if active { 11.0 } else { 10.0 }),
            if active { theme::TEXT } else { theme::TEXT2 },
        );
        if active {
            p.rect_filled(
                Rect::from_center_size(Pos2::new(cx, rect.bottom() - 3.0), Vec2::new(18.0, 3.0)),
                1.5,
                theme::PINK,
            );
        }
        let hit = Rect::from_min_size(
            Pos2::new(rect.left() + slot * i as f32, rect.top()),
            Vec2::new(slot, rect.height()),
        );
        if ui
            .interact(hit, egui::Id::new(("space-tab", i)), Sense::click())
            .clicked()
        {
            app.space_tab = *tab;
            app.space_folder = None;
        }
    }
}

/// 投稿：双列封面卡
fn videos(ui: &mut egui::Ui, app: &mut App, data: &SpaceData) {
    if data.videos.is_empty() {
        empty(ui, "TA 还没有投稿");
        return;
    }
    let w = ui.available_width();
    let gap = 7.0;
    let cw = ((w - 16.0 - gap) / 2.0).floor();
    let mut clicked = None;
    let cards: Vec<crate::api::Video> = data.videos.iter().map(SpaceVideo::as_video).collect();
    let mut i = 0;
    while i < cards.len() {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.spacing_mut().item_spacing.x = gap;
            if let Some(v) = cards.get(i) {
                if widgets::small_card(ui, v, &app.covers, cw).clicked() {
                    clicked = Some(v.id);
                }
            }
            if let Some(v) = cards.get(i + 1) {
                if widgets::small_card(ui, v, &app.covers, cw).clicked() {
                    clicked = Some(v.id);
                }
            }
        });
        ui.add_space(8.0);
        i += 2;
    }
    if let Some(id) = clicked {
        app.open_detail(id);
    }
}

/// 专栏：左封面 + 右标题/数据
fn articles(ui: &mut egui::Ui, app: &mut App, list: &[ArticleCard]) {
    if list.is_empty() {
        empty(ui, "TA 还没有写文章");
        return;
    }
    for a in list {
        let w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 82.0), Sense::hover());
        let cover = Rect::from_min_size(
            Pos2::new(rect.left() + 12.0, rect.top() + 6.0),
            Vec2::new(110.0, 66.0),
        );
        widgets::draw_cover(ui, cover, &a.cover_url, &app.covers, 4.0);
        let p = ui.painter();
        let text_x = cover.right() + 10.0;
        let text_w = (rect.right() - text_x - 10.0).max(60.0);
        let lines = widgets::wrap_lines(p, &a.title, 13.0, text_w, 2);
        let mut y = cover.top() + 2.0;
        for l in lines {
            p.text(
                Pos2::new(text_x, y),
                Align2::LEFT_TOP,
                l,
                FontId::proportional(13.0),
                theme::TEXT,
            );
            y += 19.0;
        }
        p.text(
            Pos2::new(text_x, cover.bottom() - 14.0),
            Align2::LEFT_TOP,
            format!(
                "{}阅读 · {}评论 · {}点赞",
                theme::human(a.view_count),
                theme::human(a.comment_count),
                theme::human(a.coin_count)
            ),
            FontId::proportional(10.0),
            theme::TEXT3,
        );
        p.line_segment(
            [
                Pos2::new(rect.left() + 12.0, rect.bottom()),
                Pos2::new(rect.right() - 12.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
    }
}

/// 动态：文字 + 时间 + 互动数（图文动态暂不铺图，先保证信息完整）
fn dynamics(ui: &mut egui::Ui, data: &SpaceData) {
    if data.dynamics.is_empty() {
        empty(ui, "TA 还没有发过动态");
        return;
    }
    let p_name = data.profile.nickname.clone();
    for d in &data.dynamics {
        let w = ui.available_width();
        let body = if !d.content.is_empty() {
            d.content.clone()
        } else {
            d.title.clone()
        };
        let lines = widgets::wrap_lines(ui.painter(), &body, 12.0, w - 28.0, 4);
        let h = 42.0 + lines.len() as f32 * 18.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
        let p = ui.painter();
        p.text(
            Pos2::new(rect.left() + 14.0, rect.top() + 8.0),
            Align2::LEFT_TOP,
            &p_name,
            FontId::proportional(12.0),
            theme::TEXT2,
        );
        if !d.created_at.is_empty() {
            p.text(
                Pos2::new(rect.right() - 14.0, rect.top() + 8.0),
                Align2::RIGHT_TOP,
                d.created_at.chars().take(16).collect::<String>(),
                FontId::proportional(9.0),
                theme::TEXT3,
            );
        }
        let mut y = rect.top() + 30.0;
        for l in &lines {
            p.text(
                Pos2::new(rect.left() + 14.0, y),
                Align2::LEFT_TOP,
                l,
                FontId::proportional(12.0),
                theme::TEXT,
            );
            y += 18.0;
        }
        p.text(
            Pos2::new(rect.left() + 14.0, rect.bottom() - 16.0),
            Align2::LEFT_TOP,
            format!(
                "赞 {} · 评论 {}{}",
                theme::human(d.like_count),
                theme::human(d.comment_count),
                if d.images.is_empty() {
                    String::new()
                } else {
                    format!(" · {} 张图", d.images.len())
                }
            ),
            FontId::proportional(10.0),
            theme::TEXT3,
        );
        p.line_segment(
            [
                Pos2::new(rect.left() + 14.0, rect.bottom()),
                Pos2::new(rect.right() - 14.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
    }
}

/// 收藏：横排视频行
fn favorites(ui: &mut egui::Ui, app: &mut App, list: &[FavoriteItem]) {
    if list.is_empty() {
        empty(ui, "收藏夹是空的，或者 TA 没有公开");
        return;
    }
    let mut clicked = None;
    for f in list {
        let v = crate::api::Video {
            id: f.id,
            user_id: f.uploader_id,
            title: f.title.clone(),
            cover_url: f.cover_url.clone(),
            duration: f.duration,
            play_count: f.play_count,
            danmaku_count: f.danmaku_count,
            comment_count: 0,
            like_count: 0,
        };
        if widgets::video_row(ui, &v, &app.covers).clicked() {
            clicked = Some(f.id);
        }
    }
    if let Some(id) = clicked {
        app.open_detail(id);
    }
}

/// 最近投币：横排视频行
fn recent_coins(ui: &mut egui::Ui, app: &mut App, list: &[CoinVideo]) {
    if list.is_empty() {
        empty(ui, "还没有投币记录，或者 TA 没有公开");
        return;
    }
    let mut clicked = None;
    for c in list {
        let v = crate::api::Video {
            id: c.id,
            user_id: 0,
            title: c.title.clone(),
            cover_url: c.cover_url.clone(),
            duration: c.duration as u64,
            play_count: c.play_count,
            danmaku_count: c.danmaku_count,
            comment_count: 0,
            like_count: 0,
        };
        if widgets::video_row(ui, &v, &app.covers).clicked() {
            clicked = Some(c.id);
        }
    }
    if let Some(id) = clicked {
        app.open_detail(id);
    }
}

/// 收藏夹列表：点开某个夹，下面展开该夹的视频
fn favorite_folders(ui: &mut egui::Ui, app: &mut App, data: &SpaceData) {
    if data.favorite_folders.is_empty() {
        empty(ui, "没有公开的收藏夹");
        return;
    }
    let mut open = None;
    for f in &data.favorite_folders {
        let selected = app.space_folder == Some(f.id);
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 52.0), Sense::click());
        let p = ui.painter();
        if selected {
            p.rect_filled(rect, 0.0, theme::PINK_SOFT);
        }
        p.text(
            Pos2::new(rect.left() + 14.0, rect.top() + 10.0),
            Align2::LEFT_TOP,
            &f.title,
            FontId::proportional(13.0),
            theme::TEXT,
        );
        let public = if f.is_public { "公开" } else { "私密" };
        p.text(
            Pos2::new(rect.left() + 14.0, rect.top() + 30.0),
            Align2::LEFT_TOP,
            format!("{} 个视频 · {public}", f.video_count),
            FontId::proportional(10.5),
            theme::TEXT3,
        );
        p.line_segment(
            [
                Pos2::new(rect.left() + 14.0, rect.bottom()),
                Pos2::new(rect.right() - 14.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
        if resp.clicked() {
            open = Some(f.id);
        }
    }
    if let Some(id) = open {
        app.open_space_folder(id);
    }
    let Some(folder_id) = app.space_folder else {
        return;
    };
    ui.add_space(4.0);
    match app.space_folder_state.clone() {
        LoadState::Loading => empty(ui, "加载收藏夹…"),
        LoadState::Err(e) => empty(ui, &format!("加载失败：{e}")),
        _ => {
            if app.space_folder_items.is_empty() {
                empty(ui, "这个收藏夹是空的");
            } else {
                let list = app.space_folder_items.clone();
                favorites(ui, app, &list);
            }
        }
    }
    let _ = folder_id;
}

/// 关注 / 粉丝列表：点头像进对方主页
fn users(ui: &mut egui::Ui, app: &mut App, list: &[FollowUser], empty_tip: &str) {
    if list.is_empty() {
        empty(ui, empty_tip);
        return;
    }
    let mut clicked = None;
    for u in list {
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 62.0), Sense::click());
        let name = if u.nickname.is_empty() {
            format!("用户 {}", u.user_id)
        } else {
            u.nickname.clone()
        };
        let av = Pos2::new(rect.left() + 28.0, rect.center().y);
        if u.avatar_url.is_empty() {
            widgets::avatar(ui.painter(), av, 17.0, &name, u.user_id);
        } else {
            widgets::avatar_tex(ui, av, 17.0, &name, &u.avatar_url, &app.covers, u.user_id);
        }
        let p = ui.painter();
        p.text(
            Pos2::new(rect.left() + 54.0, rect.top() + 14.0),
            Align2::LEFT_TOP,
            &name,
            FontId::proportional(13.0),
            theme::TEXT,
        );
        if !u.sign.is_empty() {
            let lines = widgets::wrap_lines(p, &u.sign, 11.0, w - 74.0, 1);
            if let Some(l) = lines.first() {
                p.text(
                    Pos2::new(rect.left() + 54.0, rect.top() + 34.0),
                    Align2::LEFT_TOP,
                    l,
                    FontId::proportional(11.0),
                    theme::TEXT3,
                );
            }
        }
        p.line_segment(
            [
                Pos2::new(rect.left() + 14.0, rect.bottom()),
                Pos2::new(rect.right() - 14.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
        if resp.clicked() {
            clicked = Some(u.user_id);
        }
    }
    if let Some(id) = clicked {
        app.open_space(id);
    }
}

fn empty(ui: &mut egui::Ui, tip: &str) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(egui::RichText::new(tip).color(theme::TEXT3).size(11.0));
    });
    ui.add_space(20.0);
}
