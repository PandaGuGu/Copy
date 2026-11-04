//! 我的收藏 + 详情页收藏夹选择弹窗
//!
//!   GET    /api/v1/users/me/favorites[?folder_id=]
//!   GET    /api/v1/users/me/favorite-folders
//!   POST   /api/v1/users/me/favorite-folders
//!   DELETE /api/v1/users/me/favorite-folders/:folderId
//!   GET    /api/v1/videos/:id/favorite-picker
//!   PUT    /api/v1/videos/:id/favorite-folders
//!
//! 收藏页按收藏夹筛选；弹窗里可以同时勾选多个夹，一个都不勾就是取消收藏。

use crate::{
    api::{FavoriteFolder, FavoriteItem},
    app::{App, LoadState},
    theme, widgets,
};
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    header(ui, app);

    if !app.client.logged_in() {
        ui.vertical_centered(|ui| {
            ui.add_space(110.0);
            ui.label(
                egui::RichText::new("登录后可以在这里看收藏")
                    .color(theme::TEXT3)
                    .size(12.0),
            );
        });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("favorites-scroll")
        .show(ui, |ui| {
            folders_section(ui, app);
            videos_section(ui, app);
            ui.add_space(12.0);
        });

    if let Some(msg) = app.folder_msg.clone() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
        });
    }
    if let Some(msg) = app.action_msg.clone() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
        });
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
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "我的收藏",
        FontId::proportional(15.0),
        theme::TEXT,
    );
    if ui
        .interact(
            Rect::from_min_size(rect.min, Vec2::new(84.0, rect.height())),
            egui::Id::new("favorites-back"),
            Sense::click(),
        )
        .clicked()
    {
        app.back();
    }
}

fn folders_section(ui: &mut egui::Ui, app: &mut App) {
    section_title(ui, "收藏夹");
    new_folder_row(ui, app);

    match app.folders_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            hint(ui, "加载收藏夹…");
        }
        LoadState::Err(e) => {
            hint(ui, &format!("收藏夹加载失败：{e}"));
            if text_action(ui, "重试") {
                app.folders_state = LoadState::Idle;
                app.load_folders();
            }
        }
        LoadState::Ready => {
            let mut pick: Option<Option<u64>> = None;
            let mut del: Option<u64> = None;

            match folder_row(ui, app, None) {
                FolderHit::Pick => pick = Some(None),
                FolderHit::Delete => {}
                FolderHit::None => {}
            }
            for f in &app.favorite_folders {
                match folder_row(ui, app, Some(f)) {
                    FolderHit::Pick => pick = Some(Some(f.id)),
                    FolderHit::Delete => del = Some(f.id),
                    FolderHit::None => {}
                }
            }

            if let Some(id) = del {
                app.delete_folder(id);
            } else if let Some(folder) = pick {
                app.fav_selected.clear();
                app.select_favorites_folder(folder);
            }

            // 选中某个收藏夹后，才有改名 / 清理 / 批量移出
            if let Some(fid) = app.favorites_folder {
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("重命名").color(theme::TEXT2).size(11.0),
                        ))
                        .clicked()
                    {
                        app.rename_folder = fid;
                        app.rename_title = app
                            .favorite_folders
                            .iter()
                            .find(|f| f.id == fid)
                            .map(|f| f.title.clone())
                            .unwrap_or_default();
                    }
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("清理失效").color(theme::TEXT2).size(11.0),
                        ))
                        .clicked()
                    {
                        app.clear_invalid_favorites(fid);
                    }
                });
                if app.rename_folder == fid {
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut app.rename_title)
                                .hint_text("新的收藏夹名称")
                                .desired_width((ui.available_width() - 120.0).max(90.0)),
                        );
                        if ui.button("保存").clicked() {
                            app.rename_favorite_folder(fid);
                        }
                        if ui.button("取消").clicked() {
                            app.rename_folder = 0;
                        }
                    });
                }
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    let mut target = app.move_to_folder;
                    let current = app
                        .favorite_folders
                        .iter()
                        .find(|f| f.id == target)
                        .map(|f| f.title.clone())
                        .unwrap_or_else(|| "移动到…".to_owned());
                    egui::ComboBox::from_id_salt("fav-move-target")
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for f in &app.favorite_folders {
                                if f.id != fid {
                                    ui.selectable_value(&mut target, f.id, &f.title);
                                }
                            }
                        });
                    app.move_to_folder = target;
                    if ui.button("移动选中").clicked() {
                        app.move_selected_favorites(fid);
                    }
                });
            }
        }
    }
}

enum FolderHit {
    None,
    Pick,
    Delete,
}

