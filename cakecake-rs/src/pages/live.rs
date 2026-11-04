//! 直播：GET /api/v1/live/rooms 列表 + /api/v1/live/room/:id 详情
//!
//! 播放器和视频详情页共用一套：MediaPlayer 硬解 HLS，点画面出蒙层控件，
//! 全屏走横屏 + 沉浸式（见 `video::set_screen_landscape`）。

use crate::{
    app::{App, LoadState},
    api::LiveRoom,
    theme,
    video::Phase,
    widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

fn status_label(status: &str) -> (&'static str, Color32) {
    match status {
        "live" => ("直播中", theme::PINK),
        "idle" => ("未开播", theme::TEXT3),
        "ended" => ("已结束", theme::TEXT3),
        "banned" => ("已封禁", Color32::from_rgb(0xE2, 0x4B, 0x4A)),
        _ => ("未知", theme::TEXT3),
    }
}

pub fn list(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);

    match app.live_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(140.0);
                ui.label(
                    egui::RichText::new("正在请求 /api/v1/live/rooms …")
                        .color(theme::TEXT3)
                        .size(12.0),
                );
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_live();
            }
        }
        LoadState::Ready => {
            if app.live.is_empty() {
                if App::empty_state(ui, "当前没有直播间", "接口返回空列表") {
                    app.load_live();
                }
                return;
            }
            let mut clicked = None;
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("live-scroll")
                .show(ui, |ui| {
                    for r in &app.live {
                        if room_card(ui, r, &app.covers).clicked() {
                            clicked = Some(r.id);
                        }
                    }
                    ui.add_space(10.0);
                });
            if let Some(id) = clicked {
                app.open_live(id);
            }
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
        "直播",
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
        .interact(btn, egui::Id::new("live-refresh"), Sense::click())
        .clicked()
    {
        app.load_live();
    }
}

fn room_card(ui: &mut egui::Ui, r: &LiveRoom, covers: &crate::images::Covers) -> egui::Response {
    let w = ui.available_width() - 16.0;
    let cover_h = (w * 0.5625).round();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, cover_h + 48.0), Sense::click());
    let cover = Rect::from_min_size(Pos2::new(rect.left() + 8.0, rect.top() + 6.0), Vec2::new(w, cover_h));

    widgets::draw_cover(ui, cover, &r.cover_url, covers, 6.0);
    let p = ui.painter();

    let (label, color) = status_label(&r.status);
    let tag = Rect::from_min_size(Pos2::new(cover.left() + 8.0, cover.top() + 8.0), Vec2::new(48.0, 18.0));
    p.rect_filled(tag, 4.0, Color32::from_black_alpha(140));
    p.text(tag.center(), Align2::CENTER_CENTER, label, FontId::proportional(10.0), color);

    if r.viewer_count > 0 {
        widgets::chip(
            p,
            Pos2::new(cover.right() - 8.0, cover.bottom() - 8.0),
            Align2::RIGHT_BOTTOM,
            &format!("{} 人在看", theme::human(r.viewer_count as u64)),
            10.0,
        );
    }

    let host = if r.host_name.is_empty() {
        format!("UP {}", r.user_id)
    } else {
        r.host_name.clone()
    };
    p.text(
        Pos2::new(cover.left() + 2.0, cover.bottom() + 8.0),
        Align2::LEFT_TOP,
        &r.title,
        FontId::proportional(14.0),
        theme::TEXT,
    );
    widgets::avatar(p, Pos2::new(cover.left() + 10.0, cover.bottom() + 32.0), 8.0, &host, r.user_id);
    p.text(
        Pos2::new(cover.left() + 23.0, cover.bottom() + 26.0),
        Align2::LEFT_TOP,
        &host,
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    ui.allocate_rect(rect, Sense::hover());
    resp
}

pub fn room(ui: &mut egui::Ui, app: &mut App, id: u64) {
    // 横屏全屏播放：整页只留画面 + 会自动隐藏的蒙层控件（和视频详情页同一套）
    if app.player_fullscreen {
        fullscreen(ui, app);
        return;
    }

    match app.room_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            if widgets::back_bar(ui, "", "live-room-back") {
                app.back();
                return;
            }
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if widgets::back_bar(ui, "", "live-room-back") {
                app.back();
                return;
            }
            if App::empty_state(ui, "加载失败", &e) {
                app.open_live(id);
            }
        }
        LoadState::Ready => {
            let Some(r) = app.room.clone() else { return };
            let w = ui.available_width();
            let video_h = (w * 0.5625).round();
            let head_h = 48.0;
            // 主播栏 + 视频之外的高度全部让给聊天：
            // 标题走聊天里的系统提示，页面上不再单占一块，也就没有中间那片空白
            let chat_h = (ui.available_height() - head_h - video_h).max(120.0);

            egui::Panel::bottom("live-room-chat")
                .exact_size(chat_h)
                .resizable(false)
                .frame(
                    egui::Frame::NONE
                        .fill(theme::BG)
                        .stroke(Stroke::new(1.0, theme::LINE))
                        .inner_margin(egui::Margin::same(0)),
                )
                .show_inside(ui, |ui| chat_panel(ui, app));

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("room-scroll")
                .show(ui, |ui| {
                    host_bar(ui, app, &r);
                    player(ui, app, &r);
                });
        }
    }
}

