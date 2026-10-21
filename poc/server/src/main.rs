// Rust Quest POC 判題服務（拋棄式）
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;

const IMAGE: &str = "rust-quest-runner:dev";
const TIMEOUT: Duration = Duration::from_secs(10);
const OUTPUT_LIMIT: usize = 64 * 1024;
const CODE_LIMIT: usize = 64 * 1024;
const RUN_MARKER: &str = "@@RQ_RUN@@";
const COMPILE_FAIL_EXIT: i32 = 86;

static SEQ: AtomicU64 = AtomicU64::new(0);

// 容器內腳本固定不變，玩家程式只透過唯讀掛載的檔案傳入
const RUN_SCRIPT: &str = r#"
s=$(date +%s%N)
rustc --edition 2024 --error-format=json -o /tmp/main /work/main.rs 2>/tmp/diag
rc=$?
e=$(date +%s%N)
cat /tmp/diag >&2
echo "@@RQ_COMPILE_MS=$(( (e - s) / 1000000 ))" >&2
if [ $rc -ne 0 ]; then exit 86; fi
echo "@@RQ_RUN@@" >&2
exec /tmp/main
"#;

const TEST_SCRIPT: &str = r#"
s=$(date +%s%N)
rustc --edition 2024 --test --error-format=json -o /tmp/main /work/main.rs 2>/tmp/diag
rc=$?
e=$(date +%s%N)
cat /tmp/diag >&2
echo "@@RQ_COMPILE_MS=$(( (e - s) / 1000000 ))" >&2
if [ $rc -ne 0 ]; then exit 86; fi
echo "@@RQ_RUN@@" >&2
exec /tmp/main --test-threads=1
"#;

fn level_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../level")
}

fn read_level_file(name: &str) -> String {
    std::fs::read_to_string(level_dir().join(name)).expect("讀取關卡檔案失敗")
}

#[derive(Deserialize)]
struct CodeReq {
    code: Option<Value>,
}

fn api_error(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(json!({ "error": { "code": code, "message": message, "details": null } }))).into_response()
}

fn extract_code(req: CodeReq) -> Result<String, Response> {
    match req.code {
        Some(Value::String(s)) if !s.is_empty() => {
            if s.len() > CODE_LIMIT {
                Err(api_error(StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE", "程式碼過長"))
            } else {
                Ok(s)
            }
        }
        _ => Err(api_error(StatusCode::BAD_REQUEST, "INVALID_REQUEST", "缺少 code 欄位或為空字串")),
    }
}

async fn get_level() -> Response {
    let mut level: Value = serde_json::from_str(&read_level_file("level.json")).expect("level.json 格式錯誤");
    level["starter"] = Value::String(read_level_file("starter.rs"));
    Json(level).into_response()
}

#[derive(Serialize, Clone)]
struct Diagnostic {
    code: Option<String>,
    message: String,
    line: Option<u64>,
    column: Option<u64>,
    end_line: Option<u64>,
    end_column: Option<u64>,
    level: String,
    explanation_url: Option<String>,
    rendered: Option<String>,
}

struct Judged {
    compile_ok: bool,
    compile_ms: u64,
    diagnostics: Vec<Diagnostic>,
    exec: Option<Value>,
    timed_out_stage: Option<&'static str>,
    stdout: String,
}

fn truncate(s: &str) -> (String, bool) {
    if s.len() <= OUTPUT_LIMIT {
        return (s.to_string(), false);
    }
    let mut end = OUTPUT_LIMIT;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    (s[..end].to_string(), true)
}

fn parse_diagnostics(raw: &str, user_lines: u64) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        if v.get("$message_type").and_then(Value::as_str) != Some("diagnostic") {
            continue;
        }
        let message = v["message"].as_str().unwrap_or_default().to_string();
        let spans = v["spans"].as_array().cloned().unwrap_or_default();
        let level = v["level"].as_str().unwrap_or("error").to_string();
        // 跳過沒有位置的彙總訊息
        if spans.is_empty() && (message.starts_with("aborting due to") || message.starts_with("For more information")) {
            continue;
        }
        if level == "failure-note" {
            continue;
        }
        let code = v["code"]["code"].as_str().map(str::to_string);
        let primary = spans.iter().find(|s| s["is_primary"].as_bool() == Some(true));
        let (mut line_s, mut col_s, mut line_e, mut col_e) = (None, None, None, None);
        if let Some(p) = primary {
            let ls = p["line_start"].as_u64().unwrap_or(0);
            if ls >= 1 && ls <= user_lines {
                line_s = Some(ls);
                col_s = p["column_start"].as_u64();
                let le = p["line_end"].as_u64().unwrap_or(ls);
                if le <= user_lines {
                    line_e = Some(le);
                    col_e = p["column_end"].as_u64();
                } else {
                    line_e = Some(user_lines);
                }
            }
        }
        let explanation_url = code
            .as_ref()
            .filter(|c| c.len() == 5 && c.starts_with('E') && c[1..].chars().all(|ch| ch.is_ascii_digit()))
            .map(|c| format!("https://doc.rust-lang.org/error_codes/{c}.html"));
        out.push(Diagnostic {
            code,
            message,
            line: line_s,
            column: col_s,
            end_line: line_e,
            end_column: col_e,
            level,
            explanation_url,
            rendered: v["rendered"].as_str().map(str::to_string),
        });
    }
    out.sort_by_key(|d| (d.level != "error", d.line.unwrap_or(u64::MAX)));
    out.truncate(50);
    out
}