fn folder_row(ui: &mut egui::Ui, app: &App, folder: Option<&FavoriteFolder>) -> FolderHit {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 54.0), Sense::click());
    let selected = match folder {
        Some(f) => app.favorites_folder == Some(f.id),
        None => app.favorites_folder.is_none(),
    };
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 0.0, Color32::from_rgb(0xFF, 0xF1, 0xF5));
    }

    let icon = Rect::from_min_size(
        Pos2::new(rect.left() + 14.0, rect.center().y - 17.0),
        Vec2::new(34.0, 34.0),
    );
    p.rect_filled(icon, 8.0, theme::SURFACE);
    if folder.is_some() {
        // 三个横条
        for i in 0..3 {
            let y = icon.top() + 10.0 + i as f32 * 8.0;
            p.line_segment(
                [
                    Pos2::new(icon.left() + 9.0, y),
                    Pos2::new(icon.right() - 9.0, y),
                ],
                Stroke::new(1.2, theme::TEXT3),
            );
        }
    } else {
        // 「全部」：叠放的两个小方块
        for i in 0..2 {
            let x = icon.left() + 8.0 + i as f32 * 4.0;
            let y = icon.top() + 9.0 + i as f32 * 4.0;
            p.rect_stroke(
                Rect::from_min_size(Pos2::new(x, y), Vec2::new(14.0, 12.0)),
                3.0,
                Stroke::new(1.2, theme::TEXT3),
                StrokeKind::Inside,
            );
        }
    }

    let text_x = icon.right() + 12.0;
    let (title, sub) = match folder {
        Some(f) => {
            let public = if f.is_public { "公开" } else { "私密" };
            (f.title.clone(), format!("{} 个视频 · {public}", f.video_count))
        }
        None => ("全部收藏".to_owned(), "所有收藏的视频".to_owned()),
    };
    p.text(
        Pos2::new(text_x, rect.center().y - 16.0),
        Align2::LEFT_TOP,
        &title,
        FontId::proportional(13.0),
        theme::TEXT,
    );
    p.text(
        Pos2::new(text_x, rect.center().y + 4.0),
        Align2::LEFT_TOP,
        &sub,
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    if selected {
        let c = Pos2::new(rect.right() - 22.0, rect.center().y);
        p.add(egui::Shape::line(
            vec![
                Pos2::new(c.x - 5.0, c.y),
                Pos2::new(c.x - 1.0, c.y + 4.0),
                Pos2::new(c.x + 6.0, c.y - 5.0),
            ],
            Stroke::new(1.6, theme::PINK),
        ));
    }

    let mut del_clicked = false;
    if let Some(f) = folder {
        if !f.is_default {
            let del = Rect::from_min_size(
                Pos2::new(rect.right() - 78.0, rect.center().y - 11.0),
                Vec2::new(34.0, 22.0),
            );
            p.text(
                del.center(),
                Align2::CENTER_CENTER,
                "删除",
                FontId::proportional(10.0),
                theme::TEXT3,
            );
            del_clicked = ui
                .interact(del, egui::Id::new(("folder-del", f.id)), Sense::click())
                .clicked();
        }
    }

    p.line_segment(
        [
            Pos2::new(rect.left() + 60.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    if del_clicked {
        FolderHit::Delete
    } else if resp.clicked() {
        FolderHit::Pick
    } else {
        FolderHit::None
    }
}

fn new_folder_row(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let btn_w = 64.0;
        ui.add(
            egui::TextEdit::singleline(&mut app.new_folder_title)
                .hint_text("新建收藏夹名称")
                .desired_width((ui.available_width() - btn_w - 22.0).max(80.0)),
        );
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(btn_w, 24.0), Sense::hover());
        if pill(ui, rect, "新建", false, "folder-create") {
            app.create_folder();
        }
    });
    ui.add_space(4.0);
}

fn videos_section(ui: &mut egui::Ui, app: &mut App) {
    let title = match app.favorites_folder {
        Some(id) => app
            .favorite_folders
            .iter()
            .find(|f| f.id == id)
            .map(|f| f.title.clone())
            .unwrap_or_else(|| "收藏的视频".to_owned()),
        None => "收藏的视频".to_owned(),
    };
    section_title(ui, &title);

    match app.favorites_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            hint(ui, "加载收藏…");
        }
        LoadState::Err(e) => {
            hint(ui, &format!("加载失败：{e}"));
            if text_action(ui, "重试") {
                app.favorites_state = LoadState::Idle;
                app.load_favorites();
            }
        }
        LoadState::Ready => {
            if app.favorites.is_empty() {
                hint(ui, "还没有收藏的视频");
            } else {
                if let Some(fid) = app.favorites_folder {
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.label(
                            egui::RichText::new(format!("已选 {} 个", app.fav_selected.len()))
                                .color(theme::TEXT3)
                                .size(10.5),
                        );
                        if ui.button("移出收藏夹").clicked() {
                            app.batch_remove_favorites(fid);
                        }
                    });
                }
                let mut open = None;
                for it in app.favorites.clone() {
                    if video_row(ui, app, &it) {
                        open = Some(it.id);
                    }
                }
                if let Some(id) = open {
                    app.open_detail(id);
                }
            }
        }
    }
}

