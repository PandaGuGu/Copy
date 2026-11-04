//! 账号与设置：资料 / 公告 / 密码 / 空间隐私 / 关注分组 / 注销
//!
//! 全部是「本人」权限：接口都在 authd 组，后端也会按 uid 校验，
//! 这里只负责把可改的项摆出来，不做任何越权跳转。

use crate::{
    api::{FollowGroup, SpacePrivacy},
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    if widgets::back_bar(ui, "设置", "settings-back") {
        app.back();
        return;
    }
    if !app.client.logged_in() {
        centered(ui, "登录后才能改这些设置");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("settings-scroll")
        .show(ui, |ui| {
            section(ui, "个人资料");
            field(ui, app, Field::Nickname);
            field(ui, app, Field::Sign);
            gender_row(ui, app);
            field(ui, app, Field::Birthday);
            actions_row(ui, app, "保存资料", Btn::SaveProfile);

            section(ui, "头像");
            avatar_block(ui, app);

            section(ui, "空间公告");
            field(ui, app, Field::Announcement);
            actions_row(ui, app, "保存公告", Btn::SaveAnnouncement);

            section(ui, "修改密码");
            field(ui, app, Field::OldPassword);
            field(ui, app, Field::NewPassword);
            actions_row(ui, app, "修改密码", Btn::SavePassword);

            section(ui, "服务器");
            field(ui, app, Field::Server);
            actions_row(ui, app, "保存并重连", Btn::SaveServer);
            lan_row(ui, app);

            section(ui, "空间隐私");
            privacy_rows(ui, app);

            section(ui, "关注分组");
            groups_block(ui, app);

            section(ui, "注销账号");
            field(ui, app, Field::DeletePassword);
            destructive_row(ui, app);

            if let Some(msg) = app.settings_msg.clone() {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
                });
            }
            ui.add_space(20.0);
        });
}

fn centered(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(title)
                .color(theme::TEXT)
                .size(13.0)
                .strong(),
        );
    });
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width() - 14.0, 6.0), Sense::hover());
        ui.painter().line_segment(
            [Pos2::new(r.left(), r.center().y), Pos2::new(r.right(), r.center().y)],
            Stroke::new(1.0, theme::LINE),
        );
    });
}

#[derive(Clone, Copy)]
enum Field {
    Nickname,
    Sign,
    Birthday,
    Announcement,
    OldPassword,
    NewPassword,
    DeletePassword,
    Server,
}

impl Field {
    fn label(&self) -> &'static str {
        match self {
            Field::Nickname => "昵称",
            Field::Sign => "签名",
            Field::Birthday => "生日",
            Field::Announcement => "公告",
            Field::OldPassword => "当前密码",
            Field::NewPassword => "新密码（8 位以上）",
            Field::DeletePassword => "账号密码",
            Field::Server => "服务器地址",
        }
    }
    fn hint(&self) -> &'static str {
        match self {
            Field::Birthday => "2000-01-01",
            Field::Announcement => "展示在个人主页顶部",
            // 真机走 Wi-Fi 时填电脑的局域网地址（和手机在同一网段）
            Field::Server => "http://192.168.1.10:8080",
            _ => "",
        }
    }
    fn password(&self) -> bool {
        matches!(
            self,
            Field::OldPassword | Field::NewPassword | Field::DeletePassword
        )
    }
}

fn field(ui: &mut egui::Ui, app: &mut App, which: Field) {
    let me = app.me.clone().unwrap_or_default();
    let value = match which {
        Field::Nickname => &mut app.set_nickname,
        Field::Sign => &mut app.set_sign,
        Field::Birthday => &mut app.set_birthday,
        Field::Announcement => &mut app.set_announcement,
        Field::OldPassword => &mut app.set_old_pw,
        Field::NewPassword => &mut app.set_new_pw,
        Field::DeletePassword => &mut app.set_del_pw,
        Field::Server => &mut app.set_server,
    };
    if matches!(which, Field::Nickname) && value.is_empty() {
        *value = me.nickname.clone();
    }
    if matches!(which, Field::Sign) && value.is_empty() {
        *value = me.sign.clone();
    }
    if matches!(which, Field::Server) && value.is_empty() {
        *value = app.client.base.clone();
    }
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(which.label())
                .color(theme::TEXT3)
                .size(11.0),
        );
    });
    let mut focused = false;
    {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            let hint = match which {
                Field::Server => match crate::api::lan_base() {
                    Some(base) => format!("局域网 {base}（下面可自动查找）"),
                    None => "和手机同一 Wi-Fi 的电脑 IP，例如 http://192.168.1.10:8080".to_owned(),
                },
                _ => which.hint().to_owned(),
            };
            let mut edit = egui::TextEdit::singleline(value)
                .hint_text(hint)
                .desired_width(ui.available_width() - 28.0);
            if which.password() {
                edit = edit.password(true);
            }
            focused = ui.add(edit).has_focus();
        });
    }
    app.note_focus(focused);
    ui.add_space(4.0);
}