async fn judge(source: &str, script: &str, user_lines: u64) -> Result<Judged, Response> {
    let dir = tempfile::Builder::new()
        .prefix("rq-poc-")
        .tempdir()
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "建立暫存目錄失敗"))?;
    let file = dir.path().join("main.rs");
    std::fs::write(&file, source).map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "寫入暫存檔失敗"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644));
    }

    let name = format!("rq-poc-{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed));
    let mount = format!("{}:/work:ro", dir.path().display());
    let args: Vec<String> = vec![
        "run", "--rm", "--name", &name, "--network", "none", "--memory", "256m", "--memory-swap", "256m",
        "--cpus", "1", "--pids-limit", "64", "--read-only", "--tmpfs", "/tmp:rw,exec,size=64m",
        "--cap-drop", "ALL", "--security-opt", "no-new-privileges", "--user", "1000:1000", "-v", &mount,
        IMAGE, "sh", "-c", script,
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    let started = Instant::now();
    let mut child = Command::new("docker")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| api_error(StatusCode::SERVICE_UNAVAILABLE, "JUDGE_UNAVAILABLE", "判題服務未就緒：無法執行 docker"))?;

    // 邊讀邊截斷，超過上限或逾時就 docker kill，保留已讀到的輸出
    let (over_tx, mut over_rx) = tokio::sync::mpsc::channel::<()>(2);
    let out_task = tokio::spawn(read_capped(child.stdout.take().expect("stdout"), OUTPUT_LIMIT, over_tx.clone()));
    let err_task = tokio::spawn(read_capped(child.stderr.take().expect("stderr"), OUTPUT_LIMIT * 4, over_tx));
    let deadline = tokio::time::sleep(TIMEOUT);
    tokio::pin!(deadline);
    let mut timed_out = false;
    let mut over_limit = false;
    let status = loop {
        tokio::select! {
            s = child.wait() => break s.ok(),
            _ = &mut deadline, if !timed_out && !over_limit => {
                timed_out = true;
                let _ = Command::new("docker").args(["kill", &name]).output().await;
            }
            Some(_) = over_rx.recv(), if !timed_out && !over_limit => {
                over_limit = true;
                let _ = Command::new("docker").args(["kill", &name]).output().await;
            }
        }
    };
    let (stdout_bytes, stdout_trunc) = out_task.await.unwrap_or_default();
    let (stderr_bytes, _) = err_task.await.unwrap_or_default();
    let stdout_raw = String::from_utf8_lossy(&stdout_bytes).to_string();
    let stderr_raw = String::from_utf8_lossy(&stderr_bytes).to_string();
    let exit = if timed_out || over_limit { None } else { status.and_then(|s| s.code()) };

    if timed_out && !stderr_raw.contains(RUN_MARKER) {
        // 編譯階段逾時
        return Ok(Judged {
            compile_ok: false,
            compile_ms: started.elapsed().as_millis() as u64,
            diagnostics: vec![],
            exec: None,
            timed_out_stage: Some("compile"),
            stdout: String::new(),
        });
    }

    if exit == Some(125) && !stderr_raw.contains("@@RQ_COMPILE_MS") {
        eprintln!("docker error: {stderr_raw}");
        return Err(api_error(StatusCode::SERVICE_UNAVAILABLE, "JUDGE_UNAVAILABLE", "判題服務未就緒：容器啟動失敗"));
    }

    let (compile_part, run_stderr) = match stderr_raw.split_once(&format!("{RUN_MARKER}\n")) {
        Some((a, b)) => (a.to_string(), b.to_string()),
        None => (stderr_raw.clone(), String::new()),
    };
    let compile_ms = compile_part
        .lines()
        .find_map(|l| l.strip_prefix("@@RQ_COMPILE_MS="))
        .and_then(|n| n.trim().parse().ok())
        .unwrap_or(0);
    let diagnostics = parse_diagnostics(&compile_part, user_lines);
    let compile_ok = exit != Some(COMPILE_FAIL_EXIT) || stderr_raw.contains(RUN_MARKER);

    if !compile_ok {
        return Ok(Judged { compile_ok, compile_ms, diagnostics, exec: None, timed_out_stage: None, stdout: String::new() });
    }

    let (stdout, so_trunc) = truncate(&stdout_raw);
    let (stderr, se_trunc) = truncate(&run_stderr);
    let total_ms = started.elapsed().as_millis() as u64;
    Ok(Judged {
        compile_ok,
        compile_ms,
        diagnostics,
        exec: Some(json!({
            "exit_code": exit, "stdout": stdout, "stderr": stderr, "timed_out": timed_out,
            "stdout_truncated": so_trunc || stdout_trunc, "stderr_truncated": se_trunc,
            "duration_ms": total_ms.saturating_sub(compile_ms),
        })),
        timed_out_stage: if timed_out { Some("execution") } else { None },
        stdout,
    })
}

