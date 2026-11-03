//! 首页：GET /api/v1/videos 的推荐列表

use crate::{
    app::{App, HomeCat, LoadState, Route, Tab},
    theme, widgets,
};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);
    cat_bar(ui, app);
    sub_zone_hint(ui, app);

    match app.videos_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            center_hint(ui, "正在加载 …");
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_videos();
            }
        }
        LoadState::Ready => {
            if app.videos.is_empty() {
                let tip = match &app.home_cat {
                    HomeCat::Zone(z) => format!("「{z}」分区还没有视频"),
                    _ => "接口返回空列表，先去后台发布一条".to_owned(),
                };
                if App::empty_state(ui, "暂无视频", &tip) {
                    app.load_videos();
                }
                return;
            }
            grid(ui, app);
        }
    }
}

/// 分区导航栏：直播 | 推荐 | 热门 | 各分区 … 最右固定「☰」进全部分区
///
/// 分区名优先取 `GET /zones` 的真实分区（按父分区汇总视频数、倒序），
/// 接口还没回来或后端一条视频都没有时，回退到内置分区表。
fn cat_bar(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let mut goto_live = false;
    let mut open_zones = false;
    let mut pick: Option<HomeCat> = None;

    // 真实分区；空则用内置表兜底（只影响 tab 文案，点开仍然走同一个推荐接口）
    let mut cats: Vec<String> = app.zone_parents().into_iter().map(|(n, _)| n).collect();
    if cats.is_empty() {
        cats = crate::zones::ZONES.iter().map(|z| z.name.to_owned()).collect();
    }

    // 整条导航是一个横向布局：左边横向滚动的 tab，右边固定「☰」
    let row = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.set_height(40.0);
        egui::ScrollArea::horizontal()
            .max_width((w - 44.0).max(80.0))
            .max_height(40.0)
            .auto_shrink([false, true])
            // 滚动条会占掉底部 8pt，把选中态的下划线裁掉，这里直接隐藏
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .id_salt("home-cat-bar")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(6.0);
                    if tab_item(ui, "直播", false) {
                        goto_live = true;
                    }
                    if tab_item(ui, "推荐", app.home_cat == HomeCat::Recommend) {
                        pick = Some(HomeCat::Recommend);
                    }
                    if tab_item(ui, "热门", app.home_cat == HomeCat::Hot) {
                        pick = Some(HomeCat::Hot);
                    }
                    for cat in &cats {
                        // 选的是子分区（生活-日常）时，父分区（生活）也高亮
                        let active = match &app.home_cat {
                            HomeCat::Zone(z) => {
                                z == cat || z.starts_with(&format!("{cat}-"))
                            }
                            _ => false,
                        };
                        if tab_item(ui, cat, active) {
                            pick = Some(HomeCat::Zone(cat.clone()));
                        }
                    }
                    ui.add_space(8.0);
                });
            });

        // ☰ 全部分区
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(44.0, 40.0), Sense::click());
        let p = ui.painter();
        for i in 0..3 {
            let y = rect.top() + 14.0 + i as f32 * 6.0;
            p.line_segment(
                [
                    Pos2::new(rect.left() + 14.0, y),
                    Pos2::new(rect.right() - 10.0, y),
                ],
                Stroke::new(1.6, if resp.hovered() { theme::TEXT } else { theme::TEXT2 }),
            );
        }
        if resp.clicked() {
            open_zones = true;
        }
    });

    // 分隔线
    let r = row.response.rect;
    ui.painter().line_segment(
        [Pos2::new(r.left(), r.bottom()), Pos2::new(r.right(), r.bottom())],
        Stroke::new(1.0, theme::LINE),
    );

    if goto_live {
        app.navigate(Route::Tab(Tab::Live));
    }
    if open_zones {
        app.navigate(Route::Zones);
    }
    if let Some(c) = pick {
        app.select_cat(c);
    }
}

/// 从「全部分区」挑了子分区时，标一下当前在看哪一块
fn sub_zone_hint(ui: &mut egui::Ui, app: &App) {
    let HomeCat::Zone(z) = &app.home_cat else {
        return;
    };
    let Some((parent, sub)) = z.split_once('-') else {
        return;
    };
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::hover());
    ui.painter().text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        format!("当前分区：{parent} / {sub}"),
        FontId::proportional(11.0),
        theme::TEXT3,
    );
}

