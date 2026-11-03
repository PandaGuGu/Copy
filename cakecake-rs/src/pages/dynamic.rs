//! 动态（关注页）：我关注的人 + 关注的人的投稿 / 动态流
//!
//! 数据全部要登录态：
//!   GET /api/v1/users/me/followings   我关注的人
//!   GET /api/v1/feed/subscription     关注的人的投稿（model.Video 原样返回）
//!   GET /api/v1/dynamics/following    关注的人发的文字/图文动态
//!
//! 投稿和动态按时间归并成一条流，对齐 B 站「关注」页的样子。

use crate::{
    api::{DynItem, FollowUser, SubVideo},
    app::{App, LoadState, Tab},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

/// 归并后的一条：要么是投稿，要么是文字动态
enum Entry<'a> {
    Video(&'a SubVideo),
    Dyn(&'a DynItem),
}

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);

    if !app.client.logged_in() {
        need_login(ui, app);
        return;
    }

    match app.dynamic_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            center_hint(ui, "正在拉取关注动态 …");
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.dynamic_state = LoadState::Idle;
                app.load_dynamic();
            }
        }
        LoadState::Ready => {
            if app.dynamic.followings.is_empty() {
                if App::empty_state(ui, "还没有关注任何人", "去首页找几个 UP 主关注一下") {
                    app.dynamic_state = LoadState::Idle;
                    app.load_dynamic();
                }
                return;
            }
            follow_strip(ui, app);
            sub_tabs(ui, app);
            feed(ui, app);
        }
    }
}

fn header(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 48.0), Sense::hover());
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        "动态",
        FontId::proportional(17.0),
        theme::TEXT,
    );
    let btn = Rect::from_min_size(
        Pos2::new(rect.right() - 62.0, rect.center().y - 12.0),
        Vec2::new(50.0, 24.0),
    );
    p.rect_stroke(btn, 12.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
    p.text(
        btn.center(),
        Align2::CENTER_CENTER,
        "刷新",
        FontId::proportional(11.0),
        theme::TEXT2,
    );
    if ui
        .interact(btn, egui::Id::new("dyn-refresh"), Sense::click())
        .clicked()
    {
        app.dynamic_state = LoadState::Idle;
        app.load_dynamic();
    }
}

fn need_login(ui: &mut egui::Ui, app: &mut App) {
    ui.add_space(120.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("登录后查看关注动态")
                .color(theme::TEXT2)
                .size(14.0),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("动态来自你关注的 UP 主：投稿和日常都在这里")
                .color(theme::TEXT3)
                .size(11.0),
        );
        ui.add_space(16.0);
        let btn = egui::Button::new(egui::RichText::new("去登录").color(Color32::WHITE).size(13.0))
            .fill(theme::PINK)
            .corner_radius(16.0)
            .min_size(Vec2::new(96.0, 32.0));
        if ui.add(btn).clicked() {
            app.navigate(crate::app::Route::Tab(Tab::Mine));
        }
    });
}

/// 「我关注的人」横向头像条
fn follow_strip(ui: &mut egui::Ui, app: &mut App) {
    let list = app.dynamic.followings.clone();
    if list.is_empty() {
        return;
    }
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 28.0), Sense::hover());
    ui.painter().text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        "我关注的人",
        FontId::proportional(13.0),
        theme::TEXT,
    );
    ui.painter().text(
        Pos2::new(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        format!("{} 位", list.len()),
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    let row_h = 74.0;
    // 横向滚动条必须限高，否则 ScrollArea 会吃掉整页剩余高度，下面的 tab 和动态都看不见
    egui::ScrollArea::horizontal()
        .max_height(row_h)
        .auto_shrink([false, true])
        .id_salt("dyn-follow-strip")
        .show(ui, |ui: &mut egui::Ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.spacing_mut().item_spacing.x = 10.0;
                for u in &list {
                    cell(ui, u, &app.covers);
                }
                ui.add_space(6.0);
            });
        });
}