fn video_row(ui: &mut egui::Ui, app: &mut App, it: &FavoriteItem) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 82.0), Sense::click());
    let cover = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.top() + 6.0),
        Vec2::new(116.0, 66.0),
    );
    widgets::draw_cover(ui, cover, &it.cover_url, &app.covers, 4.0);

    let text_x = cover.right() + 12.0;
    let multi = app.favorites_folder.is_some();
    let text_w = (rect.right() - text_x - if multi { 42.0 } else { 12.0 }).max(40.0);
    let p = ui.painter();
    let lines = widgets::wrap_lines(p, &it.title, 13.0, text_w, 2);
    let mut y = rect.top() + 4.0;
    for l in lines {
        p.text(
            Pos2::new(text_x, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(13.0),
            theme::TEXT,
        );
        y += 18.0;
    }

    let mut meta = Vec::new();
    if !it.uploader.is_empty() {
        meta.push(it.uploader.clone());
    }
    if it.duration > 0 {
        meta.push(theme::clock(it.duration));
    }
    p.text(
        Pos2::new(text_x, cover.bottom() - 26.0),
        Align2::LEFT_TOP,
        widgets::wrap_lines(p, &meta.join(" · "), 11.0, text_w, 1)
            .into_iter()
            .next()
            .unwrap_or_default(),
        FontId::proportional(11.0),
        theme::TEXT3,
    );
    if !it.favorited_at.is_empty() {
        p.text(
            Pos2::new(text_x, cover.bottom() - 11.0),
            Align2::LEFT_TOP,
            format!("收藏于 {}", short_time(&it.favorited_at)),
            FontId::proportional(11.0),
            theme::PINK,
        );
    }

    if multi {
        let c = Pos2::new(rect.right() - 24.0, rect.center().y);
        let on = app.fav_selected.contains(&it.id);
        let box_rect = Rect::from_center_size(c, Vec2::new(18.0, 18.0));
        if on {
            ui.painter().rect_filled(box_rect, 9.0, theme::PINK);
            ui.painter().add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - 5.0, c.y),
                    Pos2::new(c.x - 1.0, c.y + 4.0),
                    Pos2::new(c.x + 6.0, c.y - 5.0),
                ],
                Stroke::new(1.6, Color32::WHITE),
            ));
        } else {
            ui.painter().rect_stroke(
                box_rect,
                9.0,
                Stroke::new(1.0, theme::LINE),
                StrokeKind::Inside,
            );
        }
        let hit = Rect::from_center_size(c, Vec2::new(36.0, 36.0));
        if ui
            .interact(hit, egui::Id::new(("fav-sel", it.id)), Sense::click())
            .clicked()
        {
            if on {
                app.fav_selected.retain(|x| *x != it.id);
            } else {
                app.fav_selected.push(it.id);
            }
            return false;
        }
    }

    p.line_segment(
        [
            Pos2::new(rect.left() + 12.0, rect.bottom()),
            Pos2::new(rect.right() - 12.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
    resp.clicked()
}

/// 详情页的收藏夹选择弹窗
pub fn picker(ui: &mut egui::Ui, app: &mut App) {
    if app.picker_state == LoadState::Idle && app.favorite_picker.is_none() {
        return;
    }

    let screen = ui.ctx().viewport_rect();
    let mask = egui::Area::new(egui::Id::new("fav-picker-mask"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ui.ctx(), |ui| {
            let r = ui.allocate_rect(screen, Sense::click());
            ui.painter()
                .rect_filled(r.rect, 0.0, Color32::from_black_alpha(96));
            r
        });
    if mask.inner.clicked() {
        app.close_favorite_picker();
        return;
    }

    egui::Area::new(egui::Id::new("fav-picker-card"))
        .order(egui::Order::Tooltip)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            egui::Frame::window(&ui.style()).show(ui, |ui| {
                ui.set_width(300.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("收藏到")
                            .color(theme::TEXT)
                            .size(15.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("×").frame(false))
                            .clicked()
                        {
                            app.close_favorite_picker();
                        }
                    });
                });
                ui.label(
                    egui::RichText::new("可以同时放进多个收藏夹，都不选就是取消收藏")
                        .color(theme::TEXT3)
                        .size(10.0),
                );
                ui.add_space(8.0);

                match app.picker_state.clone() {
                    LoadState::Idle | LoadState::Loading => {
                        ui.label(
                            egui::RichText::new("加载收藏夹…")
                                .color(theme::TEXT3)
                                .size(11.0),
                        );
                    }
                    LoadState::Err(e) => {
                        ui.label(
                            egui::RichText::new(format!("加载失败：{e}"))
                                .color(theme::UP_TAG)
                                .size(11.0),
                        );
                        if ui.button("重试").clicked() {
                            app.open_favorite_picker(app.picker_video);
                        }
                    }
                    LoadState::Ready => {
                        let mut toggle = None;
                        egui::ScrollArea::vertical()
                            .max_height(220.0)
                            .auto_shrink([false, true])
                            .id_salt("picker-scroll")
                            .show(ui, |ui| {
                                if let Some(p) = &app.favorite_picker {
                                    for f in &p.items {
                                        if picker_row(ui, f, app.picker_selected.contains(&f.id)) {
                                            toggle = Some(f.id);
                                        }
                                    }
                                }
                            });
                        if let Some(id) = toggle {
                            app.toggle_picker_folder(id);
                        }
                    }
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.new_folder_title)
                            .hint_text("新建收藏夹")
                            .desired_width((ui.available_width() - 70.0).max(120.0)),
                    );
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(56.0, 24.0), Sense::hover());
                    if pill(ui, rect, "新建", false, "picker-create") {
                        app.create_folder();
                    }
                });
                if let Some(msg) = app.folder_msg.clone() {
                    ui.label(
                        egui::RichText::new(msg)
                            .color(theme::UP_TAG)
                            .size(10.0),
                    );
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let total = ui.available_width();
                    let (cancel, _) =
                        ui.allocate_exact_size(Vec2::new(64.0, 30.0), Sense::hover());
                    let (ok, _) =
                        ui.allocate_exact_size(Vec2::new(total - 74.0, 30.0), Sense::hover());
                    if pill(ui, cancel, "取消", false, "picker-cancel") {
                        app.close_favorite_picker();
                    }
                    let label = if app.picker_selected.is_empty() {
                        "取消收藏"
                    } else {
                        "完成"
                    };
                    if pill(ui, ok, label, true, "picker-save") {
                        app.save_favorite_picker();
                    }
                });
            });
        });
}

