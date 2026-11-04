//! 视频详情：播放器 + 简介/评论 tab + UP 主区 + 操作栏 + 相关推荐
//!
//! 数据全部来自接口：
//!   GET /api/v1/videos/:id（含 video_url，可直接播放）
//!   GET /api/v1/videos/:id/danmaku · /comments
//!   GET /api/v1/feed/recommendation（相关推荐）
//!   POST /videos/:id/like · /coin · /favorite · POST /users/:id/follow
//!
//! 播放走系统的 MediaPlayer（SoC 硬解），所以详情页真的有画面，不是封面图。

use crate::{
    api::{Danmaku, VideoDetail},
    app::{Action, App, DetailTab, FsMenu, LoadState},
    theme,
    video::Phase,
    widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App, id: u64) {
    // 全屏（横屏）播放：整页只留播放画面 + 会自动隐藏的蒙层控件
    if app.player_fullscreen {
        if app.detail_state == LoadState::Ready {
            if let Some(data) = app.detail.take() {
                fullscreen_player(ui, app, &data);
                app.detail = Some(data);
            }
        } else {
            app.exit_fullscreen();
        }
        return;
    }

    // 返回键压在视频画面左上角，不再单独占一条横向导航栏
    match app.detail_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            if widgets::back_bar(ui, "", "detail-back") {
                app.back();
                return;
            }
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(egui::RichText::new("加载中…").color(theme::TEXT3).size(12.0));
            });
        }
        LoadState::Err(e) => {
            if widgets::back_bar(ui, "", "detail-back") {
                app.back();
                return;
            }
            let mut retry = false;
            ui.vertical_centered(|ui| {
                ui.add_space(90.0);
                ui.label(egui::RichText::new("加载失败").color(theme::TEXT).size(14.0));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(&e).color(theme::TEXT3).size(11.0));
                ui.add_space(12.0);
                if ui.button("重试").clicked() {
                    retry = true;
                }
            });
            if retry {
                app.open_detail(id);
            }
        }
        LoadState::Ready => {
            let Some(data) = app.detail.take() else { return };
            // 评论 tab 下把输入栏做成悬浮层，钉在「键盘/导航栏之上」。
            // Panel::bottom 在页面高度不够时会被键盘挤出可视区（整条被裁掉），
            // 所以改用 Area 按屏幕坐标定位。
            if app.detail_tab == DetailTab::Comments {
                let bar_h = 54.0
                    + if app.reply_to.is_some() { 24.0 } else { 0.0 }
                    + if app.comment_image_open { 34.0 } else { 0.0 };
                let screen = ui.ctx().viewport_rect();
                let top = (screen.bottom() - app.bottom_inset - bar_h).max(0.0);
                egui::Area::new(egui::Id::new("detail-composer"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::Pos2::new(0.0, top))
                    .show(ui.ctx(), |ui| {
                        ui.set_width(screen.width());
                        ui.set_height(bar_h);
                        egui::Frame::NONE
                            .fill(theme::BG)
                            .stroke(egui::Stroke::new(1.0, theme::LINE))
                            .inner_margin(egui::Margin::same(0))
                            .show(ui, |ui| {
                                crate::pages::comment_ui::composer(ui, app, data.video.id);
                            });
                    });
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_salt("detail-scroll")
                .show(ui, |ui| {
                    player(ui, app, &data);
                    // 清晰度/字幕/倍速默认收起，点播放器「设置」才展开
                    if app.player_settings {
                        options_bar(ui, app, &data);
                    }
                    tabs(ui, app, &data);
                    if let Some(msg) = app.action_msg.clone() {
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
                        });
                    }
                    ui.add_space(4.0);
                    match app.detail_tab {
                        // 简介：UP 主区 + 标题 + 操作栏 + 简介正文
                        DetailTab::Intro => {
                            uploader_row(ui, app, &data.video);
                            title_block(ui, &data.video);
                            action_bar(ui, app, &data.video);
                            ui.add_space(4.0);
                            intro(ui, app, &data)
                        }
                        // 评论：只留评论，别把 UP 主卡片和标题重复一遍
                        DetailTab::Comments => {
                            crate::pages::comment_list::show(ui, app, data.video.id, &data.comments)
                        }
                    }
                    ui.add_space(10.0);
                });
            app.detail = Some(data);
        }
    }

    // 收藏夹选择弹窗（覆盖在详情页上）
    crate::pages::favorites::picker(ui, app);
}

// ───────────────────────── 播放器 ─────────────────────────

