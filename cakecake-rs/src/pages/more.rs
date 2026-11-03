//! 「更多」页面合集：文章 / 创作中心 / 动态详情 / 专题 / 硬币账本 / 治理客服 / 直播开播
//!
//! 这些页面结构相近（顶栏 + 列表 / 表单），数据统一走 `App::load_more` 的那一个数据槽，
//! 写操作统一走 `App::more_action`，避免十几套重复的加载状态。
//!
//! 权限：进来的每一页都是「本人」视角（我的文章 / 我的投稿 / 我的工单），
//! 列表里的删除、发布等按钮只对作者本人出现，后端还会再校验一次。

use crate::{
    api,
    app::{App, LoadState, Route},
    theme, widgets,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    match app.route {
        Route::ArticleList => article_list(ui, app),
        Route::Article(id) => article_detail(ui, app, id),
        Route::MyArticles => my_articles(ui, app),
        Route::Creator => creator(ui, app),
        Route::CreatorComments => creator_comments(ui, app),
        Route::CreatorDanmakus => creator_danmakus(ui, app),
        Route::DynamicDetail(id) => dynamic_detail(ui, app, id),
        Route::Specials => specials(ui, app),
        Route::Ledger => ledger(ui, app),
        Route::Governance => governance(ui, app),
        Route::LiveHost => live_host(ui, app),
        _ => {}
    }
}

fn back(ui: &mut egui::Ui, app: &mut App, title: &str, id: &str) -> bool {
    if widgets::back_bar(ui, title, id) {
        app.back();
        return true;
    }
    false
}

/// 统一的加载 / 失败 / 空态处理；返回 true 表示这一帧不用再画内容
fn gate(ui: &mut egui::Ui, app: &mut App) -> bool {
    match app.more_state.clone() {
        LoadState::Idle | LoadState::Loading => {
            centered(ui, "加载中…");
            true
        }
        LoadState::Err(e) => {
            if App::empty_state(ui, "加载失败", &e) {
                app.load_more();
            }
            true
        }
        LoadState::Ready => false,
    }
}

fn centered(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(110.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(12.0));
    });
}

fn empty(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(egui::RichText::new(text).color(theme::TEXT3).size(11.0));
    });
    ui.add_space(20.0);
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(title)
                .color(theme::TEXT)
                .size(13.0)
                .strong(),
        );
    });
    ui.add_space(2.0);
}

/// 通用列表行：左标题 + 副标题 + 右侧小字
fn row(ui: &mut egui::Ui, app: &App, title: &str, sub: &str, right: &str, cover: &str) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 66.0), Sense::click());
    let mut text_x = rect.left() + 14.0;
    if !cover.is_empty() {
        let c = Rect::from_min_size(
            Pos2::new(text_x, rect.top() + 8.0),
            Vec2::new(72.0, 50.0),
        );
        widgets::draw_cover(ui, c, cover, &app.covers, 4.0);
        text_x = c.right() + 10.0;
    }
    let p = ui.painter();
    let lines = widgets::wrap_lines(p, title, 13.0, rect.right() - text_x - 70.0, 1);
    if let Some(l) = lines.first() {
        p.text(
            Pos2::new(text_x, rect.top() + 12.0),
            Align2::LEFT_TOP,
            l,
            FontId::proportional(13.0),
            theme::TEXT,
        );
    }
    if !sub.is_empty() {
        p.text(
            Pos2::new(text_x, rect.top() + 36.0),
            Align2::LEFT_TOP,
            sub,
            FontId::proportional(10.5),
            theme::TEXT3,
        );
    }
    if !right.is_empty() {
        p.text(
            Pos2::new(rect.right() - 14.0, rect.center().y),
            Align2::RIGHT_CENTER,
            right,
            FontId::proportional(10.5),
            theme::TEXT3,
        );
    }
    p.line_segment(
        [
            Pos2::new(rect.left() + 14.0, rect.bottom()),
            Pos2::new(rect.right() - 14.0, rect.bottom()),
        ],
        Stroke::new(1.0, theme::LINE),
    );
    resp.clicked()
}