async fn read_capped<R: tokio::io::AsyncRead + Unpin>(
    mut r: R,
    cap: usize,
    over: tokio::sync::mpsc::Sender<()>,
) -> (Vec<u8>, bool) {
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut truncated = false;
    loop {
        match r.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if buf.len() < cap {
                    let take = n.min(cap - buf.len());
                    buf.extend_from_slice(&chunk[..take]);
                    if take < n {
                        truncated = true;
                        let _ = over.try_send(());
                    }
                } else {
                    truncated = true;
                    let _ = over.try_send(());
                }
            }
        }
    }
    (buf, truncated)
}

fn compile_json(j: &Judged) -> Value {
    json!({
        "success": j.compile_ok,
        "timed_out": j.timed_out_stage == Some("compile"),
        "duration_ms": j.compile_ms,
        "diagnostics": j.diagnostics,
    })
}

fn line_count(code: &str) -> u64 {
    code.lines().count() as u64
}

async fn run_level(Json(req): Json<CodeReq>) -> Response {
    let code = match extract_code(req) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let j = match judge(&code, RUN_SCRIPT, line_count(&code)).await {
        Ok(j) => j,
        Err(r) => return r,
    };
    let status = if j.timed_out_stage.is_some() {
        "timeout"
    } else if !j.compile_ok {
        "compile_error"
    } else if j.exec.as_ref().and_then(|e| e["stdout_truncated"].as_bool()) == Some(true) {
        "output_limit"
    } else if j.exec.as_ref().and_then(|e| e["exit_code"].as_i64()) == Some(137) {
        "memory_limit"
    } else if j.exec.as_ref().and_then(|e| e["exit_code"].as_i64()) != Some(0) {
        "runtime_panic"
    } else {
        "ok"
    };
    Json(json!({ "status": status, "compile": compile_json(&j), "execution": j.exec })).into_response()
}