fn player(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    let v = &data.video;
    let danmakus = &data.danmakus;
    let w = ui.available_width();
    let h = (w * 0.5625).round();
    let url = play_url(app, data);
    let has_src = !url.is_empty() && data.source_ready;
    let phase = app.video.phase();
    // 失败后不要每帧重试，等用户点一下再重连
    if has_src && phase != Phase::Failed {
        app.video.play(&url);
    }
    // 换视频后把用户选的倍速补回去
    if (app.video.speed() - app.speed).abs() > 0.001 {
        app.video.set_speed(app.speed);
    }

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    // 有地址就把这块交给播放器：它负责建 Surface、起 MediaPlayer、逐帧画。
    // 帧没到之前是黑底 +「缓冲中…」，不能用封面盖住——盖住了回调就不会被调用。
    if has_src {
        app.video.ui_in(ui, rect);
    } else {
        widgets::draw_cover(ui, rect, &v.cover_url, &app.covers, 0.0);
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_black_alpha(60));
    }

    // 弹幕：跟着播放进度从右往左跑（右上角「弹」可以关掉）
    if app.danmaku_on && (phase == Phase::Playing || phase == Phase::Paused) && has_src {
        let pos = app.video.position_ms();
        if pos > 0 {
            draw_danmaku(ui, rect, danmakus, pos);
        }
    }

    // 字幕：当前时刻对应的那句，压在画面底部（GET /videos/:id/subtitles）
    if has_src && matches!(phase, Phase::Playing | Phase::Paused) && !app.cues.is_empty() {
        let sec = app.video.position_ms().max(0) as f64 / 1000.0;
        if let Some(text) = crate::subtitle::at(&app.cues, sec) {
            draw_subtitle(ui, rect, text);
        }
    }

    // 返回键压在画面左上角，和进度条一样平时隐藏、点画面才出现
    let back_hit = Rect::from_center_size(
        Pos2::new(rect.left() + 22.0, rect.top() + 22.0),
        Vec2::splat(44.0),
    );

    // 暂停时的中央播放键（常显）
    let c = rect.center();
    let center_hit = Rect::from_center_size(c, Vec2::splat(72.0));
    if matches!(phase, Phase::Paused | Phase::Failed) {
        let p = ui.painter();
        p.circle_filled(c, 24.0, Color32::from_black_alpha(110));
        p.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(c.x - 5.0, c.y - 9.0),
                Pos2::new(c.x - 5.0, c.y + 9.0),
                Pos2::new(c.x + 9.0, c.y),
            ],
            Color32::WHITE,
            Stroke::NONE,
        ));
    }

    // 蒙层控件：默认什么都不显示，点一下视频才出现，3 秒后自动隐藏
    let dur = app.video.duration_ms().max(0);
    let shown = app.display_position_ms().max(0);
    let ratio = if dur > 0 {
        (shown as f32 / dur as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let cy = rect.bottom() - 18.0;
    let play_hit = Rect::from_center_size(
        Pos2::new(rect.left() + 18.0, cy),
        Vec2::new(34.0, 34.0),
    );
    let time_pos = Pos2::new(rect.left() + 34.0, cy);
    let track = Rect::from_min_max(
        Pos2::new(rect.left() + 96.0, cy - 2.0),
        Pos2::new(rect.right() - 88.0, cy + 2.0),
    );
    let track_hit = Rect::from_min_max(
        Pos2::new(track.left() - 10.0, rect.bottom() - 36.0),
        Pos2::new(track.right() + 10.0, rect.bottom()),
    );
    let set_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 58.0, cy),
        Vec2::new(40.0, 32.0),
    );
    let fs_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 22.0, cy),
        Vec2::new(40.0, 32.0),
    );

    let cvis = app.fs_controls_visible();
    if cvis {
        let p = ui.painter();
        // 返回键（左上角）
        p.circle_filled(back_hit.center(), 16.0, Color32::from_black_alpha(120));
        p.add(egui::Shape::line(
            vec![
                Pos2::new(back_hit.center().x + 4.0, back_hit.center().y - 7.0),
                Pos2::new(back_hit.center().x - 4.0, back_hit.center().y),
                Pos2::new(back_hit.center().x + 4.0, back_hit.center().y + 7.0),
            ],
            Stroke::new(1.8, Color32::WHITE),
        ));
        // 底部渐变压暗，保证白字在亮画面上可读
        for i in 0..4 {
            let y0 = rect.bottom() - 40.0 + i as f32 * 10.0;
            p.rect_filled(
                Rect::from_min_max(
                    Pos2::new(rect.left(), y0),
                    Pos2::new(rect.right(), y0 + 10.0),
                ),
                0.0,
                Color32::from_black_alpha(40 + i * 18),
            );
        }
        widgets::play_icon(p, play_hit.center(), phase == Phase::Playing, Color32::WHITE);
        p.text(
            time_pos,
            Align2::LEFT_CENTER,
            format!(
                "{}/{}",
                theme::clock((shown / 1000) as u64),
                theme::clock((dur / 1000) as u64)
            ),
            FontId::proportional(10.0),
            Color32::WHITE,
        );
        p.rect_filled(track, track.height() / 2.0, Color32::from_gray(110));
        let mut played = track;
        played.set_right(track.left() + track.width() * ratio);
        p.rect_filled(played, played.height() / 2.0, theme::PINK);
        p.circle_filled(Pos2::new(played.right(), track.center().y), 5.0, theme::PINK);
        p.text(
            set_hit.center(),
            Align2::CENTER_CENTER,
            "设置",
            FontId::proportional(10.0),
            if app.player_settings {
                theme::PINK
            } else {
                Color32::WHITE
            },
        );
        widgets::fullscreen_icon(p, fs_hit.center(), Color32::WHITE, false);
    }
    // 进度条拖动（只在控件显示时生效，避免挡住页面滚动）
    if cvis {
        let seek = ui.interact(
            track_hit,
            egui::Id::new("detail-seek"),
            Sense::click_and_drag(),
        );
        if seek.drag_started() {
            if let Some(x) = ui.input(|i| i.pointer.interact_pos()).map(|q| q.x) {
                app.begin_seek(ratio_at(track, x));
            }
        }
        if seek.dragged() && app.seek_preview.is_some() {
            if let Some(x) = ui.input(|i| i.pointer.interact_pos()).map(|q| q.x) {
                app.update_seek(ratio_at(track, x));
            }
        }
        if seek.drag_stopped() {
            app.finish_seek();
        } else if seek.clicked() {
            if let Some(x) = seek.interact_pointer_pos().map(|q| q.x) {
                app.seek_ratio(ratio_at(track, x));
            }
        }
    }

    if resp.clicked() {
        if let Some(q) = resp.interact_pointer_pos() {
            if cvis {
                if back_hit.contains(q) {
                    // back() 会先把当前进度上报再 stop
                    app.back();
                    return;
                }
                if play_hit.contains(q) {
                    match phase {
                        Phase::Failed => app.video.play(&url),
                        Phase::Idle | Phase::Connecting => {
                            if has_src {
                                app.video.play(&url);
                            }
                        }
                        _ => app.video.toggle_pause(),
                    }
                    app.poke_controls();
                    return;
                }
                if set_hit.contains(q) {
                    app.player_settings = !app.player_settings;
                    app.poke_controls();
                    return;
                }
                if fs_hit.contains(q) {
                    app.enter_fullscreen();
                    return;
                }
                if track_hit.contains(q) {
                    app.seek_ratio(ratio_at(track, q.x));
                    return;
                }
            }
            // 暂停时点画面正中 = 继续播放
            if !matches!(phase, Phase::Playing) && center_hit.contains(q) {
                match phase {
                    Phase::Failed => app.video.play(&url),
                    _ => app.video.toggle_pause(),
                }
                app.poke_controls();
                return;
            }
            // 点画面：切换控件显示
            if cvis {
                app.hide_controls();
            } else {
                app.poke_controls();
            }
        }
    }
    if !has_src {
        ui.painter().text(
            rect.center() + Vec2::new(0.0, 34.0),
            Align2::CENTER_CENTER,
            if data.source_ready {
                "这条视频没有可播放文件"
            } else {
                "源文件还没准备好（可能未上传或还在转码）"
            },
            FontId::proportional(11.0),
            Color32::from_gray(225),
        );
    }
}

