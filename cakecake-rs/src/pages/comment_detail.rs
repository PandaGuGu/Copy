//! 评论详情：一条一级评论 + 它下面的全部回复
//!
//! 从视频页的一级评论卡片（灰块「共 N 条回复」）点进来。
//! 数据不重新请求：直接复用 `app.detail` 里已经拉下来的评论树。
//!
//! 交互：底部固定输入栏，回复某条时自动带上 `@昵称`；右上角可切换「按时间 / 按热度」。

use crate::{
    api::Comment,
    app::App,
    pages::comment_ui,
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App, video_id: u64, comment_id: u64) {
    if header(ui, app) {
        return;
    }

    let Some(data) = app.detail.take() else {
        // 没数据说明详情还没回来或已失效，别硬编内容
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label(
                egui::RichText::new("评论还没加载出来")
                    .color(theme::TEXT3)
                    .size(12.0),
            );
            ui.add_space(10.0);
            if ui.button("返回视频页").clicked() {
                app.back();
            }
        });
        return;
    };

    // 底部输入栏做成悬浮层，钉在「键盘/导航栏之上」。
    // Panel::bottom 在页面高度不够时会被键盘挤出可视区（整条被裁掉），
    // 所以改用 Area 按屏幕坐标定位，和视频详情页评论 tab 保持一致。
    let bar_h = if app.reply_to.is_some() { 78.0 } else { 54.0 };
    let screen = ui.ctx().viewport_rect();
    let top = (screen.bottom() - app.bottom_inset - bar_h).max(0.0);
    egui::Area::new(egui::Id::new("comment-detail-composer"))
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::new(0.0, top))
        .show(ui.ctx(), |ui| {
            ui.set_width(screen.width());
            ui.set_height(bar_h);
            egui::Frame::NONE
                .fill(theme::BG)
                .stroke(Stroke::new(1.0, theme::LINE))
                .inner_margin(egui::Margin::same(0))
                .show(ui, |ui| {
                    comment_ui::composer(ui, app, video_id);
                });
        });

    let list = &data.comments;
    let (roots, children) = comment_ui::thread_index(list);

    // 目标评论可能是一级评论，也可能是楼里的某条回复：统一往上找到它所属的楼
    let mut root_idx = None;
    for &i in &roots {
        if list[i].id == comment_id {
            root_idx = Some(i);
            break;
        }
        let kids = comment_ui::descendants(list, &children, i);
        if kids.iter().any(|&k| list[k].id == comment_id) {
            root_idx = Some(i);
            break;
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("comment-detail-scroll")
        .show(ui, |ui| {
            let Some(ri) = root_idx else {
                ui.add_space(40.0);
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new("这条评论已被删除")
                            .color(theme::TEXT3)
                            .size(12.0),
                    );
                });
                return;
            };

            let root = &list[ri];
            let kids = comment_ui::descendants(list, &children, ri);
            let hit = root_block(ui, app, root, kids.len());
            if hit.like {
                app.toggle_comment_like(video_id, root.id);
            }
            if hit.dislike {
                app.toggle_comment_dislike(video_id, root.id);
            }

            replies_header(ui, app, kids.len());

            let latest = app.comment_sort == "latest";
            let mut ordered = kids.clone();
            ordered.sort_by(|a, z| {
                let (ca, cz) = (&list[*a], &list[*z]);
                if latest {
                    cz.created_at.cmp(&ca.created_at)
                } else {
                    cz.like_count.cmp(&ca.like_count)
                }
            });

            let mut like: Option<u64> = None;
            let mut dislike: Option<u64> = None;
            for &k in &ordered {
                let h = reply_row(ui, app, list, k, root.id);
                if h.like {
                    like = Some(list[k].id);
                }
                if h.dislike {
                    dislike = Some(list[k].id);
                }
            }
            // 悬浮输入栏会盖住列表底部，留出等高的滚动余量
            ui.add_space(12.0 + bar_h);

            if let Some(id) = like {
                app.toggle_comment_like(video_id, id);
            }
            if let Some(id) = dislike {
                app.toggle_comment_dislike(video_id, id);
            }
        });

    app.detail = Some(data);
}