fn hidden_test_names(tests_src: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut next_is_test = false;
    for line in tests_src.lines() {
        let t = line.trim();
        if t == "#[test]" {
            next_is_test = true;
            continue;
        }
        if next_is_test {
            if let Some(rest) = t.strip_prefix("fn ") {
                names.push(rest.split('(').next().unwrap_or_default().trim().to_string());
            }
            next_is_test = false;
        }
    }
    names
}

// 從 libtest 輸出解析每個測試結果
fn parse_tests(stdout: &str, names: &[String]) -> Vec<Value> {
    let prefix = "__quest_hidden_tests::";
    names
        .iter()
        .map(|name| {
            let full = format!("{prefix}{name}");
            let result_line = stdout.lines().find(|l| l.starts_with(&format!("test {full} ...")));
            let status = match result_line {
                Some(l) if l.ends_with("ok") => "passed",
                Some(_) => "failed",
                None => "not_run",
            };
            let message = match status {
                "failed" => {
                    let header = format!("---- {full} stdout ----");
                    stdout.split_once(&header).map(|(_, rest)| {
                        let body: Vec<&str> = rest
                            .lines()
                            .skip(1)
                            .skip_while(|l| l.is_empty() || l.starts_with("thread '"))
                            .take_while(|l| !l.is_empty() && !l.starts_with("note:") && !l.starts_with("----"))
                            .filter(|l| !l.starts_with("thread '"))
                            .collect();
                        body.join("\n")
                    })
                }
                "not_run" => Some("未執行：測試程式在此測試前已被終止".to_string()),
                _ => None,
            };
            json!({ "name": name, "status": status, "passed": status == "passed", "message": message })
        })
        .collect()
}

async fn submit_level(Json(req): Json<CodeReq>) -> Response {
    let code = match extract_code(req) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let tests_src = read_level_file("tests.rs");
    let names = hidden_test_names(&tests_src);
    let user_lines = line_count(&code);
    let merged = format!("{code}\n{tests_src}");
    let j = match judge(&merged, TEST_SCRIPT, user_lines).await {
        Ok(j) => j,
        Err(r) => return r,
    };
    let tests = if j.compile_ok { parse_tests(&j.stdout, &names) } else { vec![] };
    let count = |s: &str| tests.iter().filter(|t| t["status"] == s).count();
    let (passed_n, failed_n) = (count("passed"), count("failed"));
    let not_run = names.len() - passed_n - failed_n;
    let status = if j.timed_out_stage.is_some() {
        "timeout"
    } else if !j.compile_ok {
        "compile_error"
    } else if j.exec.as_ref().and_then(|e| e["stdout_truncated"].as_bool()) == Some(true) {
        "output_limit"
    } else if failed_n == 0 && not_run == 0 && j.exec.as_ref().and_then(|e| e["exit_code"].as_i64()) == Some(0) {
        "ok"
    } else {
        "test_failed"
    };
    let warning_count = j.diagnostics.iter().filter(|d| d.level == "warning").count();
    Json(json!({
        "status": status,
        "passed": status == "ok",
        "compile": compile_json(&j),
        "execution": j.exec,
        "tests": tests,
        "summary": { "total": names.len(), "passed": passed_n, "failed": failed_n, "not_run": not_run },
        "star_inputs": { "warning_count": warning_count },
    }))
    .into_response()
}

async fn not_found() -> Response {
    api_error(StatusCode::NOT_FOUND, "LEVEL_NOT_FOUND", "POC 只有第 02 關")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/levels/02", get(get_level))
        .route("/api/levels/02-mut-matters", get(get_level))
        .route("/api/levels/02/run", post(run_level))
        .route("/api/levels/02-mut-matters/run", post(run_level))
        .route("/api/levels/02/submit", post(submit_level))
        .route("/api/levels/02-mut-matters/submit", post(submit_level))
        .fallback(not_found)
        .layer(axum::extract::DefaultBodyLimit::max(128 * 1024));
    let addr = std::env::var("POC_ADDR").unwrap_or_else(|_| "127.0.0.1:3100".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("綁定埠失敗");
    println!("POC server listening on http://{addr}");
    axum::serve(listener, app).await.expect("server 結束");
}