fn pill(ui: &mut egui::Ui, label: &str, on: bool) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .color(if on { Color32::WHITE } else { theme::TEXT2 })
            .size(11.0),
    )
    .fill(if on { theme::PINK } else { theme::SURFACE })
    .corner_radius(12.0)
    .min_size(Vec2::new(0.0, 26.0));
    ui.add(btn).clicked()
}

/// 表单字段（用枚举而不是 &mut String，避开 app 同时被借两次）
#[derive(Clone, Copy)]
enum F {
    Title,
    Body,
    Text,
}

fn text_field(ui: &mut egui::Ui, app: &mut App, which: F, hint: &str) {
    let mut focused = false;
    {
        let value = match which {
            F::Title => &mut app.f_title,
            F::Body => &mut app.f_body,
            F::Text => &mut app.f_text,
        };
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            let desired = ui.available_width() - 28.0;
            if matches!(which, F::Body) {
                let resp = ui.add(
                    egui::TextEdit::multiline(value)
                        .hint_text(hint)
                        .desired_width(desired)
                        .desired_rows(4),
                );
                focused = resp.has_focus();
            } else {
                let resp = ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(hint)
                        .desired_width(desired),
                );
                focused = resp.has_focus();
            }
        });
    }
    app.note_focus(focused);
    ui.add_space(4.0);
}

// ───────────────────────── 专栏 ─────────────────────────

fn article_list(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "专栏", "art-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let items = app.more.articles.clone();
    if items.is_empty() {
        empty(ui, "还没有已发布的专栏");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("article-list")
        .show(ui, |ui| {
            let mut open = None;
            for a in &items {
                let sub = format!(
                    "{} · {} 阅读 · {} 评论",
                    a.author_name, a.view_count, a.comment_count
                );
                if row(ui, app, &a.title, &sub, "", &a.cover_url) {
                    open = Some(a.id);
                }
            }
            if let Some(id) = open {
                app.open_article(id);
            }
            ui.add_space(12.0);
        });
}

fn article_detail(ui: &mut egui::Ui, app: &mut App, id: u64) {
    if back(ui, app, "专栏", "artd-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let Some(a) = app.more.article.clone() else {
        empty(ui, "文章不存在");
        return;
    };

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("article-detail")
        .show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new(&a.title)
                        .color(theme::TEXT)
                        .size(17.0)
                        .strong(),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} · {} 阅读 · {} 评论 · {}",
                        a.author_name, a.view_count, a.comment_count, a.published_at
                    ))
                    .color(theme::TEXT3)
                    .size(10.5),
                );
            });
            ui.add_space(8.0);

            // 正文：后端同时给 md 和 html，这里按 md 纯文本排版（不引入 webview）
            let body = if a.body_md.trim().is_empty() {
                a.body_html
                    .replace("<p>", "\n")
                    .replace("</p>", "\n")
                    .replace("<br>", "\n")
                    .replace("<br/>", "\n")
            } else {
                a.body_md.clone()
            };
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new(body)
                        .color(theme::TEXT2)
                        .size(12.5),
                );
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(14.0);
                if pill(ui, &format!("收藏 {}", a.fav_count), a.favorited_by_me) {
                    app.more_action(move |c| c.toggle_article_favorite(id).map(|_| ()));
                }
                if pill(ui, &format!("投币 {}", a.coin_count), a.coined_by_me) {
                    app.more_action(move |c| c.post_article_coin(id));
                }
            });
            ui.add_space(8.0);

            section(ui, &format!("评论 {}", a.comment_count));
            if a.comments_closed {
                empty(ui, "作者关闭了评论区");
            } else {
                for c in app.more.article_comments.clone() {
                    let body = c.content.clone();
                    let name = if c.username.is_empty() {
                        "匿名".to_owned()
                    } else {
                        c.username.clone()
                    };
                    let sub = format!("{} · 赞 {}", c.created_at, c.like_count);
                    row(ui, app, &format!("{name}：{body}"), &sub, "", "");
                }
                text_field(ui, app, F::Text, "写条评论…");
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    if pill(ui, "发表评论", false) && !app.f_text.trim().is_empty() {
                        let text = app.f_text.trim().to_owned();
                        app.f_text.clear();
                        app.more_action(move |c| c.post_article_comment(id, &text, 0));
                    }
                });
            }
            ui.add_space(16.0);
        });
}

