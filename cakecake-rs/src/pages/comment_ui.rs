//! 评论相关的公共画法：时间/地区、昵称行、赞踩按钮、底部输入栏
//!
//! 视频详情页（一级评论 + 回复预览）和评论详情页（完整楼中楼）共用这些零件，
//! 避免两边各画一套导致样式跑偏。

use crate::{
    api::Comment,
    app::App,
    theme, widgets,
};
use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};
use std::collections::HashMap;

/// 昵称兜底
pub fn name_of(c: &Comment) -> String {
    if c.username.is_empty() {
        "匿名".to_owned()
    } else {
        c.username.clone()
    }
}

/// "11-03 20:39 · 辽宁"（后端给的是 "2026-11-03 20:39:00"）
pub fn time_place(c: &Comment) -> String {
    let t = short_time(&c.created_at);
    if c.ip_location.is_empty() {
        t
    } else {
        format!("{t} · {}", c.ip_location)
    }
}

fn short_time(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    // 去掉年份，手机上 16 个字太占地方
    if chars.len() >= 16 && chars.get(4) == Some(&'-') {
        chars[5..16].iter().collect()
    } else {
        chars.into_iter().take(16).collect()
    }
}

/// 头像：有图贴图，没图用首字占位
pub fn avatar(ui: &mut egui::Ui, app: &App, c: &Comment, center: Pos2, r: f32) {
    let name = name_of(c);
    if c.avatar_url.is_empty() {
        widgets::avatar(ui.painter(), center, r, &name, c.user_id);
    } else {
        widgets::avatar_tex(ui, center, r, &name, &c.avatar_url, &app.covers, c.user_id);
    }
}

/// 昵称 + LV 徽章 + UP 徽章 + 置顶标，返回这一行画完后的 x
pub fn name_line(p: &Painter, x: f32, y: f32, c: &Comment) -> f32 {
    let name = name_of(c);
    let g = p.layout_no_wrap(name, FontId::proportional(12.5), theme::TEXT2);
    let w = g.size().x;
    p.galley(Pos2::new(x, y), g, theme::TEXT2);
    let mut cx = x + w + 5.0;

    // 等级牌
    let lv_text = format!("LV{}", c.user_level.max(1));
    let lg = p.layout_no_wrap(lv_text, FontId::proportional(8.0), Color32::WHITE);
    let lrect = Rect::from_min_size(
        Pos2::new(cx, y + 1.5),
        Vec2::new(lg.size().x + 7.0, 12.0),
    );
    p.rect_filled(lrect, 3.0, theme::ORANGE);
    p.galley(
        Pos2::new(cx + 3.5, lrect.top() + (12.0 - lg.size().y) / 2.0),
        lg,
        Color32::WHITE,
    );
    cx = lrect.right() + 4.0;

    if c.is_by_uploader {
        let r = Rect::from_min_size(Pos2::new(cx, y + 1.5), Vec2::new(21.0, 12.0));
        p.rect_filled(r, 3.0, theme::UP_TAG);
        p.text(
            r.center(),
            Align2::CENTER_CENTER,
            "UP",
            FontId::proportional(8.0),
            Color32::WHITE,
        );
        cx = r.right() + 4.0;
    }
    if c.pinned {
        let r = Rect::from_min_size(Pos2::new(cx, y + 1.5), Vec2::new(26.0, 12.0));
        p.rect_stroke(r, 3.0, Stroke::new(1.0, theme::PINK), StrokeKind::Inside);
        p.text(
            r.center(),
            Align2::CENTER_CENTER,
            "置顶",
            FontId::proportional(8.0),
            theme::PINK,
        );
        cx = r.right() + 4.0;
    }
    cx
}

/// 一条评论右下角的操作热区：回复 / 赞 / 踩（从右往左排，和 B 站一致）
///
/// 这里只算矩形，命中判定交给调用方：整张卡片自己是一个可点区域，
/// 点完再按坐标分派到具体按钮（滚动区域里比再注册一层 widget 稳）。
pub struct ActionRects {
    pub reply: Rect,
    pub like: Rect,
    pub dislike: Rect,
}

pub fn action_rects(right_x: f32, center_y: f32, like_text: &str) -> ActionRects {
    let down = Rect::from_center_size(
        Pos2::new(right_x - 12.0, center_y),
        Vec2::new(26.0, 24.0),
    );
    let like_w = 24.0 + like_text.chars().count() as f32 * 7.0;
    let like = Rect::from_min_max(
        Pos2::new(down.left() - 8.0 - like_w, center_y - 12.0),
        Pos2::new(down.left() - 8.0, center_y + 12.0),
    );
    let reply = Rect::from_center_size(
        Pos2::new(like.left() - 8.0 - 17.0, center_y),
        Vec2::new(34.0, 24.0),
    );
    ActionRects {
        reply,
        like,
        dislike: down,
    }
}

pub fn draw_actions(p: &Painter, c: &Comment, r: &ActionRects, hover: Option<Pos2>) {
    let hot = |rect: &Rect| hover.map(|q| rect.contains(q)).unwrap_or(false);
    let like_text = if c.like_count == 0 {
        String::new()
    } else {
        theme::human(c.like_count)
    };

    let like_color = if c.liked_by_me {
        theme::PINK
    } else if hot(&r.like) {
        theme::TEXT
    } else {
        theme::TEXT3
    };
    thumb(p, Pos2::new(r.like.left() + 6.0, r.like.center().y), true, like_color);
    if !like_text.is_empty() {
        p.text(
            Pos2::new(r.like.left() + 15.0, r.like.center().y),
            Align2::LEFT_CENTER,
            like_text,
            FontId::proportional(11.0),
            like_color,
        );
    }

    let down_color = if c.disliked_by_me || hot(&r.dislike) {
        theme::TEXT
    } else {
        theme::TEXT3
    };
    thumb(p, Pos2::new(r.dislike.center().x, r.dislike.center().y), false, down_color);

    p.text(
        r.reply.center(),
        Align2::CENTER_CENTER,
        "回复",
        FontId::proportional(10.5),
        if hot(&r.reply) { theme::TEXT } else { theme::TEXT3 },
    );
}

