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
/// 分区名和 Web 端 `constants/videoZones.js` 一致，切换后走
/// `/api/v1/zones/:zone/recommendation`。
fn cat_bar(ui: &mut egui::Ui, app: &mut App) {
    let w = ui.available_width();
    let mut goto_live = false;
    let mut open_zones = false;
    let mut pick: Option<HomeCat> = None;

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
                    for cat in crate::zones::ZONES {
                        // 选的是子分区（生活-日常）时，父分区（生活）也高亮
                        let active = match &app.home_cat {
                            HomeCat::Zone(z) => {
                                z == cat.name || z.starts_with(&format!("{}-", cat.name))
                            }
                            _ => false,
                        };
                        if tab_item(ui, cat.name, active) {
                            pick = Some(HomeCat::Zone(cat.name.to_owned()));
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

/// 顶部：左侧头像 + 中间搜索栏 + 右侧刷新
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
    let right = rect.right() - 60.0;
    let bar = Rect::from_min_max(
        Pos2::new(left, rect.center().y - 16.0),
        Pos2::new(right.max(left + 60.0), rect.center().y + 16.0),
    );
    if widgets::search_pill(ui, bar, "搜索视频、UP主", "home-search") {
        app.navigate(Route::Search);
    }

    // ---- 右：刷新 ----
    let btn = Rect::from_min_size(
        Pos2::new(rect.right() - 56.0, rect.center().y - 12.0),
        Vec2::new(48.0, 24.0),
    );
    let p = ui.painter();
    p.rect_stroke(btn, 12.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
    p.text(
        btn.center(),
        Align2::CENTER_CENTER,
        "刷新",
        FontId::proportional(11.0),
        theme::TEXT2,
    );
    if ui
        .interact(btn, egui::Id::new("home-refresh"), Sense::click())
        .clicked()
    {
        app.load_videos();
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
