//! When：送出 HTTP 請求

use axum::body::Body;
use axum::http::{Request, header};
use cucumber::gherkin::Step;
use cucumber::when;
use serde_json::json;

use super::{ApiWorld, docstring, level};

pub fn code_request(id: &str, endpoint: &str, code: &str) -> Request<Body> {
    raw_request(
        id,
        endpoint,
        Some("application/json"),
        json!({ "code": code }).to_string(),
    )
}

fn raw_request(
    id: &str,
    endpoint: &str,
    content_type: Option<&str>,
    body: String,
) -> Request<Body> {
    let mut req = Request::post(format!("/api/levels/{id}/{endpoint}"));
    if let Some(ct) = content_type {
        req = req.header(header::CONTENT_TYPE, ct);
    }
    req.body(Body::from(body)).unwrap()
}

fn endpoint(mode: &str) -> &'static str {
    match mode {
        "Run" => "run",
        "Submit" => "submit",
        other => panic!("未知模式 {other}"),
    }
}

fn level_file(id: &str, file: &str) -> String {
    let l = level(id);
    match file {
        "solution" => l.solution.clone(),
        "starter" => l.starter.clone(),
        other => panic!("未知檔案 {other}"),
    }
}

/// 在函式本體開頭插入程式碼（每行縮排 4 格）
fn insert_at_fn_start(src: &str, func: &str, snippet: &str) -> String {
    let start = src
        .find(&format!("fn {func}("))
        .unwrap_or_else(|| panic!("找不到 fn {func}"));
    let brace = start + src[start..].find('{').expect("函式本體");
    let line_end = brace + src[brace..].find('\n').expect("換行");
    let indented: String = snippet.lines().map(|l| format!("    {l}\n")).collect();
    format!("{}\n{indented}{}", &src[..line_end], &src[line_end + 1..])
}

/// 刪除整個函式、緊鄰上方的註解行與下方一個空行
fn remove_fn(src: &str, func: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let sig = lines
        .iter()
        .position(|l| l.trim_start().starts_with(&format!("fn {func}(")))
        .unwrap_or_else(|| panic!("找不到 fn {func}"));
    let mut first = sig;
    while first > 0 && lines[first - 1].trim_start().starts_with("//") {
        first -= 1;
    }
    let mut depth = 0i32;
    let mut last = sig;
    for (i, line) in lines.iter().enumerate().skip(sig) {
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        if depth == 0 && line.contains('}') {
            last = i;
            break;
        }
    }
    if lines.get(last + 1).is_some_and(|l| l.trim().is_empty()) {
        last += 1;
    }
    let kept: Vec<&str> = lines[..first]
        .iter()
        .chain(&lines[last + 1..])
        .copied()
        .collect();
    kept.join("\n") + "\n"
}

#[when(regex = r#"^送出 GET "([^"]+)"$"#)]
async fn send_get(w: &mut ApiWorld, uri: String) {
    w.send(Request::get(uri).body(Body::empty()).unwrap()).await;
}

#[when(regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為 "([^"]*)"$"#)]
async fn send_literal(w: &mut ApiWorld, id: String, mode: String, code: String) {
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為該關 (solution|starter)\.rs$"#)]
async fn send_level_file(w: &mut ApiWorld, id: String, mode: String, file: String) {
    let code = level_file(&id, &file);
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為：$"#)]
async fn send_docstring(w: &mut ApiWorld, id: String, mode: String, step: &Step) {
    let code = docstring(step) + "\n";
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為該關 solution\.rs 後面接上：$"#)]
async fn send_solution_appended(w: &mut ApiWorld, id: String, mode: String, step: &Step) {
    let code = format!("{}\n{}\n", level_file(&id, "solution"), docstring(step));
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(
    regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為該關 solution\.rs 並在 (\w+) 開頭加入「[^」]*」：$"#
)]
async fn send_solution_injected(
    w: &mut ApiWorld,
    id: String,
    mode: String,
    func: String,
    step: &Step,
) {
    let code = insert_at_fn_start(&level_file(&id, "solution"), &func, &docstring(step));
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(
    regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為該關 solution\.rs 刪除整個 "([^"]+)" 函式（含上方註解）$"#
)]
async fn send_solution_without_fn(w: &mut ApiWorld, id: String, mode: String, func: String) {
    let code = remove_fn(&level_file(&id, "solution"), &func);
    assert!(!code.contains(&format!("fn {func}(")), "{func} 應已刪除");
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為 (\d+) 位元組的字串$"#)]
async fn send_sized(w: &mut ApiWorld, id: String, mode: String, bytes: usize) {
    let code = "a".repeat(bytes);
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(
    regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，code 為 "([^"]*)" 後接 (\d+) 個 "([^"]*)"$"#
)]
async fn send_prefixed_repeat(
    w: &mut ApiWorld,
    id: String,
    mode: String,
    prefix: String,
    n: usize,
    unit: String,
) {
    let code = prefix + &unit.repeat(n);
    w.send(code_request(&id, endpoint(&mode), &code)).await;
}

#[when(
    regex = r#"^對關卡 "([^"]+)" 送出 (Run|Submit)，body 為 code "([^"]*)" 且以 JSON 空白補到 (\d+) 位元組$"#
)]
async fn send_padded_body(w: &mut ApiWorld, id: String, mode: String, code: String, total: usize) {
    let mut body = json!({ "code": code }).to_string();
    body.pop();
    let pad = total - body.len() - 1;
    body.push_str(&" ".repeat(pad));
    body.push('}');
    assert_eq!(body.len(), total);
    w.send(raw_request(
        &id,
        endpoint(&mode),
        Some("application/json"),
        body,
    ))
    .await;
}

#[when(
    regex = r#"^對關卡 "([^"]+)" 的 (run|submit) 端點送出 (Content-Type "[^"]*"|不帶 Content-Type)、body 為 `(.*)`$"#
)]
async fn send_raw(w: &mut ApiWorld, id: String, endpoint: String, ct: String, body: String) {
    let content_type = ct
        .strip_prefix("Content-Type \"")
        .map(|s| s.trim_end_matches('"').to_string());
    w.send(raw_request(&id, &endpoint, content_type.as_deref(), body))
        .await;
}