fn my_articles(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "我的文章", "myart-back") {
        return;
    }

    // 新建/编辑区（editing == 0 表示新建）
    section(ui, if app.more.editing > 0 { "编辑文章" } else { "写新文章" });
    text_field(ui, app, F::Title, "标题");
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.add(
            egui::TextEdit::multiline(&mut app.f_body)
                .hint_text("正文（支持 Markdown）")
                .desired_width(ui.available_width() - 28.0)
                .desired_rows(4),
        );
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let editing = app.more.editing;
        if pill(ui, if editing > 0 { "保存修改" } else { "发布" }, false) {
            let (title, body) = (app.f_title.trim().to_owned(), app.f_body.clone());
            if !title.is_empty() {
                app.more_action(move |c| {
                    if editing > 0 {
                        c.put_article(editing, &title, &body)
                    } else {
                        c.post_article(&title, &body).map(|_| ())
                    }
                });
                app.f_title.clear();
                app.f_body.clear();
            }
        }
        if pill(ui, "清空", false) {
            app.f_title.clear();
            app.f_body.clear();
        }
    });
    ui.add_space(6.0);

    if gate(ui, app) {
        return;
    }
    section(ui, "已发布 / 草稿");
    let items = app.more.my_articles.clone();
    if items.is_empty() {
        empty(ui, "还没有写过文章");
        return;
    }
    let mut del: Option<u64> = None;
    let mut edit: Option<u64> = None;
    for a in &items {
        let sub = format!("{} 阅读 · {}", a.view_count, a.published_at);
        if row(ui, app, &a.title, &sub, "删除", &a.cover_url) {
            edit = Some(a.id);
        }
        let w = ui.available_width();
        let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 26.0), Sense::click());
        ui.painter().text(
            Pos2::new(r.right() - 60.0, r.center().y),
            Align2::RIGHT_CENTER,
            "删除",
            FontId::proportional(10.5),
            theme::UP_TAG,
        );
        if resp.clicked() {
            del = Some(a.id);
        }
    }
    if let Some(id) = del {
        app.more_action(move |c| c.delete_article(id));
    }
    if let Some(id) = edit {
        // 点标题把正文拉回来填进编辑器（只对自己文章开放，后端按 uid 校验）
        app.edit_article(id);
    }
}

// ───────────────────────── 创作中心 ─────────────────────────

