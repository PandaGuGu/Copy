//! 直播播放层
//!
//! 链路（Android）：
//!   SRS 的 HLS(`.m3u8`) → 系统 MediaPlayer（硬解，走手机 SoC 的解码器）
//!   → SurfaceTexture（OES 外部纹理）→ egui 的 glow 绘制回调里用自己的 shader 画进页面矩形
//!
//! 为什么不让 egui 直接显示成 `Image`：SurfaceTexture 产出的是 `GL_TEXTURE_EXTERNAL_OES`，
//! egui 的贴图只吃普通 2D 纹理，中间得多做一次 FBO 拷贝；直接在回调里画省掉那一趟。
//!
//! 桌面端没有这套东西，`ui()` 只画占位，不影响 `cargo run`。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Connecting,
    Playing,
    Paused,
    Failed,
}

pub struct Player {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    url: Option<String>,
    phase: Phase,
    message: String,
    /// 播放倍速（1.0 = 原速）；换视频后仍然生效
    speed: f32,
    /// 进入 Connecting 的时刻；超过阈值还没出画就报错，别让用户对着「缓冲中…」干等
    started_at: Instant,
    /// 上一次的播放位置 + 该位置变化的时刻，用来判断画面是不是卡住了
    last_pos_ms: i64,
    last_progress_at: Instant,
    /// 看门狗的检查节拍（跨 JNI 调用不用每帧做）
    checked_at: Instant,
    /// 是不是直播流（直播位置不前进 = 卡了；点播播完位置不前进是正常）
    live: bool,
    #[cfg(target_os = "android")]
    native: Native,
}

impl Player {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                url: None,
                phase: Phase::Idle,
                message: String::new(),
                speed: 1.0,
                started_at: Instant::now(),
                last_pos_ms: 0,
                last_progress_at: Instant::now(),
                checked_at: Instant::now(),
                live: false,
                #[cfg(target_os = "android")]
                native: Native::default(),
            })),
        }
    }

    /// 开始播放某个地址；同一个地址重复调用是空操作。
    pub fn play(&mut self, url: &str) {
        self.play_with(url, false);
    }

    /// 播放直播流。与点播的区别只有一个：直播没有「播完」，位置不前进就是卡了，
    /// 需要看门狗兜住；点播播到结尾位置同样不前进，不能报警。
    pub fn play_live(&mut self, url: &str) {
        self.play_with(url, true);
    }

    fn play_with(&mut self, url: &str, live: bool) {
        let Ok(mut st) = self.inner.lock() else { return };
        if st.url.as_deref() == Some(url) && st.phase != Phase::Failed {
            st.live = live;
            return;
        }
        // 失败后重播：先把旧的解码器拆掉，draw() 里会重新建会话
        #[cfg(target_os = "android")]
        if st.phase == Phase::Failed {
            if let Some(s) = st.native.session.take() {
                release_session(s);
            }
        }
        st.url = Some(url.to_owned());
        st.live = live;
        st.phase = Phase::Connecting;
        st.message = String::new();
        st.started_at = Instant::now();
        st.last_pos_ms = 0;
        st.last_progress_at = Instant::now();
        st.checked_at = Instant::now();
    }

    /// 播放看门狗：连接超时 / 画面卡住都变成明确的失败态，
    /// 而不是无限「缓冲中…」（真机上最常见的表现就是一直转圈）。
    #[cfg(target_os = "android")]
    fn tick(&self) {
        let Ok(mut st) = self.inner.lock() else { return };
        match st.phase {
            Phase::Connecting if st.started_at.elapsed() > Duration::from_secs(15) => {
                let url = st.url.clone().unwrap_or_default();
                st.phase = Phase::Failed;
                st.message = format!(
                    "连接超时（15 秒没有画面）\n{url}\n检查电脑 IP / 防火墙，或主播是否已开播"
                );
            }
            // 直播才判「卡住」：点播停在最后一帧是正常的播完，不能算失败
            Phase::Playing if st.live => {
                // 每秒查一次就够了，别每帧都跨 JNI 调 getCurrentPosition
                if st.checked_at.elapsed() < Duration::from_secs(1) {
                    return;
                }
                st.checked_at = Instant::now();
                let pos = st
                    .native
                    .session
                    .as_ref()
                    .and_then(|s| s.mp.as_ref())
                    .map(android::mp_position)
                    .unwrap_or(0);
                if pos > st.last_pos_ms {
                    st.last_pos_ms = pos;
                    st.last_progress_at = Instant::now();
                } else if st.last_progress_at.elapsed() > Duration::from_secs(12) {
                    let url = st.url.clone().unwrap_or_default();
                    st.phase = Phase::Failed;
                    st.message = format!("画面卡住了（12 秒没有新数据），点画面重连\n{url}");
                }
            }
            _ => {}
        }
    }

    #[cfg(not(target_os = "android"))]
    fn tick(&self) {}

    /// 播放进度（毫秒）；桌面端恒为 0
    pub fn position_ms(&self) -> i64 {
        #[cfg(target_os = "android")]
        {
            let Ok(st) = self.inner.lock() else { return 0 };
            if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                return android::mp_position(mp);
            }
            0
        }
        #[cfg(not(target_os = "android"))]
        {
            0
        }
    }

    /// 总时长（毫秒）；未 prepare 完成为 0
    pub fn duration_ms(&self) -> i64 {
        #[cfg(target_os = "android")]
        {
            let Ok(st) = self.inner.lock() else { return 0 };
            if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                return android::mp_duration(mp);
            }
            0
        }
        #[cfg(not(target_os = "android"))]
        {
            0
        }
    }

    /// 暂停 / 继续
    pub fn toggle_pause(&mut self) {
        let Ok(mut st) = self.inner.lock() else { return };
        match st.phase {
            Phase::Playing => {
                st.phase = Phase::Paused;
                #[cfg(target_os = "android")]
                if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                    android::mp_pause(mp);
                }
            }
            Phase::Paused => {
                st.phase = Phase::Playing;
                #[cfg(target_os = "android")]
                if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                    // setPlaybackParams 在暂停态会顺带恢复播放，所以倍速统一放在恢复之后应用
                    android::mp_resume(mp);
                    if (st.speed - 1.0).abs() > 0.001 {
                        android::mp_speed(mp, st.speed);
                    }
                }
            }
            _ => {}
        }
    }

    /// 设置倍速（1.0 = 原速）。播放中立即生效，暂停中下次恢复时生效。
    pub fn set_speed(&mut self, speed: f32) {
        let speed = speed.clamp(0.25, 3.0);
        let Ok(mut st) = self.inner.lock() else { return };
        st.speed = speed;
        #[cfg(target_os = "android")]
        if st.phase == Phase::Playing {
            if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                android::mp_speed(mp, speed);
            }
        }
    }

    pub fn speed(&self) -> f32 {
        self.inner.lock().map(|s| s.speed).unwrap_or(1.0)
    }

    /// 拖动进度
    pub fn seek_to(&self, ms: i64) {
        #[cfg(target_os = "android")]
        {
            let Ok(st) = self.inner.lock() else { return };
            if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                android::mp_seek(mp, ms);
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = ms;
        }
    }

    /// 停止并释放解码器
    pub fn stop(&mut self) {
        let Ok(mut st) = self.inner.lock() else { return };
        st.url = None;
        st.phase = Phase::Idle;
        st.message.clear();
        // 释放 MediaPlayer / Surface（不需要 GL 上下文，直接在这里做，
        // 否则离开房间后解码器会继续在后台播、声音不断）
        #[cfg(target_os = "android")]
        if let Some(s) = st.native.session.take() {
            release_session(s);
        }
    }

    pub fn phase(&self) -> Phase {
        self.inner.lock().map(|s| s.phase).unwrap_or(Phase::Idle)
    }

    pub fn message(&self) -> String {
        self.inner
            .lock()
            .map(|s| s.message.clone())
            .unwrap_or_default()
    }

    /// 在页面上占一块区域逐帧绘制视频
    pub fn ui(&self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        self.ui_in(ui, rect);
        resp
    }

    /// 把视频帧画进一块已经分配好的矩形（详情页要自己叠弹幕/控件）
    pub fn ui_in(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);

        #[cfg(target_os = "android")]
        {
            let shared = self.inner.clone();
            let cb = egui_glow::CallbackFn::new(move |info, painter| {
                draw(&shared, &info, painter.gl());
            });
            ui.painter().add(egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            });
        }
        #[cfg(not(target_os = "android"))]
        let _ = rect;

        // 看门狗：先把「连不上 / 卡住」判出来，再按最新状态画文案
        self.tick();

        // 状态文案盖在视频上
        let Ok(st) = self.inner.lock() else { return };
        let (phase, msg) = (st.phase, st.message.clone());
        drop(st);
        match phase {
            Phase::Playing | Phase::Paused | Phase::Idle => {}
            Phase::Failed => {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("播放失败\n{msg}"),
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(0xE2, 0x4B, 0x4A),
                );
            }
            Phase::Connecting => {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    if msg.is_empty() { "缓冲中…".to_owned() } else { msg },
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_gray(190),
                );
            }
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