fn cell(ui: &mut egui::Ui, u: &FollowUser, covers: &crate::images::Covers) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(58.0, 72.0), Sense::hover());
    let c = Pos2::new(rect.center().x, rect.top() + 24.0);
    if u.avatar_url.is_empty() {
        widgets::avatar(ui.painter(), c, 23.0, &u.nickname, u.user_id);
    } else {
        widgets::avatar_tex(ui, c, 23.0, &u.nickname, &u.avatar_url, covers, u.user_id);
    }
    let p = ui.painter();
    let name = if u.nickname.is_empty() {
        format!("UP {}", u.user_id)
    } else {
        u.nickname.clone()
    };
    let lines = widgets::wrap_lines(p, &name, 10.0, 54.0, 1);
    if let Some(l) = lines.first() {
        p.text(
            Pos2::new(rect.center().x, rect.top() + 52.0),
            Align2::CENTER_TOP,
            l,
            FontId::proportional(10.0),
            theme::TEXT2,
        );
    }
}

fn sub_tabs(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 38.0), Sense::hover());
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
    let slot = w / 2.0;
    for (i, label) in ["全部", "视频"].iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let active = app.dynamic_video_only == (i == 1);
        p.text(
            Pos2::new(cx, rect.center().y - 2.0),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(if active { 15.0 } else { 14.0 }),
            if active { theme::TEXT } else { theme::TEXT2 },
        );
        if active {
            p.rect_filled(
                Rect::from_center_size(Pos2::new(cx, rect.bottom() - 3.0), Vec2::new(20.0, 3.0)),
                1.5,
                theme::PINK,
            );
        }
        let hit = Rect::from_min_size(
            Pos2::new(rect.left() + slot * i as f32, rect.top()),
            Vec2::new(slot, rect.height()),
        );
        if ui
            .interact(hit, egui::Id::new(("dyn-tab", i)), Sense::click())
            .clicked()
        {
            app.dynamic_video_only = i == 1;
        }
    }
}

fn feed(ui: &mut egui::Ui, app: &mut App) {
    let mut entries: Vec<(String, Entry)> = Vec::new();
    if !app.dynamic_video_only {
        for d in &app.dynamic.dynamics {
            entries.push((short_time(&d.created_at), Entry::Dyn(d)));
        }
    }
    for v in &app.dynamic.videos {
        entries.push((short_time(&v.created_at), Entry::Video(v)));
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    if entries.is_empty() {
        center_hint(ui, "关注的 UP 主最近没有更新");
        return;
    }

    let mut open_video = None;

    let mut open_dyn: Option<u64> = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("dyn-feed")
        .show(ui, |ui| {
            for (time, e) in &entries {
                match e {
                    Entry::Video(v) => {
                        if video_card(ui, v, time, &app.dynamic.followings, &app.covers).clicked() {
                            open_video = Some(v.id);
                        }
                    }
                    Entry::Dyn(d) => {
                        if dyn_card(ui, d, time, &app.covers).clicked() {
                            open_dyn = Some(d.id);
                        }
                    }
                }
            }
            ui.add_space(14.0);
        });
    if let Some(id) = open_video {
        app.open_detail(id);
    }
}

/// 投稿卡：头像 +「投稿了视频」+ 封面 + 标题 + 数据
fn video_card(
    ui: &mut egui::Ui,
    v: &SubVideo,
    time: &str,
    users: &[FollowUser],
    covers: &crate::images::Covers,
) -> egui::Response {
    let w = ui.available_width();
    let author = users
        .iter()
        .find(|u| u.user_id == v.user_id)
        .cloned()
        .unwrap_or_default();
    let name = if author.nickname.is_empty() {
        format!("UP {}", v.user_id)
    } else {
        author.nickname.clone()
    };

    let cover_w = w - 24.0;
    let cover_h = (cover_w * 0.5625).round();
    let total_h = 52.0 + cover_h + 58.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, total_h), Sense::click());

    let av = Pos2::new(rect.left() + 26.0, rect.top() + 20.0);
    if author.avatar_url.is_empty() {
        widgets::avatar(ui.painter(), av, 14.0, &name, v.user_id);
    } else {
        widgets::avatar_tex(ui, av, 14.0, &name, &author.avatar_url, covers, v.user_id);
    }
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 46.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        &name,
        FontId::proportional(13.0),
        theme::PINK,
    );
    p.text(
        Pos2::new(rect.left() + 46.0, rect.top() + 28.0),
        Align2::LEFT_TOP,
        format!("{time} · 投稿了视频"),
        FontId::proportional(10.0),
        theme::TEXT3,
    );

    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.top() + 50.0),
        Vec2::new(cover_w, cover_h),
    );
    widgets::draw_cover(ui, cover, &v.cover_url, covers, 6.0);
    let p = ui.painter();
    p.circle_filled(cover.center(), 20.0, Color32::from_black_alpha(70));
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(cover.center().x - 5.0, cover.center().y - 9.0),
            Pos2::new(cover.center().x - 5.0, cover.center().y + 9.0),
            Pos2::new(cover.center().x + 9.0, cover.center().y),
        ],
        Color32::from_white_alpha(220),
        Stroke::NONE,
    ));
    widgets::chip(
        p,
        Pos2::new(cover.left() + 8.0, cover.bottom() - 8.0),
        Align2::LEFT_BOTTOM,
        &format!(
            "{}  {}播放 · {}弹幕",
            theme::clock(v.duration as u64),
            theme::human(v.play_count),
            theme::human(v.danmaku_count)
        ),
        10.0,
    );

    let lines = widgets::wrap_lines(p, &v.title, 15.0, cover_w, 2);
    let mut y = cover.bottom() + 8.0;
    for l in lines {
        p.text(
            Pos2::new(rect.left() + 12.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(15.0),
            theme::TEXT,
        );
        y += 21.0;
    }
    p.text(
        Pos2::new(rect.left() + 12.0, y + 2.0),
        Align2::LEFT_TOP,
        format!(
            "评论 {} · 点赞 {}",
            theme::human(v.comment_count),
            theme::human(v.like_count)
        ),
        FontId::proportional(10.0),
        theme::TEXT3,
    );
    ui.allocate_rect(rect, Sense::hover());
    resp
}