/// 弹窗里的一行收藏夹：勾选框 + 标题 + 数量
fn picker_row(ui: &mut egui::Ui, f: &FavoriteFolder, on: bool) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::click());
    let p = ui.painter();

    let box_rect = Rect::from_min_size(
        Pos2::new(rect.left(), rect.center().y - 9.0),
        Vec2::new(18.0, 18.0),
    );
    if on {
        p.rect_filled(box_rect, 9.0, theme::PINK);
        let c = box_rect.center();
        p.add(egui::Shape::line(
            vec![
                Pos2::new(c.x - 5.0, c.y),
                Pos2::new(c.x - 1.0, c.y + 4.0),
                Pos2::new(c.x + 6.0, c.y - 5.0),
            ],
            Stroke::new(1.6, Color32::WHITE),
        ));
    } else {
        p.rect_stroke(
            box_rect,
            9.0,
            Stroke::new(1.0, theme::LINE),
            StrokeKind::Inside,
        );
    }

    p.text(
        Pos2::new(box_rect.right() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        &f.title,
        FontId::proportional(12.0),
        theme::TEXT,
    );
    let count = if f.count_label.is_empty() {
        format!("{} 个", f.video_count)
    } else {
        f.count_label.clone()
    };
    p.text(
        Pos2::new(rect.right(), rect.center().y),
        Align2::RIGHT_CENTER,
        count,
        FontId::proportional(10.0),
        theme::TEXT3,
    );

    resp.clicked()
}

// ───────────────────────── 通用小部件 ─────────────────────────

fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(text)
                .color(theme::TEXT)
                .size(14.0)
                .strong(),
        );
    });
    ui.add_space(4.0);
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(11.0));
    });
    ui.add_space(6.0);
}

fn text_action(ui: &mut egui::Ui, text: &str) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if ui
            .add(egui::Button::new(
                egui::RichText::new(text).color(theme::PINK).size(11.0),
            ))
            .clicked()
        {
            clicked = true;
        }
    });
    clicked
}

fn pill(ui: &mut egui::Ui, rect: Rect, label: &str, primary: bool, id: &str) -> bool {
    let resp = ui.interact(rect, egui::Id::new(id), Sense::click());
    let p = ui.painter();
    if primary {
        p.rect_filled(rect, 15.0, theme::PINK);
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    } else {
        p.rect_filled(rect, 15.0, Color32::WHITE);
        p.rect_stroke(rect, 15.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0),
            theme::TEXT2,
        );
    }
    resp.clicked()
}

/// "2026-11-03 16:28:01" → "11-03 16:28"
fn short_time(s: &str) -> String {
    if s.len() >= 16 {
        s[5..16].to_owned()
    } else {
        s.to_owned()
    }
}
