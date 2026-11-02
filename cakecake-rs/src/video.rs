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

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Connecting,
    Playing,
    Failed,
}

pub struct Player {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    url: Option<String>,
    phase: Phase,
    message: String,
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
                #[cfg(target_os = "android")]
                native: Native::default(),
            })),
        }
    }

    /// 开始播放某个地址；同一个地址重复调用是空操作。
    pub fn play(&mut self, url: &str) {
        let Ok(mut st) = self.inner.lock() else { return };
        if st.url.as_deref() == Some(url) && st.phase != Phase::Failed {
            return;
        }
        st.url = Some(url.to_owned());
        st.phase = Phase::Connecting;
        st.message = "正在连接直播流…".to_owned();
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

        // 状态文案盖在视频上
        let (phase, msg) = {
            match self.inner.lock() {
                Ok(s) => (s.phase, s.message.clone()),
                Err(_) => (Phase::Idle, String::new()),
            }
        };
        match phase {
            Phase::Playing | Phase::Idle => {}
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
        resp
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
    let px = android::nav_bar_px() as f32;
    if pixels_per_point > 0.0 {
        px / pixels_per_point
    } else {
        0.0
    }
}

/// 系统状态栏占用的高度（逻辑点）。非 Android 恒为 0。
#[cfg(target_os = "android")]
pub fn status_bar_points(pixels_per_point: f32) -> f32 {
    let px = android::status_bar_px() as f32;
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
    v_uv = vec2(a_pos.x, 1.0 - a_pos.y);
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
                } else {
                    android::release_mp(&mp);
                }
            }
            Err(e) => {
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
