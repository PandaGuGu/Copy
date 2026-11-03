//! 私信：会话列表 + 聊天
//!
//!   GET    /dm/conversations
//!   GET    /dm/conversations/:id/messages
//!   POST   /dm/conversations/:id/messages
//!   PATCH  /dm/conversations/:id/settings（置顶 / 免打扰）
//!   POST   /dm/conversations/:id/reset · DELETE /dm/conversations/:id
//!
//! 权限：只有会话里的两个人能看，后端会校验，客户端不做额外过滤。

use crate::{
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    if widgets::back_bar(ui, "私信", "dm-back") {
        app.back();
        return;
    }
    if !app.client.logged_in() {
        center(ui, "登录后才能收发私信");
        return;
    }

    match app.dm_state.clone() {
        LoadState::Idle | LoadState::Loading => center(ui, "正在加载会话 …"),
        LoadState::Err(e) => {
            if App::empty_state(ui, "会话加载失败", &e) {
                app.dm_state = LoadState::Idle;
                app.load_dm();
            }
        }
        LoadState::Ready => {
            if app.dm_convs.is_empty() {
                center(ui, "还没有私信。去 UP 主主页点「私信」开始聊");
                return;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("dm-list")
                .show(ui, |ui| {
                    let convs = app.dm_convs.clone();
                    let mut open = None;
                    let mut pin = None;
                    let mut mute = None;
                    let mut del = None;
                    for c in &convs {
                        match row(ui, app, c) {
                            RowHit::Open => open = Some(c.clone()),
                            RowHit::Pin => pin = Some((c.id, c.pinned)),
                            RowHit::Mute => mute = Some((c.id, c.muted)),
                            RowHit::Delete => del = Some(c.id),
                            RowHit::None => {}
                        }
                    }
                    if let Some(c) = open {
                        app.open_dm_chat(&c);
                    }
                    if let Some((id, on)) = pin {
                        app.dm_patch(id, Some(!on), None);
                    }
                    if let Some((id, on)) = mute {
                        app.dm_patch(id, None, Some(!on));
                    }
                    if let Some(id) = del {
                        app.dm_delete(id);
                    }
                });
        }
    }
}

fn center(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}

enum RowHit {
    None,
    Open,
    Pin,
    Mute,
    Delete,
}

fn row(ui: &mut egui::Ui, app: &mut App, c: &crate::api::DmConversation) -> RowHit {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 70.0), Sense::click());
    let av = Pos2::new(rect.left() + 30.0, rect.center().y);
    if c.peer_avatar.is_empty() {
        widgets::avatar(ui.painter(), av, 19.0, &c.peer_name, c.peer_id);
    } else {
        widgets::avatar_tex(
            ui,
            av,
            19.0,
            &c.peer_name,
            &c.peer_avatar,
            &app.covers,
            c.peer_id,
        );
    }
    let p = ui.painter();
    let name = if c.peer_name.is_empty() {
        format!("用户 {}", c.peer_id)
    } else {
        c.peer_name.clone()
    };
    p.text(
        Pos2::new(rect.left() + 58.0, rect.top() + 14.0),
        Align2::LEFT_TOP,
        &name,
        FontId::proportional(13.0),
        theme::TEXT,
    );
    if c.pinned {
        p.text(
            Pos2::new(rect.left() + 58.0 + name.chars().count() as f32 * 13.5 + 6.0, rect.top() + 16.0),
            Align2::LEFT_TOP,
            "置顶",
            FontId::proportional(9.0),
            theme::PINK,
        );
    }
    if c.muted {
        p.text(
            Pos2::new(rect.right() - 74.0, rect.top() + 16.0),
            Align2::LEFT_TOP,
            "免打扰",
            FontId::proportional(9.0),
            theme::TEXT3,
        );
    }
    let lines = widgets::wrap_lines(p, &c.last_preview, 11.5, w - 130.0, 1);
    if let Some(l) = lines.first() {
        p.text(
            Pos2::new(rect.left() + 58.0, rect.top() + 38.0),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(11.5),
            theme::TEXT3,
        );
    }
    if !c.last_message_at.is_empty() {
        p.text(
            Pos2::new(rect.right() - 14.0, rect.top() + 14.0),
            Align2::RIGHT_TOP,
            c.last_message_at.chars().skip(5).take(11).collect::<String>(),
            FontId::proportional(9.0),
            theme::TEXT3,
        );
    }
    if c.unread_count > 0 {
        let badge = Rect::from_center_size(
            Pos2::new(rect.right() - 20.0, rect.center().y + 8.0),
            Vec2::new(18.0, 15.0),
        );
        p.rect_filled(badge, 7.0, theme::PINK);
        p.text(
            badge.center(),
            Align2::CENTER_CENTER,
            if c.unread_count > 99 {
                "99+".to_owned()
            } else {
                c.unread_count.to_string()
            },
            FontId::proportional(9.0),
            Color32::WHITE,
        );
    }

    // 右侧三个小动作：置顶 / 免打扰 / 删除
    let ops = [
        ("置顶", Rect::from_center_size(Pos2::new(rect.right() - 118.0, rect.bottom() - 16.0), Vec2::new(34.0, 22.0))),
        ("免打扰", Rect::from_center_size(Pos2::new(rect.right() - 78.0, rect.bottom() - 16.0), Vec2::new(46.0, 22.0))),
        ("删除", Rect::from_center_size(Pos2::new(rect.right() - 32.0, rect.bottom() - 16.0), Vec2::new(34.0, 22.0))),
    ];
    for (label, r) in ops {
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(9.5),
            theme::TEXT3,
        );
    }
    ui.painter().line_segment(
        [
            Pos2::new(rect.left() + 14.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if ops[0].1.contains(pos) {
                return RowHit::Pin;
            }
            if ops[1].1.contains(pos) {
                return RowHit::Mute;
            }
            if ops[2].1.contains(pos) {
                return RowHit::Delete;
            }
            return RowHit::Open;
        }
    }
    RowHit::None
}

