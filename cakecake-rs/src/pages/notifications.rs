//! 消息中心
//!
//!   GET    /api/v1/notifications?category=…（回复我的/@我/收到的赞/系统通知）
//!   PATCH  /api/v1/notifications/read-by-category?category=…
//!   PATCH  /api/v1/notifications/:id/read
//!   DELETE /api/v1/notifications/:id
//!
//! 每条消息的头像、发送人、时间、内容都来自接口的 formatNotification。

use crate::{
    api::Notification,
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// (接口 category, 界面文案)
const CATS: [(&str, &str); 4] = [
    ("reply_received", "回复我的"),
    ("at_me", "@我"),
    ("like_aggregation", "收到的赞"),
    ("system_notice", "系统通知"),
];

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);
    if !app.client.logged_in() {
        ui.vertical_centered(|ui| {
            ui.add_space(110.0);
            ui.label(
                egui::RichText::new("登录后可以在这里看消息")
                    .color(theme::TEXT3)
                    .size(12.0),
            );
        });
        return;
    }

    tabs(ui, app);

    match app.notifications_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(90.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_notifications();
            }
        }
        LoadState::Ready => {
            if app.notifications.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(90.0);
                    ui.label(
                        egui::RichText::new("这个分类下还没有消息")
                            .color(theme::TEXT3)
                            .size(12.0),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .id_salt("notifications-scroll")
                    .show(ui, |ui| {
                        let mut open = None;
                        let mut del = None;
                        for n in &app.notifications {
                            match row(ui, app, n) {
                                RowHit::Open => {
                                    open = Some((n.id, n.video_id, n.is_read, target_comment(n)))
                                }
                                RowHit::Delete => del = Some(n.id),
                                RowHit::None => {}
                            }
                        }
                        ui.add_space(10.0);
                        if let Some(id) = del {
                            app.delete_notification(id);
                        } else if let Some((id, video_id, is_read, comment_id)) = open {
                            if !is_read {
                                app.mark_notification_read(id);
                            }
                            if video_id > 0 {
                                if comment_id > 0 {
                                    app.open_detail_at_comment(video_id, comment_id);
                                } else {
                                    app.open_detail(video_id);
                                }
                            }
                        }
                    });
            }
        }
    }

    if let Some(msg) = app.notify_msg.clone() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
        });
    }
}

enum RowHit {
    None,
    Open,
    Delete,
}

/// 通知要定位的评论 id：回复类取新回复，点赞类取被赞评论
fn target_comment(n: &Notification) -> u64 {
    if n.reply_comment_id > 0 {
        n.reply_comment_id
    } else {
        n.liked_comment_id
    }
}

fn header(ui: &mut egui::Ui, app: &mut App) {
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
    if ui
        .interact(
            Rect::from_min_size(rect.min, Vec2::new(84.0, rect.height())),
            egui::Id::new("notifications-back"),
            Sense::click(),
        )
        .clicked()
    {
        app.back();
        return;
    }

    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "消息",
        FontId::proportional(15.0),
        theme::TEXT,
    );

    if app.client.logged_in() {
        let btn = Rect::from_min_size(
            Pos2::new(rect.right() - 80.0, rect.center().y - 12.0),
            Vec2::new(66.0, 24.0),
        );
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "全部已读",
            FontId::proportional(11.0),
            theme::PINK,
        );
        if ui
            .interact(btn, egui::Id::new("notifications-read-all"), Sense::click())
            .clicked()
        {
            app.mark_all_notifications_read();
        }
    }
}