/// 命中判定：点在哪块热区里
pub fn hit_actions(r: &ActionRects, pos: Pos2) -> u8 {
    if r.reply.contains(pos) {
        1
    } else if r.like.contains(pos) {
        2
    } else if r.dislike.contains(pos) {
        3
    } else {
        0
    }
}

fn thumb(p: &Painter, c: Pos2, up: bool, color: Color32) {
    let s = 4.2;
    let (a, b, d) = if up {
        (
            Pos2::new(c.x - s, c.y + s * 0.8),
            Pos2::new(c.x + s, c.y + s * 0.8),
            Pos2::new(c.x, c.y - s * 0.9),
        )
    } else {
        (
            Pos2::new(c.x - s, c.y - s * 0.8),
            Pos2::new(c.x + s, c.y - s * 0.8),
            Pos2::new(c.x, c.y + s * 0.9),
        )
    };
    p.add(egui::Shape::convex_polygon(
        vec![a, b, d],
        Color32::TRANSPARENT,
        Stroke::new(1.2, color),
    ));
}

/// 评论树：返回 (一级评论下标, 父 → 子下标)
pub fn thread_index(list: &[Comment]) -> (Vec<usize>, HashMap<u64, Vec<usize>>) {
    let ids: std::collections::HashSet<u64> = list.iter().map(|c| c.id).collect();
    let mut children: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();
    for (i, c) in list.iter().enumerate() {
        if c.parent_id != 0 && c.parent_id != c.id && ids.contains(&c.parent_id) {
            children.entry(c.parent_id).or_default().push(i);
        } else {
            roots.push(i);
        }
    }
    (roots, children)
}

/// 某条评论下面的全部后代（后端最多三层，这里按树摊平）
pub fn descendants(
    list: &[Comment],
    children: &HashMap<u64, Vec<usize>>,
    root_idx: usize,
) -> Vec<usize> {
    let mut out = Vec::new();
    let mut stack: Vec<usize> = children
        .get(&list[root_idx].id)
        .map(|v| v.clone())
        .unwrap_or_default();
    stack.reverse();
    while let Some(i) = stack.pop() {
        out.push(i);
        if let Some(kids) = children.get(&list[i].id) {
            for &k in kids.iter().rev() {
                stack.push(k);
            }
        }
    }
    // 按时间正序（同一楼里早的在上）
    out.sort_by_key(|&i| list[i].created_at.clone());
    out
}

/// 底部固定输入栏：`ui` 需要是已经预留好高度的区域（用 TopBottomPanel 包）
pub fn composer(ui: &mut egui::Ui, app: &mut App, video_id: u64) {
    let target = app.reply_to.clone();
    let mut cancel = false;
    let mut send = false;

    // 回复目标提示行
    if let Some((_, name)) = &target {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new(format!("回复 @{name}"))
                    .color(theme::PINK)
                    .size(11.0),
            );
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("取消").color(theme::TEXT3).size(10.0),
                ))
                .clicked()
            {
                cancel = true;
            }
        });
    }

    let w = ui.available_width();
    let (rect, bar_resp) = ui.allocate_exact_size(Vec2::new(w, 46.0), Sense::click());
    let pill = Rect::from_min_max(
        Pos2::new(rect.left() + 12.0, rect.center().y - 18.0),
        Pos2::new(rect.right() - 58.0, rect.center().y + 18.0),
    );
    let send_btn = Rect::from_center_size(
        Pos2::new(rect.right() - 30.0, rect.center().y),
        Vec2::splat(32.0),
    );

    {
        let p = ui.painter();
        p.rect_filled(pill, 18.0, theme::SURFACE);
        // 发送键：有内容才点亮
        let ready = !app.comment_text.trim().is_empty();
        p.circle_filled(
            send_btn.center(),
            15.0,
            if ready { theme::PINK } else { theme::SURFACE },
        );
        let ink = if ready { Color32::WHITE } else { theme::TEXT3 };
        p.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(send_btn.center().x - 5.0, send_btn.center().y - 6.0),
                Pos2::new(send_btn.center().x + 6.0, send_btn.center().y),
                Pos2::new(send_btn.center().x - 5.0, send_btn.center().y + 6.0),
            ],
            ink,
            Stroke::NONE,
        ));
    }

    let hint = match &target {
        Some((_, name)) => format!("回复 @{name}"),
        None => "进来和UP唠会嗑呗".to_owned(),
    };
    let field = ui.put(
        pill.shrink2(Vec2::new(14.0, 7.0)),
        egui::TextEdit::singleline(&mut app.comment_text)
            .id(egui::Id::new("comment-input"))
            .frame(egui::Frame::NONE)
            .hint_text(hint)
            .font(FontId::proportional(12.0)),
    );
    // 安卓软键盘：聚焦时允许 IME 弹出
    let focused = field.has_focus();
    app.note_focus(focused);
    if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        send = true;
    }
    // 发送键：整条输入栏是一个点击区，按坐标判断是否点在圆钮上
    if bar_resp.clicked() {
        if let Some(pos) = bar_resp.interact_pointer_pos() {
            if send_btn.contains(pos) {
                send = true;
            }
        }
    }

    if cancel {
        app.reply_to = None;
    }
    if send {
        app.post_comment(video_id);
    }
}