/// 聊天页
pub fn chat(ui: &mut egui::Ui, app: &mut App, conv_id: u64) {
    if widgets::back_bar(ui, &app.dm_peer_name, "dmchat-back") {
        app.back();
        return;
    }

    // 底部固定输入栏
    egui::Panel::bottom("dm-composer")
        .exact_size(54.0)
        .frame(egui::Frame::NONE.fill(theme::BG))
        .show(ui, |ui| {
            let w = ui.available_width();
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 46.0), Sense::click());
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
                let ready = !app.dm_text.trim().is_empty();
                p.circle_filled(
                    send_btn.center(),
                    15.0,
                    if ready { theme::PINK } else { theme::SURFACE },
                );
                p.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(send_btn.center().x - 5.0, send_btn.center().y - 6.0),
                        Pos2::new(send_btn.center().x + 6.0, send_btn.center().y),
                        Pos2::new(send_btn.center().x - 5.0, send_btn.center().y + 6.0),
                    ],
                    if ready { Color32::WHITE } else { theme::TEXT3 },
                    Stroke::NONE,
                ));
            }
            let field = ui.put(
                pill.shrink2(Vec2::new(14.0, 7.0)),
                egui::TextEdit::singleline(&mut app.dm_text)
                    .id(egui::Id::new("dm-input"))
                    .frame(egui::Frame::NONE)
                    .hint_text("发条消息…")
                    .font(FontId::proportional(12.0)),
            );
            let focused = field.has_focus();
            app.note_focus(focused);
            let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let clicked = resp.clicked()
                && resp
                    .interact_pointer_pos()
                    .map(|p| send_btn.contains(p))
                    .unwrap_or(false);
            if enter || clicked {
                app.send_dm(conv_id);
            }
        });

    match app.dm_chat_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label(egui::RichText::new("正在加载消息 …").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "消息加载失败", &e) {
                app.load_dm_chat(conv_id);
            }
        }
        LoadState::Ready => {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .id_salt("dm-chat-scroll")
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    for m in &app.dm_chat {
                        bubble(ui, m);
                    }
                    ui.add_space(8.0);
                });
        }
    }

    if let Some(msg) = app.action_msg.clone() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
        });
    }
}

fn bubble(ui: &mut egui::Ui, m: &crate::api::DmMessage) {
    let w = ui.available_width();
    let max_w = w * 0.72;
    let lines = widgets::wrap_lines(ui.painter(), &m.content, 12.5, max_w - 20.0, 20);
    let body_h = lines.len().max(1) as f32 * 18.0;
    let h = body_h + 18.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h + 6.0), Sense::hover());
    let p = ui.painter();

    let text_w = lines
        .iter()
        .map(|l| l.chars().count() as f32 * 12.0)
        .fold(0.0f32, f32::max)
        .min(max_w - 20.0);
    let bubble_w = (text_w + 20.0).max(48.0);
    let bubble = if m.from_me {
        Rect::from_min_size(
            Pos2::new(rect.right() - 14.0 - bubble_w, rect.top() + 3.0),
            Vec2::new(bubble_w, body_h + 14.0),
        )
    } else {
        Rect::from_min_size(
            Pos2::new(rect.left() + 14.0, rect.top() + 3.0),
            Vec2::new(bubble_w, body_h + 14.0),
        )
    };
    p.rect_filled(
        bubble,
        8.0,
        if m.from_me { theme::PINK } else { theme::SURFACE },
    );
    let mut y = bubble.top() + 7.0;
    for l in &lines {
        p.text(
            Pos2::new(bubble.left() + 10.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(12.5),
            if m.from_me { Color32::WHITE } else { theme::TEXT },
        );
        y += 18.0;
    }
}
