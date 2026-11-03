//! 视频详情：播放器 + 简介/评论 tab + UP 主区 + 操作栏 + 相关推荐
//!
//! 数据全部来自接口：
//!   GET /api/v1/videos/:id（含 video_url，可直接播放）
//!   GET /api/v1/videos/:id/danmaku · /comments
//!   GET /api/v1/feed/recommendation（相关推荐）
//!   POST /videos/:id/like · /coin · /favorite · POST /users/:id/follow
//!
//! 播放走系统的 MediaPlayer（SoC 硬解），所以详情页真的有画面，不是封面图。

use crate::{
    api::{Danmaku, VideoDetail},
    app::{Action, App, DetailTab, LoadState},
    theme,
    video::Phase,
    widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App, id: u64) {
    if widgets::back_bar(ui, "", "detail-back") {
        app.video.stop();
        app.back();
        return;
    }

    match app.detail_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            let mut retry = false;
            ui.vertical_centered(|ui| {
                ui.add_space(90.0);
                ui.label(egui::RichText::new("加载失败").color(theme::TEXT).size(14.0));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(&e).color(theme::TEXT3).size(11.0));
                ui.add_space(12.0);
                if ui.button("重试").clicked() {
                    retry = true;
                }
            });
            if retry {
                app.open_detail(id);
            }
        }
        LoadState::Ready => {
            let Some(data) = app.detail.take() else { return };
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("detail-scroll")
                .show(ui, |ui| {
                    player(ui, app, &data.video, &data.danmakus);
                    controls(ui, app);
                    tabs(ui, app, &data);
                    uploader_row(ui, app, &data.video);
                    title_block(ui, &data.video);
                    action_bar(ui, app, &data.video);
                    if let Some(msg) = app.action_msg.clone() {
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
                        });
                    }
                    ui.add_space(4.0);
                    match app.detail_tab {
                        DetailTab::Intro => intro(ui, app, &data),
                        DetailTab::Comments => comments(ui, &data.comments),
                    }
                    ui.add_space(10.0);
                });
            app.detail = Some(data);
        }
    }
}

// ───────────────────────── 播放器 ─────────────────────────

fn player(ui: &mut egui::Ui, app: &mut App, v: &VideoDetail, danmakus: &[Danmaku]) {
    let w = ui.available_width();
    let h = (w * 0.5625).round();
    let url = app.client.abs(&v.video_url);
    let has_src = !url.is_empty();
    let phase = app.video.phase();
    // 失败后不要每帧重试，等用户点一下再重连
    if has_src && phase != Phase::Failed {
        app.video.play(&url);
    }

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    // 有地址就把这块交给播放器：它负责建 Surface、起 MediaPlayer、逐帧画。
    // 帧没到之前是黑底 +「缓冲中…」，不能用封面盖住——盖住了回调就不会被调用。
    if has_src {
        app.video.ui_in(ui, rect);
    } else {
        widgets::draw_cover(ui, rect, &v.cover_url, &app.covers, 0.0);
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_black_alpha(60));
    }

    // 弹幕：跟着播放进度从右往左跑（右上角「弹」可以关掉）
    if app.danmaku_on && (phase == Phase::Playing || phase == Phase::Paused) && has_src {
        let pos = app.video.position_ms();
        if pos > 0 {
            draw_danmaku(ui, rect, danmakus, pos);
        }
    }

    // 中央播放/暂停按钮
    let c = rect.center();
    let p = ui.painter();
    match phase {
        Phase::Playing => {
            let hover = ui
                .input(|i| i.pointer.hover_pos().map(|q| rect.contains(q)).unwrap_or(false));
            if hover {
                p.circle_filled(c, 22.0, Color32::from_black_alpha(90));
                p.rect_filled(
                    Rect::from_center_size(Pos2::new(c.x - 5.0, c.y), Vec2::new(4.0, 18.0)),
                    2.0,
                    Color32::WHITE,
                );
                p.rect_filled(
                    Rect::from_center_size(Pos2::new(c.x + 5.0, c.y), Vec2::new(4.0, 18.0)),
                    2.0,
                    Color32::WHITE,
                );
            }
        }
        Phase::Paused | Phase::Failed => {
            p.circle_filled(c, 24.0, Color32::from_black_alpha(110));
            p.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(c.x - 5.0, c.y - 9.0),
                    Pos2::new(c.x - 5.0, c.y + 9.0),
                    Pos2::new(c.x + 9.0, c.y),
                ],
                Color32::WHITE,
                Stroke::NONE,
            ));
        }
        _ => {}
    }

    if resp.clicked() {
        match phase {
            Phase::Playing | Phase::Paused => app.video.toggle_pause(),
            Phase::Failed => app.video.play(&url),
            Phase::Idle | Phase::Connecting => {
                if has_src {
                    app.video.play(&url);
                }
            }
        }
    }
    if !has_src {
        ui.painter().text(
            rect.center() + Vec2::new(0.0, 34.0),
            Align2::CENTER_CENTER,
            "这条视频没有可播放文件",
            FontId::proportional(11.0),
            Color32::from_gray(225),
        );
    }
}

