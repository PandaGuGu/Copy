//! B 站风格浅色主题

use eframe::egui::{self, Color32};

pub const BG: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SURFACE: Color32 = Color32::from_rgb(0xF1, 0xF2, 0xF3);
pub const SURFACE2: Color32 = Color32::from_rgb(0xF6, 0xF7, 0xF8);
pub const LINE: Color32 = Color32::from_rgb(0xE3, 0xE5, 0xE7);
pub const TEXT: Color32 = Color32::from_rgb(0x18, 0x19, 0x1C);
pub const TEXT2: Color32 = Color32::from_rgb(0x61, 0x66, 0x6D);
pub const TEXT3: Color32 = Color32::from_rgb(0x94, 0x99, 0xA0);
pub const PINK: Color32 = Color32::from_rgb(0xFB, 0x72, 0x99);
pub const PINK_SOFT: Color32 = Color32::from_rgb(0xFF, 0xEC, 0xF1);
pub const BLUE: Color32 = Color32::from_rgb(0x00, 0xAE, 0xEC);
pub const ORANGE: Color32 = Color32::from_rgb(0xFF, 0x92, 0x00);
pub const UP_TAG: Color32 = Color32::from_rgb(0xFF, 0x66, 0x99);

fn apply_font(ctx: &egui::Context, bytes: Vec<u8>) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "cjk".to_owned(),
        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
    );
    if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        list.insert(0, "cjk".to_owned());
    }
    if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        list.push("cjk".to_owned());
    }
    ctx.set_fonts(fonts);
}

/// 两色线性混合
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
    )
}

pub fn install(ctx: &egui::Context) {
    let mut done = false;
    for path in [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf",
        "C:/Windows/Fonts/Deng.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            apply_font(ctx, bytes);
            done = true;
            break;
        }
    }

    // 安卓没有 C:/Windows/Fonts，用随包编译进去的字体
    #[cfg(target_os = "android")]
    if !done {
        apply_font(ctx, include_bytes!("../assets/fonts/cjk.ttf").to_vec());
    }
    let _ = done;

    ctx.all_styles_mut(|style| {
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = BG;
        style.visuals.extreme_bg_color = SURFACE;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.widgets.noninteractive.bg_fill = SURFACE;
        style.visuals.widgets.inactive.bg_fill = SURFACE;
        style.visuals.widgets.hovered.bg_fill = SURFACE2;
        style.visuals.widgets.active.bg_fill = LINE;
        style.visuals.selection.bg_fill = PINK.gamma_multiply(0.3);
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
    });
}

/// 封面缺图时的稳定取色
pub fn cover_color(seed: u64) -> Color32 {
    const SET: [Color32; 8] = [
        Color32::from_rgb(0x3A, 0x4A, 0x6E),
        Color32::from_rgb(0x60, 0x3E, 0x5C),
        Color32::from_rgb(0x34, 0x5C, 0x58),
        Color32::from_rgb(0x68, 0x4E, 0x38),
        Color32::from_rgb(0x46, 0x42, 0x6C),
        Color32::from_rgb(0x5E, 0x3A, 0x42),
        Color32::from_rgb(0x36, 0x56, 0x6C),
        Color32::from_rgb(0x54, 0x58, 0x3A),
    ];
    SET[(seed as usize) % SET.len()]
}

/// 12.3万 / 4.5万 / 999
pub fn human(n: u64) -> String {
    if n >= 100_000_000 {
        format!("{:.1}亿", n as f64 / 100_000_000.0)
    } else if n >= 10_000 {
        format!("{:.1}万", n as f64 / 10_000.0)
    } else {
        n.to_string()
    }
}

pub fn clock(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}