#[derive(Default)]
struct Hit {
    like: bool,
    dislike: bool,
}

fn header(ui: &mut egui::Ui, app: &mut App) -> bool {
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
    p.text(
        Pos2::new(rect.center().x, rect.center().y),
        Align2::CENTER_CENTER,
        "评论详情",
        FontId::proportional(15.0),
        theme::TEXT,
    );

    // 右上角关闭：和返回等价，手指更好点
    let close = Pos2::new(rect.right() - 24.0, rect.center().y);
    p.line_segment(
        [
            Pos2::new(close.x - 6.0, close.y - 6.0),
            Pos2::new(close.x + 6.0, close.y + 6.0),
        ],
        Stroke::new(1.4, theme::TEXT2),
    );
    p.line_segment(
        [
            Pos2::new(close.x + 6.0, close.y - 6.0),
            Pos2::new(close.x - 6.0, close.y + 6.0),
        ],
        Stroke::new(1.4, theme::TEXT2),
    );
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    let back_hit = Rect::from_min_size(rect.min, Vec2::new(84.0, rect.height()));
    let close_hit = Rect::from_center_size(close, Vec2::splat(40.0));
    let back = ui
        .interact(back_hit, egui::Id::new("cd-back"), Sense::click())
        .clicked();
    let closed = ui
        .interact(close_hit, egui::Id::new("cd-close"), Sense::click())
        .clicked();
    if back || closed {
        app.back();
        return true;
    }
    false
}

/// 主角：被点开的那条一级评论（完整展示，不截断）
fn root_block(ui: &mut egui::Ui, app: &mut App, c: &Comment, reply_count: usize) -> Hit {
    let w = ui.available_width();
    let text_x = 14.0 + 32.0;
    let text_w = (w - text_x - 14.0).max(80.0);
    let lines = widgets::wrap_lines(ui.painter(), &c.content, 13.0, text_w, 40);
    let body_h = lines.len().max(1) as f32 * 19.0;
    let h = 34.0 + body_h + 30.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    let av = Pos2::new(rect.left() + 14.0 + 14.0, rect.top() + 24.0);
    comment_ui::avatar(ui, app, c, av, 14.0);

    let p = ui.painter();
    comment_ui::name_line(p, rect.left() + text_x, rect.top() + 10.0, c);

    let mut y = rect.top() + 34.0;
    for l in &lines {
        p.text(
            Pos2::new(rect.left() + text_x, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(13.0),
            theme::TEXT,
        );
        y += 19.0;
    }

    // 时间 · 地区 / 回复 / 赞踩
    let meta_y = rect.top() + 34.0 + body_h + 12.0;
    p.text(
        Pos2::new(rect.left() + text_x, meta_y),
        Align2::LEFT_CENTER,
        comment_ui::time_place(c),
        FontId::proportional(10.5),
        theme::TEXT3,
    );
    let like_text = if c.like_count == 0 {
        String::new()
    } else {
        theme::human(c.like_count)
    };
    let rects = comment_ui::action_rects(rect.right() - 14.0, meta_y, &like_text);
    comment_ui::draw_actions(p, c, &rects, ui.ctx().pointer_hover_pos());

    let _ = reply_count;
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    // 整行是一个点击区，按坐标分派到「回复 / 赞 / 踩」
    let mut hit = Hit::default();
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            match comment_ui::hit_actions(&rects, pos) {
                1 => app.reply_to = Some((c.id, comment_ui::name_of(c))),
                2 => hit.like = true,
                3 => hit.dislike = true,
                _ => {}
            }
        }
    }
    hit
}

