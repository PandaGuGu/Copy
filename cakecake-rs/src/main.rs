//! 桌面入口（Android 入口在 lib.rs 的 android_main）
//!
//! 跑：
//!   cargo run --release
//!   CAKECAKE_API=mock cargo run --release   # 强制离线示例数据

fn main() -> eframe::Result<()> {
    cakecake_rs::run(None)
}
