//! 排行榜（首页右上角「榜」进）
//!
//! 走 GET /api/v1/leaderboard?by=&period=：
//!   by     = play / coin / like / fav（后端默认 play）
//!   period = all / week / month（后端默认 all）
//!
//! 后端直接返回 model.Video（大写键），排名就是列表顺序——不做本地二次排序。

use crate::{
    api::RankVideo,
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// (接口参数, 界面文案)
const BY: [(&str, &str); 4] = [
    ("play", "播放"),
    ("coin", "投币"),
    ("like", "点赞"),
    ("fav", "收藏"),
];
const PERIOD: [(&str, &str); 3] = [("all", "总榜"), ("week", "周榜"), ("month", "月榜")];

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    if widgets::back_bar(ui, "排行榜", "rank-back") {
        app.back();
        return;
    }
    filters(ui, app);

    match app.rank_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(egui::RichText::new("正在加载榜单 …").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "榜单加载失败", &e) {
                app.load_rank();
            }
        }
        LoadState::Ready => {
            if app.rank.is_empty() {
                if App::empty_state(ui, "榜单还是空的", "该维度下还没有已发布的视频") {
                    app.load_rank();
                }
                return;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("rank-scroll")
                .show(ui, |ui| {
                    let mut clicked = None;
                    for (i, v) in app.rank.iter().enumerate() {
                        if row(ui, app, v, i + 1).clicked() {
                            clicked = Some(v.id);
                        }
                    }
                    ui.add_space(12.0);
                    if let Some(id) = clicked {
                        app.open_detail(id);
                    }
                });
        }
    }
}

/// 维度 / 周期筛选
fn filters(ui: &mut egui::Ui, app: &mut App) {
    let mut by: Option<&str> = None;
    let mut period: Option<&str> = None;
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(12.0);
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
        for (key, label) in BY {
            let on = app.rank_by == key;
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .color(if on { Color32::WHITE } else { theme::TEXT2 })
                    .size(11.0),
            )
            .fill(if on { theme::PINK } else { theme::SURFACE })
            .corner_radius(12.0)
            .min_size(Vec2::new(0.0, 26.0));
            if ui.add(btn).clicked() && !on {
                by = Some(key);
            }
        }
        ui.add_space(6.0);
        for (key, label) in PERIOD {
            let on = app.rank_period == key;
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .color(if on { theme::PINK } else { theme::TEXT3 })
                    .size(11.0),
            )
            .fill(theme::SURFACE2)
            .corner_radius(12.0)
            .min_size(Vec2::new(0.0, 26.0));
            if ui.add(btn).clicked() && !on {
                period = Some(key);
            }
        }
    });
    ui.add_space(6.0);
    if by.is_some() || period.is_some() {
        app.set_rank(by, period);
    }
}

fn row(ui: &mut egui::Ui, app: &App, v: &RankVideo, rank: usize) -> egui::Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 76.0), Sense::click());
    let p = ui.painter();
    // 前三名用粉色标号，其余灰色
    let color = if rank <= 3 { theme::PINK } else { theme::TEXT3 };
    p.text(
        Pos2::new(rect.left() + 20.0, rect.center().y),
        Align2::CENTER_CENTER,
        rank.to_string(),
        FontId::proportional(if rank <= 3 { 16.0 } else { 13.0 }),
        color,
    );

    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 40.0, rect.top() + 6.0),
        Vec2::new(112.0, 64.0),
    );
    widgets::draw_cover(ui, cover, &v.cover_url, &app.covers, 4.0);
    if v.duration > 0.0 {
        widgets::chip(
            ui.painter(),
            Pos2::new(cover.right() - 5.0, cover.bottom() - 5.0),
            Align2::RIGHT_BOTTOM,
            &theme::clock(v.duration as u64),
            10.0,
        );
    }

    let p = ui.painter();
    let text_x = cover.right() + 10.0;
    let text_w = (rect.right() - text_x - 12.0).max(60.0);
    let lines = widgets::wrap_lines(p, &v.title, 13.0, text_w, 2);
    let mut y = cover.top() + 2.0;
    for l in lines {
        p.text(
            Pos2::new(text_x, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(13.0),
            theme::TEXT,
        );
        y += 19.0;
    }
    p.text(
        Pos2::new(text_x, cover.bottom() - 14.0),
        Align2::LEFT_TOP,
        format!(
            "{}播放 · {}点赞 · {}投币",
            theme::human(v.play_count),
            theme::human(v.like_count),
            theme::human(v.coin_count)
        ),
        FontId::proportional(10.0),
        theme::TEXT3,
    );
    p.line_segment(
        [
            Pos2::new(rect.left() + 14.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
    resp
}
