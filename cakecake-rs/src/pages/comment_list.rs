//! 视频详情页的评论列表：一级评论卡片 + 楼中楼预览
//!
//! 交互和 B 站一致：
//!   · 一级评论下面用灰块预览第一条回复和「共 N 条回复」，整块可点
//!   · 点进「评论详情」看这一楼的全部回复（`pages::comment_detail`）
//!   · 楼里不再逐条展开，详情页里才铺开，列表因此短很多
//!
//! 数据仍是一次性拉回来的全量评论树（`/videos/:id/comments`），
//! 进详情页不发新请求。
//!
//! 命中判定：整张卡片只注册一个有 click 语义的区域，点完按坐标分派到
//! 「回复 / 赞 / 踩 / 回复预览块」——滚动区域里这样最稳。

use crate::{
    api::Comment,
    app::App,
    pages::comment_ui,
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App, video_id: u64, list: &[Comment]) {
    if list.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("还没有评论，来抢个沙发")
                    .color(theme::TEXT3)
                    .size(11.0),
            );
        });
        ui.add_space(30.0);
        return;
    }

    let (roots, children) = comment_ui::thread_index(list);
    let latest = app.comment_sort == "latest";

    // 置顶永远最前，其余按热度 / 时间
    let mut ordered = roots;
    ordered.sort_by(|a, z| {
        let (ca, cz) = (&list[*a], &list[*z]);
        cz.pinned.cmp(&ca.pinned).then_with(|| {
            if latest {
                cz.created_at.cmp(&ca.created_at)
            } else {
                cz.like_count.cmp(&ca.like_count)
            }
        })
    });

    header(ui, app);

    let mut open_detail: Option<u64> = None;
    let mut like: Option<u64> = None;
    let mut dislike: Option<u64> = None;
    let mut reply: Option<u64> = None;

    for &i in &ordered {
        let kids = comment_ui::descendants(list, &children, i);
        let hit = card(ui, app, list, i, &kids);
        let id = list[i].id;
        if hit.open {
            open_detail = Some(id);
        }
        if hit.like {
            like = Some(id);
        }
        if hit.dislike {
            dislike = Some(id);
        }
        if hit.reply {
            reply = Some(id);
        }
    }
    ui.add_space(8.0);

    if let Some(id) = reply {
        let name = list
            .iter()
            .find(|c| c.id == id)
            .map(comment_ui::name_of)
            .unwrap_or_default();
        app.reply_to = Some((id, name));
    }
    if let Some(id) = like {
        app.toggle_comment_like(video_id, id);
    }
    if let Some(id) = dislike {
        app.toggle_comment_dislike(video_id, id);
    }
    if let Some(id) = open_detail {
        app.open_comment_detail(video_id, id);
    }
}

