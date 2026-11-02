//! 首页：GET /api/v1/videos 的推荐列表

use crate::{
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);

    match app.videos_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            center_hint(ui, "正在请求 /api/v1/videos …");
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_videos();
            }
        }
        LoadState::Ready => {
            if app.videos.is_empty() {
                if App::empty_state(ui, "暂无视频", "接口返回空列表，先去后台发布一条") {
                    app.load_videos();
                }
                return;
            }
            grid(ui, app);
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
        "cakecake",
        FontId::proportional(18.0),
        theme::PINK,
    );
    p.text(
        Pos2::new(rect.left() + 92.0, rect.center().y),
        Align2::LEFT_CENTER,
        &app.client.base,
        FontId::proportional(10.0),
        theme::TEXT3,
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
    let r = ui.interact(btn, egui::Id::new("home-refresh"), Sense::click());
    if r.clicked() {
        app.load_videos();
    }
}

fn grid(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("home-scroll")
        .show(ui, |ui| {
            ui.add_space(4.0);
            let w = ui.available_width();
            let mut clicked = None;

            if let Some(v) = app.videos.first() {
                let cw = w - 16.0;
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if widgets::big_card(ui, v, &app.covers, cw).clicked() {
                        clicked = Some(v.id);
                    }
                });
            }

            ui.add_space(10.0);
            let gap = 7.0;
            let cw = ((w - 16.0 - gap) / 2.0).floor();
            let mut i = 1;
            while i < app.videos.len() {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.x = gap;
                    if let Some(v) = app.videos.get(i) {
                        if widgets::small_card(ui, v, &app.covers, cw).clicked() {
                            clicked = Some(v.id);
                        }
                    }
                    if let Some(v) = app.videos.get(i + 1) {
                        if widgets::small_card(ui, v, &app.covers, cw).clicked() {
                            clicked = Some(v.id);
                        }
                    }
                });
                ui.add_space(10.0);
                i += 2;
            }
            ui.add_space(8.0);

            if let Some(id) = clicked {
                app.open_detail(id);
            }
        });
}

fn center_hint(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(140.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}