/// 顶部主播栏：返回 + 头像 + 昵称 / 人气 + 开播状态。
/// 主播信息挪到最上面，页面中间就不用再挤一大块介绍区。
fn host_bar(ui: &mut egui::Ui, app: &mut App, r: &LiveRoom) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 48.0), Sense::hover());

    let host = if r.host_name.is_empty() {
        format!("UP {}", r.user_id)
    } else {
        r.host_name.clone()
    };

    // 返回
    let back_hit = Rect::from_min_size(Pos2::new(rect.left(), rect.top()), Vec2::new(52.0, 44.0));
    if ui
        .interact(back_hit, egui::Id::new("live-room-back"), Sense::click())
        .clicked()
    {
        app.back();
    }
    // 点头像/昵称进主播主页
    let name_hit = Rect::from_min_max(
        Pos2::new(rect.left() + 44.0, rect.top()),
        Pos2::new(rect.left() + 240.0, rect.top() + 44.0),
    );
    if ui
        .interact(name_hit, egui::Id::new("live-room-host"), Sense::click())
        .clicked()
        && r.user_id > 0
    {
        app.open_space(r.user_id);
    }

    let av = Pos2::new(rect.left() + 58.0, rect.center().y);
    if r.avatar_url.is_empty() {
        widgets::avatar(ui.painter(), av, 14.0, &host, r.user_id);
    } else {
        widgets::avatar_tex(
            ui,
            av,
            14.0,
            &host,
            &r.avatar_url,
            &app.covers,
            r.user_id,
        );
    }

    let p = ui.painter();
    // 返回箭头
    let c = Pos2::new(rect.left() + 20.0, rect.center().y);
    p.add(egui::Shape::line(
        vec![
            Pos2::new(c.x + 4.0, c.y - 6.0),
            Pos2::new(c.x - 3.0, c.y),
            Pos2::new(c.x + 4.0, c.y + 6.0),
        ],
        Stroke::new(1.8, theme::TEXT),
    ));
    p.text(
        Pos2::new(rect.left() + 80.0, rect.center().y - 7.0),
        Align2::LEFT_CENTER,
        &host,
        FontId::proportional(13.5),
        theme::TEXT,
    );
    let mut meta = format!("{} 人在看", r.viewer_count);
    if let Some(t) = &r.started_at {
        // 2026-11-02T20:15:00+08:00 -> 2026-11-02 20:15
        let shown = match t.split_once('T') {
            Some((d, rest)) => format!("{d} {}", rest.get(0..5).unwrap_or("")),
            None => t.clone(),
        };
        meta.push_str(&format!(" · 开播 {}", shown.trim_end()));
    }
    // 长标题把这一行挤掉一半也无所谓，正文里有完整信息
    if let Some(l) = widgets::wrap_lines(p, &meta, 10.0, w - 180.0, 1).first() {
        p.text(
            Pos2::new(rect.left() + 80.0, rect.center().y + 8.0),
            Align2::LEFT_CENTER,
            l,
            FontId::proportional(10.0),
            theme::TEXT3,
        );
    }
    let (label, color) = status_label(&r.status);
    let tag = Rect::from_min_size(
        Pos2::new(rect.right() - 64.0, rect.center().y - 10.0),
        Vec2::new(52.0, 20.0),
    );
    p.rect_stroke(tag, 10.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
    p.text(
        tag.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.0),
        color,
    );
}

// ───────────────────────── 播放区（竖屏 / 横屏） ─────────────────────────

