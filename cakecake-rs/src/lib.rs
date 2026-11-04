//! cakecake-rs 库入口
//!
//! 桌面走 `main.rs` 的 `main()`；Android 走这里导出的 `android_main`（NativeActivity）。

pub mod api;
pub mod app;
pub mod images;
pub mod livechat;
pub mod pages;
pub mod subtitle;
pub mod theme;
pub mod video;
pub mod widgets;
pub mod wsclient;
pub mod zones;

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

    // 同一进程里第二次进来（系统多开了一个 Activity 实例，或者上一轮没退干净）：
    // winit 的 EventLoop 只能建一次，这里与其报 "EventLoop can't be recreated"
    // 留个黑屏，不如把多出来的这个实例关掉，用户看到的还是原来那个界面。
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        eprintln!("[app] duplicate activity, finishing it");
        finish_activity(&app);
        return;
    }

    if let Err(e) = run(Some(app)) {
        eprintln!("android_main failed: {e}");
    }
}

/// 调 Activity.finish()，只用于关掉重复实例
#[cfg(target_os = "android")]
fn finish_activity(app: &android_activity::AndroidApp) {
    use jni::objects::JObject;
    use jni::JavaVM;
    unsafe {
        let Ok(vm) = JavaVM::from_raw(app.vm_as_ptr().cast()) else {
            return;
        };
        // attach_current_thread 的 guard 借用 JavaVM，而这里是一次性调用，
        // 直接把它泄漏成 'static 最省事（进程级单例，本来也不该释放）
        let vm: &'static JavaVM = Box::leak(Box::new(vm));
        if let Ok(mut env) = vm.attach_current_thread() {
            let act = JObject::from_raw(app.activity_as_ptr() as jni::sys::jobject);
            let _ = env.call_method(&act, "finish", "()V", &[]);
            let _ = env.exception_clear();
        }
    }
}