/// 全屏（横屏）播放：画面铺满，控件蒙层几秒后自动隐藏，点画面再叫出来
fn fullscreen_player(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    let size = ui.available_size();
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());

    let v = &data.video;
    let url = play_url(app, data);
    let has_src = !url.is_empty() && data.source_ready;
    let phase = app.video.phase();
    if has_src && phase != Phase::Failed {
        app.video.play(&url);
    }
    if (app.video.speed() - app.speed).abs() > 0.001 {
        app.video.set_speed(app.speed);
    }

    // 16:9 居中，多余部分留黑边，不拉伸画面
    let vr = widgets::fit16x9(rect);
    ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
    if has_src {
        app.video.ui_in(ui, vr);
    } else {
        widgets::draw_cover(ui, vr, &v.cover_url, &app.covers, 0.0);
        ui.painter()
            .rect_filled(vr, 0.0, Color32::from_black_alpha(60));
    }

    if app.danmaku_on && matches!(phase, Phase::Playing | Phase::Paused) && has_src {
        let pos = app.video.position_ms();
        if pos > 0 {
            draw_danmaku(ui, vr, &data.danmakus, pos);
        }
    }
    if has_src && matches!(phase, Phase::Playing | Phase::Paused) && !app.cues.is_empty() {
        let sec = app.video.position_ms().max(0) as f64 / 1000.0;
        if let Some(text) = crate::subtitle::at(&app.cues, sec) {
            draw_subtitle(ui, vr, text);
        }
    }

    // 暂停时的中央播放键
    if !matches!(phase, Phase::Playing) {
        let c = vr.center();
        let p = ui.painter();
        p.circle_filled(c, 26.0, Color32::from_black_alpha(110));
        widgets::play_icon(p, c, false, Color32::WHITE);
    }

    let dur = app.video.duration_ms().max(0);
    let shown = app.display_position_ms().max(0);
    let ratio = if dur > 0 {
        (shown as f32 / dur as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // ── 控件位置（对齐 B 站横屏：时间在进度条上方，底部一排功能键） ──
    let row_y = rect.bottom() - 22.0;
    let track = Rect::from_min_max(
        Pos2::new(rect.left() + 14.0, rect.bottom() - 52.0),
        Pos2::new(rect.right() - 14.0, rect.bottom() - 49.5),
    );
    let time_pos = Pos2::new(rect.left() + 14.0, rect.bottom() - 66.0);
    let play_hit = Rect::from_center_size(
        Pos2::new(rect.left() + 28.0, row_y),
        Vec2::new(48.0, 42.0),
    );
    let danmaku_hit = Rect::from_center_size(
        Pos2::new(rect.left() + 80.0, row_y),
        Vec2::new(46.0, 36.0),
    );
    let pill = Rect::from_min_max(
        Pos2::new(rect.left() + 108.0, row_y - 15.0),
        Pos2::new(rect.right() - 206.0, row_y + 15.0),
    );
    let sub_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 176.0, row_y),
        Vec2::new(56.0, 36.0),
    );
    let speed_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 112.0, row_y),
        Vec2::new(56.0, 36.0),
    );
    let quality_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 48.0, row_y),
        Vec2::new(60.0, 36.0),
    );
    let center_hit = Rect::from_center_size(vr.center(), Vec2::splat(84.0));
    let back_hit = Rect::from_min_size(
        Pos2::new(rect.left() + 6.0, rect.top() + 4.0),
        Vec2::new(48.0, 40.0),
    );
    // 顶部右侧：点赞 / 评论 / 收藏 / 稍后再看
    let act_center = |i: f32| Pos2::new(rect.right() - 34.0 - 54.0 * i, rect.top() + 22.0);
    let like_hit = Rect::from_center_size(act_center(0.0), Vec2::new(54.0, 40.0));
    let comment_hit = Rect::from_center_size(act_center(1.0), Vec2::new(54.0, 40.0));
    let fav_hit = Rect::from_center_size(act_center(2.0), Vec2::new(54.0, 40.0));
    let later_hit = Rect::from_center_size(act_center(3.0), Vec2::new(54.0, 40.0));

    // 弹幕输入条：键盘弹起时整条上移，避免被盖住
    let ime_h = if app.danmaku_ime {
        crate::video::ime_points(ui.ctx().pixels_per_point())
    } else {
        0.0
    };
    let input_y = rect.bottom() - ime_h - 26.0;
    let input_pill = Rect::from_min_max(
        Pos2::new(rect.left() + 14.0, input_y - 16.0),
        Pos2::new(rect.right() - 64.0, input_y + 16.0),
    );
    let send_hit = Rect::from_center_size(
        Pos2::new(rect.right() - 34.0, input_y),
        Vec2::new(46.0, 40.0),
    );

    // 弹幕输入：键盘由原生 EditText 持有，内容每帧镜像回来自己画
    if app.danmaku_ime {
        if let Some(t) = crate::video::ime_text() {
            if t != app.danmaku_text {
                app.danmaku_text = t;
            }
        }
    }

    let mut menu_hits: Vec<(Rect, FsPick)> = Vec::new();
    let visible = app.fs_controls_visible();
    if app.danmaku_ime {
        let p = ui.painter();
        // 输入条压在键盘正上方
        p.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), input_y - 32.0),
                Pos2::new(rect.right(), rect.bottom()),
            ),
            0.0,
            Color32::from_black_alpha(170),
        );
        p.rect_filled(input_pill, 16.0, Color32::from_black_alpha(210));
        draw_input_text(p, input_pill, &app.danmaku_text, "发个友善的弹幕见证当下");
        let ready = !app.danmaku_text.trim().is_empty();
        p.circle_filled(
            send_hit.center(),
            16.0,
            if ready { theme::PINK } else { Color32::from_gray(90) },
        );
        draw_send_icon(p, send_hit.center(), Color32::WHITE);
    } else if visible {
        let p = ui.painter();
        // 上下渐变压暗，保证控件在亮画面上也看得清
        widgets::scrim(p, rect);

        // 顶部：返回 + 标题 + 右侧互动（点赞/评论/收藏/稍后再看）
        p.add(egui::Shape::line(
            vec![
                Pos2::new(back_hit.center().x + 4.0, back_hit.center().y - 7.0),
                Pos2::new(back_hit.center().x - 4.0, back_hit.center().y),
                Pos2::new(back_hit.center().x + 4.0, back_hit.center().y + 7.0),
            ],
            Stroke::new(1.8, Color32::WHITE),
        ));
        let title_lines = widgets::wrap_lines(p, &v.title, 13.0, rect.width() - 300.0, 1);
        if let Some(t) = title_lines.first() {
            p.text(
                Pos2::new(back_hit.right() + 6.0, back_hit.center().y),
                Align2::LEFT_CENTER,
                t,
                FontId::proportional(13.0),
                Color32::from_gray(235),
            );
        }
        // 点赞
        draw_thumb_icon(
            p,
            Pos2::new(like_hit.center().x - 10.0, like_hit.center().y),
            if v.liked_by_me { theme::PINK } else { Color32::WHITE },
        );
        p.text(
            Pos2::new(like_hit.center().x + 1.0, like_hit.center().y),
            Align2::LEFT_CENTER,
            theme::human(v.like_count),
            FontId::proportional(10.0),
            Color32::from_gray(230),
        );
        // 评论
        draw_bubble_icon(
            p,
            Pos2::new(comment_hit.center().x - 10.0, comment_hit.center().y),
            Color32::WHITE,
        );
        p.text(
            Pos2::new(comment_hit.center().x + 1.0, comment_hit.center().y),
            Align2::LEFT_CENTER,
            theme::human(data.comments.len() as u64),
            FontId::proportional(10.0),
            Color32::from_gray(230),
        );
        // 收藏
        draw_star_icon(
            p,
            Pos2::new(fav_hit.center().x - 10.0, fav_hit.center().y),
            if v.favorited_by_me { theme::PINK } else { Color32::WHITE },
        );
        p.text(
            Pos2::new(fav_hit.center().x + 1.0, fav_hit.center().y),
            Align2::LEFT_CENTER,
            theme::human(v.fav_count),
            FontId::proportional(10.0),
            Color32::from_gray(230),
        );
        // 稍后再看
        draw_bookmark_icon(
            p,
            Pos2::new(later_hit.center().x - 6.0, later_hit.center().y),
            if v.in_watch_later { theme::PINK } else { Color32::WHITE },
        );

        // 底部：播放键 / 弹幕开关 / 弹幕输入 / 时间 / 进度条 / 字幕 / 倍速 / 清晰度
        widgets::play_icon(p, play_hit.center(), phase == Phase::Playing, Color32::WHITE);
        let on = app.danmaku_on;
        p.rect_stroke(
            danmaku_hit,
            6.0,
            Stroke::new(1.0, if on { theme::PINK } else { Color32::from_gray(210) }),
            StrokeKind::Inside,
        );
        p.text(
            danmaku_hit.center(),
            Align2::CENTER_CENTER,
            "弹",
            FontId::proportional(12.0),
            if on { theme::PINK } else { Color32::WHITE },
        );
        p.rect_filled(pill, 15.0, Color32::from_white_alpha(36));
        p.text(
            Pos2::new(pill.left() + 14.0, pill.center().y),
            Align2::LEFT_CENTER,
            "发个友善的弹幕见证当下",
            FontId::proportional(12.0),
            Color32::from_gray(215),
        );

        p.text(
            time_pos,
            Align2::LEFT_CENTER,
            format!(
                "{}/{}",
                theme::clock((shown / 1000) as u64),
                theme::clock((dur / 1000) as u64)
            ),
            FontId::proportional(11.0),
            Color32::from_gray(225),
        );
        p.rect_filled(track, track.height() / 2.0, Color32::from_gray(110));
        let mut played = track;
        played.set_right(track.left() + track.width() * ratio);
        p.rect_filled(played, played.height() / 2.0, theme::PINK);
        p.circle_filled(Pos2::new(played.right(), track.center().y), 6.5, theme::PINK);

        // 右侧：字幕 / 倍速 / 清晰度（点了弹选项面板）
        p.text(
            sub_hit.center(),
            Align2::CENTER_CENTER,
            "字幕",
            FontId::proportional(12.0),
            Color32::WHITE,
        );
        p.text(
            speed_hit.center(),
            Align2::CENTER_CENTER,
            if (app.speed - 1.0).abs() < 0.001 {
                "倍速".to_owned()
            } else {
                speed_label(app.speed)
            },
            FontId::proportional(12.0),
            Color32::WHITE,
        );
        p.text(
            quality_hit.center(),
            Align2::CENTER_CENTER,
            quality_label(app, data),
            FontId::proportional(12.0),
            Color32::WHITE,
        );

        // 选项面板（倍速 / 清晰度 / 字幕）
        if let Some(menu) = app.fs_menu {
            let items: Vec<(String, bool, FsPick)> = match menu {
                FsMenu::Speed => SPEEDS
                    .iter()
                    .map(|&s| {
                        (
                            speed_label(s),
                            (app.speed - s).abs() < 0.001,
                            FsPick::Speed(s),
                        )
                    })
                    .collect(),
                FsMenu::Quality => {
                    let mut list: Vec<&crate::api::VideoBitrate> =
                        data.bitrates.iter().collect();
                    list.sort_by_key(|b| b.kbps);
                    let mut v = vec![(
                        "自动".to_owned(),
                        app.quality.is_none(),
                        FsPick::Quality(None),
                    )];
                    for b in list {
                        v.push((
                            bitrate_label(b),
                            app.quality == Some(b.id),
                            FsPick::Quality(Some(b.id)),
                        ));
                    }
                    v
                }
                FsMenu::Subtitle => {
                    let mut v = vec![(
                        "关闭".to_owned(),
                        app.subtitle_id.is_none(),
                        FsPick::Subtitle(None),
                    )];
                    for s in &data.subtitles {
                        let label = if s.title.is_empty() {
                            s.lang.clone()
                        } else {
                            s.title.clone()
                        };
                        v.push((
                            label,
                            app.subtitle_id == Some(s.id),
                            FsPick::Subtitle(Some(s.id)),
                        ));
                    }
                    v
                }
            };
            let w = 150.0;
            let rh = 34.0;
            let h = items.len() as f32 * rh + 8.0;
            let panel = Rect::from_min_size(
                Pos2::new(rect.right() - 16.0 - w, row_y - 44.0 - h),
                Vec2::new(w, h),
            );
            p.rect_filled(panel, 8.0, Color32::from_black_alpha(215));
            for (i, (label, on, pick)) in items.iter().enumerate() {
                let r = Rect::from_min_size(
                    Pos2::new(panel.left() + 4.0, panel.top() + 4.0 + i as f32 * rh),
                    Vec2::new(w - 8.0, rh),
                );
                if *on {
                    p.rect_filled(r, 6.0, Color32::from_white_alpha(28));
                }
                p.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(12.0),
                    if *on { theme::PINK } else { Color32::WHITE },
                );
                menu_hits.push((r, *pick));
            }
        }
    }

    // ── 交互 ──
    let ptr = resp.interact_pointer_pos();

    // 弹幕输入态：只处理发送 / 重新拉键盘 / 收起
    if app.danmaku_ime {
        if resp.clicked() {
            if let Some(q) = ptr {
                if send_hit.contains(q) {
                    app.danmaku_ime = false;
                    crate::video::ime_close();
                    crate::video::ime_clear();
                    app.send_danmaku(data.video.id);
                } else if input_pill.contains(q) {
                    crate::video::ime_open(&app.danmaku_text);
                } else {
                    app.danmaku_ime = false;
                    crate::video::ime_close();
                }
            }
        }
        return;
    }

    if resp.drag_started() {
        if let Some(q) = ptr {
            if track.contains(q) {
                app.begin_seek(ratio_at(track, q.x));
            }
        }
    }
    if resp.dragged() && app.seek_preview.is_some() {
        if let Some(q) = ptr {
            app.update_seek(ratio_at(track, q.x));
        }
    }
    if resp.drag_stopped() {
        app.finish_seek();
    } else if resp.clicked() {
        if let Some(q) = ptr {
            // 选项面板打开时，只处理菜单项 / 点空白关闭
            if app.fs_menu.is_some() {
                for (r, pick) in &menu_hits {
                    if r.contains(q) {
                        match *pick {
                            FsPick::Speed(s) => app.set_speed(s),
                            FsPick::Quality(None) => app.quality = None,
                            FsPick::Quality(Some(id)) => {
                                if let Some(b) = data.bitrates.iter().find(|b| b.id == id) {
                                    app.select_quality(data.video.id, b);
                                }
                            }
                            FsPick::Subtitle(id) => {
                                app.select_subtitle(&data.subtitles, id)
                            }
                        }
                        app.fs_menu = None;
                        app.poke_controls();
                        return;
                    }
                }
                app.fs_menu = None;
                return;
            }
            // 暂停状态点画面正中 = 继续播放（即使蒙层已隐藏）
            if center_hit.contains(q) && !matches!(phase, Phase::Playing) {
                match phase {
                    Phase::Failed => app.video.play(&url),
                    _ => app.video.toggle_pause(),
                }
                app.poke_controls();
                return;
            }
            if visible {
                if back_hit.contains(q) {
                    app.exit_fullscreen();
                    return;
                }
                if play_hit.contains(q) {
                    app.video.toggle_pause();
                    app.poke_controls();
                    return;
                }
                if danmaku_hit.contains(q) {
                    app.danmaku_on = !app.danmaku_on;
                    app.poke_controls();
                    return;
                }
                if pill.contains(q) {
                    app.fs_menu = None;
                    app.danmaku_ime = true;
                    crate::video::ime_open(&app.danmaku_text);
                    return;
                }
                if sub_hit.contains(q) {
                    app.fs_menu = if app.fs_menu == Some(FsMenu::Subtitle) {
                        None
                    } else {
                        Some(FsMenu::Subtitle)
                    };
                    app.poke_controls();
                    return;
                }
                if speed_hit.contains(q) {
                    app.fs_menu = if app.fs_menu == Some(FsMenu::Speed) {
                        None
                    } else {
                        Some(FsMenu::Speed)
                    };
                    app.poke_controls();
                    return;
                }
                if quality_hit.contains(q) {
                    app.fs_menu = if app.fs_menu == Some(FsMenu::Quality) {
                        None
                    } else {
                        Some(FsMenu::Quality)
                    };
                    app.poke_controls();
                    return;
                }
                if track.contains(q) {
                    app.seek_ratio(ratio_at(track, q.x));
                    return;
                }
                if like_hit.contains(q) {
                    app.act(v.id, v.user_id, Action::Like);
                    app.poke_controls();
                    return;
                }
                if comment_hit.contains(q) {
                    app.exit_fullscreen();
                    app.detail_tab = DetailTab::Comments;
                    return;
                }
                if fav_hit.contains(q) {
                    app.exit_fullscreen();
                    app.open_favorite_picker(v.id);
                    return;
                }
                if later_hit.contains(q) {
                    app.act(v.id, v.user_id, Action::WatchLater);
                    app.poke_controls();
                    return;
                }
            }
            // 点画面空白处：切换控件显示
            if visible {
                app.hide_controls();
            } else {
                app.poke_controls();
            }
        }
    }
}

