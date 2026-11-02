//! 我的：未登录显示登录表单，登录后显示 /users/me 的真实信息
//!
//! 走的接口：POST /api/v1/auth/login → GET /api/v1/users/me

use crate::{
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui);
    match app.me.clone() {
        Some(me) => logged_in(ui, app, &me),
        None => login_form(ui, app),
    }
}

fn header(ui: &mut egui::Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 48.0), Sense::hover());
    ui.painter().text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        "我的",
        FontId::proportional(17.0),
        theme::TEXT,
    );
}

fn logged_in(ui: &mut egui::Ui, app: &mut App, me: &crate::api::Me) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 110.0), Sense::hover());
    let p = ui.painter();

    let name = if me.nickname.is_empty() {
        me.username.clone()
    } else {
        me.nickname.clone()
    };
    widgets::avatar(
        p,
        Pos2::new(rect.left() + 42.0, rect.top() + 40.0),
        24.0,
        &name,
        me.user_id,
    );

    p.text(
        Pos2::new(rect.left() + 78.0, rect.top() + 20.0),
        Align2::LEFT_TOP,
        &name,
        FontId::proportional(17.0),
        theme::TEXT,
    );

    let lv = Rect::from_min_size(
        Pos2::new(rect.left() + 78.0, rect.top() + 46.0),
        Vec2::new(34.0, 15.0),
    );
    p.rect_filled(lv, 3.0, theme::ORANGE);
    p.text(
        lv.center(),
        Align2::CENTER_CENTER,
        format!("LV{}", me.level_info.current_level),
        FontId::proportional(9.0),
        egui::Color32::WHITE,
    );
    p.text(
        Pos2::new(lv.right() + 8.0, rect.top() + 47.0),
        Align2::LEFT_TOP,
        format!("硬币 {}", me.coin_balance),
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    p.text(
        Pos2::new(rect.left() + 78.0, rect.top() + 68.0),
        Align2::LEFT_TOP,
        if me.cake_id.is_empty() {
            format!("用户 ID {}", me.user_id)
        } else {
            format!("{} · 用户 ID {}", me.cake_id, me.user_id)
        },
        FontId::proportional(10.0),
        theme::TEXT3,
    );

    if !me.sign.is_empty() {
        p.text(
            Pos2::new(rect.left() + 78.0, rect.top() + 86.0),
            Align2::LEFT_TOP,
            &me.sign,
            FontId::proportional(11.0),
            theme::TEXT2,
        );
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let btn = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(80.0, 28.0));
        let resp = ui.allocate_rect(
            Rect::from_min_size(ui.cursor().min, btn.size()),
            Sense::click(),
        );
        let p = ui.painter();
        p.rect_stroke(
            resp.rect,
            14.0,
            Stroke::new(1.0, theme::LINE),
            StrokeKind::Inside,
        );
        p.text(
            resp.rect.center(),
            Align2::CENTER_CENTER,
            "退出登录",
            FontId::proportional(11.0),
            theme::TEXT2,
        );
        if resp.clicked() {
            app.logout();
        }
    });

    ui.add_space(18.0);
    note(ui, "登录态已生效：token 来自 /auth/login，用户信息来自 /users/me");
}

fn login_form(ui: &mut egui::Ui, app: &mut App) {
    ui.add_space(24.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("登录")
                .color(theme::TEXT)
                .size(16.0)
                .strong(),
        );
    });
    ui.add_space(4.0);
    note(ui, "POST /api/v1/auth/login → GET /api/v1/users/me");
    ui.add_space(14.0);

    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.add(
            egui::TextEdit::singleline(&mut app.login_user)
                .hint_text("用户名")
                .desired_width(ui.available_width() - 32.0),
        );
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.add(
            egui::TextEdit::singleline(&mut app.login_pass)
                .hint_text("密码")
                .password(true)
                .desired_width(ui.available_width() - 32.0),
        );
    });

    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let busy = app.auth_state == LoadState::Loading;
        let label = if busy { "登录中…" } else { "登录" };
        if ui
            .add_enabled(!busy, egui::Button::new(label).min_size(Vec2::new(88.0, 30.0)))
            .clicked()
        {
            app.do_login();
        }
    });

    if let LoadState::Err(e) = &app.auth_state {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(egui::RichText::new(e).color(theme::UP_TAG).size(11.0));
        });
    }

    ui.add_space(20.0);
    note(ui, &format!("当前后端：{}", app.client.base));
}

fn note(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(10.0));
    });
}