/// 服务器地址下方：显示当前实际在用的地址，并提供「自动查找电脑」。
/// 真机换 Wi-Fi / 电脑换了 IP 时，不用手敲地址也能回到局域网直连。
fn lan_row(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal_wrapped(|ui| {
        ui.add_space(14.0);
        let media = app
            .client
            .media_origin()
            .unwrap_or_else(|| format!("http://{}:{}", crate::api::host_of(&app.client.base), crate::api::media_port()));
        ui.label(
            egui::RichText::new(format!("当前 {} · 直播源 {media}", app.client.base))
                .color(theme::TEXT3)
                .size(9.5),
        );
    });
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("自动查找电脑")
                        .color(theme::PINK)
                        .size(11.0),
                )
                .fill(theme::SURFACE)
                .corner_radius(11.0)
                .min_size(Vec2::new(0.0, 26.0)),
            )
            .clicked()
        {
            app.scan_for_lan();
        }
        ui.label(
            egui::RichText::new("手机与电脑同一 Wi-Fi 时点这里")
                .color(theme::TEXT3)
                .size(9.5),
        );
    });
    ui.add_space(4.0);
}

fn avatar_block(ui: &mut egui::Ui, app: &mut App) {
    if app.image_files.is_empty() {
        app.scan_images();
    }
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if ui.button("刷新本地图片").clicked() {
            app.scan_images();
        }
        let label = if app.image_path.is_empty() {
            "未选择图片"
        } else {
            std::path::Path::new(&app.image_path)
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or("已选择")
        };
        ui.label(
            egui::RichText::new(label)
                .color(theme::TEXT3)
                .size(10.5),
        );
    });
    ui.horizontal_wrapped(|ui| {
        ui.add_space(14.0);
        for path in app.image_files.clone() {
            let name = std::path::Path::new(&path)
                .file_name()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_default();
            let on = app.image_path == path;
            if ui.selectable_label(on, name).clicked() {
                app.image_path = path;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if ui.button("上传头像").clicked() {
            app.upload_avatar();
        }
        ui.label(
            egui::RichText::new("把图片放进 adb inbox 目录后点刷新")
                .color(theme::TEXT3)
                .size(9.5),
        );
    });
    ui.add_space(4.0);
}

fn gender_row(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(egui::RichText::new("性别").color(theme::TEXT3).size(11.0));
        for (key, label) in [("secret", "保密"), ("male", "男"), ("female", "女")] {
            let on = app.set_gender == key;
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .color(if on { theme::PINK } else { theme::TEXT2 })
                    .size(11.0),
            )
            .fill(theme::SURFACE)
            .corner_radius(11.0)
            .min_size(Vec2::new(0.0, 24.0));
            if ui.add(btn).clicked() {
                app.set_gender = key.to_owned();
            }
        }
    });
    ui.add_space(4.0);
}

#[derive(Clone, Copy, PartialEq)]
enum Btn {
    SaveServer,
    SaveProfile,
    SaveAnnouncement,
    SavePassword,
}

fn actions_row(ui: &mut egui::Ui, app: &mut App, label: &str, btn: Btn) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let busy = app.settings_state == LoadState::Loading;
        if ui
            .add_enabled(
                !busy,
                egui::Button::new(
                    egui::RichText::new(if busy { "提交中…" } else { label })
                        .color(egui::Color32::WHITE)
                        .size(12.0),
                )
                .fill(theme::PINK)
                .corner_radius(14.0)
                .min_size(Vec2::new(96.0, 30.0)),
            )
            .clicked()
        {
            match btn {
                Btn::SaveServer => app.apply_server(),
                Btn::SaveProfile => app.save_profile(),
                Btn::SaveAnnouncement => app.save_announcement(),
                Btn::SavePassword => app.save_password(),
            }
        }
    });
    ui.add_space(2.0);
}