/// 「热门评论」+ 右侧排序开关
fn header(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 34.0), Sense::click());
    let latest = app.comment_sort == "latest";
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        if latest { "最新评论" } else { "热门评论" },
        FontId::proportional(12.5),
        theme::TEXT2,
    );

    // 右侧：三条横线 + 当前排序，点一下切换
    let label = if latest { "按时间" } else { "按热度" };
    let ink = if resp.hovered() {
        theme::TEXT
    } else {
        theme::TEXT3
    };
    let p = ui.painter();
    for i in 0..3 {
        let y = rect.center().y - 4.0 + i as f32 * 4.0;
        p.line_segment(
            [
                Pos2::new(rect.right() - 72.0, y),
                Pos2::new(rect.right() - 60.0, y),
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
        app.set_comment_sort(if latest { "hot" } else { "latest" });
    }
}

#[derive(Default)]
struct Hit {
    open: bool,
    like: bool,
    dislike: bool,
    reply: bool,
}

/// 一条一级评论：头像/昵称/内容/时间/赞踩 + 楼中楼预览块
fn card(ui: &mut egui::Ui, app: &mut App, list: &[Comment], idx: usize, kids: &[usize]) -> Hit {
    let c = &list[idx];
    let w = ui.available_width();
    let left = 14.0;
    let text_x = left + 30.0;
    let text_w = (w - text_x - 14.0).max(80.0);

    let lines = widgets::wrap_lines(ui.painter(), &c.content, 12.5, text_w, 6);
    let body_h = lines.len().max(1) as f32 * 18.0;
    let img_h = comment_ui::images_h(c);

    // 预览块：第一条回复（2 行）+「共 N 条回复」
    let first = kids.first().map(|&k| &list[k]);
    let first_lines = first
        .map(|f| {
            widgets::wrap_lines(
                ui.painter(),
                &format!("{}：{}", comment_ui::name_of(f), f.content),
                11.5,
                text_w - 16.0,
                2,
            )
        })
        .unwrap_or_default();
    let preview_h = if kids.is_empty() {
        0.0
    } else {
        8.0 + first_lines.len().max(1) as f32 * 16.0 + 20.0 + 8.0
    };

    let h = 30.0 + body_h + img_h + 22.0 + preview_h + 12.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    let av = Pos2::new(rect.left() + left + 13.0, rect.top() + 22.0);
    comment_ui::avatar(ui, app, c, av, 13.0);

    let p = ui.painter().clone();
    comment_ui::name_line(&p, rect.left() + text_x, rect.top() + 8.0, c);

    let mut y = rect.top() + 30.0;
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

    // 评论配图
    comment_ui::draw_images(
        ui,
        app,
        c,
        rect.left() + text_x,
        rect.top() + 30.0 + body_h + 4.0,
        text_w,
    );

    // 时间 · 地区 …… 回复 / 赞 / 踩
    let meta_y = rect.top() + 30.0 + body_h + img_h + 10.0;
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
    comment_ui::draw_actions(&p, c, &rects, ui.ctx().pointer_hover_pos());

    // 楼中楼预览块
    let mut box_rect = Rect::NOTHING;
    if !kids.is_empty() {
        box_rect = Rect::from_min_max(
            Pos2::new(rect.left() + text_x, meta_y + 14.0),
            Pos2::new(rect.right() - 14.0, rect.top() + h - 10.0),
        );
        let p = ui.painter();
        p.rect_filled(box_rect, 6.0, theme::SURFACE2);

        let mut ty = box_rect.top() + 7.0;
        for l in &first_lines {
            p.text(
                Pos2::new(box_rect.left() + 8.0, ty),
                Align2::LEFT_TOP,
                l,
                FontId::proportional(11.5),
                theme::TEXT2,
            );
            ty += 16.0;
        }
        if let Some(f) = first {
            // 第一条回复也有赞，标在右下角
            if f.like_count > 0 {
                p.text(
                    Pos2::new(box_rect.right() - 8.0, ty - 16.0),
                    Align2::RIGHT_CENTER,
                    theme::human(f.like_count),
                    FontId::proportional(10.0),
                    theme::TEXT3,
                );
            }
        }
        // 「共 N 条回复 ›」——按实际文字宽度接箭头，不要留大缝
        let more = format!("共 {} 条回复", kids.len());
        let g = p.layout_no_wrap(more.clone(), FontId::proportional(11.0), theme::BLUE);
        let more_y = box_rect.bottom() - 16.0;
        p.text(
            Pos2::new(box_rect.left() + 8.0, more_y),
            Align2::LEFT_CENTER,
            more,
            FontId::proportional(11.0),
            theme::BLUE,
        );
        // 手画一个「›」，字体里这个符号太小还有基线偏移
        let ax = box_rect.left() + 12.0 + g.size().x;
        p.add(egui::Shape::line(
            vec![
                Pos2::new(ax, more_y - 3.5),
                Pos2::new(ax + 4.0, more_y),
                Pos2::new(ax, more_y + 3.5),
            ],
            Stroke::new(1.4, theme::BLUE),
        ));
    }

    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left() + left, rect.bottom()),
            Pos2::new(rect.right() - left, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    // 命中分派：按钮优先，其余落到预览块 = 进详情
    let mut hit = Hit::default();
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            match comment_ui::hit_actions(&rects, pos) {
                1 => hit.reply = true,
                2 => hit.like = true,
                3 => hit.dislike = true,
                _ => {
                    if box_rect.contains(pos) {
                        hit.open = true;
                    }
                }
            }
        }
    }
    hit
}