/// 「相关回复共 N 条」+ 排序开关
fn replies_header(ui: &mut egui::Ui, app: &mut App, count: usize) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::click());
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        format!("相关回复共 {count} 条"),
        FontId::proportional(12.0),
        theme::TEXT2,
    );

    let latest = app.comment_sort == "latest";
    let label = if latest { "按时间" } else { "按热度" };
    let hit = Rect::from_min_size(
        Pos2::new(rect.right() - 80.0, rect.top()),
        Vec2::new(66.0, rect.height()),
    );
    let hovered = hit.contains(ui.ctx().pointer_hover_pos().unwrap_or(Pos2::ZERO));
    let ink = if hovered { theme::TEXT } else { theme::TEXT3 };
    let p = ui.painter();
    for i in 0..3 {
        let y = rect.center().y - 4.0 + i as f32 * 4.0;
        p.line_segment(
            [
                Pos2::new(rect.right() - 76.0, y),
                Pos2::new(rect.right() - 64.0, y),
            ],
            Stroke::new(1.2, ink),
        );
    }
    p.text(
        Pos2::new(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        label,
        FontId::proportional(11.5),
        ink,
    );
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if hit.contains(pos) {
                app.set_comment_sort(if latest { "hot" } else { "latest" });
            }
        }
    }
}

/// 一条楼内回复（带「回复 @某某」的前缀）
fn reply_row(ui: &mut egui::Ui, app: &mut App, list: &[Comment], idx: usize, root_id: u64) -> Hit {
    let c = &list[idx];
    let w = ui.available_width();
    let text_x = 14.0 + 28.0;
    let text_w = (w - text_x - 14.0).max(80.0);

    // 回复的不是楼主时，把被回复的人标出来
    let to_name = if c.parent_id != 0 && c.parent_id != root_id {
        list.iter()
            .find(|p| p.id == c.parent_id)
            .map(comment_ui::name_of)
    } else {
        None
    };
    let to_line = to_name
        .as_ref()
        .map(|n| format!("回复 @{n}"))
        .unwrap_or_default();
    let lines = widgets::wrap_lines(ui.painter(), &c.content, 12.5, text_w, 8);
    let body_h = lines.len().max(1) as f32 * 18.0;
    let head_h = if to_name.is_some() { 18.0 } else { 0.0 };
    let h = 10.0 + 20.0 + head_h + body_h + 22.0 + 12.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    let av = Pos2::new(rect.left() + 14.0 + 12.0, rect.top() + 22.0);
    comment_ui::avatar(ui, app, c, av, 12.0);

    let p = ui.painter();
    comment_ui::name_line(p, rect.left() + text_x, rect.top() + 8.0, c);

    let mut y = rect.top() + 30.0;
    if to_name.is_some() {
        p.text(
            Pos2::new(rect.left() + text_x, y - 2.0),
            Align2::LEFT_TOP,
            &to_line,
            FontId::proportional(10.5),
            theme::BLUE,
        );
        y += head_h;
    }
    for l in &lines {
        p.text(
            Pos2::new(rect.left() + text_x, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(12.5),
            theme::TEXT,
        );
        y += 18.0;
    }

    let meta_y = rect.top() + 30.0 + head_h + body_h + 12.0;
    p.text(
        Pos2::new(rect.left() + text_x, meta_y),
        Align2::LEFT_CENTER,
        comment_ui::time_place(c),
        FontId::proportional(10.5),
        theme::TEXT3,
    );
    let like_text = if c.like_count == 0 {
        String::new()
    } else {
        theme::human(c.like_count)
    };
    let rects = comment_ui::action_rects(rect.right() - 14.0, meta_y, &like_text);
    comment_ui::draw_actions(p, c, &rects, ui.ctx().pointer_hover_pos());

    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left() + 14.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    let mut hit = Hit::default();
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            match comment_ui::hit_actions(&rects, pos) {
                1 => app.reply_to = Some((c.id, comment_ui::name_of(c))),
                2 => hit.like = true,
                3 => hit.dislike = true,
                _ => {}
            }
        }
    }
    hit
}