fn bitrate_label(b: &crate::api::VideoBitrate) -> String {
    if !b.label.is_empty() {
        b.label.clone()
    } else if b.height > 0 {
        format!("{}P", b.height)
    } else {
        format!("{}kbps", b.kbps)
    }
}

fn ratio_at(bar: Rect, x: f32) -> f32 {
    if bar.width() > 0.0 {
        ((x - bar.left()) / bar.width()).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn quality_label(app: &App, data: &crate::app::DetailData) -> String {
    let Some(id) = app.quality else {
        return "自动".to_owned();
    };
    data.bitrates
        .iter()
        .find(|b| b.id == id)
        .map(|b| bitrate_label(b))
        .unwrap_or_else(|| "自动".to_owned())
}

/// 全屏弹幕输入框里的一行文字 + 光标
fn draw_input_text(p: &egui::Painter, pill: Rect, text: &str, hint: &str) {
    let tr = pill.shrink2(Vec2::new(14.0, 6.0));
    if text.is_empty() {
        p.text(
            Pos2::new(tr.left(), tr.center().y),
            Align2::LEFT_CENTER,
            hint,
            FontId::proportional(12.0),
            Color32::from_gray(170),
        );
        return;
    }
    let clipped = p.with_clip_rect(tr);
    let g = clipped.layout_no_wrap(
        text.to_owned(),
        FontId::proportional(12.0),
        Color32::WHITE,
    );
    let x = if g.size().x > tr.width() {
        tr.right() - g.size().x
    } else {
        tr.left()
    };
    clipped.galley(Pos2::new(x, tr.center().y - g.size().y / 2.0), g.clone(), Color32::WHITE);
    let caret_x = (tr.left() + g.size().x).min(tr.right());
    p.line_segment(
        [
            Pos2::new(caret_x, tr.center().y - 8.0),
            Pos2::new(caret_x, tr.center().y + 8.0),
        ],
        Stroke::new(1.5, theme::PINK),
    );
}

fn draw_send_icon(p: &egui::Painter, c: Pos2, color: Color32) {
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(c.x - 5.0, c.y - 6.0),
            Pos2::new(c.x + 6.0, c.y),
            Pos2::new(c.x - 5.0, c.y + 6.0),
        ],
        color,
        Stroke::NONE,
    ));
}

