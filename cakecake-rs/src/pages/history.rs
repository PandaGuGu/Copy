//! 观看历史 + 断点续播
//!
//!   GET    /api/v1/users/me/view-history
//!   POST   /api/v1/videos/:id/view-history（详情页每 20 秒上报进度）
//!   DELETE /api/v1/users/me/view-history[/:videoId|/live/:id|/articles/:id]
//!   GET/PUT /api/v1/users/me/view-history/settings（记录开关）
//!
//! 列表里混有视频/直播/专栏：视频点开走详情（自动续播），直播进直播间，
//! 专栏 App 里没有页面，点了给提示。

use crate::{
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);

    if !app.client.logged_in() {
        ui.vertical_centered(|ui| {
            ui.add_space(110.0);
            ui.label(
                egui::RichText::new("登录后可以在这里看观看历史")
                    .color(theme::TEXT3)
                    .size(12.0),
            );
        });
        return;
    }

    settings_row(ui, app);

    match app.history_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(90.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_history();
            }
        }
        LoadState::Ready => {
            if app.view_history.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(90.0);
                    ui.label(
                        egui::RichText::new("还没有看过视频")
                            .color(theme::TEXT3)
                            .size(12.0),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .id_salt("history-scroll")
                    .show(ui, |ui| {
                        let mut open = None;
                        let mut del = None;
                        for it in &app.view_history {
                            match row(ui, app, it) {
                                RowHit::Open => {
                                    open =
                                        Some((it.media_type.clone(), it.video_id, it.live_room_id))
                                }
                                RowHit::Delete => {
                                    del = Some((it.media_type.clone(), it.target_id()))
                                }
                                RowHit::None => {}
                            }
                        }
                        if let Some((kind, id)) = del {
                            app.delete_history(&kind, id);
                        } else if let Some((kind, video_id, live_id)) = open {
                            match kind.as_str() {
                                "live" => app.open_live(live_id),
                                "video" => app.open_detail(video_id),
                                _ => {
                                    app.action_msg = Some("专栏暂不支持在 App 内打开".to_owned());
                                }
                            }
                        }
                    });
            }
        }
    }

    if let Some(msg) = app.action_msg.clone() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
        });
    }
}

enum RowHit {
    None,
    Open,
    Delete,
}

fn header(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 44.0), Sense::hover());
    let p = ui.painter();

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
        Pos2::new(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        "返回",
        FontId::proportional(13.0),
        theme::TEXT2,
    );
    if ui
        .interact(
            Rect::from_min_size(rect.min, Vec2::new(84.0, rect.height())),
            egui::Id::new("history-back"),
            Sense::click(),
        )
        .clicked()
    {
        app.back();
        return;
    }

    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "观看历史",
        FontId::proportional(15.0),
        theme::TEXT,
    );

    if !app.view_history.is_empty() {
        let clear = Rect::from_min_size(
            Pos2::new(rect.right() - 46.0, rect.center().y - 12.0),
            Vec2::new(36.0, 24.0),
        );
        p.text(
            clear.center(),
            Align2::CENTER_CENTER,
            "清空",
            FontId::proportional(11.0),
            theme::TEXT2,
        );
        if ui
            .interact(clear, egui::Id::new("history-clear"), Sense::click())
            .clicked()
        {
            app.clear_history();
        }
    }
}

/// 「记录观看历史」开关（后端 /view-history/settings）
fn settings_row(ui: &mut egui::Ui, app: &mut App) {
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
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        "记录观看历史",
        FontId::proportional(12.0),
        theme::TEXT2,
    );

    let on = !app.history_paused;
    let btn = Rect::from_min_size(
        Pos2::new(rect.right() - 72.0, rect.center().y - 12.0),
        Vec2::new(58.0, 24.0),
    );
    if on {
        p.rect_stroke(btn, 12.0, Stroke::new(1.0, theme::PINK), StrokeKind::Inside);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "已开启",
            FontId::proportional(10.0),
            theme::PINK,
        );
    } else {
        p.rect_stroke(btn, 12.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "已暂停",
            FontId::proportional(10.0),
            theme::TEXT3,
        );
    }
    if ui
        .interact(btn, egui::Id::new("history-paused"), Sense::click())
        .clicked()
    {
        app.set_history_paused(on);
    }
}