fn creator(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "创作中心", "creator-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    if let Some(s) = app.more.stats.clone() {
        let w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 74.0), Sense::hover());
        let p = ui.painter();
        let items = [
            ("播放", s.total_plays),
            ("粉丝", s.total_fans),
            ("点赞", s.total_likes),
            ("投币", s.total_coins),
        ];
        let slot = w / 4.0;
        for (i, (label, v)) in items.iter().enumerate() {
            let cx = rect.left() + slot * (i as f32 + 0.5);
            p.text(
                Pos2::new(cx, rect.top() + 18.0),
                Align2::CENTER_TOP,
                theme::human(*v as u64),
                FontId::proportional(16.0),
                theme::TEXT,
            );
            p.text(
                Pos2::new(cx, rect.top() + 44.0),
                Align2::CENTER_TOP,
                *label,
                FontId::proportional(10.5),
                theme::TEXT3,
            );
        }
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if pill(ui, "评论管理", false) {
            app.open_more(Route::CreatorComments);
        }
        if pill(ui, "弹幕管理", false) {
            app.open_more(Route::CreatorDanmakus);
        }
        if pill(ui, "我的文章", false) {
            app.open_more(Route::MyArticles);
        }
        if pill(ui, "+ 投稿", false) {
            app.open_upload();
        }
        if pill(ui, "写文章", false) {
            app.more.editing = 0;
            app.f_title.clear();
            app.f_body.clear();
            app.open_more(Route::MyArticles);
        }
    });
    section(ui, "我的投稿");

    let videos = app.more.videos.clone();
    if videos.is_empty() {
        empty(ui, "还没有投稿");
        return;
    }
    let mut publish: Option<u64> = None;
    let mut del: Option<u64> = None;
    let mut edit: Option<u64> = None;
    for v in &videos {
        let sub = format!(
            "{} · {} 播放 · {} 评论 · {}",
            v.status, v.play_count, v.comment_count, v.created_at
        );
        row(ui, app, &v.title, &sub, "", &v.cover_url);
        let w = ui.available_width();
        let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::click());
        let p = ui.painter();
        let actions = if v.status == "draft" {
            vec![
                ("发布", theme::PINK),
                ("编辑", theme::TEXT2),
                ("删除", theme::UP_TAG),
            ]
        } else {
            vec![("编辑", theme::TEXT2), ("删除", theme::UP_TAG)]
        };
        let mut rects = Vec::new();
        let mut x = r.right() - 14.0;
        for (label, color) in &actions {
            let rect = Rect::from_min_max(
                Pos2::new(x - 44.0, r.top()),
                Pos2::new(x, r.bottom()),
            );
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                *label,
                FontId::proportional(10.5),
                *color,
            );
            rects.push((label.to_string(), rect));
            x -= 52.0;
        }
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                for (label, rect) in &rects {
                    if rect.contains(pos) {
                        match label.as_str() {
                            "发布" => publish = Some(v.id),
                            "编辑" => edit = Some(v.id),
                            _ => del = Some(v.id),
                        }
                    }
                }
            }
        }
    }
    if let Some(id) = edit {
        app.open_video_edit(id);
    }
    if let Some(id) = publish {
        app.more_action(move |c| c.publish_draft(id));
    }
    if let Some(id) = del {
        app.more_action(move |c| c.delete_my_video(id));
    }
    ui.add_space(12.0);
}

fn creator_comments(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "评论管理", "cc-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let list = app.more.creator_comments.clone();
    if list.is_empty() {
        empty(ui, "还没有收到评论");
        return;
    }
    let mut del: Option<u64> = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("cc-scroll")
        .show(ui, |ui| {
            for c in &list {
                let name = if c.username.is_empty() {
                    "匿名".to_owned()
                } else {
                    c.username.clone()
                };
                let sub = format!("{} · 赞 {} · {}", name, c.like_count, c.created_at);
                if row(ui, app, &c.content, &sub, "删除", "") {
                    del = Some(c.id);
                }
            }
        });
    if let Some(id) = del {
        // 自己视频下的评论：UP 主权限，后端校验
        app.more_action(move |c| c.delete_comment(id));
    }
}

fn creator_danmakus(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "弹幕管理", "cd-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let list = app.more.danmakus.clone();
    if list.is_empty() {
        empty(ui, "还没有收到弹幕");
        return;
    }
    let mut del: Option<u64> = None;
    for d in &list {
        let sub = format!(
            "{} · {} · {}:{:02}",
            d.video_title,
            d.username,
            (d.video_time as u64) / 60,
            (d.video_time as u64) % 60
        );
        if row(ui, app, &d.content, &sub, "删除", "") {
            del = Some(d.id);
        }
    }
    if let Some(id) = del {
        app.more_action(move |c| c.delete_danmaku(id));
    }
}