fn draw_thumb_icon(p: &egui::Painter, c: Pos2, color: Color32) {
    let s = 4.6;
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(c.x - s, c.y + s * 0.9),
            Pos2::new(c.x + s, c.y + s * 0.9),
            Pos2::new(c.x, c.y - s * 1.1),
        ],
        Color32::TRANSPARENT,
        Stroke::new(1.2, color),
    ));
}

fn draw_bubble_icon(p: &egui::Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, Vec2::new(13.0, 10.0));
    p.rect_stroke(r, 3.0, Stroke::new(1.1, color), StrokeKind::Inside);
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(c.x - 2.5, r.bottom() - 1.0),
            Pos2::new(c.x + 2.5, r.bottom() - 1.0),
            Pos2::new(c.x - 1.0, r.bottom() + 3.0),
        ],
        color,
        Stroke::NONE,
    ));
}

fn draw_star_icon(p: &egui::Painter, c: Pos2, color: Color32) {
    let pts: Vec<Pos2> = (0..10)
        .map(|i| {
            let a = std::f32::consts::PI * 2.0 * (i as f32) / 10.0
                - std::f32::consts::PI / 2.0;
            let r = if i % 2 == 0 { 6.0 } else { 2.6 };
            Pos2::new(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect();
    p.add(egui::Shape::convex_polygon(
        pts,
        Color32::TRANSPARENT,
        Stroke::new(1.1, color),
    ));
}

fn draw_bookmark_icon(p: &egui::Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, Vec2::new(10.0, 13.0));
    p.add(egui::Shape::line(
        vec![
            r.left_top(),
            r.right_top(),
            r.right_bottom(),
            Pos2::new(c.x, r.bottom() - 4.0),
            r.left_bottom(),
            r.left_top(),
        ],
        Stroke::new(1.2, color),
    ));
}

/// 清晰度 / 字幕 / 倍速切换条
fn options_bar(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    if !data.bitrates.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.add_space(12.0);
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            ui.label(egui::RichText::new("清晰度").color(theme::TEXT3).size(10.0));
            // 后端已按 kbps 升序返回，这里显式再排一次，免得换实现后顺序变
            let mut list: Vec<&crate::api::VideoBitrate> = data.bitrates.iter().collect();
            list.sort_by_key(|b| b.kbps);
            for b in list {
                let label = if !b.label.is_empty() {
                    b.label.clone()
                } else if b.height > 0 {
                    format!("{}P", b.height)
                } else {
                    format!("{}kbps", b.kbps)
                };
                let on = app.quality == Some(b.id);
                if chip(ui, &label, on) && !on {
                    let video_id = data.video.id;
                    app.select_quality(video_id, b);
                }
            }
        });
    }
    if !data.subtitles.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.add_space(12.0);
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            ui.label(egui::RichText::new("字幕").color(theme::TEXT3).size(10.0));
            let off = app.subtitle_id.is_none();
            if chip(ui, "关", off) && !off {
                app.select_subtitle(&data.subtitles, None);
            }
            for s in &data.subtitles {
                let label = if s.title.is_empty() {
                    s.lang.clone()
                } else {
                    s.title.clone()
                };
                let on = app.subtitle_id == Some(s.id);
                if chip(ui, &label, on) && !on {
                    app.select_subtitle(&data.subtitles, Some(s.id));
                }
            }
        });
    }
    // 倍速
    ui.horizontal_wrapped(|ui| {
        ui.add_space(12.0);
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
        ui.label(egui::RichText::new("倍速").color(theme::TEXT3).size(10.0));
        for s in SPEEDS {
            let on = (app.speed - s).abs() < 0.001;
            if chip(ui, &speed_label(s), on) && !on {
                app.set_speed(s);
            }
        }
    });
    ui.add_space(2.0);
}