/// 直播播放区（竖屏）：控件和视频详情页一样，点画面出现、3 秒自动隐藏。
/// 直播没有总时长，所以底部只留「播放/暂停 + LIVE + 全屏（横屏）」，不做进度条。
fn player(ui: &mut egui::Ui, app: &mut App, r: &LiveRoom) -> Rect {
    let w = ui.available_width();
    let h = (w * 0.5625).round();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    // 直播中且有 stream_key 才可能真出流；电脑上的 m3u8 还没探到就先别起解码器
    let is_live = r.status == "live" && !r.stream_key.is_empty();
    let ready = app.room_media_state == LoadState::Ready;
    let url = if is_live {
        live_url(app, &r.stream_key)
    } else {
        String::new()
    };
    let media_err = match &app.room_media_state {
        LoadState::Err(e) if is_live => Some(e.clone()),
        _ => None,
    };

    if is_live && ready {
        // 失败后不要每帧自动重启（会反复建解码器），点画面才重连
        if app.video.phase() != Phase::Failed {
            app.video.play_live(&url);
        }
        app.video.ui_in(ui, rect);
    } else if let Some(e) = &media_err {
        draw_media_error(ui, rect, w, e);
    } else if is_live {
        // 正在探 m3u8：还没帧可画，先用封面 + 等待文案顶住
        widgets::draw_cover(ui, rect, &r.cover_url, &app.covers, 0.0);
        let p = ui.painter();
        p.rect_filled(rect, 0.0, Color32::from_black_alpha(130));
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "正在连接电脑上的直播…",
            FontId::proportional(12.0),
            Color32::from_gray(225),
        );
    } else {
        // 未开播：封面压暗，主播一开播这页会自动接入
        widgets::draw_cover(ui, rect, &r.cover_url, &app.covers, 0.0);
        let p = ui.painter();
        p.rect_filled(rect, 0.0, Color32::from_black_alpha(70));
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "主播未开播",
            FontId::proportional(12.0),
            Color32::from_gray(225),
        );
    }

    let phase = app.video.phase();
    let playable = is_live && ready;

    // 聊天当弹幕飘过画面（和横屏同一套渲染）
    if app.danmaku_on && playable {
        draw_chat_danmaku(ui, rect, &app.chat.msgs);
    }

    // ── 控件位置（返回键在顶部主播栏上，这里只留播放键 / LIVE / 弹幕 / 全屏）──
    let cvis = app.fs_controls_visible();
    let cy = rect.bottom() - 18.0;
    let play_hit = Rect::from_center_size(
        Pos2::new(rect.left() + 18.0, cy),
        Vec2::new(34.0, 34.0),
    );
    let badge_pos = Pos2::new(rect.left() + 40.0, cy);
    let dm_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 62.0, cy),
        Vec2::new(34.0, 30.0),
    );
    let q_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 114.0, cy),
        Vec2::new(46.0, 30.0),
    );
    let fs_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 22.0, cy),
        Vec2::new(40.0, 32.0),
    );
    let center_hit = Rect::from_center_size(rect.center(), Vec2::splat(72.0));

    // 暂停 / 播放失败时中间常显一个播放键（点它继续 / 重连）
    if playable && matches!(phase, Phase::Paused | Phase::Failed) {
        let c = rect.center();
        let p = ui.painter();
        p.circle_filled(c, 24.0, Color32::from_black_alpha(110));
        widgets::play_icon(p, c, false, Color32::WHITE);
    }

    if cvis {
        let p = ui.painter();
        widgets::scrim(p, rect);
        if playable {
            widgets::play_icon(
                p,
                play_hit.center(),
                phase == Phase::Playing,
                Color32::WHITE,
            );
            live_badge(p, badge_pos, "直播中");
        } else {
            p.text(
                badge_pos,
                Align2::LEFT_CENTER,
                "未开播",
                FontId::proportional(11.0),
                Color32::from_gray(220),
            );
        }
        draw_quality(p, q_hit, app.live_sd_url.is_some(), app.live_quality_sd);
        draw_danmaku_toggle(p, dm_hit, app.danmaku_on);
        widgets::fullscreen_icon(p, fs_hit.center(), Color32::WHITE, false);
    }

    if resp.clicked() {
        if let Some(q) = resp.interact_pointer_pos() {
            if cvis {
                if play_hit.contains(q) {
                    if playable {
                        app.video.toggle_pause();
                    }
                    app.poke_controls();
                    return rect;
                }
                if dm_hit.contains(q) {
                    app.danmaku_on = !app.danmaku_on;
                    app.poke_controls();
                    return rect;
                }
                if q_hit.contains(q) && app.live_sd_url.is_some() {
                    app.live_quality_sd = !app.live_quality_sd;
                    app.poke_controls();
                    return rect;
                }
                if fs_hit.contains(q) {
                    app.enter_fullscreen();
                    return rect;
                }
            }
            // 媒体源探不到：点画面直接重试（和服务端地址切换同一套流程）
            if media_err.is_some() {
                app.retry_live_media();
                return rect;
            }
            // 播放器自己挂了：点画面重连，别让用户对着「播放失败」干等
            if playable && phase == Phase::Failed {
                app.video.play_live(&url);
                app.poke_controls();
                return rect;
            }
            if playable && !matches!(phase, Phase::Playing) && center_hit.contains(q) {
                app.video.toggle_pause();
                app.poke_controls();
                return rect;
            }
            if cvis {
                app.hide_controls();
            } else {
                app.poke_controls();
            }
        }
    }
    rect
}

