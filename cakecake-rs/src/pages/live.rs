//! 直播：GET /api/v1/live/rooms 列表 + /api/v1/live/room/:id 详情
//!
//! 后端只给了房间元信息（标题/主播/观看数/状态/开播时间），没有播放地址、
//! 没有聊天接口，所以这里就不放播放入口和假弹幕。

use crate::{
    app::{App, LoadState},
    api::LiveRoom,
    theme, widgets,
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
    // 返回
    let w = ui.available_width();
    let (bar, _) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::hover());
    let p = ui.painter();
    let c = Pos2::new(bar.left() + 20.0, bar.center().y);
    p.add(egui::Shape::line(
        vec![
            Pos2::new(c.x + 4.0, c.y - 6.0),
            Pos2::new(c.x - 3.0, c.y),
            Pos2::new(c.x + 4.0, c.y + 6.0),
        ],
        Stroke::new(1.8, theme::TEXT),
    ));
    let hit = Rect::from_min_size(Pos2::new(bar.left(), bar.top()), Vec2::new(60.0, bar.height()));
    if ui
        .interact(hit, egui::Id::new("room-back"), Sense::click())
        .clicked()
    {
        app.back();
    }

    match app.room_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.open_live(id);
            }
        }
        LoadState::Ready => {
            let Some(r) = app.room.clone() else { return };
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("room-scroll")
                .show(ui, |ui| {
                    let w = ui.available_width();
                    let h = (w * 0.5625).round();

                    // 直播中且有 stream_key：占位交给播放器逐帧自绘；
                    // 否则还是显示封面。
                    let is_live = r.status == "live" && !r.stream_key.is_empty();
                    let cover = if is_live {
                        let url = app.client.hls_url(&r.stream_key);
                        app.video.play(&url);
                        let resp = app.video.ui(ui, Vec2::new(w, h));
                        if resp.clicked() && app.video.phase() == crate::video::Phase::Failed {
                            // 仅失败态点一下重连，正常播放时误触不打断
                            app.video.stop();
                        }
                        resp.rect
                    } else {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
                        widgets::draw_cover(ui, rect, &r.cover_url, &app.covers, 0.0);
                        rect
                    };
                    let p = ui.painter();

                    let (label, color) = status_label(&r.status);
                    let tag = Rect::from_min_size(
                        Pos2::new(cover.left() + 10.0, cover.top() + 10.0),
                        Vec2::new(52.0, 20.0),
                    );
                    p.rect_filled(tag, 4.0, Color32::from_black_alpha(150));
                    p.text(
                        tag.center(),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::proportional(10.0),
                        color,
                    );

                    let mut y = cover.bottom() + 12.0;
                    let lines = widgets::wrap_lines(p, &r.title, 16.0, w - 28.0, 3);
                    for l in lines {
                        p.text(
                            Pos2::new(cover.left() + 14.0, y),
                            Align2::LEFT_TOP,
                            l,
                            FontId::proportional(16.0),
                            theme::TEXT,
                        );
                        y += 23.0;
                    }

                    let host = if r.host_name.is_empty() {
                        format!("UP {}", r.user_id)
                    } else {
                        r.host_name.clone()
                    };
                    widgets::avatar(
                        p,
                        Pos2::new(cover.left() + 24.0, y + 14.0),
                        14.0,
                        &host,
                        r.user_id,
                    );
                    p.text(
                        Pos2::new(cover.left() + 46.0, y + 6.0),
                        Align2::LEFT_TOP,
                        &host,
                        FontId::proportional(12.0),
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
                    p.text(
                        Pos2::new(cover.left() + 46.0, y + 24.0),
                        Align2::LEFT_TOP,
                        meta,
                        FontId::proportional(10.0),
                        theme::TEXT3,
                    );

                    ui.add_space(y + 48.0 - ui.cursor().top());
                    let p = ui.painter();
                    p.line_segment(
                        [
                            Pos2::new(cover.left(), y + 46.0),
                            Pos2::new(cover.right(), y + 46.0),
                        ],
                        Stroke::new(1.0, theme::LINE),
                    );
                    p.text(
                        Pos2::new(cover.left() + 14.0, y + 56.0),
                        Align2::LEFT_TOP,
                        if is_live {
                            format!("HLS {}", app.client.hls_url(&r.stream_key))
                        } else {
                            "主播未开播：开始推流后本页会自动接入 HLS 播放".to_owned()
                        },
                        FontId::proportional(10.0),
                        theme::TEXT3,
                    );
                    ui.add_space(40.0);
                });
        }
    }
}