// ───────────────────────── 动态 ─────────────────────────

fn dynamic_detail(ui: &mut egui::Ui, app: &mut App, id: u64) {
    if back(ui, app, "动态", "dy-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let Some(d) = app.more.dynamic.clone() else {
        empty(ui, "动态已被删除");
        return;
    };
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(&d.author_name)
                .color(theme::TEXT2)
                .size(12.5),
        );
        ui.label(
            egui::RichText::new(&d.created_at)
                .color(theme::TEXT3)
                .size(10.0),
        );
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(egui::RichText::new(&d.content).color(theme::TEXT).size(13.0));
    });
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if pill(ui, &format!("赞 {}", d.like_count), d.liked_by_me) {
            app.more_action(move |c| c.toggle_dynamic_like(id).map(|_| ()));
        }
    });
    section(ui, &format!("评论 {}", d.comment_count));

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("dyn-comments")
        .show(ui, |ui| {
            for c in app.more.dynamic_comments.clone() {
                let name = if c.username.is_empty() {
                    "匿名".to_owned()
                } else {
                    c.username.clone()
                };
                let sub = format!("{} · 赞 {}", c.created_at, c.like_count);
                row(ui, app, &format!("{name}：{}", c.content), &sub, "", "");
            }
            ui.add_space(6.0);
        });

    text_field(ui, app, F::Text, "说点什么…");
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        if pill(ui, "发表", false) && !app.f_text.trim().is_empty() {
            let text = app.f_text.trim().to_owned();
            app.f_text.clear();
            app.more_action(move |c| c.post_dynamic_comment(id, &text, 0));
        }
    });
}

// ───────────────────────── 专题 / 账本 / 治理 / 直播 ─────────────────────────

fn specials(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "专题", "sp-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let list = app.more.specials.clone();
    if list.is_empty() {
        empty(ui, "运营还没有配置专题");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("sp-scroll")
        .show(ui, |ui| {
            for s in &list {
                let sub = if s.subtitle.is_empty() {
                    format!("{} 个视频", s.video_count)
                } else {
                    format!("{} · {} 个视频", s.subtitle, s.video_count)
                };
                let url = s.cover_url.clone();
                row(ui, app, &s.title, &sub, "", &url);
            }
            ui.add_space(12.0);
        });
}

fn ledger(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "硬币与任务", "ld-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    if let Some(r) = app.more.rewards.clone() {
        section(ui, "今日任务");
        let items = [
            ("登录", &r.login),
            ("看视频", &r.watch),
            ("投币", &r.coin),
            ("分享", &r.share),
        ];
        for (label, t) in items {
            let sub = if t.max > 0 {
                format!("{}/{} · +{} 经验", t.progress, t.max, t.exp)
            } else {
                format!("+{} 经验", t.exp)
            };
            let right = if t.done { "已完成" } else { "未完成" };
            row(ui, app, label, &sub, right, "");
        }
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            if pill(ui, "标记今日已看视频", false) {
                app.more_action(|c| c.post_daily_watch().map(|_| ()));
            }
        });
        ui.add_space(6.0);
    }
    section(ui, "硬币明细（近 30 天）");
    let Some(page) = app.more.ledger.clone() else {
        empty(ui, "暂无记录");
        return;
    };
    if page.items.is_empty() {
        empty(ui, "这段时间没有硬币变动");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("ledger-scroll")
        .show(ui, |ui| {
            for e in &page.items {
                let sign = if e.amount >= 0 { "+" } else { "" };
                let sub = format!("{} · {}", e.reason, e.created_at);
                row(ui, app, &sub, "", &format!("{sign}{}", e.amount), "");
            }
            ui.add_space(12.0);
        });
}

