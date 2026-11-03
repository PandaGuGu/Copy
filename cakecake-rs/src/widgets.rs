//! 通用组件：封面卡、角标、头像、底部导航图标
//!
//! 只画接口里有的字段：标题、播放/弹幕数、时长、主播名。

use crate::{api::Video, images::Covers, theme};
use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2,
};

#[derive(Clone, Copy)]
pub enum NavIcon {
    Home,
    Bag,
    Person,
    /// 动态（五角星）
    Star,
}

/// 中文字宽与拉丁字宽（用于精确换行）
fn advance(p: &Painter, size: f32) -> (f32, f32) {
    let f = FontId::proportional(size);
    let cjk = p
        .layout_no_wrap("字".to_owned(), f.clone(), theme::TEXT)
        .size()
        .x;
    let latin = p.layout_no_wrap("x".to_owned(), f, theme::TEXT).size().x * 1.05;
    (cjk, latin)
}

pub fn wrap_lines(p: &Painter, text: &str, size: f32, max_w: f32, max_lines: usize) -> Vec<String> {
    let (cjk, latin) = advance(p, size);
    let adv = |c: char| if (c as u32) > 0x2E80 { cjk } else { latin };
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() && out.len() < max_lines {
        let last = out.len() + 1 == max_lines;
        let mut line = String::new();
        let mut w = 0.0;
        while i < chars.len() {
            let a = adv(chars[i]);
            let reserve = if last { cjk } else { 0.0 };
            if w + a + reserve > max_w {
                break;
            }
            w += a;
            line.push(chars[i]);
            i += 1;
        }
        if last && i < chars.len() {
            line.push('…');
            i = chars.len();
        }
        out.push(line);
    }
    out
}

pub fn draw_cover(ui: &mut egui::Ui, rect: Rect, url: &str, covers: &Covers, radius: f32) {
    if let Some(tex) = covers.get(url) {
        let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), rect.size()))
            .corner_radius(radius)
            .fit_to_exact_size(rect.size());
        ui.put(rect, img);
    } else {
        // 没有封面文件时的占位渐变（安卓端就是这样）
        let seed = url
            .bytes()
            .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
        let base = theme::cover_color(seed as u64);
        let p = ui.painter();
        p.rect_filled(rect, radius, base);
        let top = Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.height() * 0.55));
        p.rect_filled(top, radius, theme::mix(base, Color32::WHITE, 0.16));
    }
}

/// 半透明黑底白字角标
pub fn chip(p: &Painter, pos: Pos2, anchor: Align2, text: &str, size: f32) {
    let g = p.layout_no_wrap(text.to_owned(), FontId::proportional(size), Color32::WHITE);
    let pad = Vec2::new(5.0, 2.0);
    let sz = g.size() + pad * 2.0;
    let min = if anchor == Align2::LEFT_BOTTOM {
        Pos2::new(pos.x, pos.y - sz.y)
    } else if anchor == Align2::RIGHT_BOTTOM {
        Pos2::new(pos.x - sz.x, pos.y - sz.y)
    } else {
        pos
    };
    let rect = Rect::from_min_size(min, sz);
    p.rect_filled(rect, 3.0, Color32::from_black_alpha(130));
    p.galley(rect.min + pad, g, Color32::WHITE);
}

fn play_triangle(p: &Painter, c: Pos2, s: f32) {
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(c.x - s * 0.45, c.y - s),
            Pos2::new(c.x - s * 0.45, c.y + s),
            Pos2::new(c.x + s * 0.8, c.y),
        ],
        Color32::from_white_alpha(210),
        Stroke::NONE,
    ));
}

/// 首页大卡：封面 + 数据角标 + 标题
pub fn big_card(ui: &mut egui::Ui, v: &Video, covers: &Covers, width: f32) -> Response {
    let cover_h = (width * 0.5625).round();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, cover_h + 52.0), Sense::click());
    let cover = Rect::from_min_size(rect.min, Vec2::new(width, cover_h));

    draw_cover(ui, cover, &v.cover_url, covers, 6.0);
    let p = ui.painter();
    let c = cover.center();
    p.circle_filled(c, 22.0, Color32::from_black_alpha(60));
    play_triangle(p, c, 9.0);

    let stats = format!(
        "{}  {}",
        theme::human(v.play_count),
        theme::human(v.danmaku_count)
    );
    chip(p, Pos2::new(cover.left() + 8.0, cover.bottom() - 8.0), Align2::LEFT_BOTTOM, &stats, 11.0);
    if v.duration > 0 {
        chip(p, Pos2::new(cover.right() - 8.0, cover.bottom() - 8.0), Align2::RIGHT_BOTTOM, &theme::clock(v.duration), 11.0);
    }

    let lines = wrap_lines(p, &v.title, 16.0, width - 8.0, 2);
    let mut y = cover.bottom() + 8.0;
    for l in lines {
        p.text(
            Pos2::new(rect.left() + 2.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(16.0),
            theme::TEXT,
        );
        y += 22.0;
    }

    // ui.put 画封面会移动布局光标，这里推回卡片底部
    ui.allocate_rect(rect, Sense::hover());
    resp
}