/// 连不上直播源：把试过的地址摊开，点一下重试
fn draw_media_error(ui: &mut egui::Ui, rect: Rect, w: f32, e: &str) {
    let p = ui.painter();
    p.rect_filled(rect, 0.0, Color32::BLACK);
    let cp = p.with_clip_rect(rect.shrink(12.0));
    cp.text(
        Pos2::new(rect.left() + 14.0, rect.top() + 16.0),
        Align2::LEFT_TOP,
        "连不上直播源（点这里重试）",
        FontId::proportional(12.5),
        theme::UP_TAG,
    );
    let mut y = rect.top() + 42.0;
    for line in e.lines().take(6) {
        for l in widgets::wrap_lines(&cp, line, 10.0, w - 28.0, 2) {
            cp.text(
                Pos2::new(rect.left() + 14.0, y),
                Align2::LEFT_TOP,
                l,
                FontId::proportional(10.0),
                Color32::from_gray(170),
            );
            y += 15.0;
        }
    }
    cp.text(
        Pos2::new(rect.left() + 14.0, rect.bottom() - 26.0),
        Align2::LEFT_TOP,
        "同一 Wi-Fi 下应直连电脑 IP；可在「我的 → 设置」里改后端地址",
        FontId::proportional(9.5),
        Color32::from_gray(150),
    );
}

/// 当前该播的流：选了「流畅」且探到了转码流就用它，否则原画
fn live_url(app: &App, stream_key: &str) -> String {
    if app.live_quality_sd {
        if let Some(sd) = &app.live_sd_url {
            return sd.clone();
        }
    }
    app.room_media
        .clone()
        .unwrap_or_else(|| app.client.hls_url(stream_key))
}

/// 清晰度按钮：显示当前档位，点一下在原画 / 流畅之间切
fn draw_quality(p: &egui::Painter, hit: Rect, has_sd: bool, sd: bool) {
    let label = if sd && has_sd { "流畅" } else { "原画" };
    // 只有一路流时按钮变灰：点了也没得切，别让人以为坏了
    let color = if has_sd {
        Color32::WHITE
    } else {
        Color32::from_gray(150)
    };
    p.text(
        hit.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        color,
    );
}

/// 「弹」开关：亮粉色 = 开，白框 = 关（关的时候划一道斜杠）
fn draw_danmaku_toggle(p: &egui::Painter, hit: Rect, on: bool) {
    let color = if on { theme::PINK } else { Color32::from_gray(215) };
    p.rect_stroke(hit, 6.0, Stroke::new(1.0, color), StrokeKind::Inside);
    p.text(
        hit.center(),
        Align2::CENTER_CENTER,
        "弹",
        FontId::proportional(12.0),
        color,
    );
    if !on {
        p.line_segment(
            [
                Pos2::new(hit.left() + 8.0, hit.bottom() - 8.0),
                Pos2::new(hit.right() - 8.0, hit.top() + 8.0),
            ],
            Stroke::new(1.2, color),
        );
    }
}

/// 「● 直播中」：红点 + 白字
fn live_badge(p: &egui::Painter, pos: Pos2, text: &str) {
    p.circle_filled(
        Pos2::new(pos.x + 3.5, pos.y),
        3.5,
        Color32::from_rgb(0xF2, 0x4C, 0x5A),
    );
    p.text(
        Pos2::new(pos.x + 13.0, pos.y),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(11.0),
        Color32::WHITE,
    );
}