fn governance(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "帮助与反馈", "gov-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("gov-scroll")
        .show(ui, |ui| {
            section(ui, "提交工单");
            text_field(ui, app, F::Title, "标题");
            text_field(ui, app, F::Text, "问题描述");
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                if pill(ui, "提交", false)
                    && !app.f_title.trim().is_empty()
                    && !app.f_text.trim().is_empty()
                {
                    let (s, d) = (app.f_title.trim().to_owned(), app.f_text.trim().to_owned());
                    app.f_title.clear();
                    app.f_text.clear();
                    app.more_action(move |c| c.post_ticket("other", &s, &d));
                }
                if pill(ui, "转人工客服", false) && !app.f_text.trim().is_empty() {
                    let m = app.f_text.trim().to_owned();
                    app.f_text.clear();
                    app.more_action(move |c| c.post_cs_conversation(&m));
                }
            });
            ui.add_space(6.0);

            section(ui, "我的工单");
            if app.more.tickets.is_empty() {
                empty(ui, "还没有工单");
            } else {
                for t in app.more.tickets.clone() {
                    let sub = format!("{} · {}", t.category, t.created_at);
                    row(ui, app, &t.subject, &sub, &t.status, "");
                }
            }

            section(ui, "我的申诉");
            if app.more.appeals.is_empty() {
                empty(ui, "没有进行中的申诉");
            } else {
                for a in app.more.appeals.clone() {
                    let sub = format!("{} {} · {}", a.target_type, a.target_id, a.created_at);
                    row(ui, app, &a.content, &sub, &a.status, "");
                }
            }

            section(ui, "客服会话");
            if app.more.cs.is_empty() {
                empty(ui, "暂无客服会话");
            } else {
                for c in app.more.cs.clone() {
                    row(ui, app, &c.title, &c.last_message, &c.status, "");
                }
            }
            ui.add_space(16.0);
        });
}

pub fn live_host(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "开播", "lh-back") {
        return;
    }
    if !app.client.logged_in() {
        centered(ui, "登录后才能开播");
        return;
    }
    if gate(ui, app) {
        return;
    }
    let Some(room) = app.more.live.clone() else {
        empty(ui, "还没有直播间，点下面「创建直播间」");
        return;
    };
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(&room.title)
                .color(theme::TEXT)
                .size(15.0)
                .strong(),
        );
    });
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(format!("状态：{} · 观众 {}", room.status, room.viewer_count))
                .color(theme::TEXT3)
                .size(11.0),
        );
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(format!("推流地址：{}", app.client.hls_url(&room.stream_key)))
                .color(theme::TEXT3)
                .size(10.0),
        );
    });
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let id = room.id;
        let live = room.status == "live";
        if pill(ui, if live { "下播" } else { "开始直播" }, live) {
            app.more_action(move |c| {
                if live {
                    c.end_live(id)
                } else {
                    c.start_live(id)
                }
            });
        }
        if pill(ui, "更换推流码", false) {
            app.more_action(move |c| c.regenerate_stream_key(id).map(|_| ()));
        }
    });
}

// ───────────────────────── 稿件编辑 ─────────────────────────