/// 进度条 + 时间 + 暂停键
fn controls(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 34.0), Sense::hover());
    let pos = app.video.position_ms().max(0);
    let dur = app.video.duration_ms().max(0);
    let p = ui.painter();

    // 暂停/播放小按钮
    let btn_c = Pos2::new(rect.left() + 16.0, rect.center().y);
    let playing = app.video.phase() == Phase::Playing;
    if playing {
        p.rect_filled(
            Rect::from_center_size(Pos2::new(btn_c.x - 3.0, btn_c.y), Vec2::new(3.0, 12.0)),
            1.0,
            theme::TEXT2,
        );
        p.rect_filled(
            Rect::from_center_size(Pos2::new(btn_c.x + 3.0, btn_c.y), Vec2::new(3.0, 12.0)),
            1.0,
            theme::TEXT2,
        );
    } else {
        p.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(btn_c.x - 4.0, btn_c.y - 6.0),
                Pos2::new(btn_c.x - 4.0, btn_c.y + 6.0),
                Pos2::new(btn_c.x + 6.0, btn_c.y),
            ],
            theme::TEXT2,
            Stroke::NONE,
        ));
    }

    let t1 = theme::clock((pos / 1000) as u64);
    let t2 = theme::clock((dur / 1000) as u64);
    p.text(
        Pos2::new(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        &t1,
        FontId::proportional(10.0),
        theme::TEXT3,
    );

    // 轨道
    let bar = Rect::from_min_max(
        Pos2::new(rect.left() + 78.0, rect.center().y - 1.5),
        Pos2::new(rect.right() - 52.0, rect.center().y + 1.5),
    );
    p.rect_filled(bar, 2.0, theme::SURFACE);
    let ratio = if dur > 0 {
        (pos as f32 / dur as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut played = bar;
    played.set_right(bar.left() + bar.width() * ratio);
    p.rect_filled(played, 2.0, theme::PINK);
    p.circle_filled(Pos2::new(played.right(), bar.center().y), 4.0, theme::PINK);

    p.text(
        Pos2::new(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        &t2,
        FontId::proportional(10.0),
        theme::TEXT3,
    );

    // 点时间轴 = 跳转；点左侧图标 = 暂停/继续
    let track_hit = Rect::from_min_max(
        Pos2::new(bar.left(), rect.top() + 6.0),
        Pos2::new(bar.right(), rect.bottom() - 6.0),
    );
    if ui
        .interact(track_hit, egui::Id::new("detail-seek"), Sense::click())
        .clicked()
    {
        if dur > 0 {
            if let Some(px) = ui.ctx().pointer_interact_pos() {
                let r = ((px.x - bar.left()) / bar.width()).clamp(0.0, 1.0);
                app.video.seek_to((dur as f32 * r) as i64);
            }
        }
    }
    let btn_hit = Rect::from_center_size(btn_c, Vec2::new(30.0, 30.0));
    if ui
        .interact(btn_hit, egui::Id::new("detail-toggle"), Sense::click())
        .clicked()
    {
        app.video.toggle_pause();
    }
}

fn draw_danmaku(ui: &mut egui::Ui, rect: Rect, list: &[Danmaku], pos_ms: i64) {
    if pos_ms <= 0 || list.is_empty() {
        return;
    }
    const LIFE: i64 = 7000;
    let lanes = (((rect.height() - 16.0) / 20.0).floor() as usize).max(1);
    let p = ui.painter().with_clip_rect(rect);
    for (i, d) in list.iter().enumerate() {
        let start = (d.video_time * 1000.0) as i64;
        let dt = pos_ms - start;
        if dt < 0 || dt > LIFE {
            continue;
        }
        let t = dt as f32 / LIFE as f32;
        let galley = p.layout_no_wrap(d.content.clone(), FontId::proportional(12.0), Color32::WHITE);
        let text_w = galley.size().x;
        let x = rect.right() - t * (rect.width() + text_w);
        let y = rect.top() + 8.0 + (i % lanes) as f32 * 20.0;
        // 先描一层黑边保证在任何画面上都读得清，再画白字
        p.galley(
            Pos2::new(x + 1.0, y + 1.0),
            galley.clone(),
            Color32::from_black_alpha(150),
        );
        p.galley(Pos2::new(x, y), galley, Color32::WHITE);
    }
}

// ───────────────────────── tab / 信息区 ─────────────────────────

fn tabs(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::hover());
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    // 左：简介 / 评论；右：弹幕输入框 + 「弹」开关
    let n = data.comments.len();
    let items = [
        (DetailTab::Intro, "简介".to_owned()),
        (DetailTab::Comments, format!("评论 {n}")),
    ];
    let slot = 70.0;
    for (i, (tab, label)) in items.iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let active = app.detail_tab == *tab;
        p.text(
            Pos2::new(cx, rect.center().y - 3.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(if active { 14.0 } else { 13.0 }),
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
            .interact(hit, egui::Id::new(("detail-tab", i)), Sense::click())
            .clicked()
        {
            app.detail_tab = *tab;
        }
    }

    // 弹幕开关（最右）
    let toggle = Rect::from_min_size(
        Pos2::new(rect.right() - 40.0, rect.center().y - 13.0),
        Vec2::new(30.0, 26.0),
    );
    let p = ui.painter();
    if app.danmaku_on {
        p.rect_stroke(toggle, 6.0, Stroke::new(1.0, theme::PINK), StrokeKind::Inside);
        p.text(
            toggle.center(),
            Align2::CENTER_CENTER,
            "弹",
            FontId::proportional(12.0),
            theme::PINK,
        );
    } else {
        p.rect_stroke(toggle, 6.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            toggle.center(),
            Align2::CENTER_CENTER,
            "弹",
            FontId::proportional(12.0),
            theme::TEXT3,
        );
        // 关掉时画一道斜杠
        p.line_segment(
            [
                Pos2::new(toggle.left() + 8.0, toggle.bottom() - 8.0),
                Pos2::new(toggle.right() - 8.0, toggle.top() + 8.0),
            ],
            Stroke::new(1.2, theme::TEXT3),
        );
    }
    if ui
        .interact(toggle, egui::Id::new("detail-danmaku-toggle"), Sense::click())
        .clicked()
    {
        app.danmaku_on = !app.danmaku_on;
    }

    // 弹幕输入框（tab 与开关之间）
    let input = Rect::from_min_max(
        Pos2::new(rect.left() + slot * 2.0 + 4.0, rect.center().y - 13.0),
        Pos2::new(toggle.left() - 8.0, rect.center().y + 13.0),
    );
    if input.width() > 60.0 {
        let p = ui.painter();
        p.rect_filled(input, 13.0, theme::SURFACE);
        let field = ui.put(
            input.shrink2(Vec2::new(10.0, 4.0)),
            egui::TextEdit::singleline(&mut app.danmaku_text)
                .frame(egui::Frame::NONE)
                .hint_text("发个弹幕见证当下")
                .font(FontId::proportional(11.0)),
        );
        let focused = field.has_focus();
        if app.ime_on != focused {
            app.ime_on = focused;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::IMEAllowed(focused));
        }
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.send_danmaku(data.video.id);
        }
    }
}

fn uploader_row(ui: &mut egui::Ui, app: &mut App, v: &VideoDetail) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 64.0), Sense::hover());
    let name = if v.uploader.is_empty() {
        format!("UP {}", v.user_id)
    } else {
        v.uploader.clone()
    };
    let av = Pos2::new(rect.left() + 30.0, rect.center().y);
    if v.uploader_avatar_url.is_empty() {
        widgets::avatar(ui.painter(), av, 17.0, &name, v.user_id);
    } else {
        widgets::avatar_tex(
            ui,
            av,
            17.0,
            &name,
            &v.uploader_avatar_url,
            &app.covers,
            v.user_id,
        );
    }
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 56.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        &name,
        FontId::proportional(14.0),
        theme::TEXT,
    );
    p.text(
        Pos2::new(rect.left() + 56.0, rect.top() + 32.0),
        Align2::LEFT_TOP,
        format!(
            "{}粉丝 · {}视频",
            theme::human(v.uploader_follower_count),
            theme::human(v.uploader_published_count)
        ),
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    // 关注按钮
    let btn = Rect::from_min_size(
        Pos2::new(rect.right() - 76.0, rect.center().y - 13.0),
        Vec2::new(64.0, 26.0),
    );
    if v.followed_by_me {
        p.rect_stroke(btn, 13.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "已关注",
            FontId::proportional(11.0),
            theme::TEXT3,
        );
    } else {
        p.rect_filled(btn, 13.0, theme::PINK);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "+ 关注",
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    }
    if ui
        .interact(btn, egui::Id::new("detail-follow"), Sense::click())
        .clicked()
    {
        app.act(v.id, Action::Follow);
    }
}