fn privacy_rows(ui: &mut egui::Ui, app: &mut App) {
    let Some(p) = app.privacy.clone() else {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new("读取中…").color(theme::TEXT3).size(11.0));
        });
        return;
    };
    let items: [(&str, bool); 5] = [
        ("公开我的收藏", p.public_favorites),
        ("公开最近投币", p.public_recent_coins),
        ("公开我的关注", p.public_following),
        ("公开我的粉丝", p.public_fans),
        ("公开生日", p.public_birthday),
    ];
    let mut next = p.clone();
    let mut changed = false;
    for (i, (label, on)) in items.iter().enumerate() {
        let (rect, resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 40.0),
            Sense::click(),
        );
        let p2 = ui.painter();
        p2.text(
            Pos2::new(rect.left() + 14.0, rect.center().y),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(12.5),
            theme::TEXT,
        );
        // 开关
        let track = Rect::from_center_size(
            Pos2::new(rect.right() - 36.0, rect.center().y),
            Vec2::new(38.0, 20.0),
        );
        p2.rect_filled(track, 10.0, if *on { theme::PINK } else { theme::LINE });
        p2.circle_filled(
            Pos2::new(
                if *on { track.right() - 10.0 } else { track.left() + 10.0 },
                track.center().y,
            ),
            7.5,
            egui::Color32::WHITE,
        );
        p2.line_segment(
            [
                Pos2::new(rect.left() + 14.0, rect.bottom()),
                Pos2::new(rect.right() - 14.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
        if resp.clicked() {
            match i {
                0 => next.public_favorites = !*on,
                1 => next.public_recent_coins = !*on,
                2 => next.public_following = !*on,
                3 => next.public_fans = !*on,
                _ => next.public_birthday = !*on,
            }
            changed = true;
        }
    }
    if changed {
        app.save_privacy(next);
    }
}

fn groups_block(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.new_group)
                .hint_text("新建分组名")
                .desired_width((ui.available_width() - 96.0).max(120.0)),
        );
        let focused = resp.has_focus();
        app.note_focus(focused);
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("新建")
                        .color(egui::Color32::WHITE)
                        .size(11.0),
                )
                .fill(theme::PINK)
                .corner_radius(12.0)
                .min_size(Vec2::new(60.0, 28.0)),
            )
            .clicked()
            && !app.new_group.trim().is_empty()
        {
            app.create_group();
        }
    });
    ui.add_space(4.0);
    if app.groups.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("还没有分组，关注很多人时可以按分组管理")
                    .color(theme::TEXT3)
                    .size(11.0),
            );
        });
        return;
    }
    for g in app.groups.clone() {
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 44.0), Sense::click());
        let p = ui.painter();
        p.text(
            Pos2::new(rect.left() + 14.0, rect.center().y),
            Align2::LEFT_CENTER,
            &g.name,
            FontId::proportional(12.5),
            theme::TEXT,
        );
        p.text(
            Pos2::new(rect.left() + 14.0 + g.name.chars().count() as f32 * 13.0 + 8.0, rect.center().y),
            Align2::LEFT_CENTER,
            format!("{} 人", g.member_count),
            FontId::proportional(10.0),
            theme::TEXT3,
        );
        let del = Rect::from_center_size(
            Pos2::new(rect.right() - 34.0, rect.center().y),
            Vec2::new(48.0, 26.0),
        );
        p.text(
            del.center(),
            Align2::CENTER_CENTER,
            "删除",
            FontId::proportional(10.5),
            theme::TEXT3,
        );
        p.line_segment(
            [
                Pos2::new(rect.left() + 14.0, rect.bottom()),
                Pos2::new(rect.right() - 14.0, rect.bottom()),
            ],
            Stroke::new(1.0, theme::LINE),
        );
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if del.contains(pos) {
                    app.delete_group(g.id);
                }
            }
        }
    }
}

fn destructive_row(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let pending = app.deletion_pending;
        let (label, color) = if pending {
            ("已申请注销 · 撤销", theme::TEXT2)
        } else {
            ("申请注销（7 天后生效）", theme::UP_TAG)
        };
        let resp = ui.allocate_exact_size(Vec2::new(ui.available_width() - 28.0, 34.0), Sense::click());
        let p = ui.painter();
        p.rect_stroke(resp.0, 8.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            resp.0.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.5),
            color,
        );
        if resp.1.clicked() {
            if pending {
                app.revoke_deletion();
            } else {
                app.request_deletion();
            }
        }
    });
}
