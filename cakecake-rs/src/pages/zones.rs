//! 全部分区页（首页导航栏最右「☰」进入）
//!
//! 分区与视频数来自 `GET /zones`（后端按 `videos.zone` 聚合，父子形式如「生活-日常」）。
//! 接口还没有数据时回退到内置分区表（`crate::zones`），只作展示，点击仍然走
//! `/zones/:zone/recommendation`，不会伪造视频。

use crate::{app::App, theme, widgets};
use eframe::egui::{self, Align2, FontId, Pos2, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    if widgets::back_bar(ui, "全部分区", "zones-back") {
        app.back();
    }

    let parents = app.zone_parents();
    let real = !parents.is_empty();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("zones-scroll")
        .show(ui, |ui| {
            if !real {
                hint(ui, "后端还没有分区数据，先按内置分区表展示");
            }
            if real {
                for (parent, total) in &parents {
                    let children = app.zone_children(parent);
                    cat_block(ui, app, parent, *total, &children);
                }
            } else {
                for cat in crate::zones::ZONES {
                    let children: Vec<(String, u64)> =
                        cat.items.iter().map(|s| ((*s).to_owned(), 0)).collect();
                    cat_block(ui, app, cat.name, 0, &children);
                }
            }
            ui.add_space(16.0);
        });
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(10.0));
    });
    ui.add_space(4.0);
}

/// 一个父分区：标题行（含总视频数）+ 子分区胶囊
fn cat_block(ui: &mut egui::Ui, app: &mut App, name: &str, total: u64, children: &[(String, u64)]) {
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
    if total > 0 {
        p.text(
            Pos2::new(head.left() + 14.0 + name.chars().count() as f32 * 16.0, head.center().y),
            Align2::LEFT_CENTER,
            format!("{} 个视频", theme::human(total)),
            FontId::proportional(10.0),
            theme::TEXT3,
        );
    }
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

    if children.is_empty() {
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
            for (sub, cnt) in children {
                let value = format!("{name}-{sub}");
                let on = matches!(&app.home_cat, crate::app::HomeCat::Zone(z) if *z == value);
                // 有视频数的把数字带上，0 的不显示，避免误以为点进去一定有内容
                let label = if *cnt > 0 {
                    format!("{sub} {}", theme::human(*cnt))
                } else {
                    sub.clone()
                };
                let btn = egui::Button::new(
                    egui::RichText::new(label)
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
