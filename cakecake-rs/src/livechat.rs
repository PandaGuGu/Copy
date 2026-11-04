//! 直播间聊天（WebSocket 客户端，手写帧协议）
//!
//! 后端只有 `GET /api/v1/ws/live?room_id=&token=` 一条 WS 通道，
//! 收 `{"content": "..."}` 发消息，推过来的是
//! `{"type":"message"|"system"|"audience"|"user_info", ...}`。
//!
//! 这里不引第三方 WS 库：只用得到「明文 ws:// + 文本帧」，自己握手 + 拆帧更省事，
//! 也不用再为安卓交叉编译多拉一堆依赖。

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum Evt {
    /// 普通聊天消息
    Msg { user: String, content: String },
    /// 系统提示（进入/离开/关注成功…）
    System(String),
    /// 在线人数
    Count(u64),
    /// 服务端认定的自己的昵称（`user_info` 里回传，未登录是「游客」）
    Me(String),
    Closed(String),
}

/// 一条连接：发消息用 `cmd`，收事件用 `evt`
pub struct Conn {
    cmd: Sender<String>,
    evt: Receiver<Evt>,
}

impl Conn {
    pub fn send(&self, content: &str) {
        let _ = self.cmd.send(content.to_owned());
    }

    pub fn drain(&self) -> Vec<Evt> {
        let mut out = Vec::new();
        loop {
            match self.evt.try_recv() {
                Ok(e) => out.push(e),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        out
    }
}

/// 连接直播间；`base` 形如 `http://127.0.0.1:8080`
pub fn connect(base: &str, room_id: u64, token: Option<&str>) -> Conn {
    let (cmd_tx, cmd_rx) = channel::<String>();
    let (evt_tx, evt_rx) = channel::<Evt>();

    let url = ws_url(base, room_id, token);
    std::thread::spawn(move || {
        if let Err(e) = run(&url, cmd_rx, &evt_tx) {
            let _ = evt_tx.send(Evt::Closed(e));
        }
    });

    Conn {
        cmd: cmd_tx,
        evt: evt_rx,
    }
}

fn ws_url(base: &str, room_id: u64, token: Option<&str>) -> String {
    let rest = base
        .trim_end_matches('/')
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    match token {
        Some(t) if !t.is_empty() => format!("ws://{rest}/api/v1/ws/live?room_id={room_id}&token={t}"),
        _ => format!("ws://{rest}/api/v1/ws/live?room_id={room_id}"),
    }
}

fn run(url: &str, cmd_rx: Receiver<String>, evt_tx: &Sender<Evt>) -> Result<(), String> {
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

    let mut stream =
        TcpStream::connect(&host_port).map_err(|e| format!("连接失败: {e}"))?;
    handshake(&mut stream, authority, path)?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(120)));

    let _ = evt_tx.send(Evt::System("已连接直播间".to_owned()));

    // 服务端对「只看不说话」的连接有 120 秒读超时，超时直接掐线，
    // 表现就是过一会儿聊天发不出去。这里每 25 秒发一个 WS ping 保活。
    const PING_INTERVAL: Duration = Duration::from_secs(25);
    let mut last_ping = Instant::now();

    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    loop {
        // 0) 心跳
        if last_ping.elapsed() >= PING_INTERVAL {
            if let Err(e) = send_frame(&mut stream, 0x9, b"hb") {
                return Err(format!("心跳失败: {e}"));
            }
            last_ping = Instant::now();
        }

        // 1) 发送待发消息
        loop {
            match cmd_rx.try_recv() {
                Ok(text) => {
                    if text.trim().is_empty() {
                        continue;
                    }
                    let payload = serde_json::json!({ "content": text }).to_string();
                    if let Err(e) = send_text(&mut stream, payload.as_bytes()) {
                        return Err(format!("发送失败: {e}"));
                    }
                }
                Err(_) => break,
            }
        }

        // 2) 读一包（超时就继续循环，攒够整帧再解析）
        match stream.read(&mut tmp) {
            Ok(0) => return Err("连接已关闭".to_owned()),
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
            Err(e) => return Err(format!("连接中断: {e}")),
        }

        // 3) 拆帧
        while let Some((op, payload)) = parse_frame(&mut buf) {
            match op {
                0x1 => handle_text(&payload, evt_tx),
                0x8 => return Err("服务器关闭了连接".to_owned()),
                0x9 => {
                    // ping → pong（同 payload）
                    let _ = send_frame(&mut stream, 0xA, &payload);
                }
                // 0xA pong（我们心跳的回包）与其余控制帧直接忽略
                _ => {}
            }
        }
    }
}

fn handle_text(payload: &[u8], evt_tx: &Sender<Evt>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(payload) else {
        return;
    };
    match v.get("type").and_then(|t| t.as_str()).unwrap_or("") {
        "message" => {
            let user = v.get("username").and_then(|u| u.as_str()).unwrap_or("游客");
            let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
            if !content.is_empty() {
                let _ = evt_tx.send(Evt::Msg {
                    user: user.to_owned(),
                    content: content.to_owned(),
                });
            }
        }
        "system" => {
            if let Some(msg) = v.get("msg").and_then(|m| m.as_str()) {
                if !msg.is_empty() {
                    let _ = evt_tx.send(Evt::System(msg.to_owned()));
                }
            }
        }
        "audience" => {
            if let Some(n) = v.get("count").and_then(|c| c.as_u64()) {
                let _ = evt_tx.send(Evt::Count(n));
            }
        }
        "user_info" => {
            if let Some(name) = v.get("username").and_then(|c| c.as_str()) {
                if !name.is_empty() {
                    let _ = evt_tx.send(Evt::Me(name.to_owned()));
                }
            }
            if let Some(n) = v.get("user_count").and_then(|c| c.as_u64()) {
                let _ = evt_tx.send(Evt::Count(n));
            }
        }
        _ => {}
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

/// 客户端发出的帧必须带掩码
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

/// 返回 (opcode, payload)；数据不够一帧时返回 None（余量留在 buf 里）
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
        // xorshift64*
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
