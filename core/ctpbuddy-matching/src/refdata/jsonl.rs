use std::fs::File;
use std::io::{BufRead, BufReader};

// ---- JSONL helpers ---------------------------------------------------------
// The core builds with zero external crates (DESIGN §13), and the server's
// `json` module lives in another crate, so this carries a minimal reader for
// the *flat* objects the provider writes: string keys, scalar values, no
// nesting. Numbers may be quoted (`"0.1"`) so a hand-edited or spreadsheet-
// derived file still loads. It is deliberately not a general JSON parser —
// anything richer belongs in the Python provider, which is where the real
// vendor formats get parsed.

/// A parsed row: key -> scalar value. A newtype (not a type alias) so the
/// accessors can be inherent methods.
#[derive(Clone, Debug, Default)]
pub(super) struct JsonRow(Vec<(String, Scalar)>);

#[derive(Clone, Debug)]
enum Scalar {
    Num(f64),
    Bool(bool),
    Str(String),
}

impl JsonRow {
    pub(super) fn num(&self, key: &str) -> Option<f64> {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| match v {
                Scalar::Num(n) => Some(*n),
                Scalar::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                // A quoted number is still a number to us — providers quote
                // values freely, and refusing them would make hand-written
                // and spreadsheet-derived files fail to load.
                Scalar::Str(s) => s.trim().parse::<f64>().ok(),
            })
    }

    pub(super) fn int(&self, key: &str) -> Option<i32> {
        self.num(key).map(|v| v as i32)
    }

    /// A non-empty string value; a numeric value is stringified so a provider
    /// that writes `delivery_year: 2026` still yields `"2026"`.
    pub(super) fn str(&self, key: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| match v {
                Scalar::Str(s) => s.clone(),
                Scalar::Num(n) => format!("{}", *n as i64),
                Scalar::Bool(b) => {
                    if *b {
                        "1".into()
                    } else {
                        "0".into()
                    }
                }
            })
            .filter(|s| !s.is_empty())
    }
}

/// Read a JSONL file into rows. A missing file is an empty table (legal: a
/// desk may have no 申报费), anything malformed is an error naming the line.
pub(super) fn read_jsonl(path: &str) -> Result<Vec<JsonRow>, String> {
    let f = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{path}: {e}")),
    };
    let mut out = Vec::new();
    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line.map_err(|e| format!("{path} line {}: {e}", i + 1))?;
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        out.push(parse_flat_object(t).map_err(|e| format!("{path} line {}: {e}", i + 1))?);
    }
    Ok(out)
}

pub(super) fn parse_flat_object(text: &str) -> Result<JsonRow, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0usize;
    skip_ws(&chars, &mut pos);
    if chars.get(pos) != Some(&'{') {
        return Err("不是 JSON 对象".into());
    }
    pos += 1;
    let mut row = JsonRow::default();
    skip_ws(&chars, &mut pos);
    if chars.get(pos) == Some(&'}') {
        return Ok(row);
    }
    loop {
        skip_ws(&chars, &mut pos);
        let key = parse_string(&chars, &mut pos)?;
        skip_ws(&chars, &mut pos);
        if chars.get(pos) != Some(&':') {
            return Err(format!("key {key:?} 后缺 ':'"));
        }
        pos += 1;
        skip_ws(&chars, &mut pos);
        let val = parse_scalar(&chars, &mut pos)?;
        row.0.push((key, val));
        skip_ws(&chars, &mut pos);
        match chars.get(pos) {
            Some(',') => pos += 1,
            Some('}') => return Ok(row),
            _ => return Err(format!("缺 ',' 或 '}}'（位置 {pos}）")),
        }
    }
}

fn skip_ws(c: &[char], pos: &mut usize) {
    while matches!(
        c.get(*pos),
        Some(' ') | Some('\t') | Some('\n') | Some('\r')
    ) {
        *pos += 1;
    }
}

fn parse_string(c: &[char], pos: &mut usize) -> Result<String, String> {
    if c.get(*pos) != Some(&'"') {
        return Err(format!("位置 {} 期望字符串", pos));
    }
    *pos += 1;
    let mut out = String::new();
    loop {
        match c.get(*pos) {
            Some('"') => {
                *pos += 1;
                return Ok(out);
            }
            Some('\\') => {
                *pos += 1;
                match c.get(*pos) {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('b') => out.push('\u{0008}'),
                    Some('f') => out.push('\u{000C}'),
                    Some('u') => {
                        let hex: String = c.iter().skip(*pos + 1).take(4).collect();
                        let code = u32::from_str_radix(&hex, 16)
                            .ok()
                            .filter(|_| hex.len() == 4)
                            .ok_or_else(|| format!("位置 {} 的 \\u 转义无效", pos))?;
                        out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        *pos += 4;
                    }
                    Some(other) => return Err(format!("位置 {} 的转义 \\{other} 无效", pos)),
                    None => return Err("字符串未结束".into()),
                }
                *pos += 1;
            }
            Some(ch) => {
                out.push(*ch);
                *pos += 1;
            }
            None => return Err("字符串未结束".into()),
        }
    }
}

fn parse_scalar(c: &[char], pos: &mut usize) -> Result<Scalar, String> {
    match c.get(*pos) {
        Some('"') => Ok(Scalar::Str(parse_string(c, pos)?)),
        Some('t') | Some('f') | Some('n') => {
            let word: String = c
                .iter()
                .skip(*pos)
                .take_while(|ch| ch.is_ascii_alphabetic())
                .collect();
            *pos += word.chars().count();
            match word.as_str() {
                "true" => Ok(Scalar::Bool(true)),
                "false" => Ok(Scalar::Bool(false)),
                "null" => Ok(Scalar::Str(String::new())),
                other => Err(format!("无效字面量 {other:?}")),
            }
        }
        Some(_) => {
            let start = *pos;
            while matches!(c.get(*pos), Some(ch) if ch.is_ascii_digit()
                || *ch == '-' || *ch == '+' || *ch == '.' || *ch == 'e' || *ch == 'E')
            {
                *pos += 1;
            }
            let text: String = c[start..*pos].iter().collect();
            text.parse::<f64>()
                .map(Scalar::Num)
                .map_err(|_| format!("无法解析数字 {text:?}"))
        }
        None => Err("值缺失".into()),
    }
}
