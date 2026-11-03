//! 稍后再看
//!
//!   GET    /api/v1/users/me/watch-later
//!   POST   /api/v1/users/me/watch-later/:id/watched
//!   DELETE /api/v1/users/me/watch-later/watched
//!   DELETE /api/v1/users/me/watch-later
//!
//! 列表里的播放数、时长、UP 主都直接来自接口，不在本地补数据。

use crate::{
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);

    if !app.client.logged_in() {
        ui.vertical_centered(|ui| {
            ui.add_space(110.0);
            ui.label(
                egui::RichText::new("登录后可以在这里看稍后再看")
                    .color(theme::TEXT3)
                    .size(12.0),
            );
        });
        return;
    }

    match app.watch_later_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(110.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_watch_later();
            }
        }
        LoadState::Ready => {
            if app.watch_later.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(110.0);
                    ui.label(
                        egui::RichText::new("还没有稍后再看的视频")
                            .color(theme::TEXT3)
                            .size(12.0),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .id_salt("watch-later-scroll")
                    .show(ui, |ui| {
                        let mut open = None;
                        let mut mark = None;
                        for it in &app.watch_later {
                            match row(ui, app, it) {
                                RowHit::Open => open = Some(it.id),
                                RowHit::Mark => mark = Some(it.id),
                                RowHit::None => {}
                            }
                        }
                        ui.add_space(10.0);
                        if let Some(id) = open {
                            // 从稍后再看点开就算看过了，顺手标记，回到列表时状态是真的
                            app.mark_watched(id);
                            app.open_detail(id);
                        } else if let Some(id) = mark {
                            app.mark_watched(id);
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
    Mark,
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
            egui::Id::new("watch-later-back"),
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
        "稍后再看",
        FontId::proportional(15.0),
        theme::TEXT,
    );

    // 右侧：清除已看 / 清空（列表为空时不显示）
    if !app.watch_later.is_empty() {
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
        let has_watched = app.watch_later.iter().any(|v| v.watched);
        let clear_watched = Rect::from_min_size(
            Pos2::new(clear.left() - 72.0, rect.center().y - 12.0),
            Vec2::new(66.0, 24.0),
        );
        if has_watched {
            p.text(
                clear_watched.center(),
                Align2::CENTER_CENTER,
                "清除已看",
                FontId::proportional(11.0),
                theme::TEXT3,
            );
        }
        if ui
            .interact(
                clear,
                egui::Id::new("watch-later-clear"),
                Sense::click(),
            )
            .clicked()
        {
            app.watch_later_clear(false);
        }
        if has_watched
            && ui
                .interact(
                    clear_watched,
                    egui::Id::new("watch-later-clear-watched"),
                    Sense::click(),
                )
                .clicked()
        {
            app.watch_later_clear(true);
        }
    }
}

fn row(ui: &mut egui::Ui, app: &App, it: &crate::api::WatchLaterItem) -> RowHit {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 82.0), Sense::click());
    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.top() + 4.0),
        Vec2::new(120.0, 68.0),
    );
    widgets::draw_cover(ui, cover, &it.cover_url, &app.covers, 4.0);
    let p = ui.painter();
    if it.duration > 0.0 {
        widgets::chip(
            p,
            Pos2::new(cover.right() - 5.0, cover.bottom() - 5.0),
            Align2::RIGHT_BOTTOM,
            &theme::clock(it.duration as u64),
            10.0,
        );
    }

    let text_x = cover.right() + 12.0;
    let text_w = (rect.right() - text_x - 82.0).max(40.0);
    let lines = widgets::wrap_lines(p, &it.title, 14.0, text_w, 2);
    let mut y = cover.top() + 2.0;
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

    let mut meta = if it.uploader.is_empty() {
        format!("{}播放 · {}弹幕", theme::human(it.play_count), theme::human(it.danmaku_count))
    } else {
        format!(
            "{} · {}播放 · {}弹幕",
            it.uploader,
            theme::human(it.play_count),
            theme::human(it.danmaku_count)
        )
    };
    if !it.added_at.is_empty() {
        meta.push_str(&format!(" · 收藏于 {}", it.added_at));
    }
    let meta = widgets::wrap_lines(p, &meta, 11.0, text_w, 1)
        .into_iter()
        .next()
        .unwrap_or_default();
    p.text(
        Pos2::new(text_x, cover.bottom() - 15.0),
        Align2::LEFT_TOP,
        meta,
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    // 右侧：标记已看（已经看过的只显示状态）
    let btn = Rect::from_min_size(
        Pos2::new(rect.right() - 74.0, rect.center().y - 13.0),
        Vec2::new(62.0, 26.0),
    );
    let mark_hit = if it.watched {
        p.rect_stroke(btn, 13.0, Stroke::new(1.0, theme::LINE), egui::StrokeKind::Inside);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "已看过",
            FontId::proportional(10.0),
            theme::TEXT3,
        );
        false
    } else {
        p.rect_stroke(btn, 13.0, Stroke::new(1.0, theme::PINK), egui::StrokeKind::Inside);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "标记已看",
            FontId::proportional(10.0),
            theme::PINK,
        );
        ui.interact(btn, egui::Id::new(("watch-later-mark", it.id)), Sense::click())
            .clicked()
    };

    p.line_segment(
        [
            Pos2::new(rect.left() + 12.0, rect.bottom()),
            Pos2::new(rect.right() - 12.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    if mark_hit {
        RowHit::Mark
    } else if resp.clicked() {
        RowHit::Open
    } else {
        RowHit::None
    }
}