/// 横屏全屏播放：画面铺满，控件点一下才出现、3 秒后自动隐藏。
///
/// 横屏不放侧边聊天栏（占画面、还得跟键盘抢地方），
/// 聊天内容直接当弹幕从画面上飘过去，发消息用底部居中的那条输入框。
fn fullscreen(ui: &mut egui::Ui, app: &mut App) {
    let size = ui.available_size();
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    ui.painter().rect_filled(rect, 0.0, Color32::BLACK);

    let room = app.room.clone();
    let is_live = room
        .as_ref()
        .map(|r| r.status == "live" && !r.stream_key.is_empty())
        .unwrap_or(false);
    let ready = app.room_media_state == LoadState::Ready;
    let url = match &room {
        Some(r) if is_live => live_url(app, &r.stream_key),
        _ => String::new(),
    };

    // 系统导航栏在本机横屏时会一直被画在右边（沉浸式被 ROM 覆盖），
    // 右侧控件按它的实际宽度让位，否则会被压住点不到。
    let nav_w = crate::video::nav_inset_right_points(ui.ctx().pixels_per_point());
    let right = (rect.right() - nav_w).max(rect.left() + 160.0);
    let content = Rect::from_min_max(rect.min, Pos2::new(right, rect.bottom()));

    // 16:9 居中，多余部分留黑边，不拉伸画面
    let vr = widgets::fit16x9(content);
    if is_live && ready {
        if app.video.phase() != Phase::Failed {
            app.video.play_live(&url);
        }
        app.video.ui_in(ui, vr);
    } else if let Some(r) = &room {
        widgets::draw_cover(ui, vr, &r.cover_url, &app.covers, 0.0);
        ui.painter()
            .rect_filled(vr, 0.0, Color32::from_black_alpha(80));
    }

    // 还没画面时的说明（转屏那一下房间信息可能还在路上）
    let hint = if room.is_none() {
        Some("加载中…".to_owned())
    } else if !is_live {
        Some("主播未开播".to_owned())
    } else if matches!(app.room_media_state, LoadState::Err(_)) {
        Some("连不上直播源（点画面重试）".to_owned())
    } else if !ready {
        Some("正在连接电脑上的直播…".to_owned())
    } else {
        None
    };
    let phase = app.video.phase();
    if let Some(t) = hint {
        ui.painter().text(
            vr.center(),
            Align2::CENTER_CENTER,
            t,
            FontId::proportional(12.0),
            Color32::from_gray(225),
        );
    }

    // 聊天当弹幕：从右往左飘，9 秒走完一条
    if app.danmaku_on && room.is_some() {
        draw_chat_danmaku(ui, vr, &app.chat.msgs);
    }

    // 打字时键盘由原生 EditText 持有，内容每帧镜像回来自己画（和竖屏聊天同一套）
    if app.chat_ime {
        app.chat_sync_ime();
    }

    // ── 控件位置 ──
    let typing = app.chat_ime;
    // 键盘弹起时整排跟着上移，输入框始终停在键盘正上方
    let ime_h = if typing {
        crate::video::ime_points(ui.ctx().pixels_per_point())
    } else {
        0.0
    };
    let cvis = app.fs_controls_visible() || typing;
    let row_y = rect.bottom() - ime_h - 24.0;
    let back_hit = Rect::from_min_size(
        Pos2::new(rect.left() + 6.0, rect.top() + 4.0),
        Vec2::new(48.0, 40.0),
    );
    let play_hit = Rect::from_center_size(
        Pos2::new(rect.left() + 28.0, row_y),
        Vec2::new(48.0, 42.0),
    );
    let badge_pos = Pos2::new(rect.left() + 58.0, row_y);
    let dm_hit = Rect::from_center_size(
        Pos2::new(right - 100.0, row_y),
        Vec2::new(44.0, 34.0),
    );
    let q_hit = Rect::from_center_size(
        Pos2::new(right - 160.0, row_y),
        Vec2::new(54.0, 34.0),
    );
    let fs_hit = Rect::from_center_size(
        Pos2::new(right - 44.0, row_y),
        Vec2::new(56.0, 36.0),
    );
    // 输入框压在画面下方正中：两边给播放键和退出全屏留出位置
    let pill_w = ((right - rect.left()) - 380.0).clamp(140.0, 460.0);
    let pill = Rect::from_center_size(
        Pos2::new((rect.left() + right) * 0.5, row_y),
        Vec2::new(pill_w, 32.0),
    );
    let send_hit = Rect::from_center_size(
        Pos2::new(pill.right() + 34.0, row_y),
        Vec2::new(56.0, 36.0),
    );
    let center_hit = Rect::from_center_size(vr.center(), Vec2::splat(84.0));

    // 暂停时中间常显播放键
    if matches!(phase, Phase::Paused | Phase::Failed) && is_live && !typing {
        let c = vr.center();
        let p = ui.painter();
        p.circle_filled(c, 26.0, Color32::from_black_alpha(110));
        widgets::play_icon(p, c, false, Color32::WHITE);
    }

    if cvis {
        let p = ui.painter();
        widgets::scrim(p, rect);
        // 顶部：返回 + 标题
        p.add(egui::Shape::line(
            vec![
                Pos2::new(back_hit.center().x + 4.0, back_hit.center().y - 7.0),
                Pos2::new(back_hit.center().x - 4.0, back_hit.center().y),
                Pos2::new(back_hit.center().x + 4.0, back_hit.center().y + 7.0),
            ],
            Stroke::new(1.8, Color32::WHITE),
        ));
        if let Some(r) = &room {
            let lines = widgets::wrap_lines(p, &r.title, 13.0, pill.left() - back_hit.right() - 24.0, 1);
            if let Some(t) = lines.first() {
                p.text(
                    Pos2::new(back_hit.right() + 6.0, back_hit.center().y),
                    Align2::LEFT_CENTER,
                    t,
                    FontId::proportional(13.0),
                    Color32::from_gray(235),
                );
            }
        }
        // 底部：播放键 / LIVE / 输入框 / 退出全屏
        if is_live && ready {
            widgets::play_icon(
                p,
                play_hit.center(),
                phase == Phase::Playing,
                Color32::WHITE,
            );
            let secs = app.video.position_ms().max(0) as u64 / 1000;
            if secs > 0 && secs < 86_400 {
                live_badge(p, badge_pos, &format!("直播中 {}", theme::clock(secs)));
            } else {
                live_badge(p, badge_pos, "直播中");
            }
        }
        // 输入框：平时是提示语，打字时显示镜像回来的内容 + 光标
        p.rect_filled(
            pill,
            pill.height() / 2.0,
            if typing {
                Color32::from_black_alpha(190)
            } else {
                Color32::from_white_alpha(36)
            },
        );
        if typing {
            draw_input_text(p, pill, &app.chat.input, "说点什么…");
            let ready_send = !app.chat.input.trim().is_empty();
            p.circle_filled(
                send_hit.center(),
                16.0,
                if ready_send {
                    theme::PINK
                } else {
                    Color32::from_gray(90)
                },
            );
            draw_send_icon(p, send_hit.center(), Color32::WHITE);
        } else {
            p.text(
                Pos2::new(pill.left() + 14.0, pill.center().y),
                Align2::LEFT_CENTER,
                "发个弹幕聊两句…",
                FontId::proportional(12.0),
                Color32::from_gray(215),
            );
        }
        draw_quality(p, q_hit, app.live_sd_url.is_some(), app.live_quality_sd);
        draw_danmaku_toggle(p, dm_hit, app.danmaku_on);
        widgets::fullscreen_icon(p, fs_hit.center(), Color32::WHITE, true);
    }

    // ── 交互 ──
    if resp.clicked() {
        if let Some(q) = resp.interact_pointer_pos() {
            // 打字态：只处理发送 / 重新叫键盘 / 点空白收起
            if typing {
                if send_hit.contains(q) {
                    app.chat_ime = false;
                    crate::video::ime_close();
                    crate::video::ime_clear();
                    app.chat_send();
                    app.poke_controls();
                } else if pill.contains(q) {
                    crate::video::ime_open(&app.chat.input);
                } else {
                    app.chat_ime = false;
                    crate::video::ime_close();
                    app.poke_controls();
                }
                return;
            }
            if cvis {
                if back_hit.contains(q) {
                    app.exit_fullscreen();
                    return;
                }
                if play_hit.contains(q) {
                    if is_live && ready {
                        app.video.toggle_pause();
                    }
                    app.poke_controls();
                    return;
                }
                if dm_hit.contains(q) {
                    app.danmaku_on = !app.danmaku_on;
                    app.poke_controls();
                    return;
                }
                if q_hit.contains(q) && app.live_sd_url.is_some() {
                    app.live_quality_sd = !app.live_quality_sd;
                    app.poke_controls();
                    return;
                }
                if fs_hit.contains(q) {
                    app.exit_fullscreen();
                    return;
                }
                if pill.contains(q) {
                    app.chat_ime = true;
                    app.poke_controls();
                    crate::video::ime_open(&app.chat.input);
                    return;
                }
            }
            // 媒体源没了：点画面重新探一次
            if is_live && matches!(app.room_media_state, LoadState::Err(_)) {
                app.retry_live_media();
                return;
            }
            if is_live && ready && phase == Phase::Failed {
                app.video.play_live(&url);
                app.poke_controls();
                return;
            }
            if is_live && ready && !matches!(phase, Phase::Playing) && center_hit.contains(q) {
                app.video.toggle_pause();
                app.poke_controls();
                return;
            }
            if cvis {
                app.hide_controls();
            } else {
                app.poke_controls();
            }
        }
    }
}