const SPEEDS: [f32; 6] = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

/// 全屏选项面板里的一个可点选项
#[derive(Clone, Copy)]
enum FsPick {
    Speed(f32),
    Quality(Option<u64>),
    Subtitle(Option<u64>),
}

fn speed_label(s: f32) -> String {
    if (s - s.round()).abs() < 0.001 {
        format!("{s:.1}x")
    } else {
        format!("{s}x")
    }
}

fn chip(ui: &mut egui::Ui, label: &str, on: bool) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .color(if on { Color32::WHITE } else { theme::TEXT2 })
            .size(11.0),
    )
    .fill(if on { theme::PINK } else { theme::SURFACE })
    .corner_radius(11.0)
    .min_size(Vec2::new(0.0, 24.0));
    ui.add(btn).clicked()
}

/// 当前实际播放地址：选了清晰度就用那条，否则回原画
fn play_url(app: &App, data: &crate::app::DetailData) -> String {
    if let Some(q) = app.quality {
        if let Some(b) = data.bitrates.iter().find(|b| b.id == q) {
            let url = app.client.abs(&b.url);
            if !url.is_empty() {
                return url;
            }
        }
    }
    // 原画地址经常指向还没上传/还没转码完成的文件，优先用多清晰度里的产物
    if let Some(b) = data.bitrates.iter().max_by_key(|b| b.kbps) {
        let url = app.client.abs(&b.url);
        if !url.is_empty() {
            return url;
        }
    }
    app.client.abs(&data.video.video_url)
}

