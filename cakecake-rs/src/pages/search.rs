//! 搜索页：GET /api/v1/hot-search 热搜 + GET /api/v1/search 结果
//!
//! 只画接口给的字段：标题、封面、时长、播放/弹幕数；
//! 没有搜索结果时展示热搜词，点一下直接搜。

use crate::{
    api::VideoHit,
    app::{App, LoadState},
    images::Covers,
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    if widgets::back_bar(ui, "搜索", "search-back") {
        app.back();
    }
    input_row(ui, app);
    ui.add_space(2.0);

    match app.search_state.clone() {
        LoadState::Idle => hot_words(ui, app),
        LoadState::Loading => hint(ui, "正在搜索…"),
        LoadState::Err(e) => {
            hint(ui, "搜索失败");
            hint(ui, &e);
            if ui
                .add_sized(Vec2::new(72.0, 28.0), egui::Button::new("重试"))
                .clicked()
            {
                app.do_search();
            }
        }
        LoadState::Ready => {
            if app.search_hits.is_empty() {
                hint(ui, "没有找到相关视频");
                hot_words(ui, app);
            } else {
                results(ui, app);
            }
        }
    }
}

fn input_row(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        let total = ui.available_width();
        let field_w = (total - 78.0).max(120.0);
        let resp = ui.add_sized(
            Vec2::new(field_w, 34.0),
            egui::TextEdit::singleline(&mut app.search_kw)
                .hint_text("搜索视频、UP主")
                .margin(egui::Margin::symmetric(10, 8)),
        );

        // 安卓上软键盘要靠 InputMethodManager 显式拉起，只在焦点变化时下发
        let focused = resp.has_focus();
        if app.ime_on != focused {
            app.ime_on = focused;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::IMEAllowed(focused));
        }
        if app.search_autofocus {
            resp.request_focus();
            app.search_autofocus = false;
        }
        // 键盘回车或输入法「搜索」键触发
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.do_search();
        }

        let btn = egui::Button::new(egui::RichText::new("搜索").color(Color32::WHITE).size(13.0))
            .fill(theme::PINK)
            .corner_radius(16.0);
        if ui.add_sized(Vec2::new(58.0, 34.0), btn).clicked() {
            app.do_search();
        }
    });
}

fn hot_words(ui: &mut egui::Ui, app: &mut App) {
    if !matches!(app.hot_state, LoadState::Ready) {
        if app.hot_state == LoadState::Loading {
            hint(ui, "正在拉取热搜…");
        }
        return;
    }
    let hot = app.hot.clone();
    if hot.is_empty() {
        return;
    }
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("热搜")
                .color(theme::TEXT)
                .size(13.0)
                .strong(),
        );
    });
    ui.add_space(4.0);
    let mut picked: Option<String> = None;
    ui.horizontal_wrapped(|ui| {
        ui.add_space(14.0);
        for (i, kw) in hot.iter().enumerate() {
            let label = format!("{}. {}", i + 1, kw);
            let btn = egui::Button::new(egui::RichText::new(label).color(theme::TEXT2).size(12.0))
                .fill(theme::SURFACE)
                .corner_radius(13.0)
                .min_size(Vec2::new(0.0, 28.0));
            if ui.add(btn).clicked() {
                picked = Some(kw.clone());
            }
        }
    });
    if let Some(kw) = picked {
        app.search_kw = kw;
        app.do_search();
    }
}

fn results(ui: &mut egui::Ui, app: &mut App) {
    ui.add_space(4.0);
    let mut clicked = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("search-scroll")
        .show(ui, |ui| {
            let hits = app.search_hits.clone();
            for h in &hits {
                if hit_row(ui, h, &app.covers).clicked() && h.aid > 0 {
                    clicked = Some(h.aid);
                }
            }
            ui.add_space(12.0);
        });
    if let Some(aid) = clicked {
        app.open_detail(aid);
    }
}

fn hit_row(ui: &mut egui::Ui, h: &VideoHit, covers: &Covers) -> egui::Response {
    let w = ui.available_width() - 24.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 84.0), Sense::click());
    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.top() + 4.0),
        Vec2::new(120.0, 68.0),
    );
    widgets::draw_cover(ui, cover, &h.pic, covers, 4.0);

    let p = ui.painter();
    if !h.duration.is_empty() {
        widgets::chip(
            p,
            Pos2::new(cover.right() - 5.0, cover.bottom() - 5.0),
            Align2::RIGHT_BOTTOM,
            &h.duration,
            10.0,
        );
    }

    let text_x = cover.right() + 12.0;
    let text_w = (rect.right() - text_x).max(40.0);
    let lines = widgets::wrap_lines(p, &h.title, 14.0, text_w, 2);
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
    p.text(
        Pos2::new(text_x, cover.bottom() - 15.0),
        Align2::LEFT_TOP,
        format!(
            "{}播放 · {}弹幕",
            theme::human(h.play),
            theme::human(h.video_review)
        ),
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    ui.allocate_rect(rect, Sense::hover());
    resp
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
    ui.add_space(6.0);
}