/// 横屏没有侧边聊天栏，聊天内容当弹幕从右往左飘过画面
fn draw_chat_danmaku(ui: &mut egui::Ui, rect: Rect, msgs: &[crate::app::ChatMsg]) {
    /// 一条弹幕飞完全程用的时间
    const LIFE: f32 = 9.0;
    if rect.height() < 40.0 {
        return;
    }
    let lanes = (((rect.height() - 20.0) / 26.0).floor() as usize).max(1);
    let p = ui.painter().with_clip_rect(rect);
    for (i, m) in msgs.iter().enumerate() {
        // 系统提示（谁进了直播间）不当弹幕，只飘用户说的话
        if m.system || m.content.trim().is_empty() {
            continue;
        }
        let t = m.at.elapsed().as_secs_f32() / LIFE;
        if t >= 1.0 {
            continue;
        }
        let text = format!("{}：{}", m.user, m.content);
        let g = p.layout_no_wrap(text, FontId::proportional(13.0), Color32::WHITE);
        let w = g.size().x;
        let x = rect.right() - t * (rect.width() + w);
        let y = rect.top() + 10.0 + (i % lanes) as f32 * 26.0;
        // 黑描边先铺一层，亮画面上也读得清
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            p.galley(
                Pos2::new(x + dx, y + dy),
                g.clone(),
                Color32::from_black_alpha(170),
            );
        }
        p.galley(Pos2::new(x, y), g, Color32::WHITE);
    }
}