/// 双列小卡
pub fn small_card(ui: &mut egui::Ui, v: &Video, covers: &Covers, width: f32) -> Response {
    let cover_h = (width * 0.5625).round();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, cover_h + 44.0), Sense::click());
    let cover = Rect::from_min_size(rect.min, Vec2::new(width, cover_h));

    draw_cover(ui, cover, &v.cover_url, covers, 6.0);
    let p = ui.painter();
    let stats = format!(
        "{}  {}",
        theme::human(v.play_count),
        theme::human(v.danmaku_count)
    );
    chip(p, Pos2::new(cover.left() + 6.0, cover.bottom() - 6.0), Align2::LEFT_BOTTOM, &stats, 10.0);
    if v.duration > 0 {
        chip(p, Pos2::new(cover.right() - 6.0, cover.bottom() - 6.0), Align2::RIGHT_BOTTOM, &theme::clock(v.duration), 10.0);
    }

    let lines = wrap_lines(p, &v.title, 14.0, width - 6.0, 2);
    let mut y = cover.bottom() + 6.0;
    for l in lines {
        p.text(
            Pos2::new(rect.left() + 1.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(14.0),
            theme::TEXT,
        );
        y += 19.0;
    }

    ui.allocate_rect(rect, Sense::hover());
    resp
}

/// 圆形头像占位：取名字首字
pub fn avatar(p: &Painter, center: Pos2, r: f32, name: &str, seed: u64) {
    const BG: [Color32; 6] = [
        Color32::from_rgb(0xF2, 0xC9, 0xD2),
        Color32::from_rgb(0xC9, 0xDD, 0xF2),
        Color32::from_rgb(0xD3, 0xE8, 0xD1),
        Color32::from_rgb(0xF0, 0xDF, 0xC0),
        Color32::from_rgb(0xDD, 0xD3, 0xF0),
        Color32::from_rgb(0xF2, 0xD8, 0xC9),
    ];
    let bg = BG[(seed as usize) % BG.len()];
    p.circle_filled(center, r, bg);
    let ch = name.chars().next().unwrap_or('?').to_string();
    p.text(
        center,
        Align2::CENTER_CENTER,
        ch,
        FontId::proportional(r * 0.95),
        Color32::from_rgb(0x5A, 0x5A, 0x66),
    );
}

/// 未登录头像：灰底小人，别让用户以为自己头像加载失败了
pub fn avatar_anon(p: &Painter, center: Pos2, r: f32) {
    p.circle_filled(center, r, Color32::from_rgb(0xEE, 0xEF, 0xF1));
    let ink = Color32::from_rgb(0x9A, 0x9F, 0xA6);
    let stroke = Stroke::new(1.4, ink);
    p.circle_stroke(Pos2::new(center.x, center.y - r * 0.22), r * 0.30, stroke);
    p.add(egui::Shape::line(
        vec![
            Pos2::new(center.x - r * 0.52, center.y + r * 0.62),
            Pos2::new(center.x - r * 0.34, center.y + r * 0.16),
            Pos2::new(center.x + r * 0.34, center.y + r * 0.16),
            Pos2::new(center.x + r * 0.52, center.y + r * 0.62),
        ],
        stroke,
    ));
}

/// 头像：有真实头像 URL 且已解码就贴图，否则退回首字占位
pub fn avatar_tex(
    ui: &mut egui::Ui,
    center: Pos2,
    r: f32,
    name: &str,
    url: &str,
    covers: &Covers,
    seed: u64,
) {
    let side = r * 2.0;
    let rect = Rect::from_center_size(center, Vec2::splat(side));
    if let Some(tex) = covers.get(url) {
        let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), rect.size()))
            .corner_radius(r)
            .fit_to_exact_size(rect.size());
        ui.put(rect, img);
        return;
    }
    avatar(ui.painter(), center, r, name, seed);
}