/// 稿件编辑：标题 / 简介 / 标签 / 定时发布 / 章节
///
/// 只对本人投稿开放；后端在 PUT /videos/:id、章节增删时都会再校验归属。
pub fn video_edit(ui: &mut egui::Ui, app: &mut App, id: u64) {
    if back(ui, app, "稿件编辑", "ve-back") {
        return;
    }
    if gate(ui, app) {
        return;
    }
    let Some(v) = app.more.video.clone() else {
        empty(ui, "稿件不存在");
        return;
    };

    // 首次进入时把现有内容填进表单
    if app.f_title.is_empty() {
        app.f_title = v.title.clone();
        app.f_body = v.description.clone();
        app.f_tags = v.tags.join("，");
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("video-edit-scroll")
        .show(ui, |ui| {
            section(ui, "基础信息");
            text_field(ui, app, F::Title, "标题");
            text_field(ui, app, F::Body, "简介");
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.add(
                    egui::TextEdit::singleline(&mut app.f_tags)
                        .hint_text("标签，用逗号分隔")
                        .desired_width(ui.available_width() - 28.0),
                );
            });
            ui.add_space(6.0);
            let (t, d, tags) = (
                app.f_title.trim().to_owned(),
                app.f_body.clone(),
                app.f_tags
                    .split(|c| c == '，' || c == ',')
                    .map(|x| x.trim().to_owned())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>(),
            );
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                if pill(ui, "保存修改", false) && !t.is_empty() {
                    let (t2, d2) = (t.clone(), d.clone());
                    app.more_action(move |c| c.update_my_video(id, &t2, &d2, &tags));
                }
                if pill(ui, "删除稿件", false) {
                    app.more_action(move |c| c.delete_my_video(id));
                }
            });
            ui.add_space(8.0);

            section(ui, "定时发布");
            text_field(ui, app, F::Text, "2026-11-05 20:00");
            let at = app.f_text.trim().to_owned();
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                if pill(ui, "设为定时", false) && !at.is_empty() {
                    let at2 = at.clone();
                    app.more_action(move |c| c.schedule_publish(id, &at2));
                }
                if pill(ui, "取消定时", false) {
                    app.more_action(move |c| c.cancel_schedule(id));
                }
            });
            ui.add_space(8.0);

            section(ui, "章节");
            if app.more.chapters.is_empty() {
                empty(ui, "还没有章节，可以按时间点切分");
            } else {
                for ch in app.more.chapters.clone() {
                    let sub = theme::clock(ch.time_sec.max(0.0) as u64);
                    let w = ui.available_width();
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(w, 40.0), Sense::click());
                    let p = ui.painter();
                    p.text(
                        Pos2::new(rect.left() + 14.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        &ch.title,
                        FontId::proportional(12.5),
                        theme::TEXT,
                    );
                    p.text(
                        Pos2::new(
                            rect.left() + 14.0 + ch.title.chars().count() as f32 * 13.0 + 8.0,
                            rect.center().y,
                        ),
                        Align2::LEFT_CENTER,
                        sub,
                        FontId::proportional(10.0),
                        theme::TEXT3,
                    );
                    p.text(
                        Pos2::new(rect.right() - 14.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        "删除",
                        FontId::proportional(10.5),
                        theme::UP_TAG,
                    );
                    p.line_segment(
                        [
                            Pos2::new(rect.left() + 14.0, rect.bottom()),
                            Pos2::new(rect.right() - 14.0, rect.bottom()),
                        ],
                        Stroke::new(1.0, theme::LINE),
                    );
                    if resp.clicked() {
                        let cid = ch.id;
                        app.more_action(move |c| c.creator_delete_chapter(id, cid));
                    }
                }
            }
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.add(
                    egui::TextEdit::singleline(&mut app.f_schedule)
                        .hint_text("新增章节：开场|30（标题|秒）")
                        .desired_width(ui.available_width() - 28.0),
                );
            });
            ui.add_space(4.0);
            let raw = app.f_schedule.clone();
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                if pill(ui, "添加章节", false) && !raw.trim().is_empty() {
                    let (title, secs) = raw.split_once('|').unwrap_or((raw.as_str(), "0"));
                    let title = title.trim().to_owned();
                    let secs = secs.trim().parse::<f64>().unwrap_or(0.0);
                    app.f_schedule.clear();
                    if !title.is_empty() {
                        app.more_action(move |c| c.creator_add_chapter(id, &title, secs));
                    }
                }
                ui.label(
                    egui::RichText::new("修改后即刻生效")
                        .color(theme::TEXT3)
                        .size(10.0),
                );
            });
            ui.add_space(20.0);
        });
}

// ───────────────────────── 投稿上传 ─────────────────────────