/// 输入框里的一行文字 + 光标
fn draw_input_text(p: &egui::Painter, pill: Rect, text: &str, hint: &str) {
    let tr = pill.shrink2(Vec2::new(14.0, 6.0));
    if text.is_empty() {
        p.text(
            Pos2::new(tr.left(), tr.center().y),
            Align2::LEFT_CENTER,
            hint,
            FontId::proportional(12.0),
            Color32::from_gray(170),
        );
        return;
    }
    let clipped = p.with_clip_rect(tr);
    let g = clipped.layout_no_wrap(text.to_owned(), FontId::proportional(12.0), Color32::WHITE);
    let x = if g.size().x > tr.width() {
        tr.right() - g.size().x
    } else {
        tr.left()
    };
    clipped.galley(
        Pos2::new(x, tr.center().y - g.size().y / 2.0),
        g.clone(),
        Color32::WHITE,
    );
    let caret_x = (tr.left() + g.size().x).min(tr.right());
    p.line_segment(
        [
            Pos2::new(caret_x, tr.center().y - 8.0),
            Pos2::new(caret_x, tr.center().y + 8.0),
        ],
        Stroke::new(1.5, theme::PINK),
    );
}

/// 发送键（纸飞机）
fn draw_send_icon(p: &egui::Painter, c: Pos2, color: Color32) {
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(c.x - 5.0, c.y - 6.0),
            Pos2::new(c.x + 6.0, c.y),
            Pos2::new(c.x - 5.0, c.y + 6.0),
        ],
        color,
        Stroke::NONE,
    ));
}


// ───────────────────────── 直播间聊天 ─────────────────────────

