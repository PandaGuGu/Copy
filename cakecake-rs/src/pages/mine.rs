//! 我的：未登录显示登录/注册表单，登录后显示 /users/me 的真实信息
//!
//! 走的接口：POST /api/v1/auth/login → GET /api/v1/users/me
//! 注册：POST /api/v1/users → 自动登录（access token 过期时自动 refresh）

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
    // 点头像/昵称进自己的主页（和别人主页同一页，只是 id 是自己）
    let home_hit = Rect::from_min_size(rect.min, Vec2::new(rect.width() - 90.0, 92.0));
    let open_home = ui
        .interact(home_hit, egui::Id::new("mine-home"), Sense::click())
        .clicked();
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

    if open_home && me.user_id > 0 {
        app.open_space(me.user_id);
    }

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
    entry(ui, app, "消息中心", crate::app::Route::Notifications);
    entry(ui, app, "私信", crate::app::Route::Dm);
    entry(ui, app, "观看历史", crate::app::Route::History);
    entry(ui, app, "稍后再看", crate::app::Route::WatchLater);
    entry(ui, app, "我的收藏", crate::app::Route::Favorites);
    entry(ui, app, "专栏", crate::app::Route::ArticleList);
    entry(ui, app, "图文收藏", crate::app::Route::ArticleFavorites);
    entry(ui, app, "创作中心", crate::app::Route::Creator);
    entry(ui, app, "硬币与任务", crate::app::Route::Ledger);
    entry(ui, app, "专题", crate::app::Route::Specials);
    entry(ui, app, "我的直播间", crate::app::Route::LiveHost);
    entry(ui, app, "帮助与反馈", crate::app::Route::Governance);
    entry(ui, app, "设置", crate::app::Route::Settings);

    ui.add_space(18.0);
    note(
        ui,
        "登录态已生效：token 过期会自动 refresh，用户信息来自 /users/me",
    );
}

/// 「我的」里的功能入口：左侧文案 + 右侧箭头
fn entry(ui: &mut egui::Ui, app: &mut App, label: &str, route: crate::app::Route) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 46.0), Sense::click());
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left() + 16.0, rect.top()),
            Pos2::new(rect.right() - 16.0, rect.top()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
    p.text(
        Pos2::new(rect.left() + 16.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        theme::TEXT,
    );
    // 消息中心入口带未读角标
    if route == crate::app::Route::Notifications {
        let n = app.unread.total();
        if n > 0 {
            let badge = Rect::from_center_size(
                Pos2::new(rect.right() - 52.0, rect.center().y),
                Vec2::new(if n > 9 { 20.0 } else { 15.0 }, 15.0),
            );
            p.rect_filled(badge, 7.0, theme::PINK);
            p.text(
                badge.center(),
                Align2::CENTER_CENTER,
                if n > 99 {
                    "99+".to_owned()
                } else {
                    n.to_string()
                },
                FontId::proportional(9.0),
                egui::Color32::WHITE,
            );
        }
    }
    let c = Pos2::new(rect.right() - 22.0, rect.center().y);
    p.add(egui::Shape::line(
        vec![
            Pos2::new(c.x - 5.0, c.y - 5.0),
            Pos2::new(c.x, c.y),
            Pos2::new(c.x - 5.0, c.y + 5.0),
        ],
        Stroke::new(1.2, theme::TEXT3),
    ));
    if resp.clicked() {
        match route {
            crate::app::Route::Notifications => app.open_notifications(),
            crate::app::Route::WatchLater => app.open_watch_later(),
            crate::app::Route::History => app.open_history(),
            crate::app::Route::Favorites => app.open_favorites(),
            crate::app::Route::Settings => app.open_settings(),
            crate::app::Route::Dm => app.open_dm(),
            _ => app.open_more(route),
        }
    }
}

fn login_form(ui: &mut egui::Ui, app: &mut App) {
    let register = app.show_register;

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("账号")
                .color(theme::TEXT)
                .size(16.0)
                .strong(),
        );
        ui.add_space(10.0);
        tab(ui, app, "登录", false);
        tab(ui, app, "注册", true);
    });
    ui.add_space(4.0);
    note(
        ui,
        if register {
            "POST /api/v1/users → 自动登录 → GET /users/me"
        } else {
            "POST /api/v1/auth/login → GET /api/v1/users/me"
        },
    );
    ui.add_space(14.0);

    let (state_name, busy_label, idle_label) = if register {
        (app.reg_state.clone(), "注册中…", "注册")
    } else {
        (app.auth_state.clone(), "登录中…", "登录")
    };
    let busy = state_name == LoadState::Loading;

    if register {
        field(ui, &mut app.reg_user, "用户名", false);
        ui.add_space(8.0);
        field(ui, &mut app.reg_pass, "密码（至少 8 位）", true);
    } else {
        field(ui, &mut app.login_user, "用户名", false);
        ui.add_space(8.0);
        field(ui, &mut app.login_pass, "密码", true);
    }

    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let label = if busy { busy_label } else { idle_label };
        if ui
            .add_enabled(!busy, egui::Button::new(label).min_size(Vec2::new(88.0, 30.0)))
            .clicked()
        {
            if app.show_register {
                app.do_register();
            } else {
                app.do_login();
            }
        }
    });

    if let LoadState::Err(e) = &state_name {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(egui::RichText::new(e).color(theme::UP_TAG).size(11.0));
        });
    }

    ui.add_space(20.0);
    note(ui, &format!("当前后端：{}", app.client.base));
}

/// 登录 / 注册切换
fn tab(ui: &mut egui::Ui, app: &mut App, label: &str, register: bool) {
    let selected = app.show_register == register;
    let text = egui::RichText::new(label)
        .size(13.0)
        .color(if selected { theme::ORANGE } else { theme::TEXT3 });
    if ui.selectable_label(selected, text).clicked() && !selected {
        app.show_register = register;
        if register {
            app.reg_state = LoadState::Idle;
        } else {
            app.auth_state = LoadState::Idle;
        }
    }
}

/// 一行输入框（用户名 / 密码）
fn field(ui: &mut egui::Ui, value: &mut String, hint: &str, password: bool) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let mut edit = egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(ui.available_width() - 32.0);
        if password {
            edit = edit.password(true);
        }
        ui.add(edit);
    });
}

fn note(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(10.0));
    });
}