fn title_block(ui: &mut egui::Ui, v: &VideoDetail) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 76.0), Sense::hover());
    let p = ui.painter();
    let lines = widgets::wrap_lines(p, &v.title, 16.0, w - 28.0, 2);
    let mut y = rect.top() + 2.0;
    for l in lines {
        p.text(
            Pos2::new(rect.left() + 14.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(16.0),
            theme::TEXT,
        );
        y += 23.0;
    }
    let mut meta = format!(
        "{}播放 · {}弹幕",
        theme::human(v.play_count),
        theme::human(v.danmaku_count)
    );
    if !v.created_at.is_empty() {
        meta.push_str(&format!(" · {}", v.created_at));
    }
    if v.watching_count > 0 {
        meta.push_str(&format!(" · {}人正在看", v.watching_count));
    }
    p.text(
        Pos2::new(rect.left() + 14.0, y + 4.0),
        Align2::LEFT_TOP,
        meta,
        FontId::proportional(11.0),
        theme::TEXT3,
    );
}

fn action_bar(ui: &mut egui::Ui, app: &mut App, v: &VideoDetail) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 56.0), Sense::hover());
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.top()),
            Pos2::new(rect.right(), rect.top()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    let items: [(&str, u64, bool, Action); 3] = [
        ("点赞", v.like_count, v.liked_by_me, Action::Like),
        ("投币", v.coin_count, v.coined_by_me, Action::Coin),
        ("收藏", v.fav_count, v.favorited_by_me, Action::Favorite),
    ];
    let slot = w / 3.0;
    for (i, (label, count, on, act)) in items.iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let color = if *on { theme::PINK } else { theme::TEXT2 };
        p.text(
            Pos2::new(cx, rect.top() + 14.0),
            Align2::CENTER_TOP,
            theme::human(*count),
            FontId::proportional(14.0),
            color,
        );
        p.text(
            Pos2::new(cx, rect.top() + 34.0),
            Align2::CENTER_TOP,
            *label,
            FontId::proportional(11.0),
            theme::TEXT3,
        );
        let hit = Rect::from_min_size(
            Pos2::new(rect.left() + slot * i as f32, rect.top()),
            Vec2::new(slot, rect.height()),
        );
        if ui
            .interact(hit, egui::Id::new(("detail-act", i)), Sense::click())
            .clicked()
        {
            app.act(v.id, *act);
        }
    }
}