fn chat_panel(ui: &mut egui::Ui, app: &mut App) {
    // 先把原生输入框里的内容同步进来（安卓），再画列表和输入行
    app.chat_sync_ime();

    let w = ui.available_width();
    let (head, _) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::hover());
    let p = ui.painter();
    p.text(
        Pos2::new(head.left() + 14.0, head.center().y),
        Align2::LEFT_CENTER,
        "聊天",
        FontId::proportional(13.0),
        theme::TEXT,
    );
    let (meta, meta_color) = match &app.chat.state {
        LoadState::Loading => ("连接中…".to_owned(), theme::TEXT3),
        // 断线不隐藏输入框，只提示并给一个「重连」，否则用户以为发不出去是 App 卡了
        LoadState::Err(_) => ("已断开".to_owned(), theme::UP_TAG),
        _ if app.chat.online > 0 => (format!("{} 人在看", app.chat.online), theme::TEXT3),
        _ => ("已连接".to_owned(), theme::TEXT3),
    };
    let show_retry = matches!(app.chat.state, LoadState::Err(_));
    let retry_hit = Rect::from_min_size(
        Pos2::new(head.right() - 56.0, head.center().y - 10.0),
        Vec2::new(42.0, 20.0),
    );
    let meta_right = if show_retry { 64.0 } else { 14.0 };
    p.text(
        Pos2::new(head.right() - meta_right, head.center().y),
        Align2::RIGHT_CENTER,
        meta,
        FontId::proportional(10.0),
        meta_color,
    );
    if show_retry {
        p.rect_stroke(
            retry_hit,
            10.0,
            Stroke::new(1.0, theme::PINK),
            StrokeKind::Inside,
        );
        p.text(
            retry_hit.center(),
            Align2::CENTER_CENTER,
            "重连",
            FontId::proportional(10.0),
            theme::PINK,
        );
        if ui
            .interact(retry_hit, egui::Id::new("live-chat-retry"), Sense::click())
            .clicked()
        {
            app.chat_retry();
        }
    }

    let input_h = 42.0;
    // 键盘弹起后可用高度会被压得很扁：这里不能给列表兜底太高的下限，
    // 否则列表把输入行顶出面板，输入框正好落在键盘底下（横屏尤其明显）
    let list_h = (ui.available_height() - input_h).max(0.0);
    if list_h > 18.0 {
        egui::ScrollArea::vertical()
            .max_height(list_h)
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .id_salt("live-chat-list")
            .show(ui, |ui| {
                if app.chat.msgs.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.label(
                            egui::RichText::new("还没有人说话，来打个招呼")
                                .color(theme::TEXT3)
                                .size(11.0),
                        );
                    });
                    return;
                }
                for m in &app.chat.msgs {
                    if m.system {
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(14.0);
                            ui.label(egui::RichText::new(&m.content).color(theme::TEXT3).size(11.0));
                        });
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(14.0);
                            ui.label(
                                egui::RichText::new(format!("{}：", m.user))
                                    .color(theme::PINK)
                                    .size(12.0),
                            );
                            ui.label(egui::RichText::new(&m.content).color(theme::TEXT).size(12.0));
                        });
                    }
                }
            });
    }

    // 输入行
    ui.add_space(2.0);

    let mut want_send = false;
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        let total = ui.available_width();
        let field_w = (total - 84.0).max(80.0);

        // 安卓：WebView 之外的 NativeActivity 没有输入连接，egui 的 TextEdit
        // 收不到中文（软键盘只会吐数字键），所以聊天输入和评论/弹幕一样交给
        // Java 壳里那个隐藏 EditText，这里只把内容镜像回来自己画。
        #[cfg(target_os = "android")]
        {
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(field_w, 30.0), Sense::click());
            let p = ui.painter();
            p.rect_filled(rect, 8.0, theme::SURFACE);
            let inner = rect.shrink2(Vec2::new(8.0, 6.0));
            if app.chat.input.is_empty() {
                p.text(
                    Pos2::new(inner.left(), inner.center().y),
                    Align2::LEFT_CENTER,
                    "和大家聊聊…",
                    FontId::proportional(12.0),
                    theme::TEXT3,
                );
            } else {
                let clipped = p.with_clip_rect(inner);
                let g = clipped.layout_no_wrap(
                    app.chat.input.clone(),
                    FontId::proportional(12.0),
                    theme::TEXT,
                );
                let x = if g.size().x > inner.width() {
                    inner.right() - g.size().x
                } else {
                    inner.left()
                };
                clipped.galley(Pos2::new(x, inner.center().y - g.size().y / 2.0), g, theme::TEXT);
            }
            if resp.clicked() {
                // 每次点输入区都重新叫起键盘（系统收下去之后还能再点开）
                app.chat_ime = true;
                crate::video::ime_open(&app.chat.input);
            }
        }

        // 桌面：egui 自己的输入框，开发时直接敲键盘
        #[cfg(not(target_os = "android"))]
        {
            let field = ui.add_sized(
                Vec2::new(field_w, 30.0),
                egui::TextEdit::singleline(&mut app.chat.input)
                    .id(egui::Id::new("live-chat-input"))
                    .hint_text("和大家聊聊…")
                    .margin(egui::Margin::symmetric(8, 6)),
            );
            // 软键盘统一由 App 每帧汇总下发，页面只上报焦点
            app.note_focus(field.has_focus());
            if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                want_send = true;
            }
        }

        let btn = egui::Button::new(egui::RichText::new("发送").color(Color32::WHITE).size(12.0))
            .fill(theme::PINK)
            .corner_radius(15.0);
        if ui.add_sized(Vec2::new(68.0, 30.0), btn).clicked() {
            want_send = true;
        }
    });
    if want_send {
        app.chat_send();
    }
    ui.add_space(6.0);
}