fn row(ui: &mut egui::Ui, app: &App, it: &crate::api::ViewHistoryItem) -> RowHit {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 90.0), Sense::click());
    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.top() + 6.0),
        Vec2::new(120.0, 68.0),
    );
    widgets::draw_cover(ui, cover, &it.cover_url, &app.covers, 4.0);

    let p = ui.painter();
    // 封面底部的观看进度条
    let ratio = if it.duration_sec > 0.0 {
        (it.progress_sec / it.duration_sec).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if ratio > 0.0 {
        let bar = Rect::from_min_max(
            Pos2::new(cover.left(), cover.bottom() - 3.0),
            Pos2::new(cover.right(), cover.bottom()),
        );
        p.rect_filled(bar, 0.0, egui::Color32::from_black_alpha(120));
        p.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new(bar.width() * ratio as f32, bar.height())),
            0.0,
            theme::PINK,
        );
    }

    let text_x = cover.right() + 12.0;
    let text_w = (rect.right() - text_x - 52.0).max(40.0);
    let lines = widgets::wrap_lines(p, &it.title, 14.0, text_w, 2);
    let mut y = rect.top() + 2.0;
    for l in lines {
        p.text(
            Pos2::new(text_x, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(14.0),
            theme::TEXT,
        );
        y += 20.0;
    }

    let progress = if it.media_type != "video" {
        String::new()
    } else if it.duration_sec > 0.0 {
        if it.progress_sec >= it.duration_sec - 1.0 {
            "已看完".to_owned()
        } else if it.progress_sec > 0.0 {
            format!(
                "看到 {} / {}",
                theme::clock(it.progress_sec as u64),
                theme::clock(it.duration_sec as u64)
            )
        } else {
            String::new()
        }
    } else if it.progress_sec > 0.0 {
        format!("看到 {}", theme::clock(it.progress_sec as u64))
    } else {
        String::new()
    };

    let mut meta = Vec::new();
    if !it.uploader_name.is_empty() {
        meta.push(it.uploader_name.clone());
    }
    if !it.category.is_empty() {
        meta.push(it.category.clone());
    }
    p.text(
        Pos2::new(text_x, cover.bottom() - 30.0),
        Align2::LEFT_TOP,
        widgets::wrap_lines(p, &meta.join(" · "), 11.0, text_w, 1)
            .into_iter()
            .next()
            .unwrap_or_default(),
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    let mut sub = Vec::new();
    let has_progress = !progress.is_empty();
    if has_progress {
        sub.push(progress.clone());
    }
    if !it.viewed_at.is_empty() {
        sub.push(if it.viewed_time.is_empty() {
            short_time(&it.viewed_at)
        } else {
            it.viewed_time.clone()
        });
    }
    p.text(
        Pos2::new(text_x, cover.bottom() - 13.0),
        Align2::LEFT_TOP,
        widgets::wrap_lines(p, &sub.join(" · "), 11.0, text_w, 1)
            .into_iter()
            .next()
            .unwrap_or_default(),
        FontId::proportional(11.0),
        if has_progress {
            theme::PINK
        } else {
            theme::TEXT3
        },
    );

    // 右侧：删除（视频/直播/专栏都对应自己的删除接口）
    let del = Rect::from_min_size(
        Pos2::new(rect.right() - 46.0, rect.center().y - 11.0),
        Vec2::new(34.0, 22.0),
    );
    p.text(
        del.center(),
        Align2::CENTER_CENTER,
        "删除",
        FontId::proportional(10.0),
        theme::TEXT3,
    );
    let del_clicked = ui
        .interact(del, egui::Id::new(("history-del", it.media_type.clone(), it.target_id())), Sense::click())
        .clicked();

    p.line_segment(
        [
            Pos2::new(rect.left() + 12.0, rect.bottom()),
            Pos2::new(rect.right() - 12.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    if del_clicked {
        RowHit::Delete
    } else if resp.clicked() {
        RowHit::Open
    } else {
        RowHit::None
    }
}

/// "2026-11-03 16:28:01" → "11-03 16:28"
fn short_time(s: &str) -> String {
    if s.len() >= 16 {
        s[5..16].to_owned()
    } else {
        s.to_owned()
    }
}

impl crate::api::ViewHistoryItem {
    /// 删除接口要的 id（按 media_type 区分）
    fn target_id(&self) -> u64 {
        match self.media_type.as_str() {
            "live" => self.live_room_id,
            "article" => self.article_id,
            _ => self.video_id,
        }
    }
}
