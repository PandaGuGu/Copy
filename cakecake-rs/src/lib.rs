//! cakecake-rs 库入口
//!
//! 桌面走 `main.rs` 的 `main()`；Android 走这里导出的 `android_main`（NativeActivity）。

pub mod api;
pub mod app;
pub mod images;
pub mod pages;
pub mod theme;
pub mod video;
pub mod widgets;

use eframe::egui;

#[cfg(target_os = "android")]
pub type AndroidAppHandle = android_activity::AndroidApp;

#[cfg(not(target_os = "android"))]
pub type AndroidAppHandle = ();

/// 桌面与安卓共用的启动逻辑
pub fn run(android_app: Option<AndroidAppHandle>) -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([412.0, 880.0])
            .with_min_inner_size([360.0, 600.0])
            .with_title("cakecake · rust"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    #[cfg(target_os = "android")]
    if let Some(app) = android_app {
        options.android_app = Some(app);
    }
    #[cfg(not(target_os = "android"))]
    let _ = android_app;
    eframe::run_native(
        "cakecake-rs",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc)))
        }),
    )
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default());
    video::init_android(&app);
    if let Err(e) = run(Some(app)) {
        eprintln!("android_main failed: {e}");
    }
}