/// Android 启动时调用一次，缓存 JavaVM（JNI 调 MediaPlayer 要用）
#[cfg(target_os = "android")]
pub fn init_android(app: &android_activity::AndroidApp) {
    android::init(app);
}

/// 系统导航栏占用的高度（逻辑点）。非 Android 恒为 0。
#[cfg(target_os = "android")]
pub fn nav_bar_points(pixels_per_point: f32) -> f32 {
    // 优先用实际 inset：系统栏被隐藏时就是 0，别再凭空补一条黑边
    let px = android::inset_px("navigationBars", "bottom")
        .unwrap_or_else(android::nav_bar_px) as f32;
    if pixels_per_point > 0.0 {
        px / pixels_per_point
    } else {
        0.0
    }
}

/// 系统状态栏占用的高度（逻辑点）。非 Android 恒为 0。
#[cfg(target_os = "android")]
pub fn status_bar_points(pixels_per_point: f32) -> f32 {
    let px = android::inset_px("statusBars", "top")
        .unwrap_or_else(android::status_bar_px) as f32;
    if pixels_per_point > 0.0 {
        px / pixels_per_point
    } else {
        0.0
    }
}

#[cfg(not(target_os = "android"))]
pub fn status_bar_points(_pixels_per_point: f32) -> f32 {
    0.0
}
#[cfg(not(target_os = "android"))]
pub fn nav_bar_points(_pixels_per_point: f32) -> f32 {
    0.0
}

/// 根页面按返回键：安卓退到后台，桌面忽略
#[cfg(target_os = "android")]
pub fn move_task_to_back() {
    android::move_task_to_back();
}

#[cfg(not(target_os = "android"))]
pub fn move_task_to_back() {}

/// 软键盘占用的高度（逻辑点）。非 Android / 键盘收起时恒为 0。
#[cfg(target_os = "android")]
pub fn ime_points(pixels_per_point: f32) -> f32 {
    let px = android::ime_px() as f32;
    if pixels_per_point > 0.0 {
        px / pixels_per_point
    } else {
        0.0
    }
}

#[cfg(not(target_os = "android"))]
pub fn ime_points(_pixels_per_point: f32) -> f32 {
    0.0
}

// ───────────────────────── Android: JNI + GL 自绘 ─────────────────────────

#[cfg(target_os = "android")]
use egui_glow::glow;

#[cfg(target_os = "android")]
const OES_TARGET: u32 = 0x8D65; // GL_TEXTURE_EXTERNAL_OES

#[cfg(target_os = "android")]
#[derive(Default)]
struct Native {
    gl: Option<Gl>,
    session: Option<Session>,
}

#[cfg(target_os = "android")]
struct Gl {
    tex: glow::Texture,
    program: glow::Program,
    vao: Option<glow::VertexArray>,
    vbo: glow::Buffer,
    a_pos: u32,
    u_tex: Option<UniformLoc>,
    u_mtx: Option<UniformLoc>,
    matrix: [f32; 16],
}

#[cfg(target_os = "android")]
type UniformLoc = <glow::Context as glow::HasContext>::UniformLocation;

#[cfg(target_os = "android")]
struct Session {
    url: String,
    surface: jni::objects::GlobalRef,
    st: jni::objects::GlobalRef,
    mp: Option<jni::objects::GlobalRef>,
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{GlobalRef, JObject, JValue};
    use jni::JavaVM;
    use std::sync::OnceLock;

    static VM: OnceLock<JavaVM> = OnceLock::new();
    static ACTIVITY: OnceLock<GlobalRef> = OnceLock::new();
    static NAV_PX: OnceLock<i32> = OnceLock::new();
    static STATUS_PX: OnceLock<i32> = OnceLock::new();