/// 字幕：白字 + 黑描边，居中压在画面底部，多行自动往上叠
fn draw_subtitle(ui: &mut egui::Ui, rect: Rect, text: &str) {
    let p = ui.painter().with_clip_rect(rect);
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len() as f32;
    for (i, line) in lines.iter().enumerate() {
        let galley = p.layout_no_wrap(line.to_string(), FontId::proportional(13.0), Color32::WHITE);
        let pos = Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            rect.bottom() - 22.0 - (n - 1.0 - i as f32) * 18.0,
        );
        // 描边：往四周各偏 1px 画一遍黑字，保证在亮画面上也读得清
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            p.galley(
                pos + Vec2::new(dx, dy),
                galley.clone(),
                Color32::from_black_alpha(190),
            );
        }
        p.galley(pos, galley, Color32::WHITE);
    }
}

fn draw_danmaku(ui: &mut egui::Ui, rect: Rect, list: &[Danmaku], pos_ms: i64) {
    if pos_ms <= 0 || list.is_empty() {
        return;
    }
    const LIFE: i64 = 7000;
    let lanes = (((rect.height() - 16.0) / 20.0).floor() as usize).max(1);
    let p = ui.painter().with_clip_rect(rect);
    for (i, d) in list.iter().enumerate() {
        let start = (d.video_time * 1000.0) as i64;
        let dt = pos_ms - start;
        if dt < 0 || dt > LIFE {
            continue;
        }
        let t = dt as f32 / LIFE as f32;
        let galley = p.layout_no_wrap(d.content.clone(), FontId::proportional(12.0), Color32::WHITE);
        let text_w = galley.size().x;
        let x = rect.right() - t * (rect.width() + text_w);
        let y = rect.top() + 8.0 + (i % lanes) as f32 * 20.0;
        // 先描一层黑边保证在任何画面上都读得清，再画白字
        p.galley(
            Pos2::new(x + 1.0, y + 1.0),
            galley.clone(),
            Color32::from_black_alpha(150),
        );
        p.galley(Pos2::new(x, y), galley, Color32::WHITE);
    }
}

// ───────────────────────── tab / 信息区 ─────────────────────────

fn tabs(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
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

    // 左：简介 / 评论；右：弹幕输入框 + 「弹」开关
    let n = data.comments.len();
    let items = [
        (DetailTab::Intro, "简介".to_owned()),
        (DetailTab::Comments, format!("评论 {n}")),
    ];
    let slot = 70.0;
    for (i, (tab, label)) in items.iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let active = app.detail_tab == *tab;
        p.text(
            Pos2::new(cx, rect.center().y - 3.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(if active { 14.0 } else { 13.0 }),
            if active { theme::TEXT } else { theme::TEXT2 },
        );
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
            .interact(hit, egui::Id::new(("detail-tab", i)), Sense::click())
            .clicked()
        {
            app.detail_tab = *tab;
        }
    }

    // 弹幕开关（最右）
    let toggle = Rect::from_min_size(
        Pos2::new(rect.right() - 40.0, rect.center().y - 13.0),
        Vec2::new(30.0, 26.0),
    );
    let p = ui.painter();
    if app.danmaku_on {
        p.rect_stroke(toggle, 6.0, Stroke::new(1.0, theme::PINK), StrokeKind::Inside);
        p.text(
            toggle.center(),
            Align2::CENTER_CENTER,
            "弹",
            FontId::proportional(12.0),
            theme::PINK,
        );
    } else {
        p.rect_stroke(toggle, 6.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            toggle.center(),
            Align2::CENTER_CENTER,
            "弹",
            FontId::proportional(12.0),
            theme::TEXT3,
        );
        // 关掉时画一道斜杠
        p.line_segment(
            [
                Pos2::new(toggle.left() + 8.0, toggle.bottom() - 8.0),
                Pos2::new(toggle.right() - 8.0, toggle.top() + 8.0),
            ],
            Stroke::new(1.2, theme::TEXT3),
        );
    }
    if ui
        .interact(toggle, egui::Id::new("detail-danmaku-toggle"), Sense::click())
        .clicked()
    {
        app.danmaku_on = !app.danmaku_on;
    }

    // 弹幕输入框（tab 与开关之间）
    let input = Rect::from_min_max(
        Pos2::new(rect.left() + slot * 2.0 + 4.0, rect.center().y - 13.0),
        Pos2::new(toggle.left() - 8.0, rect.center().y + 13.0),
    );
    if input.width() > 60.0 {
        let p = ui.painter();
        p.rect_filled(input, 13.0, theme::SURFACE);
        let field = ui.put(
            input.shrink2(Vec2::new(10.0, 4.0)),
            egui::TextEdit::singleline(&mut app.danmaku_text)
                .frame(egui::Frame::NONE)
                .hint_text("发个弹幕见证当下")
                .font(FontId::proportional(11.0)),
        );
        let focused = field.has_focus();
        app.note_focus(focused);
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.send_danmaku(data.video.id);
        }
    }
}

fn uploader_row(ui: &mut egui::Ui, app: &mut App, v: &VideoDetail) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 64.0), Sense::hover());
    let name = if v.uploader.is_empty() {
        format!("UP {}", v.user_id)
    } else {
        v.uploader.clone()
    };
    let av = Pos2::new(rect.left() + 30.0, rect.center().y);
    if v.uploader_avatar_url.is_empty() {
        widgets::avatar(ui.painter(), av, 17.0, &name, v.user_id);
    } else {
        widgets::avatar_tex(
            ui,
            av,
            17.0,
            &name,
            &v.uploader_avatar_url,
            &app.covers,
            v.user_id,
        );
    }
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left() + 56.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        &name,
        FontId::proportional(14.0),
        theme::TEXT,
    );
    p.text(
        Pos2::new(rect.left() + 56.0, rect.top() + 32.0),
        Align2::LEFT_TOP,
        format!(
            "{}粉丝 · {}视频",
            theme::human(v.uploader_follower_count),
            theme::human(v.uploader_published_count)
        ),
        FontId::proportional(11.0),
        theme::TEXT3,
    );

    // 关注按钮
    let btn = Rect::from_min_size(
        Pos2::new(rect.right() - 76.0, rect.center().y - 13.0),
        Vec2::new(64.0, 26.0),
    );
    if v.followed_by_me {
        p.rect_stroke(btn, 13.0, Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "已关注",
            FontId::proportional(11.0),
            theme::TEXT3,
        );
    } else {
        p.rect_filled(btn, 13.0, theme::PINK);
        p.text(
            btn.center(),
            Align2::CENTER_CENTER,
            "+ 关注",
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    }
    if ui
        .interact(btn, egui::Id::new("detail-follow"), Sense::click())
        .clicked()
    {
        app.act(v.id, v.user_id, Action::Follow);
    }

    // 点头像 / 昵称这一片进 UP 主主页（右侧关注按钮已经单独占了热区）
    let hit = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top()),
        Pos2::new(btn.left() - 8.0, rect.bottom()),
    );
    if ui
        .interact(hit, egui::Id::new("detail-up-space"), Sense::click())
        .clicked()
        && v.user_id > 0
    {
        app.open_space(v.user_id);
    }
}