fn intro(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    let v = &data.video;
    if !v.tags.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.add_space(14.0);
            for t in &v.tags {
                let btn = egui::Button::new(
                    egui::RichText::new(format!("#{t}"))
                        .color(theme::TEXT2)
                        .size(11.0),
                )
                .fill(theme::SURFACE)
                .corner_radius(11.0)
                .min_size(Vec2::new(0.0, 24.0));
                ui.add(btn);
            }
        });
        ui.add_space(6.0);
    }
    if !v.description.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new(&v.description)
                    .color(theme::TEXT2)
                    .size(12.0),
            );
        });
        ui.add_space(8.0);
    }
    if data.danmakus.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("这条视频还没有弹幕")
                    .color(theme::TEXT3)
                    .size(11.0),
            );
        });
    }

    // 相关推荐
    if data.related.is_empty() {
        return;
    }
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("相关推荐")
                .color(theme::TEXT)
                .size(14.0)
                .strong(),
        );
    });
    ui.add_space(2.0);
    let mut clicked = None;
    for r in &data.related {
        if widgets::video_row(ui, r, &app.covers).clicked() {
            clicked = Some(r.id);
        }
    }
    if let Some(next) = clicked {
        app.open_detail(next);
    }
}

fn comments(ui: &mut egui::Ui, list: &[crate::api::Comment]) {
    if list.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("还没有评论，来抢个沙发")
                    .color(theme::TEXT3)
                    .size(11.0),
            );
        });
        return;
    }
    for c in list {
        let w = ui.available_width();
        let h = if c.content.chars().count() > 26 { 86.0 } else { 70.0 };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
        let p = ui.painter();
        let name = if c.username.is_empty() {
            "匿名".to_owned()
        } else {
            c.username.clone()
        };
        widgets::avatar(
            p,
            Pos2::new(rect.left() + 24.0, rect.top() + 20.0),
            12.0,
            &name,
            7,
        );
        p.text(
            Pos2::new(rect.left() + 44.0, rect.top() + 12.0),
            Align2::LEFT_TOP,
            &name,
            FontId::proportional(12.0),
            theme::TEXT2,
        );
        if c.is_by_uploader {
            let tag = Rect::from_min_size(
                Pos2::new(
                    rect.left() + 46.0 + name.chars().count() as f32 * 13.0,
                    rect.top() + 13.0,
                ),
                Vec2::new(20.0, 12.0),
            );
            p.rect_filled(tag, 2.0, theme::UP_TAG);
            p.text(
                tag.center(),
                Align2::CENTER_CENTER,
                "UP",
                FontId::proportional(8.0),
                Color32::WHITE,
            );
        }
        p.text(
            Pos2::new(rect.right() - 14.0, rect.top() + 12.0),
            Align2::RIGHT_TOP,
            format!("赞 {}", theme::human(c.like_count)),
            FontId::proportional(10.0),
            theme::TEXT3,
        );
        let lines = widgets::wrap_lines(p, &c.content, 12.0, w - 70.0, 3);
        let mut y = rect.top() + 32.0;
        for l in lines {
            p.text(
                Pos2::new(rect.left() + 44.0, y),
                Align2::LEFT_TOP,
                l,
                FontId::proportional(12.0),
                theme::TEXT,
            );
            y += 17.0;
        }
        p.line_segment(
            [
                Pos2::new(rect.left() + 20.0, rect.bottom()),
                Pos2::new(rect.right() - 14.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
    }
}