    /// 在 `android_main` 里调用一次，缓存 JavaVM 和 Activity
    pub fn init(app: &android_activity::AndroidApp) {
        if VM.get().is_none() {
            let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
            if let Ok(vm) = vm {
                let _ = VM.set(vm);
            }
        }
        if ACTIVITY.get().is_none() {
            if let Some(vm) = VM.get() {
                if let Ok(mut env) = vm.attach_current_thread() {
                    let obj = unsafe {
                        JObject::from_raw(app.activity_as_ptr() as jni::sys::jobject)
                    };
                    if let Ok(g) = env.new_global_ref(&obj) {
                        let _ = ACTIVITY.set(g);
                    }
                }
            }
        }
    }

    /// 系统导航栏/手势条高度（px）。App 不是全屏沉浸式，底部会被它压住，
    /// 需要给底部导航留出这段内衬，否则按钮点不到。
    pub fn nav_bar_px() -> i32 {
        bar_px("navigation_bar_height", &NAV_PX)
    }

    /// 系统状态栏高度（px）。同理，顶部内容（返回按钮）会被压住点不到。
    pub fn status_bar_px() -> i32 {
        bar_px("status_bar_height", &STATUS_PX)
    }

    /// 系统栏当前**实际**占用的尺寸（px）。`kind` 是
    /// `WindowInsets$Type` 上的静态方法名（statusBars / navigationBars），
    /// `field` 取 top / bottom / right。
    ///
    /// 和 `bar_px` 的区别：系统栏被隐藏时这里返回 0，而 dimen 永远是那个数。
    /// 全屏/沉浸式状态残留时，按 dimen 留白就会在顶部/底部多出一条黑边。
    /// 拿不到（系统太旧 / 调用异常）返回 None，调用方再退回 dimen。
    pub fn inset_px(kind: &str, field: &str) -> Option<i32> {
        let (vm, act) = (VM.get()?, ACTIVITY.get()?);
        let mut env = vm.attach_current_thread().ok()?;
        let r = (|| -> Result<i32, jni::errors::Error> {
            let window = env
                .call_method(act.as_obj(), "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;
            let decor = env
                .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
                .l()?;
            let insets = env
                .call_method(
                    &decor,
                    "getRootWindowInsets",
                    "()Landroid/view/WindowInsets;",
                    &[],
                )?
                .l()?;
            if insets.is_null() {
                return Ok(0);
            }
            let type_cls = env.find_class("android/view/WindowInsets$Type")?;
            let ty = env
                .call_static_method(&type_cls, kind, "()I", &[])?
                .i()?;
            let bar = env
                .call_method(
                    &insets,
                    "getInsets",
                    "(I)Landroid/graphics/Insets;",
                    &[JValue::Int(ty)],
                )?
                .l()?;
            if bar.is_null() {
                return Ok(0);
            }
            Ok(env.get_field(&bar, field, "I")?.i()?.max(0))
        })();
        match r {
            Ok(v) => Some(v),
            Err(_) => {
                let _ = env.exception_clear();
                None
            }
        }
    }

    fn bar_px(name: &str, cache: &'static OnceLock<i32>) -> i32 {
        if let Some(v) = cache.get() {
            return *v;
        }
        let mut px = 0;
        if let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) {
            if let Ok(mut env) = vm.attach_current_thread() {
                let r = (|| -> Result<i32, jni::errors::Error> {
                    let res = env
                        .call_method(act.as_obj(), "getResources", "()Landroid/content/res/Resources;", &[])?
                        .l()?;
                    let key = env.new_string(name)?;
                    let dimen = env.new_string("dimen")?;
                    let pkg = env.new_string("android")?;
                    let id = env
                        .call_method(
                            &res,
                            "getIdentifier",
                            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I",
                            &[
                                JValue::Object(&key),
                                JValue::Object(&dimen),
                                JValue::Object(&pkg),
                            ],
                        )?
                        .i()?;
                    if id <= 0 {
                        return Ok(0);
                    }
                    Ok(env
                        .call_method(&res, "getDimensionPixelSize", "(I)I", &[JValue::Int(id)])?
                        .i()?)
                })();
                if let Ok(v) = r {
                    px = v;
                } else {
                    let _ = env.exception_clear();
                }
            }
        }
        let _ = cache.set(px);
        px
    }

