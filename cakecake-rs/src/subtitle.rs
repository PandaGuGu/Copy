//! 字幕解析：后端存的是 WebVTT / SRT 原文，这里只做「时间轴 ↔ 文本」的切分
//!
//! 两种格式的时间轴长得很像，差别只在毫秒分隔符（VTT 用 `.`、SRT 用 `,`）和文件头，
//! 所以一个解析器就能同时吃 VTT 和 SRT，不需要额外的 crate。

/// 一条字幕
#[derive(Debug, Clone, Default)]
pub struct Cue {
    /// 起始时间（秒）
    pub start: f64,
    /// 结束时间（秒）
    pub end: f64,
    pub text: String,
}

/// 把 VTT / SRT 原文切成 cue 列表；内容为空时返回空列表（UI 显示「无字幕」）
pub fn parse(content: &str) -> Vec<Cue> {
    let mut cues: Vec<Cue> = Vec::new();
    let mut cur: Option<Cue> = None;

    for raw in content.lines() {
        let line = raw.trim_end_matches('\r');
        if let Some((start, end)) = split_timing(line) {
            if let Some(c) = cur.take() {
                if !c.text.is_empty() {
                    cues.push(c);
                }
            }
            cur = Some(Cue {
                start,
                end,
                text: String::new(),
            });
            continue;
        }
        let Some(c) = cur.as_mut() else {
            // 时间轴之前的内容（WEBVTT 头、序号）直接跳过
            continue;
        };
        let text = clean(line);
        if text.is_empty() {
            if !c.text.is_empty() {
                cues.push(Cue {
                    start: c.start,
                    end: c.end,
                    text: std::mem::take(&mut c.text),
                });
            }
            continue;
        }
        if !c.text.is_empty() {
            c.text.push('\n');
        }
        c.text.push_str(&text);
    }
    if let Some(c) = cur {
        if !c.text.is_empty() {
            cues.push(c);
        }
    }
    cues
}

/// 当前时刻该显示哪条字幕（多条重叠时取最先开始的那条）
pub fn at<'a>(cues: &'a [Cue], sec: f64) -> Option<&'a str> {
    cues.iter()
        .find(|c| sec >= c.start && sec <= c.end && !c.text.is_empty())
        .map(|c| c.text.as_str())
}

/// `00:00:01.000 --> 00:00:03.500` / `00:00:01,000 --> 00:00:03,500`
fn split_timing(line: &str) -> Option<(f64, f64)> {
    let (left, right) = line.split_once("-->")?;
    let start = parse_time(left)?;
    // 结束时间后面可能跟着 VTT 的排版参数（align:start 之类），只取第一个词
    let end_tok = right.split_whitespace().next()?;
    let end = parse_time(end_tok)?;
    Some((start, end))
}

/// `HH:MM:SS.mmm` / `MM:SS.mmm` / `SS.mmm` 统一按「冒号分段、逐段乘 60」算
fn parse_time(raw: &str) -> Option<f64> {
    let s = raw.trim().replace(',', ".");
    if s.is_empty() {
        return None;
    }
    let mut total = 0.0f64;
    for part in s.split(':') {
        total = total * 60.0 + part.trim().parse::<f64>().ok()?;
    }
    Some(total)
}

/// 去掉 `<i>` / `{\an8}` 这类排版标记，留下能直接画的纯文本
fn clean(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_tag = false;
    for ch in line.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            _ => out.push(ch),
        }
    }
    out.replace("{\\an8}", "").trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vtt() {
        let src = "WEBVTT\n\n00:00:01.000 --> 00:00:03.500\n第一句\n\n00:00:04.000 --> 00:00:06.000\n第二句\n第二行\n";
        let cues = parse(src);
        assert_eq!(cues.len(), 2);
        assert!((cues[0].start - 1.0).abs() < 1e-6);
        assert!((cues[0].end - 3.5).abs() < 1e-6);
        assert_eq!(cues[0].text, "第一句");
        assert_eq!(cues[1].text, "第二句\n第二行");
        assert_eq!(at(&cues, 4.5), Some("第二句\n第二行"));
        assert_eq!(at(&cues, 3.8), None);
    }

    #[test]
    fn parses_srt_with_tags() {
        let src = "1\n00:00:02,000 --> 00:00:04,000\n<i>斜体</i>\n\n";
        let cues = parse(src);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].text, "斜体");
        assert!((cues[0].start - 2.0).abs() < 1e-6);
    }
}