/// 投稿：从 App 私有目录里选一个视频，填标题/简介/分区后上传
///
/// 手机端没有系统文件选择器（NativeActivity 拿不到 onActivityResult），
/// 所以约定一个收件目录：`/sdcard/Android/data/com.cakecake.rust/files/inbox/`，
/// `adb push` 或用系统「文件」App 都能放进去，刷新即可看到。
pub fn upload(ui: &mut egui::Ui, app: &mut App) {
    if back(ui, app, "投稿", "up-back") {
        return;
    }
    if !app.client.logged_in() {
        centered(ui, "登录后才能投稿");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .id_salt("upload-scroll")
        .show(ui, |ui| {
            section(ui, "1. 选择视频");
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                if pill(ui, "刷新文件列表", false) {
                    app.scan_inbox();
                }
                ui.label(
                    egui::RichText::new(App::inbox_dir())
                        .color(theme::TEXT3)
                        .size(9.5),
                );
            });
            ui.add_space(2.0);

            if app.inbox.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new(
                            format!("目录里还没有视频。电脑上执行：\nadb push 你的视频.mp4 {}", App::inbox_dir()),
                        )
                        .color(theme::TEXT3)
                        .size(10.5),
                    );
                });
                ui.add_space(8.0);
            } else {
                let files = app.inbox.clone();
                for (path, size) in files {
                    let selected = app.up_path.as_deref() == Some(path.as_str());
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .map(|x| x.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let mb = format!("{:.1} MB", size as f64 / 1_048_576.0);
                    let w = ui.available_width();
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(w, 44.0), Sense::click());
                    let p = ui.painter();
                    if selected {
                        p.rect_filled(rect.shrink2(Vec2::new(10.0, 2.0)), 6.0, theme::PINK_SOFT);
                    }
                    p.text(
                        Pos2::new(rect.left() + 14.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        if selected { format!("● {name}") } else { format!("○ {name}") },
                        FontId::proportional(12.0),
                        if selected { theme::PINK } else { theme::TEXT },
                    );
                    p.text(
                        Pos2::new(rect.right() - 14.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        mb,
                        FontId::proportional(10.0),
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
                        app.up_path = Some(path.clone());
                    }
                }
            }
            ui.add_space(8.0);

            section(ui, "2. 稿件信息");
            text_field(ui, app, F::Title, "标题");
            text_field(ui, app, F::Body, "简介（可选）");
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(egui::RichText::new("分区").color(theme::TEXT3).size(10.0));
            });
            ui.horizontal_wrapped(|ui| {
                ui.add_space(14.0);
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                // 优先用后端真实分区（有父子），没有就用内置分区表
                let mut zones: Vec<String> = app
                    .home_ops
                    .zones
                    .iter()
                    .map(|z| z.name.clone())
                    .take(12)
                    .collect();
                if zones.is_empty() {
                    zones = crate::zones::ZONES
                        .iter()
                        .map(|z| z.name.to_owned())
                        .collect();
                }
                if app.up_zone.is_empty() {
                    app.up_zone = zones.first().cloned().unwrap_or_default();
                }
                for z in &zones {
                    let on = app.up_zone == *z;
                    if pill(ui, z, on) {
                        app.up_zone = z.clone();
                    }
                }
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(14.0);
                let busy = app.up_state == LoadState::Loading;
                let label = if busy { "上传中…" } else { "开始上传" };
                if ui
                    .add_enabled(
                        !busy,
                        egui::Button::new(
                            egui::RichText::new(label)
                                .color(Color32::WHITE)
                                .size(12.5),
                        )
                        .fill(if busy { theme::TEXT3 } else { theme::PINK })
                        .corner_radius(16.0)
                        .min_size(Vec2::new(120.0, 34.0)),
                    )
                    .clicked()
                {
                    app.do_upload();
                }
            });
            if let Some(msg) = app.up_msg.clone() {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label(egui::RichText::new(msg).color(theme::UP_TAG).size(11.0));
                });
            }
            ui.add_space(20.0);
        });
}