    pub fn vm() -> Option<&'static JavaVM> {
        VM.get()
    }

    /// 建 SurfaceTexture + Surface（必须在持有 GL 上下文的线程调用）
    pub fn new_surface(tex: u32, w: i32, h: i32) -> Result<(GlobalRef, GlobalRef), String> {
        let vm = vm().ok_or_else(|| "JavaVM 未初始化".to_owned())?;
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let r = (|| -> Result<(GlobalRef, GlobalRef), jni::errors::Error> {
            let st = env.new_object(
                "android/graphics/SurfaceTexture",
                "(I)V",
                &[JValue::Int(tex as i32)],
            )?;
            let _ = env.call_method(
                &st,
                "setDefaultBufferSize",
                "(II)V",
                &[JValue::Int(w), JValue::Int(h)],
            );
            let surface = env.new_object(
                "android/view/Surface",
                "(Landroid/graphics/SurfaceTexture;)V",
                &[JValue::Object(&st)],
            )?;
            let a = env.new_global_ref(&st)?;
            let b = env.new_global_ref(&surface)?;
            Ok((a, b))
        })();
        match r {
            Ok(v) => Ok(v),
            Err(e) => {
                let _ = env.exception_clear();
                Err(format!("创建 Surface 失败: {e}"))
            }
        }
    }

    /// 建 MediaPlayer 并同步 prepare/start（会阻塞，必须放在工作线程）
    pub fn start(url: &str, surface: &GlobalRef) -> Result<GlobalRef, String> {
        let vm = vm().ok_or_else(|| "JavaVM 未初始化".to_owned())?;
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let r = (|| -> Result<GlobalRef, jni::errors::Error> {
            let mp = env.new_object("android/media/MediaPlayer", "()V", &[])?;
            let js = env.new_string(url)?;
            env.call_method(
                &mp,
                "setDataSource",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&js)],
            )?;
            env.call_method(
                &mp,
                "setSurface",
                "(Landroid/view/Surface;)V",
                &[JValue::Object(surface.as_obj())],
            )?;
            env.call_method(&mp, "prepare", "()V", &[])?;
            env.call_method(&mp, "start", "()V", &[])?;
            env.new_global_ref(&mp)
        })();
        match r {
            Ok(g) => Ok(g),
            Err(e) => {
                let _ = env.exception_clear();
                Err(format!("播放器启动失败: {e}"))
            }
        }
    }

    /// 拉取最新一帧，返回 SurfaceTexture 的纹理坐标变换矩阵
    pub fn update_frame(st: &GlobalRef) -> Option<[f32; 16]> {
        let vm = vm()?;
        let mut env = vm.attach_current_thread().ok()?;
        let _ = env.call_method(st.as_obj(), "updateTexImage", "()V", &[]);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return None;
        }
        let arr = env.new_float_array(16).ok()?;
        if env
            .call_method(
                st.as_obj(),
                "getTransformMatrix",
                "([F)V",
                &[JValue::Object(&arr)],
            )
            .is_err()
        {
            let _ = env.exception_clear();
            return None;
        }
        let mut m = [0f32; 16];
        env.get_float_array_region(&arr, 0, &mut m).ok()?;
        Some(m)
    }

    pub fn release_mp(mp: &GlobalRef) {
        if let Some(vm) = vm() {
            if let Ok(mut env) = vm.attach_current_thread() {
                let _ = env.call_method(mp.as_obj(), "stop", "()V", &[]);
                let _ = env.exception_clear();
                let _ = env.call_method(mp.as_obj(), "release", "()V", &[]);
                let _ = env.exception_clear();
            }
        }
    }

    /// 屏幕被软键盘占掉的高度（px）。API 30 以下拿不到，返回 0。
    ///
    /// Android 15 强制 edge-to-edge 后 `adjustResize` 不再缩窗口，
    /// 键盘是盖在画布上的，所以只能自己查 IME inset 再把内容顶上去。
    /// 直接调系统输入法（eframe 的 ViewportCommand::IMEAllowed 在本机不生效）
    pub fn set_soft_input(show: bool) {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        let r = (|| -> Result<(), jni::errors::Error> {
            let name = env.new_string("input_method")?;
            let imm = env
                .call_method(
                    act.as_obj(),
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&JObject::from(name))],
                )?
                .l()?;
            if imm.is_null() {
                return Ok(());
            }
            let window = env
                .call_method(act.as_obj(), "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;
            let decor = env
                .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
                .l()?;
            if show {
                // NativeActivity 的 decor 不是编辑器视图，showSoftInput 在部分 ROM 上无效，
                // 用 toggleSoftInput(SHOW_FORCED, 0) 直接把输入法拉起来
                env.call_method(&imm, "toggleSoftInput", "(II)V", &[JValue::Int(1), JValue::Int(0)])?;
            } else {
                let token = env
                    .call_method(&decor, "getWindowToken", "()Landroid/os/IBinder;", &[])?
                    .l()?;
                env.call_method(
                    &imm,
                    "hideSoftInputFromWindow",
                    "(Landroid/os/IBinder;I)Z",
                    &[JValue::Object(&token), JValue::Int(0)],
                )?;
            }
            Ok(())
        })();
        if r.is_err() {
            let _ = env.exception_clear();
        }
    }

    pub fn ime_px() -> i32 {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return 0;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return 0;
        };
        let r = (|| -> Result<i32, jni::errors::Error> {
            let window = env
                .call_method(act.as_obj(), "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;
            let decor = env
                .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
                .l()?;

            // 首选 WindowInsets.Type.ime()：竖屏、横屏、全屏（窗口没被缩）都准。
            // 老办法（DisplayMetrics 与可见区域相减）在横屏 edge-to-edge 下会算成 0，
            // 表现就是键盘弹出来了、输入行还压在键盘底下。
            let from_insets = (|| -> Result<i32, jni::errors::Error> {
                let insets = env
                    .call_method(
                        &decor,
                        "getRootWindowInsets",
                        "()Landroid/view/WindowInsets;",
                        &[],
                    )?
                    .l()?;
                if insets.is_null() {
                    return Ok(0);
                }
                let type_cls = env.find_class("android/view/WindowInsets$Type")?;
                let ime_type = env
                    .call_static_method(&type_cls, "ime", "()I", &[])?
                    .i()?;
                let bar = env
                    .call_method(
                        &insets,
                        "getInsets",
                        "(I)Landroid/graphics/Insets;",
                        &[JValue::Int(ime_type)],
                    )?
                    .l()?;
                if bar.is_null() {
                    return Ok(0);
                }
                Ok(env.get_field(&bar, "bottom", "I")?.i()?.max(0))
            })()
            .unwrap_or_else(|_| {
                let _ = env.exception_clear();
                0
            });
            if from_insets > 0 {
                return Ok(from_insets);
            }

            // 真实屏幕高度
            let res = env
                .call_method(act.as_obj(), "getResources", "()Landroid/content/res/Resources;", &[])?
                .l()?;
            let dm = env
                .call_method(&res, "getDisplayMetrics", "()Landroid/util/DisplayMetrics;", &[])?
                .l()?;
            let screen_h = env.get_field(&dm, "heightPixels", "I")?.i()?;
            let decor_h = env
                .call_method(&decor, "getHeight", "()I", &[])?
                .i()?;

            // 窗口自己已经被系统缩小（adjustResize）时，页面天然让位，不要再减一次，
            // 否则会凭空多出一大块空白
            if decor_h < screen_h - 100 {
                return Ok(0);
            }

            // 可见区域底边 = 键盘上沿；差值就是键盘高度。
            // 注意要用窗口(decor)的实际高度而不是 DisplayMetrics：国产 ROM 上
            // DisplayMetrics.heightPixels 会扣掉状态栏/导航栏（本机 1600 → 1470），
            // 直接相减会少算一截，悬浮输入栏就正好被键盘盖住。
            let rect_cls = env.find_class("android/graphics/Rect")?;
            let m_init = env.get_method_id(&rect_cls, "<init>", "()V")?;
            let rect = unsafe { env.new_object_unchecked(&rect_cls, m_init, &[]) }?;
            env.call_method(&decor, "getWindowVisibleDisplayFrame", "(Landroid/graphics/Rect;)V", &[JValue::Object(&rect)])?;
            let bottom = env.get_field(&rect, "bottom", "I")?.i()?;
            let ime = decor_h - bottom;
            Ok(if ime > 0 { ime } else { 0 })
        })();
        match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                0
            }
        }
    }

    /// Java 壳 MainBack.onBackPressed 打过来的标记
