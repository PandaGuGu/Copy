//! 封面图缓存：后台线程解码 jpg/png，主线程上传纹理

use eframe::egui::{self, ColorImage, TextureHandle};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};

/// 后端返回 /uploads/xxx.jpg，桌面端映射到仓库 data 目录；
/// 安卓端没有该目录（文件不存在），返回 None，由调用方画占位。
pub fn resolve(url: &str) -> Option<String> {
    if url.is_empty() || url.starts_with("http") {
        return None;
    }
    let dir = std::env::var("CAKECAKE_DATA_DIR").unwrap_or_else(|_| "../data".to_owned());
    let path = format!("{}{}", dir.trim_end_matches('/'), url);
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

pub struct Covers {
    base: String,
    tex: HashMap<String, TextureHandle>,
    requested: HashSet<String>,
    /// 常驻通道：多次 request 共用，避免后一批把前一批的回包丢掉
    tx: Sender<(String, ColorImage)>,
    rx: Receiver<(String, ColorImage)>,
}

impl Covers {
    pub fn new(base: String) -> Self {
        let (tx, rx) = channel();
        Self {
            base,
            tex: HashMap::new(),
            requested: HashSet::new(),
            tx,
            rx,
        }
    }

    /// 本地有文件就读盘，没有就走 HTTP 从后端 /uploads 拉
    fn key_for(&self, url: &str) -> Option<String> {
        if url.is_empty() {
            return None;
        }
        if let Some(p) = resolve(url) {
            return Some(p);
        }
        if url.starts_with("http") {
            return Some(url.to_owned());
        }
        Some(format!("{}{}", self.base.trim_end_matches('/'), url))
    }

    pub fn request(&mut self, urls: &[String]) {
        let mut targets = Vec::new();
        for u in urls {
            if let Some(p) = self.key_for(u) {
                if self.requested.insert(p.clone()) {
                    targets.push(p);
                }
            }
        }
        if targets.is_empty() {
            return;
        }
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            for p in targets {
                let bytes = if p.starts_with("http") {
                    let Ok(resp) = ureq::get(&p).timeout(std::time::Duration::from_secs(8)).call() else {
                        continue;
                    };
                    let mut buf = Vec::new();
                    if resp.into_reader().read_to_end(&mut buf).is_err() {
                        continue;
                    }
                    buf
                } else {
                    match std::fs::read(&p) {
                        Ok(b) => b,
                        Err(_) => continue,
                    }
                };
                let Ok(img) = image::load_from_memory(&bytes) else { continue };
                let rgba = img.to_rgba8();
                let ci = ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                );
                if tx.send((p, ci)).is_err() {
                    break;
                }
            }
        });
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        loop {
            match self.rx.try_recv() {
                Ok((path, img)) => {
                    let h = ctx.load_texture(path.clone(), img, egui::TextureOptions::LINEAR);
                    self.tex.insert(path, h);
                }
                Err(_) => break,
            }
        }
    }

    pub fn get(&self, url: &str) -> Option<&TextureHandle> {
        self.key_for(url).and_then(|p| self.tex.get(&p))
    }
}