fn title_block(ui: &mut egui::Ui, v: &VideoDetail) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 76.0), Sense::hover());
    let p = ui.painter();
    let lines = widgets::wrap_lines(p, &v.title, 16.0, w - 28.0, 2);
    let mut y = rect.top() + 2.0;
    for l in lines {
        p.text(
            Pos2::new(rect.left() + 14.0, y),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(16.0),
            theme::TEXT,
        );
        y += 23.0;
    }
    let mut meta = format!(
        "{}播放 · {}弹幕",
        theme::human(v.play_count),
        theme::human(v.danmaku_count)
    );
    if !v.created_at.is_empty() {
        meta.push_str(&format!(" · {}", v.created_at));
    }
    if v.watching_count > 0 {
        meta.push_str(&format!(" · {}人正在看", v.watching_count));
    }
    p.text(
        Pos2::new(rect.left() + 14.0, y + 4.0),
        Align2::LEFT_TOP,
        meta,
        FontId::proportional(11.0),
        theme::TEXT3,
    );
}

fn action_bar(ui: &mut egui::Ui, app: &mut App, v: &VideoDetail) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 56.0), Sense::hover());
    let p = ui.painter();
    p.line_segment(
        [
            Pos2::new(rect.left(), rect.top()),
            Pos2::new(rect.right(), rect.top()),
        ],
        Stroke::new(1.0, theme::LINE),
    );

    let items: [(&str, String, bool, Action); 4] = [
        ("点赞", theme::human(v.like_count), v.liked_by_me, Action::Like),
        ("投币", theme::human(v.coin_count), v.coined_by_me, Action::Coin),
        ("收藏", theme::human(v.fav_count), v.favorited_by_me, Action::Favorite),
        (
            "稍后再看",
            if v.in_watch_later { "已加" } else { "＋" }.to_owned(),
            v.in_watch_later,
            Action::WatchLater,
        ),
    ];
    let slot = w / 4.0;
    for (i, (label, count, on, act)) in items.iter().enumerate() {
        let cx = rect.left() + slot * (i as f32 + 0.5);
        let color = if *on { theme::PINK } else { theme::TEXT2 };
        p.text(
            Pos2::new(cx, rect.top() + 14.0),
            Align2::CENTER_TOP,
            count,
            FontId::proportional(14.0),
            color,
        );
        p.text(
            Pos2::new(cx, rect.top() + 34.0),
            Align2::CENTER_TOP,
            *label,
            FontId::proportional(11.0),
            theme::TEXT3,
        );
        let hit = Rect::from_min_size(
            Pos2::new(rect.left() + slot * i as f32, rect.top()),
            Vec2::new(slot, rect.height()),
        );
        if ui
            .interact(hit, egui::Id::new(("detail-act", i)), Sense::click())
            .clicked()
        {
            // 收藏要先选收藏夹（可以同时放进多个夹），其余操作直接发请求
            if *act == Action::Favorite {
                app.open_favorite_picker(v.id);
            } else {
                app.act(v.id, v.user_id, *act);
            }
        }
    }
}

fn intro(ui: &mut egui::Ui, app: &mut App, data: &crate::app::DetailData) {
    let v = &data.video;

    // 章节：点一下跳到对应时间（GET /videos/:id/chapters）
    if !data.chapters.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("章节")
                    .color(theme::TEXT)
                    .size(14.0)
                    .strong(),
            );
        });
        ui.add_space(2.0);
        let mut seek: Option<f64> = None;
        for c in &data.chapters {
            let w = ui.available_width();
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 28.0), Sense::click());
            let p = ui.painter();
            p.circle_filled(Pos2::new(rect.left() + 22.0, rect.center().y), 3.0, theme::PINK);
            p.text(
                Pos2::new(rect.left() + 32.0, rect.center().y),
                Align2::LEFT_CENTER,
                theme::clock(c.time_sec.max(0.0) as u64),
                FontId::proportional(11.0),
                theme::TEXT3,
            );
            let lines = widgets::wrap_lines(p, &c.title, 12.0, w - 110.0, 1);
            if let Some(l) = lines.first() {
                p.text(
                    Pos2::new(rect.left() + 88.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    l,
                    FontId::proportional(12.0),
                    theme::TEXT,
                );
            }
            if resp.clicked() {
                seek = Some(c.time_sec);
            }
        }
        if let Some(t) = seek {
            app.video.seek_to((t * 1000.0) as i64);
        }
        ui.add_space(8.0);
    }

    if !v.tags.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.add_space(14.0);
            for t in &v.tags {
                let btn = egui::Button::new(
                    egui::RichText::new(format!("#{t}"))
                        .color(theme::TEXT2)
                        .size(11.0),
                )
                .fill(theme::SURFACE)
                .corner_radius(11.0)
                .min_size(Vec2::new(0.0, 24.0));
                ui.add(btn);
            }
        });
        ui.add_space(6.0);
    }
    if !v.description.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new(&v.description)
                    .color(theme::TEXT2)
                    .size(12.0),
            );
        });
        ui.add_space(8.0);
    }
    if data.danmakus.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("这条视频还没有弹幕")
                    .color(theme::TEXT3)
                    .size(11.0),
            );
        });
    }

    // 相关推荐
    if data.related.is_empty() {
        return;
    }
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("相关推荐")
                .color(theme::TEXT)
                .size(14.0)
                .strong(),
        );
    });
    ui.add_space(2.0);
    let mut clicked = None;
    for r in &data.related {
        if widgets::video_row(ui, r, &app.covers).clicked() {
            clicked = Some(r.id);
        }
    }
    if let Some(next) = clicked {
        app.open_detail(next);
    }
}
