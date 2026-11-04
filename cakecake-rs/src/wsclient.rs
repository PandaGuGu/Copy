//! 通用 WebSocket 接收客户端（弹幕 / 私信推送）
//!
//! 与 `livechat` 同一套手写帧协议，但这里只负责收 JSON 事件：
//!   · 弹幕：`GET /api/v1/ws/danmaku?video_id=&token=`
//!   · 私信：`GET /api/v1/ws/chat?token=`
//!
//! 服务端读超时是 120s，客户端每 30s 发一个空 JSON 帧保活。

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Client {
    rx: Receiver<serde_json::Value>,
    alive: Arc<AtomicBool>,
}

impl Client {
    /// 连接后返回一个只读事件流；连接失败/断开时线程直接结束。
    pub fn connect(url: &str) -> Client {
        let (tx, rx) = channel();
        let alive = Arc::new(AtomicBool::new(true));
        let alive_thread = alive.clone();
        let url = url.to_owned();
        std::thread::spawn(move || {
            let _ = run(&url, &tx);
            alive_thread.store(false, Ordering::SeqCst);
        });
        Client { rx, alive }
    }

    /// 线程是否仍在跑（断开后可据此重连）
    pub fn alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// 取出当前已到达的全部事件
    pub fn drain(&self) -> Vec<serde_json::Value> {
        let mut out = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(v) => out.push(v),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        out
    }
}

fn run(url: &str, tx: &std::sync::mpsc::Sender<serde_json::Value>) -> Result<(), String> {
    let rest = url.strip_prefix("ws://").unwrap_or(url);
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let host_port = if authority.contains(':') {
        authority.to_owned()
    } else {
        format!("{authority}:80")
    };

    let mut stream = TcpStream::connect(&host_port).map_err(|e| format!("连接失败: {e}"))?;
    handshake(&mut stream, authority, path)?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(120)));

    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    let mut last_beat = Instant::now();

    loop {
        if last_beat.elapsed() >= Duration::from_secs(30) {
            last_beat = Instant::now();
            let _ = send_text(&mut stream, b"{}");
        }

        match stream.read(&mut tmp) {
            Ok(0) => return Err("连接已关闭".to_owned()),
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
            Err(e) => return Err(format!("连接中断: {e}")),
        }

        while let Some((op, payload)) = parse_frame(&mut buf) {
            match op {
                0x1 => {
                    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&payload) {
                        let auth_failed = v.get("type").and_then(|t| t.as_str()) == Some("auth_failed");
                        let _ = tx.send(v);
                        if auth_failed {
                            return Err("鉴权失败".to_owned());
                        }
                    }
                }
                0x8 => return Err("服务器关闭了连接".to_owned()),
                0x9 => {
                    let _ = send_frame(&mut stream, 0xA, &payload);
                }
                _ => {}
            }
        }
    }
}

fn handshake(stream: &mut TcpStream, host: &str, path: &str) -> Result<(), String> {
    let key = base64(&random_bytes());
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("握手失败: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

    let mut head = Vec::new();
    let mut b = [0u8; 1];
    loop {
        let n = stream.read(&mut b).map_err(|e| format!("握手失败: {e}"))?;
        if n == 0 {
            return Err("握手被对端关闭".to_owned());
        }
        head.push(b[0]);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
        if head.len() > 8192 {
            return Err("握手响应过长".to_owned());
        }
    }
    let text = String::from_utf8_lossy(&head);
    let first = text.lines().next().unwrap_or("");
    if !first.contains(" 101") {
        return Err(format!("握手被拒绝: {first}"));
    }
    Ok(())
}

fn send_text(stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
    send_frame(stream, 0x1, payload)
}

fn send_frame(stream: &mut TcpStream, opcode: u8, payload: &[u8]) -> std::io::Result<()> {
    let mut frame = Vec::with_capacity(payload.len() + 14);
    frame.push(0x80 | (opcode & 0x0f));
    let len = payload.len();
    if len < 126 {
        frame.push(0x80 | len as u8);
    } else if len <= u16::MAX as usize {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    let mask = random_bytes();
    let mask = [mask[0], mask[1], mask[2], mask[3]];
    frame.extend_from_slice(&mask);
    for (i, b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i % 4]);
    }
    stream.write_all(&frame)?;
    stream.flush()
}

fn parse_frame(buf: &mut Vec<u8>) -> Option<(u8, Vec<u8>)> {
    if buf.len() < 2 {
        return None;
    }
    let op = buf[0] & 0x0f;
    let masked = buf[1] & 0x80 != 0;
    let mut len = (buf[1] & 0x7f) as usize;
    let mut idx = 2;
    if len == 126 {
        if buf.len() < 4 {
            return None;
        }
        len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
        idx = 4;
    } else if len == 127 {
        if buf.len() < 10 {
            return None;
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&buf[2..10]);
        len = u64::from_be_bytes(b) as usize;
        idx = 10;
    }
    let mut mask = [0u8; 4];
    if masked {
        if buf.len() < idx + 4 {
            return None;
        }
        mask.copy_from_slice(&buf[idx..idx + 4]);
        idx += 4;
    }
    if buf.len() < idx + len {
        return None;
    }
    let mut payload = buf[idx..idx + len].to_vec();
    if masked {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
    }
    buf.drain(..idx + len);
    Some((op, payload))
}

fn random_bytes() -> [u8; 16] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15);
    let addr = &now as *const u64 as u64;
    let mut s = now ^ addr.rotate_left(17) ^ 0xA5A5_5A5A_1234_5678;
    let mut out = [0u8; 16];
    for chunk in out.chunks_mut(8) {
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        let v = s.wrapping_mul(0x2545_F491_4F6C_DD1D);
        chunk.copy_from_slice(&v.to_be_bytes()[..chunk.len()]);
    }
    out
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