#[cfg(target_os = "android")]
static BACK_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// JNI 入口：Java 的 nativeBack() 落到这里
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_cakecake_rust_MainBack_nativeBack(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    BACK_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// 取一次「按了返回键」标记（取完就清）
#[cfg(target_os = "android")]
pub fn take_back_request() -> bool {
    BACK_REQUESTED.swap(false, std::sync::atomic::Ordering::SeqCst)
}

#[cfg(not(target_os = "android"))]
pub fn take_back_request() -> bool {
    false
}

/// 把 App 退到后台（根页面按返回键用）。
    ///
    /// 不能用 ViewportCommand::Close：那会 finish 掉 Activity，但进程还活着，
    /// 下次点图标会在同一进程里重建 Activity → winit 报
    /// "EventLoop can't be recreated"，App 直接起不来。moveTaskToBack 只切后台。
    pub fn move_task_to_back() {
        if let Some(vm) = vm() {
            if let Some(act) = ACTIVITY.get() {
                if let Ok(mut env) = vm.attach_current_thread() {
                    let _ = env.call_method(
                        act.as_obj(),
                        "moveTaskToBack",
                        "(Z)Z",
                        &[JValue::Bool(1)],
                    );
                    let _ = env.exception_clear();
                }
            }
        }
    }

    pub fn release_surface(surface: &GlobalRef, st: &GlobalRef) {
        if let Some(vm) = vm() {
            if let Ok(mut env) = vm.attach_current_thread() {
                let _ = env.call_method(surface.as_obj(), "release", "()V", &[]);
                let _ = env.exception_clear();
                let _ = env.call_method(st.as_obj(), "release", "()V", &[]);
                let _ = env.exception_clear();
            }
        }
    }

    /// 下面几个都是 MediaPlayer 的轻量调用：失败就返回 0 / 空操作，
    /// 播放器状态自己会通过回调走 Failed 分支。
    pub fn mp_position(mp: &GlobalRef) -> i64 {
        call_mp_i(mp, "getCurrentPosition").unwrap_or(0) as i64
    }

    pub fn mp_duration(mp: &GlobalRef) -> i64 {
        call_mp_i(mp, "getDuration").unwrap_or(0) as i64
    }

    fn call_mp_i(mp: &GlobalRef, method: &str) -> Option<i32> {
        let vm = vm()?;
        let mut env = vm.attach_current_thread().ok()?;
        let r = env.call_method(mp.as_obj(), method, "()I", &[]).ok()?.i().ok();
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return None;
        }
        r
    }

    fn call_mp_v(mp: &GlobalRef, method: &str, sig: &str, args: &[jni::objects::JValue]) {
        if let Some(vm) = vm() {
            if let Ok(mut env) = vm.attach_current_thread() {
                let _ = env.call_method(mp.as_obj(), method, sig, args);
                let _ = env.exception_clear();
            }
        }
    }

    pub fn mp_pause(mp: &GlobalRef) {
        call_mp_v(mp, "pause", "()V", &[]);
    }

    pub fn mp_resume(mp: &GlobalRef) {
        call_mp_v(mp, "start", "()V", &[]);
    }

    pub fn mp_seek(mp: &GlobalRef, ms: i64) {
        call_mp_v(mp, "seekTo", "(I)V", &[jni::objects::JValue::Int(ms as i32)]);
    }

    /// 倍速：MediaPlayer.setPlaybackParams(PlaybackParams().setSpeed(s))
    pub fn mp_speed(mp: &GlobalRef, speed: f32) {
        if let Some(vm) = vm() {
            if let Ok(mut env) = vm.attach_current_thread() {
                let r = (|| -> Result<(), jni::errors::Error> {
                    let params = env.new_object("android/media/PlaybackParams", "()V", &[])?;
                    env.call_method(
                        &params,
                        "setSpeed",
                        "(F)Landroid/media/PlaybackParams;",
                        &[JValue::Float(speed)],
                    )?;
                    env.call_method(
                        mp.as_obj(),
                        "setPlaybackParams",
                        "(Landroid/media/PlaybackParams;)V",
                        &[JValue::Object(&params)],
                    )?;
                    Ok(())
                })();
                if r.is_err() {
                    let _ = env.exception_clear();
                }
            }
        }
    }

    /// 横屏 + 沉浸式（隐藏状态栏/导航栏）；退出时还原竖屏和系统栏
    pub fn set_screen_landscape(on: bool) {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        // SCREEN_ORIENTATION_SENSOR_LANDSCAPE = 6 / PORTRAIT = 1
        let orientation = if on { 6 } else { 1 };
        let r = env.call_method(
            act.as_obj(),
            "setRequestedOrientation",
            "(I)V",
            &[JValue::Int(orientation)],
        );
        if r.is_err() {
            let _ = env.exception_clear();
        }
        set_immersive_in(&mut env, act, on);
    }

    /// 系统导航栏当前实际占用的宽度（px）。横屏时它贴在屏幕右侧，
    /// 部分 ROM（本机 vivo/OriginOS）即使 App 请求了沉浸式也仍会把它画出来，
    /// 全屏控件必须按这个宽度让位，否则会被导航栏压住点不到。
    /// 已隐藏 / 取不到时返回 0。
    pub fn nav_inset_right_px() -> i32 {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return 0;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return 0;
        };
        let r = (|| -> Result<i32, jni::errors::Error> {
            let window = env
                .call_method(act.as_obj(), "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;
            let decor = env
                .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
                .l()?;
            let insets = env
                .call_method(
                    &decor,
                    "getRootWindowInsets",
                    "()Landroid/view/WindowInsets;",
                    &[],
                )?
                .l()?;
            if insets.is_null() {
                return Ok(0);
            }
            let type_cls = env.find_class("android/view/WindowInsets$Type")?;
            let nav = env
                .call_static_method(&type_cls, "navigationBars", "()I", &[])?
                .i()?;
            let bar = env
                .call_method(
                    &insets,
                    "getInsets",
                    "(I)Landroid/graphics/Insets;",
                    &[JValue::Int(nav)],
                )?
                .l()?;
            if bar.is_null() {
                return Ok(0);
            }
            Ok(env.get_field(&bar, "right", "I")?.i()?.max(0))
        })();
        match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                0
            }
        }
    }

    /// 只处理系统栏（横竖屏切换后系统会重置窗口属性，需要重新调用一次）
    pub fn set_immersive(on: bool) {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        set_immersive_in(&mut env, act, on);
    }

    fn set_immersive_in(env: &mut jni::JNIEnv, act: &GlobalRef, on: bool) {
        let r = (|| -> Result<(), jni::errors::Error> {
            let window = env
                .call_method(act.as_obj(), "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;
            let decor = env
                .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
                .l()?;
            // FULLSCREEN|HIDE_NAVIGATION|IMMERSIVE_STICKY|LAYOUT_STABLE
            // |LAYOUT_HIDE_NAVIGATION|LAYOUT_FULLSCREEN
            let flags = if on { 0x0406u32 | 0x0200 | 0x0100 | 0x1000 } else { 0 };
            env.call_method(
                &decor,
                "setSystemUiVisibility",
                "(I)V",
                &[JValue::Int(flags as i32)],
            )?;

            // Android 11+（尤其 Android 15 强制 edge-to-edge）上 setSystemUiVisibility
            // 已经压不住状态栏/导航栏了，必须再用 WindowInsetsController 隐藏
            let wic = env
                .call_method(
                    &window,
                    "getInsetsController",
                    "()Landroid/view/WindowInsetsController;",
                    &[],
                )
                .ok()
                .and_then(|v| v.l().ok());
            if let Some(wic) = wic {
                if !wic.is_null() {
                    let type_cls = env.find_class("android/view/WindowInsets$Type")?;
                    let bars = env
                        .call_static_method(&type_cls, "systemBars", "()I", &[])?
                        .i()?;
                    if on {
                        env.call_method(&wic, "hide", "(I)V", &[JValue::Int(bars)])?;
                        // BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE = 2
                        env.call_method(
                            &wic,
                            "setSystemBarsBehavior",
                            "(I)V",
                            &[JValue::Int(2)],
                        )?;
                    } else {
                        env.call_method(&wic, "show", "(I)V", &[JValue::Int(bars)])?;
                    }
                }
            }
            Ok(())
        })();
        if r.is_err() {
            eprintln!("[video] immersive({on}) failed: {r:?}");
            let _ = env.exception_clear();
        }
    }

    /// 拉起原生键盘：真正持有输入连接的 EditText 在 Java 壳（MainBack）里
    /// 多行输入（文章正文）：回车换行，键盘右下角不变成「发送」
    pub fn ime_open_multi(text: &str) {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        let r = (|| -> Result<(), jni::errors::Error> {
            let js = env.new_string(text)?;
            env.call_method(
                act.as_obj(),
                "beginInputMulti",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&js)],
            )?;
            Ok(())
        })();
        if r.is_err() {
            let _ = env.exception_clear();
        }
    }

    pub fn ime_open(initial: &str) {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        let r = (|| -> Result<(), jni::errors::Error> {
            let js = env.new_string(initial)?;
            env.call_method(
                act.as_obj(),
                "beginInput",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&js)],
            )?;
            Ok(())
        })();
        if r.is_err() {
            eprintln!("[ime] beginInput failed: {r:?}");
            let _ = env.exception_clear();
        }
    }

    /// 收起键盘
    pub fn ime_close() {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        if env
            .call_method(act.as_obj(), "endInput", "()V", &[])
            .is_err()
        {
            let _ = env.exception_clear();
        }
    }

    pub fn ime_clear() {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        if env
            .call_method(act.as_obj(), "clearInput", "()V", &[])
            .is_err()
        {
            let _ = env.exception_clear();
        }
    }

    /// 读当前编辑内容（包含输入法的候选拼音）
    pub fn ime_text() -> Option<String> {
        let (vm, act) = (vm()?, ACTIVITY.get()?);
        let mut env = vm.attach_current_thread().ok()?;
        let r = (|| -> Result<String, jni::errors::Error> {
            let s = env
                .call_method(act.as_obj(), "inputText", "()Ljava/lang/String;", &[])?
                .l()?;
            let js: jni::objects::JString = s.into();
            let s = env.get_string(&js)?;
            Ok(s.into())
        })();
        match r {
            Ok(s) => Some(s),
            Err(_) => {
                let _ = env.exception_clear();
                None
            }
        }
    }

    /// 取一次键盘右下角「发送」键的标记（Java 壳里的 OnEditorActionListener 置位）
    pub fn ime_action() -> bool {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return false;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return false;
        };
        let r = (|| -> Result<bool, jni::errors::Error> {
            Ok(env
                .call_method(act.as_obj(), "takeInputAction", "()Z", &[])?
                .z()?)
        })();
        match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                false
            }
        }
    }

    /// 打开系统相册选图
    pub fn pick_image() {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        if env
            .call_method(act.as_obj(), "pickImage", "()V", &[])
            .is_err()
        {
            let _ = env.exception_clear();
        }
    }

    /// 选中视频的时长(ms) / 宽 / 高（投稿页展示用）
    pub fn video_meta(path: &str) -> Option<(i64, i32, i32)> {
        let (vm, act) = (vm()?, ACTIVITY.get()?);
        let mut env = vm.attach_current_thread().ok()?;
        let r = (|| -> Result<String, jni::errors::Error> {
            let js = env.new_string(path)?;
            let out = env
                .call_method(
                    act.as_obj(),
                    "videoMeta",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    &[JValue::Object(&js)],
                )?
                .l()?;
            if out.is_null() {
                return Ok(String::new());
            }
            let js: jni::objects::JString = out.into();
            let s = env.get_string(&js)?;
            Ok(s.into())
        })();
        let text = match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                return None;
            }
        };
        let mut it = text.split(',');
        let dur: i64 = it.next()?.trim().parse().ok()?;
        let w: i32 = it.next()?.trim().parse().ok()?;
        let h: i32 = it.next()?.trim().parse().ok()?;
        if dur <= 0 && w <= 0 {
            return None;
        }
        Some((dur, w, h))
    }

    /// 取视频首帧（RGBA8888，固定 w×h），用来当投稿页缩略图
    pub fn video_frame(path: &str, w: i32, h: i32) -> Option<Vec<u8>> {
        let (vm, act) = (vm()?, ACTIVITY.get()?);
        let mut env = vm.attach_current_thread().ok()?;
        let r = (|| -> Result<Option<Vec<u8>>, jni::errors::Error> {
            let js = env.new_string(path)?;
            let out = env
                .call_method(
                    act.as_obj(),
                    "videoFrameRgba",
                    "(Ljava/lang/String;II)[B",
                    &[JValue::Object(&js), JValue::Int(w), JValue::Int(h)],
                )?
                .l()?;
            if out.is_null() {
                return Ok(None);
            }
            let arr = jni::objects::JByteArray::from(out);
            let raw = env.convert_byte_array(&arr)?;
            Ok(Some(raw.into_iter().map(|b| b as u8).collect()))
        })();
        match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                None
            }
        }
    }

    /// 打开系统文件选择器挑一个视频（投稿用）
    pub fn pick_video() {
        let (Some(vm), Some(act)) = (VM.get(), ACTIVITY.get()) else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        if env
            .call_method(act.as_obj(), "pickVideo", "()V", &[])
            .is_err()
        {
            let _ = env.exception_clear();
        }
    }

    /// 取一次视频选择结果：None=还没选，Some("")=取消，Some(路径)=选好了
    pub fn picked_video() -> Option<String> {
        let (vm, act) = (vm()?, ACTIVITY.get()?);
        let mut env = vm.attach_current_thread().ok()?;
        let r = (|| -> Result<Option<String>, jni::errors::Error> {
            let s = env
                .call_method(
                    act.as_obj(),
                    "takePickedVideo",
                    "()Ljava/lang/String;",
                    &[],
                )?
                .l()?;
            if s.is_null() {
                return Ok(None);
            }
            let js: jni::objects::JString = s.into();
            let s = env.get_string(&js)?;
            Ok(Some(s.into()))
        })();
        match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                None
            }
        }
    }

    /// 取一次相册选择结果：None=还没选，Some("")=取消，Some(路径)=选好了
    pub fn picked_image() -> Option<String> {
        let (vm, act) = (vm()?, ACTIVITY.get()?);
        let mut env = vm.attach_current_thread().ok()?;
        let r = (|| -> Result<Option<String>, jni::errors::Error> {
            let s = env
                .call_method(
                    act.as_obj(),
                    "takePickedImage",
                    "()Ljava/lang/String;",
                    &[],
                )?
                .l()?;
            if s.is_null() {
                return Ok(None);
            }
            let js: jni::objects::JString = s.into();
            let s = env.get_string(&js)?;
            Ok(Some(s.into()))
        })();
        match r {
            Ok(v) => v,
            Err(_) => {
                let _ = env.exception_clear();
                None
            }
        }
    }
}