/// 单个分类 tab（宽度随文字，选中态粉色下划线）
fn tab_item(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        FontId::proportional(if active { 14.0 } else { 13.0 }),
        theme::TEXT,
    );
    let w = galley.size().x + 20.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::click());
    let p = ui.painter();
    p.galley(
        Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0 - 1.0,
        ),
        galley,
        if active { theme::TEXT } else { theme::TEXT2 },
    );
    if active {
        p.rect_filled(
            Rect::from_center_size(
                Pos2::new(rect.center().x, rect.bottom() - 3.0),
                Vec2::new(20.0, 3.0),
            ),
            1.5,
            theme::PINK,
        );
    }
    resp.clicked()
}

/// 顶部：左侧头像 + 中间搜索栏 + 右侧消息中心（邮件图标）
///
/// 头像取 /users/me（未登录用首字占位），搜索栏点开搜索页。
fn header(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 54.0), Sense::hover());

    // ---- 左：小头像 ----
    let av_r = 15.0;
    let av_c = Pos2::new(rect.left() + 24.0, rect.center().y);
    let (name, uid, avatar_url) = match &app.me {
        Some(me) => (
            if me.nickname.is_empty() {
                me.username.clone()
            } else {
                me.nickname.clone()
            },
            me.user_id,
            me.avatar_url.clone(),
        ),
        None => (String::new(), 0, String::new()),
    };
    if avatar_url.is_empty() {
        widgets::avatar_anon(ui.painter(), av_c, av_r);
    } else {
        app.covers.request(&[avatar_url.clone()]);
        widgets::avatar_tex(ui, av_c, av_r, &name, &avatar_url, &app.covers, uid);
    }
    let av_hit = Rect::from_center_size(av_c, Vec2::splat(av_r * 2.0 + 10.0));
    if ui
        .interact(av_hit, egui::Id::new("home-avatar"), Sense::click())
        .clicked()
    {
        app.navigate(Route::Tab(Tab::Mine));
    }

    // ---- 中：搜索栏 ----
    let left = rect.left() + 50.0;
    let right = rect.right() - 96.0;
    let bar = Rect::from_min_max(
        Pos2::new(left, rect.center().y - 16.0),
        Pos2::new(right.max(left + 60.0), rect.center().y + 16.0),
    );
    if widgets::search_pill(ui, bar, "搜索视频、UP主", "home-search") {
        app.navigate(Route::Search);
    }

    // ---- 右：排行榜 ----
    let rank_btn = Rect::from_center_size(
        Pos2::new(rect.right() - 68.0, rect.center().y),
        Vec2::new(30.0, 24.0),
    );
    let rank_resp = ui.interact(rank_btn, egui::Id::new("home-rank"), Sense::click());
    {
        let p = ui.painter();
        p.text(
            rank_btn.center(),
            Align2::CENTER_CENTER,
            "榜",
            FontId::proportional(13.0),
            if rank_resp.hovered() {
                theme::TEXT
            } else {
                theme::TEXT2
            },
        );
    }
    if rank_resp.clicked() {
        app.open_rank();
    }

    // ---- 右：消息中心（邮件图标，右下角带未读角标） ----
    let btn = Rect::from_center_size(
        Pos2::new(rect.right() - 24.0, rect.center().y),
        Vec2::new(40.0, 40.0),
    );
    let resp = ui.interact(btn, egui::Id::new("home-mail"), Sense::click());
    let p = ui.painter();
    let color = if resp.hovered() { theme::TEXT } else { theme::TEXT2 };
    let envelope = Rect::from_center_size(btn.center(), Vec2::new(22.0, 16.0));
    p.rect_stroke(envelope, 3.0, Stroke::new(1.2, color), StrokeKind::Inside);
    // 信封的折角
    p.line_segment(
        [
            Pos2::new(envelope.left() + 3.0, envelope.top() + 3.0),
            Pos2::new(envelope.center().x, envelope.center().y + 1.0),
        ],
        Stroke::new(1.2, color),
    );
    p.line_segment(
        [
            Pos2::new(envelope.right() - 3.0, envelope.top() + 3.0),
            Pos2::new(envelope.center().x, envelope.center().y + 1.0),
        ],
        Stroke::new(1.2, color),
    );
    // 未读角标（总数来自 /notifications/unread-summary）
    let unread = app.unread.total();
    if unread > 0 {
        let text = if unread > 99 {
            "99+".to_owned()
        } else {
            unread.to_string()
        };
        let badge = Rect::from_center_size(
            Pos2::new(envelope.right() + 1.0, envelope.top() - 2.0),
            Vec2::new(if unread > 9 { 18.0 } else { 14.0 }, 14.0),
        );
        p.rect_filled(badge, 7.0, theme::PINK);
        p.text(
            badge.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(9.0),
            egui::Color32::WHITE,
        );
    }
    if resp.clicked() {
        app.open_notifications();
    }
}

