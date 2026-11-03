//! 全部分区页（首页导航栏最右「☰」进入）
//!
//! 分区表直接复用 Web 端 `constants/videoZones.js`（`crate::zones`），
//! 点父分区按 `/zones/:zone/recommendation` 拉整块，点子分区按「父-子」拉细分。

use crate::{app::App, theme, widgets};
use eframe::egui::{self, Align2, FontId, Pos2, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    if widgets::back_bar(ui, "全部分区", "zones-back") {
        app.back();
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("zones-scroll")
        .show(ui, |ui| {
            for cat in crate::zones::ZONES {
                cat_block(ui, app, cat.name, cat.items);
            }
            ui.add_space(16.0);
        });
}

fn cat_block(ui: &mut egui::Ui, app: &mut App, name: &str, items: &[&str]) {
    let w = ui.available_width();
    let (head, resp) = ui.allocate_exact_size(Vec2::new(w, 36.0), Sense::click());
    let active = matches!(&app.home_cat, crate::app::HomeCat::Zone(z) if z == name);
    let p = ui.painter();
    p.text(
        Pos2::new(head.left() + 14.0, head.center().y),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(15.0),
        if active { theme::PINK } else { theme::TEXT },
    );
    p.text(
        Pos2::new(head.right() - 14.0, head.center().y),
        Align2::RIGHT_CENTER,
        "全部 ›",
        FontId::proportional(10.0),
        theme::TEXT3,
    );
    if resp.clicked() {
        app.open_zone(name.to_owned());
    }

    if items.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("该分区暂无子分区")
                    .color(theme::TEXT3)
                    .size(11.0),
            );
        });
        ui.add_space(6.0);
    } else {
        let mut picked: Option<String> = None;
        ui.horizontal_wrapped(|ui| {
            ui.add_space(14.0);
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            for sub in items {
                let value = format!("{name}-{sub}");
                let on = matches!(&app.home_cat, crate::app::HomeCat::Zone(z) if *z == value);
                let btn = egui::Button::new(
                    egui::RichText::new(*sub)
                        .color(if on { theme::PINK } else { theme::TEXT2 })
                        .size(11.0),
                )
                .fill(theme::SURFACE)
                .corner_radius(12.0)
                .min_size(Vec2::new(0.0, 26.0));
                if ui.add(btn).clicked() {
                    picked = Some(value.clone());
                }
            }
        });
        ui.add_space(4.0);
        if let Some(v) = picked {
            app.open_zone(v);
        }
    }

    let (line, _) = ui.allocate_exact_size(Vec2::new(w, 8.0), Sense::hover());
    ui.painter().line_segment(
        [
            Pos2::new(line.left() + 14.0, line.center().y),
            Pos2::new(line.right() - 14.0, line.center().y),
        ],
        Stroke::new(1.0, theme::LINE),
    );
}
