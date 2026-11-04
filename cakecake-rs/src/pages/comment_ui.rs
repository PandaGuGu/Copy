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

/// 评论配图尺寸（一条评论最多铺 3 张）
pub const IMG_SIDE: f32 = 72.0;

/// 配图占用的高度（含与正文之间的间距）；没有配图返回 0
pub fn images_h(c: &Comment) -> f32 {
    if c.images.is_empty() {
        0.0
    } else {
        IMG_SIDE + 8.0
    }
}

/// 画一条评论的配图（最多 3 张一排，超出宽度就截断）。点任意一张放大查看。
pub fn draw_images(ui: &mut egui::Ui, app: &mut App, c: &Comment, left: f32, top: f32, max_w: f32) {
    if c.images.is_empty() {
        return;
    }
    let gap = 6.0;
    let mut x = left;
    for (i, url) in c.images.iter().take(3).enumerate() {
        if x + IMG_SIDE > left + max_w + 0.5 {
            break;
        }
        let rect = Rect::from_min_size(Pos2::new(x, top), Vec2::splat(IMG_SIDE));
        widgets::draw_cover(ui, rect, url, &app.covers, 6.0);
        // 缩略图本身不占布局（父级已经 allocate 过整块），这里只补一个点击热区
        if ui
            .interact(rect, egui::Id::new(("comment-img", c.id, i)), Sense::click())
            .clicked()
        {
            app.image_view = Some(url.clone());
        }
        x += IMG_SIDE + gap;
    }
}

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

    // 系统相册选图结果（安卓）：选好了就用，取消则收起配图行
    #[cfg(target_os = "android")]
    if let Some(path) = crate::video::picked_image() {
        if path.is_empty() {
            if app.comment_image_path.is_empty() {
                app.comment_image_open = false;
            }
        } else {
            app.comment_image_path = path;
            app.comment_image_open = true;
        }
    }

    // 评论配图：展开时在输入栏上方加一行，高度由调用方同步预留
    if app.comment_image_open {
        #[cfg(not(target_os = "android"))]
        if app.image_files.is_empty() {
            app.scan_images();
        }
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("配图")
                    .color(theme::TEXT3)
                    .size(10.0),
            );
            if app.comment_image_path.is_empty() {
                ui.label(
                    egui::RichText::new("未选择")
                        .color(theme::TEXT3)
                        .size(10.0),
                );
            } else {
                ui.label(
                    egui::RichText::new(
                        std::path::Path::new(&app.comment_image_path)
                            .file_name()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    )
                    .color(theme::PINK)
                    .size(10.0),
                );
            }
            #[cfg(not(target_os = "android"))]
            if ui.button("刷新").clicked() {
                app.scan_images();
            }
            if !app.comment_image_path.is_empty() && ui.button("移除").clicked() {
                app.comment_image_path.clear();
            }
        });
        // 桌面端保留本地目录选择；安卓走系统相册，不需要再列文件
        #[cfg(not(target_os = "android"))]
        egui::ScrollArea::horizontal()
            .max_height(24.0)
            .id_salt("comment-img-picker")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for path in app.image_files.clone() {
                        let name = std::path::Path::new(&path)
                            .file_name()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let on = app.comment_image_path == path;
                        if ui.selectable_label(on, name).clicked() {
                            app.comment_image_path = path;
                        }
                    }
                });
            });
    }

    let w = ui.available_width();
    let (rect, bar_resp) = ui.allocate_exact_size(Vec2::new(w, 46.0), Sense::click());
    let pill = Rect::from_min_max(
        Pos2::new(rect.left() + 12.0, rect.center().y - 18.0),
        Pos2::new(rect.right() - 88.0, rect.center().y + 18.0),
    );
    let img_btn = Rect::from_center_size(
        Pos2::new(rect.right() - 64.0, rect.center().y),
        Vec2::splat(28.0),
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
        // 配图按钮：有配图时高亮
        let has_img = !app.comment_image_path.is_empty();
        p.text(
            img_btn.center(),
            Align2::CENTER_CENTER,
            "图",
            FontId::proportional(12.0),
            if has_img { theme::PINK } else { theme::TEXT3 },
        );
    }

    let hint = match &target {
        Some((_, name)) => format!("回复 @{name}"),
        None => "进来和UP唠会嗑呗".to_owned(),
    };

    // 原生输入框同步：NativeActivity 的窗口没有输入连接，软键盘只会吐数字，
    // 所以打字交给隐藏的 EditText，这里每帧把内容镜像回来自己画。
    if app.comment_ime {
        if let Some(t) = crate::video::ime_text() {
            if t != app.comment_text {
                app.comment_text = t;
            }
        }
    }
    {
        let p = ui.painter();
        let text_rect = pill.shrink2(Vec2::new(14.0, 7.0));
        if app.comment_text.is_empty() {
            p.text(
                Pos2::new(text_rect.left(), text_rect.center().y),
                Align2::LEFT_CENTER,
                &hint,
                FontId::proportional(12.0),
                theme::TEXT3,
            );
        } else {
            let clipped = p.with_clip_rect(text_rect);
            let g = clipped.layout_no_wrap(
                app.comment_text.clone(),
                FontId::proportional(12.0),
                theme::TEXT,
            );
            // 太长时让光标停在末尾，前面被裁掉
            let x = if g.size().x > text_rect.width() {
                text_rect.right() - g.size().x
            } else {
                text_rect.left()
            };
            let y = text_rect.center().y - g.size().y / 2.0;
            clipped.galley(Pos2::new(x, y), g.clone(), theme::TEXT);
            if app.comment_ime {
                let caret_x = (text_rect.left() + g.size().x).min(text_rect.right());
                p.line_segment(
                    [
                        Pos2::new(caret_x, text_rect.center().y - 7.0),
                        Pos2::new(caret_x, text_rect.center().y + 7.0),
                    ],
                    Stroke::new(1.5, theme::PINK),
                );
            }
        }
    }

    // 整条输入栏是一个点击区，按坐标分派：输入区拉起原生键盘 / 发送 / 配图
    if bar_resp.clicked() {
        if let Some(pos) = bar_resp.interact_pointer_pos() {
            if send_btn.contains(pos) {
                send = true;
            } else if img_btn.contains(pos) {
                #[cfg(target_os = "android")]
                {
                    // 打开系统相册选图；选完由 picked_image 回填路径
                    app.comment_image_open = true;
                    crate::video::pick_image();
                }
                #[cfg(not(target_os = "android"))]
                {
                    app.comment_image_open = !app.comment_image_open;
                }
            } else if pill.contains(pos) {
                // 每次点输入区都把原生键盘重新叫起来（系统收下去之后还能再点开）
                app.comment_ime = true;
                crate::video::ime_open(&app.comment_text);
            }
        }
    }
    if app.comment_ime && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        send = true;
    }

    if cancel {
        app.reply_to = None;
    }
    if send {
        // 先收键盘，再发；文本留给 post_comment 读，失败还能重试
        app.comment_ime = false;
        crate::video::ime_close();
        crate::video::ime_clear();
        app.post_comment(video_id);
    }
}
