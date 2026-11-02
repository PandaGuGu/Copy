//! 视频详情：GET /videos/:id + /danmaku + /comments
//!
//! 没有视频解码器，所以不做"假装在播放"的播放器；
//! 封面 + 真实弹幕列表 + 真实评论列表，就是接口能给的全部。

use crate::{
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App, id: u64) {
    back_bar(ui, app);

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
            body(ui, app, &data);
            app.detail = Some(data);
        }
    }
}

fn back_bar(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::hover());
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
    let hit = Rect::from_min_size(
        Pos2::new(rect.left(), rect.top()),
        Vec2::new(80.0, rect.height()),
    );
    if ui
        .interact(hit, egui::Id::new("detail-back"), Sense::click())
        .clicked()
    {
        app.back();
    }
}

fn body(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("detail-scroll")
        .show(ui, |ui| {
            let w = ui.available_width();
            let v = &data.video;

            let (cover, _) =
                ui.allocate_exact_size(Vec2::new(w, (w * 0.5625).round()), Sense::hover());
            widgets::draw_cover(ui, cover, &v.cover_url, &app.covers, 0.0);

            let p = ui.painter();
            let lines = widgets::wrap_lines(p, &v.title, 16.0, w - 28.0, 3);
            let mut y = cover.bottom() + 10.0;
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
            let stats = format!(
                "{}播放 · {}弹幕 · {}评论 · {}点赞",
                theme::human(v.play_count),
                theme::human(v.danmaku_count),
                theme::human(v.comment_count),
                theme::human(v.like_count)
            );
            p.text(
                Pos2::new(cover.left() + 14.0, y + 4.0),
                Align2::LEFT_TOP,
                stats,
                FontId::proportional(11.0),
                theme::TEXT3,
            );
            let need = (y + 26.0) - cover.bottom();
            ui.add_space(need);

            section(ui, &format!("弹幕 {}", data.danmakus.len()));
            if data.danmakus.is_empty() {
                empty_line(ui, "这条视频还没有弹幕");
            } else {
                for d in &data.danmakus {
                    danmaku_row(ui, d.video_time, &d.content, &d.color);
                }
            }

            ui.add_space(8.0);
            section(ui, &format!("评论 {}", data.comments.len()));
            if data.comments.is_empty() {
                empty_line(ui, "还没有评论");
            } else {
                for c in &data.comments {
                    comment_row(ui, c);
                }
            }
            ui.add_space(20.0);
        });
}

fn section(ui: &mut egui::Ui, title: &str) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 34.0), Sense::hover());
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.0),
        theme::TEXT,
    );
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
}

fn empty_line(ui: &mut egui::Ui, text: &str) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 44.0), Sense::hover());
    ui.painter().text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(11.0),
        theme::TEXT3,
    );
}

fn danmaku_row(ui: &mut egui::Ui, t: f64, content: &str, color: &str) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::hover());
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        theme::clock(t as u64),
        FontId::proportional(10.0),
        theme::TEXT3,
    );
    // 白底上白色弹幕会看不见，过亮的颜色回退成正文色
    let col = parse_color(color)
        .filter(|c| (c.r() as u32 + c.g() as u32 + c.b() as u32) < 600)
        .unwrap_or(theme::TEXT);
    p.text(
        Pos2::new(rect.left() + 58.0, rect.center().y),
        Align2::LEFT_CENTER,
        content,
        FontId::proportional(12.0),
        col,
    );
}

fn parse_color(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}

fn comment_row(ui: &mut egui::Ui, c: &crate::api::Comment) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 62.0), Sense::hover());
    let p = ui.painter();
    let seed = c
        .username
        .bytes()
        .fold(0u64, |a, b| a.wrapping_mul(31) + b as u64);
    widgets::avatar(
        p,
        Pos2::new(rect.left() + 28.0, rect.top() + 20.0),
        14.0,
        &c.username,
        seed,
    );

    let mut x = rect.left() + 48.0;
    p.text(
        Pos2::new(x, rect.top() + 10.0),
        Align2::LEFT_TOP,
        &c.username,
        FontId::proportional(12.0),
        theme::TEXT2,
    );
    x += 52.0;
    let lv = Rect::from_min_size(Pos2::new(x, rect.top() + 11.0), Vec2::new(26.0, 13.0));
    p.rect_filled(lv, 2.0, theme::SURFACE);
    p.text(
        lv.center(),
        Align2::CENTER_CENTER,
        format!("LV{}", c.user_level.max(1)),
        FontId::proportional(8.0),
        theme::TEXT3,
    );
    if c.is_by_uploader {
        let tag = Rect::from_min_size(Pos2::new(lv.right() + 4.0, lv.top()), Vec2::new(22.0, 13.0));
        p.rect_filled(tag, 2.0, theme::PINK_SOFT);
        p.text(
            tag.center(),
            Align2::CENTER_CENTER,
            "UP",
            FontId::proportional(8.0),
            theme::PINK,
        );
    }

    let lines = widgets::wrap_lines(p, &c.content, 12.0, rect.width() - 62.0, 2);
    let mut y = rect.top() + 29.0;
    for l in lines {
        p.text(
            Pos2::new(rect.left() + 48.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(12.0),
            theme::TEXT,
        );
        y += 17.0;
    }
    p.text(
        Pos2::new(rect.right() - 14.0, rect.top() + 30.0),
        Align2::RIGHT_TOP,
        format!("赞 {}", theme::human(c.like_count)),
        FontId::proportional(9.0),
        theme::TEXT3,
    );
    p.line_segment(
        [
            Pos2::new(rect.left() + 14.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::SURFACE),
    );
}
