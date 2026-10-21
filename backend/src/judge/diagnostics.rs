//! rustc --error-format=json 診斷解析

use serde::Deserialize;

/// 最多回傳的診斷筆數
const MAX_DIAGNOSTICS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Diagnostic {
    pub code: Option<String>,
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub level: DiagnosticLevel,
    pub explanation_url: Option<String>,
    pub rendered: Option<String>,
}

#[derive(Deserialize)]
struct RawDiagnostic {
    message: String,
    code: Option<RawCode>,
    level: String,
    #[serde(default)]
    spans: Vec<RawSpan>,
    rendered: Option<String>,
}

#[derive(Deserialize)]
struct RawCode {
    code: String,
}

#[derive(Deserialize)]
struct RawSpan {
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
    is_primary: bool,
}

fn is_summary(message: &str) -> bool {
    message.starts_with("aborting due to")
        || message.ends_with("warning emitted")
        || message.ends_with("warnings emitted")
}

fn map_level(level: &str) -> Option<DiagnosticLevel> {
    match level {
        "warning" => Some(DiagnosticLevel::Warning),
        "note" => Some(DiagnosticLevel::Note),
        "help" => Some(DiagnosticLevel::Help),
        "failure-note" => None,
        // error 與 error: internal compiler error
        _ => Some(DiagnosticLevel::Error),
    }
}

fn explanation_url(code: Option<&str>) -> Option<String> {
    let code = code?;
    let b = code.as_bytes();
    let is_error_code = b.len() == 5 && b[0] == b'E' && b[1..].iter().all(u8::is_ascii_digit);
    is_error_code.then(|| format!("https://doc.rust-lang.org/error_codes/{code}.html"))
}

fn from_raw(raw: RawDiagnostic, user_line_count: Option<usize>) -> Option<Diagnostic> {
    let level = map_level(&raw.level)?;
    if is_summary(&raw.message) {
        return None;
    }
    let code = raw.code.map(|c| c.code);
    let mut d = Diagnostic {
        explanation_url: explanation_url(code.as_deref()),
        code,
        message: raw.message,
        line: None,
        column: None,
        end_line: None,
        end_column: None,
        level,
        rendered: raw.rendered,
    };
    let Some(span) = raw.spans.iter().find(|s| s.is_primary) else {
        return Some(d);
    };
    match user_line_count {
        Some(count) if span.line_start as usize > count => {
            // 隱藏區段：warning 不回傳，其餘不回位置與原文
            if level == DiagnosticLevel::Warning {
                return None;
            }
            d.rendered = None;
        }
        Some(count) if span.line_end as usize > count => {
            d.line = Some(span.line_start);
            d.column = Some(span.column_start);
            d.end_line = Some(count as u32);
        }
        _ => {
            d.line = Some(span.line_start);
            d.column = Some(span.column_start);
            d.end_line = Some(span.line_end);
            d.end_column = Some(span.column_end);
        }
    }
    Some(d)
}

/// user_line_count：Submit 時為使用者程式碼行數，Run 為 None
pub fn parse_diagnostics(stderr: &str, user_line_count: Option<usize>) -> Vec<Diagnostic> {
    stderr
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| match serde_json::from_str::<RawDiagnostic>(line) {
            Ok(raw) => from_raw(raw, user_line_count),
            Err(_) => Some(Diagnostic {
                code: None,
                message: line.to_string(),
                line: None,
                column: None,
                end_line: None,
                end_column: None,
                level: DiagnosticLevel::Error,
                explanation_url: None,
                rendered: Some(line.to_string()),
            }),
        })
        .take(MAX_DIAGNOSTICS)
        .collect()
}