fn grid(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("home-scroll")
        .show(ui, |ui| {
            ui.add_space(4.0);
            let w = ui.available_width();
            let mut clicked = None;

            // 运营位只在「推荐」下出现，切到分区就不占篇幅了
            if app.home_cat == HomeCat::Recommend {
                clicked = hero(ui, app, w);
            }

            if let Some(v) = app.videos.first() {
                let cw = w - 16.0;
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if widgets::big_card(ui, v, &app.covers, cw).clicked() {
                        clicked = Some(v.id);
                    }
                });
            }

            ui.add_space(10.0);
            let gap = 7.0;
            let cw = ((w - 16.0 - gap) / 2.0).floor();
            let mut i = 1;
            while i < app.videos.len() {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.x = gap;
                    if let Some(v) = app.videos.get(i) {
                        if widgets::small_card(ui, v, &app.covers, cw).clicked() {
                            clicked = Some(v.id);
                        }
                    }
                    if let Some(v) = app.videos.get(i + 1) {
                        if widgets::small_card(ui, v, &app.covers, cw).clicked() {
                            clicked = Some(v.id);
                        }
                    }
                });
                ui.add_space(10.0);
                i += 2;
            }
            ui.add_space(8.0);

            if let Some(id) = clicked {
                app.open_detail(id);
            }
        });
}

fn center_hint(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(140.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}

/// 运营位：轮播图（GET /home-banners）+ 全站统计（GET /stats/home）
///
/// 返回被点中的视频 id（轮播图链接指向视频时才有）。
fn hero(ui: &mut egui::Ui, app: &mut App, w: f32) -> Option<u64> {
    let mut open = None;

    if !app.home_ops.banners.is_empty() {
        let cw = (w - 32.0).max(140.0);
        let ch = (cw / 2.35).round();
        egui::ScrollArea::horizontal()
            .id_salt("home-banners")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .max_height(ch + 4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for b in &app.home_ops.banners {
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(cw, ch), Sense::click());
                        widgets::draw_cover(ui, rect, &b.pic, &app.covers, 8.0);
                        if !b.name.is_empty() {
                            // 标题压在左下角，长标题截断
                            let lines = widgets::wrap_lines(
                                ui.painter(),
                                &b.name,
                                12.0,
                                cw - 24.0,
                                1,
                            );
                            if let Some(l) = lines.first() {
                                widgets::chip(
                                    ui.painter(),
                                    Pos2::new(rect.left() + 10.0, rect.bottom() - 8.0),
                                    Align2::LEFT_BOTTOM,
                                    l,
                                    12.0,
                                );
                            }
                        }
                        if resp.clicked() {
                            open = banner_target(&b.url);
                        }
                    }
                });
            });
        ui.add_space(6.0);
    }

    if let Some(s) = &app.home_ops.stats {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::hover());
        ui.painter().text(
            Pos2::new(rect.left() + 12.0, rect.center().y),
            Align2::LEFT_CENTER,
            format!(
                "{} 人在线 · 全站 {} 个视频",
                theme::human(s.web_online),
                theme::human(s.all_count)
            ),
            FontId::proportional(10.0),
            theme::TEXT3,
        );
    }
    ui.add_space(2.0);
    open
}

/// 轮播图链接形如 `/#/video/BV123`，取出其中的视频 id；外链返回 None
fn banner_target(url: &str) -> Option<u64> {
    let idx = url.find("BV")?;
    let digits: String = url[idx + 2..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}