/// 横排视频卡：左封面 + 右标题/UP主/数据（相关推荐、搜索结果共用）
pub fn video_row(ui: &mut egui::Ui, v: &Video, covers: &Covers) -> Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 82.0), Sense::click());
    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.top() + 4.0),
        Vec2::new(120.0, 68.0),
    );
    draw_cover(ui, cover, &v.cover_url, covers, 4.0);
    let p = ui.painter();
    if v.duration > 0 {
        chip(
            p,
            Pos2::new(cover.right() - 5.0, cover.bottom() - 5.0),
            Align2::RIGHT_BOTTOM,
            &theme::clock(v.duration),
            10.0,
        );
    }
    let text_x = cover.right() + 12.0;
    let text_w = (rect.right() - text_x - 10.0).max(40.0);
    let lines = wrap_lines(p, &v.title, 14.0, text_w, 2);
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
            theme::human(v.play_count),
            theme::human(v.danmaku_count)
        ),
        FontId::proportional(11.0),
        theme::TEXT3,
    );
    ui.allocate_rect(rect, Sense::hover());
    resp
}

/// 通用返回栏：左上角「‹ 返回」+ 可选标题，命中返回 true
pub fn back_bar(ui: &mut egui::Ui, title: &str, id: &str) -> bool {
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
    if !title.is_empty() {
        p.text(
            Pos2::new(rect.center().x, rect.center().y),
            Align2::CENTER_CENTER,
            title,
            FontId::proportional(15.0),
            theme::TEXT,
        );
    }
    let hit = Rect::from_min_size(rect.min, Vec2::new(84.0, rect.height()));
    ui.interact(hit, egui::Id::new(id), Sense::click()).clicked()
}

/// 搜索胶囊：放大镜 + 占位文案，命中返回 true
pub fn search_pill(ui: &mut egui::Ui, rect: Rect, hint: &str, id: &str) -> bool {
    {
        let p = ui.painter();
        p.rect_filled(rect, rect.height() / 2.0, theme::SURFACE);
        let ic = Pos2::new(rect.left() + 15.0, rect.center().y - 1.0);
        p.circle_stroke(ic, 4.6, Stroke::new(1.4, theme::TEXT3));
        p.line_segment(
            [
                Pos2::new(ic.x + 3.4, ic.y + 3.4),
                Pos2::new(ic.x + 6.4, ic.y + 6.4),
            ],
            Stroke::new(1.4, theme::TEXT3),
        );
        p.text(
            Pos2::new(rect.left() + 27.0, rect.center().y),
            Align2::LEFT_CENTER,
            hint,
            FontId::proportional(12.0),
            theme::TEXT3,
        );
    }
    ui.interact(rect, egui::Id::new(id), Sense::click()).clicked()
}

pub fn nav_icon(p: &Painter, c: Pos2, kind: NavIcon, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    match kind {
        NavIcon::Home => {
            let w = 8.0;
            p.add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - w, c.y - 1.0),
                    Pos2::new(c.x, c.y - 9.0),
                    Pos2::new(c.x + w, c.y - 1.0),
                ],
                stroke,
            ));
            p.rect_stroke(
                Rect::from_min_max(
                    Pos2::new(c.x - w + 2.5, c.y - 1.5),
                    Pos2::new(c.x + w - 2.5, c.y + 7.0),
                ),
                2.0,
                stroke,
                StrokeKind::Inside,
            );
        }
        NavIcon::Bag => {
            p.rect_stroke(
                Rect::from_min_max(
                    Pos2::new(c.x - 7.0, c.y - 4.0),
                    Pos2::new(c.x + 7.0, c.y + 8.0),
                ),
                2.0,
                stroke,
                StrokeKind::Inside,
            );
            p.add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - 3.0, c.y - 4.0),
                    Pos2::new(c.x - 3.0, c.y - 7.0),
                    Pos2::new(c.x + 3.0, c.y - 7.0),
                    Pos2::new(c.x + 3.0, c.y - 4.0),
                ],
                stroke,
            ));
        }
        NavIcon::Person => {
            p.circle_stroke(Pos2::new(c.x, c.y - 3.5), 3.8, stroke);
            p.add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - 7.0, c.y + 8.0),
                    Pos2::new(c.x - 4.0, c.y + 2.5),
                    Pos2::new(c.x + 4.0, c.y + 2.5),
                    Pos2::new(c.x + 7.0, c.y + 8.0),
                ],
                stroke,
            ));
        }
        NavIcon::Star => {
            let mut pts = Vec::with_capacity(11);
            for i in 0..10 {
                let r = if i % 2 == 0 { 8.6 } else { 3.8 };
                let a = -std::f32::consts::FRAC_PI_2
                    + i as f32 * std::f32::consts::PI / 5.0;
                pts.push(Pos2::new(c.x + r * a.cos(), c.y - 1.0 + r * a.sin()));
            }
            pts.push(pts[0]);
            p.add(egui::Shape::line(pts, stroke));
        }
    }
}