#[cfg(target_os = "android")]
fn release_session(s: Session) {
    if let Some(mp) = s.mp.as_ref() {
        android::release_mp(mp);
    }
    android::release_surface(&s.surface, &s.st);
}

#[cfg(target_os = "android")]
fn bytes_of(f: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(f.as_ptr() as *const u8, std::mem::size_of_val(f)) }
}

#[cfg(target_os = "android")]
fn create_gl(gl: &glow::Context) -> Result<Gl, String> {
    use glow::HasContext;
    unsafe {
        let tex = gl.create_texture().map_err(|e| e.to_string())?;
        gl.bind_texture(OES_TARGET, Some(tex));
        gl.tex_parameter_i32(OES_TARGET, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(OES_TARGET, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(OES_TARGET, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(OES_TARGET, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        gl.bind_texture(OES_TARGET, None);

        const VS: &str = r#"
attribute vec2 a_pos;
varying vec2 v_uv;
void main() {
    // 纹理坐标不要自己翻转：SurfaceTexture 的变换矩阵已经带上了设备需要的
    // 翻转/裁剪（真机上再翻一次就会上下颠倒）。
    v_uv = a_pos;
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;
        const FS: &str = r#"
#extension GL_OES_EGL_image_external : require
precision mediump float;
uniform samplerExternalOES u_tex;
uniform mat4 u_mtx;
varying vec2 v_uv;
void main() {
    vec2 uv = (u_mtx * vec4(v_uv, 0.0, 1.0)).xy;
    gl_FragColor = texture2D(u_tex, uv);
}
"#;

        let program = gl.create_program().map_err(|e| e.to_string())?;
        let mut shaders = Vec::new();
        for (kind, src) in [(glow::VERTEX_SHADER, VS), (glow::FRAGMENT_SHADER, FS)] {
            let sh = gl.create_shader(kind).map_err(|e| e.to_string())?;
            gl.shader_source(sh, src);
            gl.compile_shader(sh);
            if !gl.get_shader_compile_status(sh) {
                let log = gl.get_shader_info_log(sh);
                return Err(format!("shader 编译失败: {log}"));
            }
            gl.attach_shader(program, sh);
            shaders.push(sh);
        }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            return Err(format!("shader 链接失败: {log}"));
        }
        for sh in shaders {
            gl.detach_shader(program, sh);
            gl.delete_shader(sh);
        }

        let vbo = gl.create_buffer().map_err(|e| e.to_string())?;
        // 必须用自己的 VAO：egui 的 VAO 此刻是绑着的，直接改 attribute 指针会把它改坏，
        // 导致本帧之后所有 egui 图形都画不出来（整页只剩视频）。
        let vao = gl.create_vertex_array().ok();
        if let Some(v) = vao {
            gl.bind_vertex_array(Some(v));
        }
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        // 单位方片：左下 → 右下 → 左上 → 右上
        let quad: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes_of(&quad), glow::STATIC_DRAW);

        let a_pos = gl.get_attrib_location(program, "a_pos").unwrap_or(0);
        gl.enable_vertex_attrib_array(a_pos);
        gl.vertex_attrib_pointer_f32(a_pos, 2, glow::FLOAT, false, 8, 0);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        if vao.is_some() {
            gl.bind_vertex_array(None);
        }

        let u_tex = gl.get_uniform_location(program, "u_tex");
        let u_mtx = gl.get_uniform_location(program, "u_mtx");

        Ok(Gl {
            tex,
            program,
            vao,
            vbo,
            a_pos,
            u_tex,
            u_mtx,
            matrix: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        })
    }
}

#[cfg(target_os = "android")]
fn start_session(shared: &Arc<Mutex<Inner>>, url: &str) -> Result<(), String> {
    let mut st = shared.lock().map_err(|_| "状态锁失败".to_owned())?;
    let tex = match st.native.gl.as_ref() {
        Some(g) => g.tex.0.get(),
        None => return Err("GL 资源未就绪".to_owned()),
    };
    let (st_ref, surface) = android::new_surface(tex, 1280, 720)?;
    st.native.session = Some(Session {
        url: url.to_owned(),
        surface: surface.clone(),
        st: st_ref,
        mp: None,
    });
    drop(st);

    let inner = shared.clone();
    let want = url.to_owned();
    std::thread::spawn(move || {
        let r = android::start(&want, &surface);
        let Ok(mut st) = inner.lock() else { return };
        match r {
            Ok(mp) => {
                let still_current = st
                    .native
                    .session
                    .as_ref()
                    .map(|s| s.url == want)
                    .unwrap_or(false);
                if still_current {
                    if let Some(s) = st.native.session.as_mut() {
                        s.mp = Some(mp);
                    }
                    st.phase = Phase::Playing;
                    st.message.clear();
                    st.last_pos_ms = 0;
                    st.last_progress_at = Instant::now();
                    // 新起的播放器默认是 1.0 倍速，这里把用户选过的倍速补上
                    if (st.speed - 1.0).abs() > 0.001 {
                        if let Some(mp) = st.native.session.as_ref().and_then(|s| s.mp.as_ref()) {
                            android::mp_speed(mp, st.speed);
                        }
                    }
                    eprintln!("[video] playing {want}");
                } else {
                    android::release_mp(&mp);
                }
            }
            Err(e) => {
                eprintln!("[video] failed: {e} ({want})");
                st.phase = Phase::Failed;
                st.message = e;
            }
        }
    });
    Ok(())
}

#[cfg(target_os = "android")]
fn draw(shared: &Arc<Mutex<Inner>>, info: &egui::PaintCallbackInfo, gl: &glow::Context) {
    use glow::HasContext;

    let vp = info.viewport_in_pixels();
    if vp.width_px <= 0 || vp.height_px <= 0 {
        return;
    }

    // 1) 处理停止 / 首次建 GL / 换流，锁在这里释放后再起会话
    let (url, need_new) = {
        let Ok(mut st) = shared.lock() else { return };
        let Some(url) = st.url.clone() else {
            if let Some(s) = st.native.session.take() {
                release_session(s);
            }
            return;
        };
        if st.native.gl.is_none() {
            match create_gl(gl) {
                Ok(g) => st.native.gl = Some(g),
                Err(e) => {
                    st.phase = Phase::Failed;
                    st.message = e;
                    return;
                }
            }
        }
        let need_new = match st.native.session.as_ref() {
            Some(s) => s.url != url,
            None => true,
        };
        if need_new {
            if let Some(s) = st.native.session.take() {
                release_session(s);
            }
        }
        (url, need_new)
    };

    if need_new {
        if let Err(e) = start_session(shared, &url) {
            if let Ok(mut st) = shared.lock() {
                st.phase = Phase::Failed;
                st.message = e;
            }
            return;
        }
    }

    // 2) 取最新帧 + 绘制
    let Ok(mut st) = shared.lock() else { return };
    if let Some(s) = st.native.session.as_ref() {
        if let Some(m) = android::update_frame(&s.st) {
            if let Some(g) = st.native.gl.as_mut() {
                g.matrix = m;
            }
        }
    }
    let Some(g) = st.native.gl.as_ref() else { return };
    unsafe {
        gl.viewport(vp.left_px, vp.from_bottom_px, vp.width_px, vp.height_px);
        gl.disable(glow::SCISSOR_TEST);
        gl.disable(glow::BLEND);
        gl.disable(glow::DEPTH_TEST);
        gl.use_program(Some(g.program));
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(OES_TARGET, Some(g.tex));
        if let Some(loc) = g.u_tex.as_ref() {
            gl.uniform_1_i32(Some(loc), 0);
        }
        if let Some(loc) = g.u_mtx.as_ref() {
            gl.uniform_matrix_4_f32_slice(Some(loc), false, &g.matrix);
        }
        // 绑自己的 VAO（顶点属性已记录在里面），用完解绑，别碰 egui 的 VAO
        if let Some(v) = g.vao.as_ref() {
            gl.bind_vertex_array(Some(*v));
        } else {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(g.vbo));
            gl.enable_vertex_attrib_array(g.a_pos);
            gl.vertex_attrib_pointer_f32(g.a_pos, 2, glow::FLOAT, false, 8, 0);
        }
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        if g.vao.is_some() {
            gl.bind_vertex_array(None);
        } else {
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        // 还原视口，避免影响 egui 后续绘制
        let [sw, sh] = info.screen_size_px;
        gl.viewport(0, 0, sw as i32, sh as i32);
    }
}

/// 顶层导出：安卓走 JNI 标记，桌面恒为 false
#[cfg(target_os = "android")]
pub use android::take_back_request;
#[cfg(not(target_os = "android"))]
pub fn take_back_request() -> bool {
    false
}

#[cfg(target_os = "android")]
pub use android::set_soft_input;
#[cfg(not(target_os = "android"))]
pub fn set_soft_input(_show: bool) {}

/// 横屏沉浸式播放（安卓改 Activity 朝向；桌面仅由 egui 全屏接管）
#[cfg(target_os = "android")]
pub use android::set_screen_landscape;
#[cfg(not(target_os = "android"))]
pub fn set_screen_landscape(_on: bool) {}

/// 单独重复隐藏/恢复系统栏（转屏后窗口属性会被系统重置，需要再调一次）
#[cfg(target_os = "android")]
pub use android::set_immersive;
#[cfg(not(target_os = "android"))]
pub fn set_immersive(_on: bool) {}

/// 系统导航栏实际占用的宽度（逻辑点）。横屏时它贴在右侧，
/// 部分 ROM 即使 App 请求了沉浸式也照画不误，全屏控件要按它让位。
#[cfg(target_os = "android")]
pub fn nav_inset_right_points(pixels_per_point: f32) -> f32 {
    let px = android::nav_inset_right_px() as f32;
    if pixels_per_point > 0.0 {
        px / pixels_per_point
    } else {
        0.0
    }
}
#[cfg(not(target_os = "android"))]
pub fn nav_inset_right_points(_pixels_per_point: f32) -> f32 {
    0.0
}

/// 原生输入框：拉起 / 读内容 / 收起 / 清空 / 键盘发送键
#[cfg(target_os = "android")]
pub use android::{ime_action, ime_clear, ime_close, ime_open, ime_open_multi, ime_text};
#[cfg(not(target_os = "android"))]
pub fn ime_open(_initial: &str) {}
#[cfg(not(target_os = "android"))]
pub fn ime_open_multi(_initial: &str) {}
#[cfg(not(target_os = "android"))]
pub fn ime_close() {}
#[cfg(not(target_os = "android"))]
pub fn ime_clear() {}
#[cfg(not(target_os = "android"))]
pub fn ime_text() -> Option<String> {
    None
}
#[cfg(not(target_os = "android"))]
pub fn ime_action() -> bool {
    false
}

/// 系统相册选图 / 选视频
#[cfg(target_os = "android")]
pub use android::{pick_image, pick_video, picked_image, picked_video, video_frame, video_meta};
#[cfg(not(target_os = "android"))]
pub fn pick_image() {}
#[cfg(not(target_os = "android"))]
pub fn pick_video() {}
#[cfg(not(target_os = "android"))]
pub fn picked_video() -> Option<String> {
    None
}
#[cfg(not(target_os = "android"))]
pub fn video_meta(_path: &str) -> Option<(i64, i32, i32)> {
    None
}
#[cfg(not(target_os = "android"))]
pub fn video_frame(_path: &str, _w: i32, _h: i32) -> Option<Vec<u8>> {
    None
}
#[cfg(not(target_os = "android"))]
pub fn picked_image() -> Option<String> {
    None
}