/// 文字/图文动态卡
fn dyn_card(
    ui: &mut egui::Ui,
    d: &DynItem,
    time: &str,
    covers: &crate::images::Covers,
) -> egui::Response {
    let w = ui.available_width();
    let body_lines = widgets::wrap_lines(ui.painter(), &d.content, 13.0, w - 68.0, 4);
    let imgs = d.images.len().min(3);
    let h = 46.0 + body_lines.len() as f32 * 19.0 + if imgs > 0 { 96.0 } else { 0.0 } + 34.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    let name = if d.author_name.is_empty() {
        format!("UP {}", d.author_id)
    } else {
        d.author_name.clone()
    };
    let av = Pos2::new(rect.left() + 26.0, rect.top() + 20.0);
    if d.author_avatar.is_empty() {
        widgets::avatar(ui.painter(), av, 14.0, &name, d.author_id);
    } else {
        widgets::avatar_tex(ui, av, 14.0, &name, &d.author_avatar, covers, d.author_id);
    }
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 46.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        &name,
        FontId::proportional(13.0),
        theme::PINK,
    );
    p.text(
        Pos2::new(rect.left() + 46.0, rect.top() + 28.0),
        Align2::LEFT_TOP,
        format!("{time} · 发布了动态"),
        FontId::proportional(10.0),
        theme::TEXT3,
    );

    let mut y = rect.top() + 46.0;
    for l in &body_lines {
        p.text(
            Pos2::new(rect.left() + 46.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(13.0),
            theme::TEXT,
        );
        y += 19.0;
    }

    if imgs > 0 {
        let cw = (w - 46.0 - 24.0 - (imgs as f32 - 1.0) * 6.0) / imgs as f32;
        for (i, url) in d.images.iter().take(imgs).enumerate() {
            let r = Rect::from_min_size(
                Pos2::new(rect.left() + 46.0 + i as f32 * (cw + 6.0), y + 4.0),
                Vec2::new(cw, cw * 0.75),
            );
            widgets::draw_cover(ui, r, url, covers, 4.0);
        }
        y += cw * 0.75 + 8.0;
    }

    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 46.0, y + 4.0),
        Align2::LEFT_TOP,
        format!(
            "评论 {} · 点赞 {}",
            theme::human(d.comment_count),
            theme::human(d.like_count)
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
    ui.allocate_rect(rect, Sense::hover());
    resp
}

/// "2026-10-31 23:25:10" / "2026-10-31T23:25:10+08:00" → "10-31 23:25"
fn short_time(s: &str) -> String {
    let s = s.trim();
    if s.len() < 16 {
        return s.to_owned();
    }
    let (date, rest) = match s.split_once('T') {
        Some((d, r)) => (d, r),
        None => match s.split_once(' ') {
            Some((d, r)) => (d, r),
            None => return s.to_owned(),
        },
    };
    let hm: String = rest.chars().take(5).collect();
    let md: String = date.chars().skip(5).collect();
    format!("{md} {hm}")
}

fn center_hint(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(130.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}