fn tabs(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::hover());
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    let slot = w / CATS.len() as f32;
    for (i, (cat, label)) in CATS.iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let active = app.notification_cat == *cat;
        p.text(
            Pos2::new(cx, rect.center().y - 2.0),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(if active { 13.0 } else { 12.0 }),
            if active { theme::TEXT } else { theme::TEXT2 },
        );
        // 未读角标：数字来自 /notifications/unread-summary
        let unread = app.unread.of(cat);
        if unread > 0 {
            let tw = p
                .layout_no_wrap(
                    (*label).to_owned(),
                    FontId::proportional(12.0),
                    theme::TEXT,
                )
                .size()
                .x;
            let badge = Rect::from_center_size(
                Pos2::new(cx + tw / 2.0 + 8.0, rect.center().y - 9.0),
                Vec2::new(14.0, 12.0),
            );
            p.rect_filled(badge, 6.0, theme::PINK);
            p.text(
                badge.center(),
                Align2::CENTER_CENTER,
                if unread > 9 {
                    "9+".to_owned()
                } else {
                    unread.to_string()
                },
                FontId::proportional(8.0),
                Color32::WHITE,
            );
        }
        if active {
            p.rect_filled(
                Rect::from_center_size(Pos2::new(cx, rect.bottom() - 3.0), Vec2::new(20.0, 3.0)),
                1.5,
                theme::PINK,
            );
        }
        let hit = Rect::from_min_size(
            Pos2::new(rect.left() + slot * i as f32, rect.top()),
            Vec2::new(slot, rect.height()),
        );
        if ui
            .interact(hit, egui::Id::new(("notif-cat", i)), Sense::click())
            .clicked()
        {
            app.select_notification_cat(cat);
        }
    }
}

fn row(ui: &mut egui::Ui, app: &App, n: &Notification) -> RowHit {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 74.0), Sense::click());

    let name = if !n.sender_username.is_empty() {
        n.sender_username.clone()
    } else if n.sender_names.is_empty() {
        if n.kind == "system_notice" {
            "系统通知".to_owned()
        } else {
            "有人".to_owned()
        }
    } else {
        n.sender_names.join("、")
    };
    let avatar = n.sender_avatar_urls.first().cloned().unwrap_or_default();
    let seed = n.sender_names.first().map(|s| s.len() as u64).unwrap_or(n.id);
    let av = Pos2::new(rect.left() + 26.0, rect.top() + 24.0);
    if avatar.is_empty() {
        widgets::avatar(ui.painter(), av, 14.0, &name, seed);
    } else {
        widgets::avatar_tex(ui, av, 14.0, &name, &avatar, &app.covers, seed);
    }

    let p = ui.painter();

    // 未读小圆点
    if !n.is_read {
        p.circle_filled(Pos2::new(rect.left() + 7.0, rect.top() + 14.0), 2.5, theme::PINK);
    }

    let name_lines = widgets::wrap_lines(p, &name, 12.0, w - 90.0, 1);
    let mut y = rect.top() + 12.0;
    for l in name_lines {
        p.text(
            Pos2::new(rect.left() + 48.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(12.0),
            theme::TEXT2,
        );
    }

    let body = if !n.message.is_empty() {
        n.message.clone()
    } else if !n.reply_content.is_empty() {
        n.reply_content.clone()
    } else if !n.comment_full_text.is_empty() {
        format!("“{}”", n.comment_full_text)
    } else if !n.comment_preview.is_empty() {
        n.comment_preview.clone()
    } else {
        "（无更多内容）".to_owned()
    };
    let body_lines = widgets::wrap_lines(p, &body, 12.0, w - 90.0, 2);
    y += 20.0;
    for l in body_lines {
        p.text(
            Pos2::new(rect.left() + 48.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(12.0),
            if n.is_read { theme::TEXT2 } else { theme::TEXT },
        );
        y += 17.0;
    }

    let time = n.created_at.chars().take(16).collect::<String>();
    p.text(
        Pos2::new(rect.right() - 14.0, rect.top() + 14.0),
        Align2::RIGHT_TOP,
        &time,
        FontId::proportional(9.0),
        theme::TEXT3,
    );

    // 右下角：删除
    let del = Rect::from_min_size(
        Pos2::new(rect.right() - 46.0, rect.bottom() - 26.0),
        Vec2::new(34.0, 20.0),
    );
    p.text(
        del.center(),
        Align2::CENTER_CENTER,
        "删除",
        FontId::proportional(10.0),
        theme::TEXT3,
    );
    let del_clicked = ui
        .interact(del, egui::Id::new(("notif-del", n.id)), Sense::click())
        .clicked();

    p.line_segment(
        [
            Pos2::new(rect.left() + 14.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    if del_clicked {
        RowHit::Delete
    } else if resp.clicked() {
        RowHit::Open
    } else {
        RowHit::None
    }
}
