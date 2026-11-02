//! egui 弹幕渲染压力测试
//!
//! 目的：验证 "egui 当壳 + 自绘内容层" 能不能扛住真实弹幕密度。
//! 对照组 = cakecake 现在的 WebView 版（uni-app + DOM/canvas）。
//!
//! 跑：cargo run --release
//! 自动计时跑（用于取数据）：SPIKE_SECONDS=10 SPIKE_COUNT=2000 cargo run --release

use eframe::egui;
use std::time::Instant;

const PALETTE: [egui::Color32; 6] = [
    egui::Color32::from_rgb(255, 255, 255),
    egui::Color32::from_rgb(254, 132, 148),
    egui::Color32::from_rgb(126, 197, 255),
    egui::Color32::from_rgb(255, 214, 102),
    egui::Color32::from_rgb(160, 240, 180),
    egui::Color32::from_rgb(210, 170, 255),
];

const TEXTS: [&str; 10] = [
    "前方高能预警",
    "这个转场是真的丝滑",
    "打卡第三遍",
    "1.5 倍速才是正确打开方式",
    "UP 主快更新",
    "笑死，根本停不下来",
    "这段配乐绝了",
    "第一次看到这种运镜",
    "泪目，爷青结",
    "弹幕护体",
];

struct Item {
    x: f32,
    y: f32,
    vx: f32,
    text: String,
    size: f32,
    color: egui::Color32,
}

/// 不引 rand，省一个依赖
struct Rng(u32);
impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / 16_777_216.0
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
    fn pick<T: Copy>(&mut self, arr: &[T]) -> T {
        arr[(self.next_f32() * arr.len() as f32) as usize % arr.len()]
    }
}

struct App {
    items: Vec<Item>,
    target_count: usize,
    rng: Rng,
    last: Instant,
    started: Instant,
    auto_stop: Option<f32>,
    frames: u64,
    frame_ms: Vec<f32>,
    worst_ms: f32,
    paused: bool,
    font_loaded: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // egui 默认字体不含 CJK，中文会变成方块，必须自己塞
        let mut font_loaded = "none".to_string();
        for path in [
            "C:/Windows/Fonts/simhei.ttf",
            "C:/Windows/Fonts/Deng.ttf",
            "C:/Windows/Fonts/msyh.ttc",
        ] {
            if let Ok(bytes) = std::fs::read(path) {
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
                cc.egui_ctx.set_fonts(fonts);
                font_loaded = path.to_string();
                break;
            }
        }

        let target_count = std::env::var("SPIKE_COUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2000usize);
        let auto_stop = std::env::var("SPIKE_SECONDS")
            .ok()
            .and_then(|s| s.parse::<f32>().ok());

        let now = Instant::now();
        let mut app = Self {
            items: Vec::new(),
            target_count,
            rng: Rng(0x9E37_79B9),
            last: now,
            started: now,
            auto_stop,
            frames: 0,
            frame_ms: Vec::with_capacity(240),
            worst_ms: 0.0,
            paused: false,
            font_loaded,
        };
        app.respawn();
        app
    }

    fn respawn(&mut self) {
        self.items.clear();
        self.items.reserve(self.target_count);
        for _ in 0..self.target_count {
            let x = self.rng.range(-600.0, 1200.0);
            let y = self.rng.range(40.0, 620.0);
            let vx = -self.rng.range(90.0, 260.0);
            // 真实弹幕字号是离散档位，连续随机字号会让字形图集爆炸
            let size = self.rng.pick(&[16.0f32, 18.0, 20.0, 22.0, 24.0, 26.0, 30.0]);
            let color = self.rng.pick(&PALETTE);
            let text = self.rng.pick(&TEXTS).to_string();
            self.items.push(Item { x, y, vx, text, size, color });
        }
    }

    fn stats(&self) -> (f32, f32, f32) {
        if self.frame_ms.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let mut v = self.frame_ms.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = v.iter().sum::<f32>() / v.len() as f32;
        let p95 = v[(v.len() as f32 * 0.95) as usize % v.len()];
        (avg, p95, v[v.len() - 1])
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = self.last.elapsed().as_secs_f32();
        self.last = Instant::now();
        self.frames += 1;

        if dt > 0.0 && dt < 1.0 {
            if self.frame_ms.len() >= 240 {
                self.frame_ms.remove(0);
            }
            self.frame_ms.push(dt * 1000.0);
            self.worst_ms = self.worst_ms.max(dt * 1000.0);
        }

        egui::Panel::top("hud").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("egui 弹幕压力测试");
                ui.separator();
                ui.label(format!("同屏 {}", self.items.len()));
                if ui.button("重置").clicked() {
                    self.respawn();
                }
                if ui.button(if self.paused { "继续" } else { "暂停" }).clicked() {
                    self.paused = !self.paused;
                }
                ui.separator();
                let (avg, p95, worst) = self.stats();
                let color = if p95 <= 16.7 {
                    egui::Color32::from_rgb(120, 230, 150)
                } else if p95 <= 33.0 {
                    egui::Color32::from_rgb(255, 210, 100)
                } else {
                    egui::Color32::from_rgb(255, 120, 120)
                };
                ui.colored_label(
                    color,
                    format!("avg {:.2}ms  p95 {:.2}ms  worst {:.2}ms", avg, p95, worst),
                );
                ui.label(format!("≈{:.0} fps", 1000.0 / avg.max(0.01)));
            });
            ui.horizontal(|ui| {
                let mut c = self.target_count;
                if ui
                    .add(egui::Slider::new(&mut c, 0..=8000).text("目标条数"))
                    .changed()
                {
                    self.target_count = c;
                    self.respawn();
                }
                ui.label(format!("字体: {}", self.font_loaded));
                ui.label(format!("累计最差 {:.2}ms", self.worst_ms));
            });
        });

        egui::CentralPanel::no_frame()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 8, 12)))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                if !self.paused {
                    for it in &mut self.items {
                        it.x += it.vx * dt;
                        if it.x < rect.left() - 400.0 {
                            it.x = rect.right() + self.rng.range(0.0, 900.0);
                            it.y = rect.top() + self.rng.range(8.0, rect.height() - 40.0);
                        }
                    }
                }
                let painter = ui.painter();
                for it in &self.items {
                    painter.text(
                        egui::pos2(it.x, rect.top() + it.y),
                        egui::Align2::LEFT_TOP,
                        &it.text,
                        egui::FontId::proportional(it.size),
                        it.color,
                    );
                }
            });

        // 每 30 帧打一次，`SPIKE_SECONDS` 到点自动收尾
        if self.frames % 30 == 0 {
            let (avg, p95, worst) = self.stats();
            eprintln!(
                "[spike] frame={} n={} avg={:.2}ms p95={:.2}ms worst={:.2}ms fps={:.1}",
                self.frames,
                self.items.len(),
                avg,
                p95,
                worst,
                1000.0 / avg.max(0.01)
            );
        }
        if let Some(limit) = self.auto_stop {
            if self.started.elapsed().as_secs_f32() >= limit {
                let (avg, p95, worst) = self.stats();
                eprintln!(
                    "[spike] FINAL n={} avg={:.2}ms p95={:.2}ms worst={:.2}ms fps={:.1} font={}",
                    self.items.len(),
                    avg,
                    p95,
                    worst,
                    1000.0 / avg.max(0.01),
                    self.font_loaded
                );
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        ui.ctx().request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 660.0])
            .with_title("egui danmaku spike"),
        ..Default::default()
    };
    eframe::run_native(
        "egui-danmaku-spike",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
