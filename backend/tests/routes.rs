//! B6 路由：levels、run、submit、health、統一錯誤格式、503
//! 不需 Docker 的以 tower::ServiceExt::oneshot 測；標 #[ignore] 的需要 Docker

mod common;

use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::IntoResponse;
use common::{ContentDir, load_fixture_store};
use http_body_util::BodyExt;
use rust_quest_backend::error::ApiError;
use rust_quest_backend::judge::{Judge, JudgeConfig};
use rust_quest_backend::{AppState, app};
use serde_json::{Value, json};
use tower::ServiceExt;

const MISSING_IMAGE: &str = "rust-quest-runner:does-not-exist";
const MISSING_DOCKER: &str = "/nonexistent/docker";
const KIB: usize = 1024;

fn app_with(f: impl FnOnce(&mut JudgeConfig)) -> Router {
    let mut cfg = JudgeConfig::default();
    f(&mut cfg);
    app(AppState::new(load_fixture_store(), Judge::new(cfg)))
}

/// 排隊名額為 0：一進來就 503 JUDGE_BUSY，不碰 Docker
fn busy_app() -> Router {
    app_with(|c| {
        c.docker_bin = MISSING_DOCKER;
        c.max_concurrent = 0;
        c.max_queue = 0;
    })
}

fn default_app() -> Router {
    app_with(|_| {})
}

async fn send(app: Router, req: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let resp = app.oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp
        .into_body()
        .collect()
        .await
        .expect("讀 body")
        .to_bytes()
        .to_vec();
    (status, headers, body)
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post_json(uri: &str, body: impl Into<Body>) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.into())
        .unwrap()
}

fn code_body(code: &str) -> String {
    json!({ "code": code }).to_string()
}

fn as_json(body: &[u8]) -> Value {
    serde_json::from_slice(body)
        .unwrap_or_else(|e| panic!("body 不是 JSON（{e}）：{}", String::from_utf8_lossy(body)))
}

fn is_json(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"))
}

/// 斷言錯誤回應恰為 {error: {code, message}}
fn assert_error_body(status: StatusCode, expected: StatusCode, body: &[u8], code: &str) {
    assert_eq!(status, expected, "body：{}", String::from_utf8_lossy(body));
    let v = as_json(body);
    let obj = v.as_object().expect("錯誤 body 應為物件");
    assert_eq!(obj.keys().collect::<Vec<_>>(), vec!["error"], "{v}");
    let err = v["error"].as_object().expect("error 應為物件");
    let mut keys: Vec<&String> = err.keys().collect();
    keys.sort();
    assert_eq!(keys, vec!["code", "message"], "{v}");
    assert_eq!(v["error"]["code"], code, "{v}");
    assert!(
        v["error"]["message"]
            .as_str()
            .is_some_and(|m| !m.is_empty()),
        "message 應為非空字串：{v}"
    );
}

/// 遞迴檢查 JSON 不含指定 key
fn assert_no_key(v: &Value, forbidden: &[&str]) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                assert!(!forbidden.contains(&k.as_str()), "不應出現 key `{k}`");
                assert_no_key(child, forbidden);
            }
        }
        Value::Array(items) => items.iter().for_each(|c| assert_no_key(c, forbidden)),
        _ => {}
    }
}

fn sorted_keys(v: &Value) -> Vec<String> {
    let mut keys: Vec<String> = v.as_object().expect("物件").keys().cloned().collect();
    keys.sort();
    keys
}

const ENDPOINTS: [&str; 2] = ["run", "submit"];

// ───────── GET /api/levels ─────────

#[tokio::test]
async fn list_levels_returns_chapters_with_levels_only() {
    let (status, headers, body) = send(default_app(), get("/api/levels")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(is_json(&headers));
    let v = as_json(&body);
    assert_eq!(
        v["chapters"],
        json!([
            { "chapter": 1, "title": "沙灘甦醒" },
            { "chapter": 2, "title": "潮汐羅盤" },
            { "chapter": 3, "title": "所有權之海" },
        ]),
        "第 4 章沒有關卡，不應出現"
    );
}

#[tokio::test]
async fn list_levels_sorted_by_chapter_then_order_with_summary_fields() {
    let (status, _, body) = send(default_app(), get("/api/levels")).await;
    assert_eq!(status, StatusCode::OK);
    let v = as_json(&body);
    assert_eq!(
        v["levels"],
        json!([
            { "id": "01-walk-grid", "chapter": 1, "order": 1, "title": "第一步", "kind": "build", "visual": "grid" },
            { "id": "09-late-grid", "chapter": 1, "order": 9, "title": "開門撿寶", "kind": "fix-compile", "visual": "grid" },
            { "id": "02-plain-sum", "chapter": 2, "order": 2, "title": "數貝殼", "kind": "debug", "visual": "none" },
            { "id": "03-move-memory", "chapter": 3, "order": 3, "title": "交出去的背包", "kind": "fix-compile", "visual": "memory" },
            { "id": "04-bridge-timeline", "chapter": 3, "order": 4, "title": "兩座窄橋", "kind": "debug", "visual": "timeline" },
        ])
    );
    assert_eq!(sorted_keys(&v), vec!["chapters", "levels"]);
}

#[tokio::test]
async fn list_levels_never_contains_solution_or_tests() {
    let (status, _, body) = send(default_app(), get("/api/levels")).await;
    assert_eq!(status, StatusCode::OK);
    assert_no_key(&as_json(&body), &["solution", "tests"]);
}

// ───────── GET /api/levels/:id ─────────

#[tokio::test]
async fn level_detail_grid_level_full_shape() {
    let (status, headers, body) = send(default_app(), get("/api/levels/01-walk-grid")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(is_json(&headers));
    let starter = common::read_fixture("content/levels/01-walk-grid/starter.rs");
    assert_eq!(
        as_json(&body),
        json!({
            "id": "01-walk-grid",
            "chapter": 1,
            "order": 1,
            "title": "第一步",
            "kind": "build",
            "visual": "grid",
            "story": "我在沙灘上醒來。",
            "concept": "## 函式\n\n- 回傳指令清單",
            "hints": ["先往下走", "再往右走"],
            "starter": starter,
            "grid": {
                "width": 5,
                "height": 3,
                "start": [0, 0],
                "goal": [4, 1],
                "walls": [[2, 0], [2, 1]],
                "gems": [],
                "doors": []
            },
            "memory_snapshots": null
        })
    );
}

#[tokio::test]
async fn level_detail_omitted_hints_is_empty_array() {
    let (status, _, body) = send(default_app(), get("/api/levels/09-late-grid")).await;
    assert_eq!(status, StatusCode::OK);
    let v = as_json(&body);
    assert_eq!(v["hints"], json!([]));
    assert_eq!(v["grid"]["gems"], json!([[0, 2]]));
    assert_eq!(v["grid"]["doors"], json!([[3, 1]]));
}

#[tokio::test]
async fn level_detail_memory_level_snapshots_shape() {
    let (status, _, body) = send(default_app(), get("/api/levels/03-move-memory")).await;
    assert_eq!(status, StatusCode::OK);
    let v = as_json(&body);
    assert_eq!(v["visual"], "memory");
    assert_eq!(v["grid"], Value::Null);
    assert_eq!(
        v["memory_snapshots"],
        json!([
            {
                "line": 8,
                "note": "bag 擁有 heap 上的三個字串",
                "frames": [
                    { "id": "report", "fn": "report", "vars": [
                        { "id": "report.bag", "name": "bag", "type": "Vec<String>", "value": "len=3 cap=3", "state": "live" }
                    ] }
                ],
                "heap": [
                    { "id": "h1", "type": "[String; 3]", "value": "珍珠 | 貝殼 | 海星", "state": "live" }
                ],
                "pointers": [
                    { "from": "report.bag", "to": "h1", "kind": "own" }
                ],
                "error": null
            },
            {
                "line": 10,
                "note": "再讀 bag[0]，編譯器回報 E0382",
                "frames": [
                    { "id": "report", "fn": "report", "vars": [
                        { "id": "report.bag", "name": "bag", "type": "Vec<String>", "value": null, "state": "moved" }
                    ] }
                ],
                "heap": [],
                "pointers": [],
                "error": { "code": "E0382", "target": "report.bag" }
            }
        ])
    );
}

#[tokio::test]
async fn level_detail_none_visual_has_null_grid_and_snapshots() {
    let (status, _, body) = send(default_app(), get("/api/levels/02-plain-sum")).await;
    assert_eq!(status, StatusCode::OK);
    let v = as_json(&body);
    assert_eq!(v["grid"], Value::Null);
    assert_eq!(v["memory_snapshots"], Value::Null);
    assert_eq!(v["kind"], "debug");
}

#[tokio::test]
async fn level_detail_timeline_level_returns_starter_with_helper() {
    let (status, _, body) = send(default_app(), get("/api/levels/04-bridge-timeline")).await;
    assert_eq!(status, StatusCode::OK);
    let starter = common::read_fixture("content/levels/04-bridge-timeline/starter.rs");
    let helper = common::read_fixture("content/shared/timeline.rs");
    assert_eq!(
        as_json(&body),
        json!({
            "id": "04-bridge-timeline",
            "chapter": 3,
            "order": 4,
            "title": "兩座窄橋",
            "kind": "debug",
            "visual": "timeline",
            "story": "兩隻分身要一起過兩座窄橋。",
            "concept": "## 鎖的順序\n\n- 每條執行緒依同一順序上鎖",
            "hints": ["看時間軸誰在等誰"],
            "starter": format!("{starter}\n{helper}"),
            "grid": null,
            "memory_snapshots": null
        })
    );
}

#[tokio::test]
async fn level_detail_has_exact_keys_and_no_solution_or_tests() {
    for id in [
        "01-walk-grid",
        "09-late-grid",
        "02-plain-sum",
        "03-move-memory",
        "04-bridge-timeline",
    ] {
        let (status, _, body) = send(default_app(), get(&format!("/api/levels/{id}"))).await;
        assert_eq!(status, StatusCode::OK, "{id}");
        let v = as_json(&body);
        assert_eq!(
            sorted_keys(&v),
            vec![
                "chapter",
                "concept",
                "grid",
                "hints",
                "id",
                "kind",
                "memory_snapshots",
                "order",
                "starter",
                "story",
                "title",
                "visual"
            ],
            "{id}"
        );
        assert_no_key(&v, &["solution", "tests", "test_names", "level_grid_src"]);
    }
}

#[tokio::test]
async fn level_detail_unknown_id_is_404_level_not_found() {
    let (status, headers, body) = send(default_app(), get("/api/levels/99-unknown")).await;
    assert_error_body(status, StatusCode::NOT_FOUND, &body, "LEVEL_NOT_FOUND");
    assert!(is_json(&headers));
}

// ───────── POST run / submit：400 ─────────

#[tokio::test]
async fn invalid_json_body_is_400() {
    for ep in ENDPOINTS {
        let (s, _, b) = send(
            busy_app(),
            post_json(&format!("/api/levels/02-plain-sum/{ep}"), "{not json"),
        )
        .await;
        assert_error_body(s, StatusCode::BAD_REQUEST, &b, "INVALID_REQUEST");
    }
}

#[tokio::test]
async fn missing_content_type_is_400_not_415() {
    for ep in ENDPOINTS {
        let req = Request::post(format!("/api/levels/02-plain-sum/{ep}"))
            .body(Body::from(code_body("fn main() {}")))
            .unwrap();
        let (s, _, b) = send(busy_app(), req).await;
        assert_error_body(s, StatusCode::BAD_REQUEST, &b, "INVALID_REQUEST");
    }
}

#[tokio::test]
async fn non_json_content_type_is_400() {
    for ep in ENDPOINTS {
        let req = Request::post(format!("/api/levels/02-plain-sum/{ep}"))
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(code_body("fn main() {}")))
            .unwrap();
        let (s, _, b) = send(busy_app(), req).await;
        assert_error_body(s, StatusCode::BAD_REQUEST, &b, "INVALID_REQUEST");
    }
}

#[tokio::test]
async fn missing_null_non_string_or_empty_code_is_400() {
    for ep in ENDPOINTS {
        for body in [
            "{}",
            "{\"code\":null}",
            "{\"code\":42}",
            "{\"code\":[\"x\"]}",
            "{\"code\":\"\"}",
            "[]",
            "\"fn main() {}\"",
        ] {
            let (s, _, b) = send(
                busy_app(),
                post_json(&format!("/api/levels/02-plain-sum/{ep}"), body),
            )
            .await;
            assert_error_body(s, StatusCode::BAD_REQUEST, &b, "INVALID_REQUEST");
        }
    }
}

// ───────── POST run / submit：413 ─────────

#[tokio::test]
async fn code_over_64_kib_is_413() {
    for ep in ENDPOINTS {
        let code = "a".repeat(64 * KIB + 1);
        let (s, _, b) = send(
            busy_app(),
            post_json(&format!("/api/levels/02-plain-sum/{ep}"), code_body(&code)),
        )
        .await;
        assert_error_body(s, StatusCode::PAYLOAD_TOO_LARGE, &b, "PAYLOAD_TOO_LARGE");
    }
}

#[tokio::test]
async fn code_limit_counts_utf8_bytes_not_chars() {
    // 21846 個「海」= 65538 位元組，字元數少於 64K
    let code = "海".repeat(21846);
    assert!(code.chars().count() < 64 * KIB && code.len() > 64 * KIB);
    for ep in ENDPOINTS {
        let (s, _, b) = send(
            busy_app(),
            post_json(&format!("/api/levels/02-plain-sum/{ep}"), code_body(&code)),
        )
        .await;
        assert_error_body(s, StatusCode::PAYLOAD_TOO_LARGE, &b, "PAYLOAD_TOO_LARGE");
    }
}

#[tokio::test]
async fn code_exactly_64_kib_passes_validation() {
    // 通過驗證後進入判題，排隊名額為 0 所以回 503
    for ep in ENDPOINTS {
        let code = "a".repeat(64 * KIB);
        let (s, _, b) = send(
            busy_app(),
            post_json(&format!("/api/levels/02-plain-sum/{ep}"), code_body(&code)),
        )
        .await;
        assert_error_body(s, StatusCode::SERVICE_UNAVAILABLE, &b, "JUDGE_BUSY");
    }
}

#[tokio::test]
async fn body_over_128_kib_is_413() {
    for ep in ENDPOINTS {
        let body = json!({ "code": "fn main() {}", "pad": "x".repeat(130 * KIB) }).to_string();
        let (s, _, b) = send(
            busy_app(),
            post_json(&format!("/api/levels/02-plain-sum/{ep}"), body),
        )
        .await;
        assert_error_body(s, StatusCode::PAYLOAD_TOO_LARGE, &b, "PAYLOAD_TOO_LARGE");
    }
}

// ───────── POST run / submit：404 ─────────

#[tokio::test]
async fn unknown_level_is_404() {
    for ep in ENDPOINTS {
        let (s, _, b) = send(
            busy_app(),
            post_json(
                &format!("/api/levels/99-unknown/{ep}"),
                code_body("fn main() {}"),
            ),
        )
        .await;
        assert_error_body(s, StatusCode::NOT_FOUND, &b, "LEVEL_NOT_FOUND");
    }
}

#[tokio::test]
async fn invalid_body_on_unknown_level_is_400_before_404() {
    for ep in ENDPOINTS {
        for body in ["{not json", "{}", "{\"code\":\"\"}"] {
            let (s, _, b) = send(
                busy_app(),
                post_json(&format!("/api/levels/99-unknown/{ep}"), body),
            )
            .await;
            assert_error_body(s, StatusCode::BAD_REQUEST, &b, "INVALID_REQUEST");
        }
    }
}

#[tokio::test]
async fn oversized_code_on_unknown_level_is_413_before_404() {
    for ep in ENDPOINTS {
        let code = "a".repeat(64 * KIB + 1);
        let (s, _, b) = send(
            busy_app(),
            post_json(&format!("/api/levels/99-unknown/{ep}"), code_body(&code)),
        )
        .await;
        assert_error_body(s, StatusCode::PAYLOAD_TOO_LARGE, &b, "PAYLOAD_TOO_LARGE");
    }
}

// ───────── POST run / submit：503 ─────────

#[tokio::test]
async fn queue_full_is_503_judge_busy_with_retry_after() {
    for ep in ENDPOINTS {
        let (s, h, b) = send(
            busy_app(),
            post_json(
                &format!("/api/levels/02-plain-sum/{ep}"),
                code_body("fn main() {}"),
            ),
        )
        .await;
        assert_error_body(s, StatusCode::SERVICE_UNAVAILABLE, &b, "JUDGE_BUSY");
        assert_eq!(
            h.get(header::RETRY_AFTER).and_then(|v| v.to_str().ok()),
            Some("2")
        );
    }
}

#[tokio::test]
async fn queue_wait_timeout_is_503_judge_busy() {
    let app = app_with(|c| {
        c.docker_bin = MISSING_DOCKER;
        c.max_concurrent = 0;
        c.max_queue = 1;
        c.queue_wait_timeout = Duration::from_millis(50);
    });
    let (s, h, b) = send(
        app,
        post_json("/api/levels/02-plain-sum/run", code_body("fn main() {}")),
    )
    .await;
    assert_error_body(s, StatusCode::SERVICE_UNAVAILABLE, &b, "JUDGE_BUSY");
    assert_eq!(
        h.get(header::RETRY_AFTER).and_then(|v| v.to_str().ok()),
        Some("2")
    );
}

#[tokio::test]
async fn docker_unavailable_is_503_judge_unavailable_without_retry_after() {
    for ep in ENDPOINTS {
        let app = app_with(|c| c.docker_bin = MISSING_DOCKER);
        let (s, h, b) = send(
            app,
            post_json(
                &format!("/api/levels/02-plain-sum/{ep}"),
                code_body("fn main() {}"),
            ),
        )
        .await;
        assert_error_body(s, StatusCode::SERVICE_UNAVAILABLE, &b, "JUDGE_UNAVAILABLE");
        assert!(h.get(header::RETRY_AFTER).is_none());
    }
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn missing_runner_image_is_503_judge_unavailable() {
    for ep in ENDPOINTS {
        let app = app_with(|c| c.image = MISSING_IMAGE);
        let (s, h, b) = send(
            app,
            post_json(
                &format!("/api/levels/02-plain-sum/{ep}"),
                code_body("fn main() {}"),
            ),
        )
        .await;
        assert_error_body(s, StatusCode::SERVICE_UNAVAILABLE, &b, "JUDGE_UNAVAILABLE");
        assert!(h.get(header::RETRY_AFTER).is_none());
    }
}

// ───────── GET /api/health ─────────

#[tokio::test]
async fn health_degraded_when_docker_unavailable() {
    let app = app_with(|c| c.docker_bin = MISSING_DOCKER);
    let (s, h, b) = send(app, get("/api/health")).await;
    assert_eq!(s, StatusCode::OK, "降級仍回 200");
    assert!(is_json(&h));
    let v = as_json(&b);
    assert_eq!(sorted_keys(&v), vec!["judge", "status"]);
    assert_eq!(sorted_keys(&v["judge"]), vec!["available", "reason"]);
    assert_eq!(v["status"], "degraded");
    assert_eq!(v["judge"]["available"], false);
    assert_eq!(v["judge"]["reason"], "docker_unreachable");
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn health_degraded_reason_is_runner_image_missing() {
    let app = app_with(|c| c.image = MISSING_IMAGE);
    let (_, _, b) = send(app, get("/api/health")).await;
    assert_eq!(
        as_json(&b),
        json!({ "status": "degraded", "judge": { "available": false, "reason": "runner_image_missing" } })
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn health_ok_when_docker_and_image_available() {
    let (s, _, b) = send(default_app(), get("/api/health")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        as_json(&b),
        json!({ "status": "ok", "judge": { "available": true, "reason": null } })
    );
}

// ───────── axum 預設 404／405 ─────────

#[tokio::test]
async fn unknown_path_and_wrong_method_use_axum_defaults() {
    let (s, _, _) = send(default_app(), get("/api/nope")).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _, _) = send(default_app(), get("/api/levels/02-plain-sum/run")).await;
    assert_eq!(s, StatusCode::METHOD_NOT_ALLOWED);
    let (s, _, _) = send(default_app(), get("/api/levels")).await;
    assert_eq!(s, StatusCode::OK, "已知路由應存在，區隔 404 來源");
}

// ───────── ApiError → 回應 ─────────

async fn error_response(err: ApiError) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let resp = err.into_response();
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, headers, body)
}

#[tokio::test]
async fn api_error_maps_every_code() {
    for (err, status, code) in [
        (
            ApiError::InvalidRequest("壞掉".into()),
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
        ),
        (
            ApiError::LevelNotFound("99-unknown".into()),
            StatusCode::NOT_FOUND,
            "LEVEL_NOT_FOUND",
        ),
        (
            ApiError::PayloadTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
            "PAYLOAD_TOO_LARGE",
        ),
        (
            ApiError::JudgeBusy,
            StatusCode::SERVICE_UNAVAILABLE,
            "JUDGE_BUSY",
        ),
        (
            ApiError::JudgeUnavailable("找不到 runner 映像".into()),
            StatusCode::SERVICE_UNAVAILABLE,
            "JUDGE_UNAVAILABLE",
        ),
        (
            ApiError::Internal("boom".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
        ),
    ] {
        let is_busy = code == "JUDGE_BUSY";
        let (s, h, b) = error_response(err).await;
        assert_error_body(s, status, &b, code);
        assert!(is_json(&h), "{code} 應為 JSON");
        assert_eq!(
            h.get(header::RETRY_AFTER).and_then(|v| v.to_str().ok()),
            is_busy.then_some("2"),
            "只有 JUDGE_BUSY 帶 Retry-After"
        );
    }
}

#[tokio::test]
async fn level_not_found_message_names_the_id() {
    let (_, _, b) = error_response(ApiError::LevelNotFound("99-unknown".into())).await;
    let v = as_json(&b);
    assert!(
        v["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("99-unknown")),
        "{v}"
    );
}

// ───────── 真實判題回應形狀（需要 Docker）─────────

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_response_json_shape() {
    let (s, _, b) = send(
        default_app(),
        post_json(
            "/api/levels/02-plain-sum/run",
            code_body("fn main() {\n    println!(\"hi\");\n}\n"),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let v = as_json(&b);
    assert_eq!(
        v,
        json!({
            "status": "ok",
            "compile": { "success": true, "timed_out": false, "diagnostics": [] },
            "execution": {
                "exit_code": 0, "stdout": "hi\n", "stderr": "",
                "timed_out": false, "stdout_truncated": false, "stderr_truncated": false
            }
        })
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_compile_error_json_has_all_diagnostic_keys() {
    let starter = common::read_fixture("content/levels/03-move-memory/starter.rs");
    let (s, _, b) = send(
        default_app(),
        post_json("/api/levels/03-move-memory/run", code_body(&starter)),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "編譯錯誤是正常判題結果");
    let v = as_json(&b);
    assert_eq!(v["status"], "compile_error");
    assert_eq!(v["execution"], Value::Null);
    let d = &v["compile"]["diagnostics"][0];
    assert_eq!(
        sorted_keys(d),
        vec![
            "code",
            "column",
            "end_column",
            "end_line",
            "explanation_url",
            "level",
            "line",
            "message",
            "rendered"
        ]
    );
    assert_eq!(d["code"], "E0382");
    assert_eq!(d["level"], "error");
    assert_eq!(d["line"], 10);
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_response_json_shape() {
    let starter = common::read_fixture("content/levels/02-plain-sum/starter.rs");
    let (s, _, b) = send(
        default_app(),
        post_json("/api/levels/02-plain-sum/submit", code_body(&starter)),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let v = as_json(&b);
    assert_eq!(
        sorted_keys(&v),
        vec!["compile", "execution", "passed", "status", "tests"]
    );
    assert_eq!(v["status"], "test_failed");
    assert_eq!(v["passed"], false);
    assert_eq!(
        v["tests"],
        json!([
            { "name": "sums_three", "status": "failed", "message": "assertion `left == right` failed: 總和不對\n  left: 14\n right: 6" },
            { "name": "empty_is_zero", "status": "passed", "message": null },
            { "name": "all_ones", "status": "passed", "message": null }
        ])
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_compile_error_json_lists_not_run_tests() {
    let (s, _, b) = send(
        default_app(),
        post_json(
            "/api/levels/02-plain-sum/submit",
            code_body("fn main() {}\n"),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let v = as_json(&b);
    assert_eq!(v["status"], "compile_error");
    assert_eq!(v["execution"], Value::Null);
    assert_eq!(v["tests"].as_array().map(|a| a.len()), Some(3));
    assert!(
        v["tests"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t["status"] == "not_run" && t["message"].is_null())
    );
    let hidden = v["compile"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "E0425")
        .expect("E0425");
    assert_eq!(hidden["line"], Value::Null);
    assert_eq!(hidden["rendered"], Value::Null);
}

// ───────── 驗收補測 ─────────

#[tokio::test]
async fn permit_is_released_after_unavailable() {
    let app = app_with(|c| {
        c.docker_bin = MISSING_DOCKER;
        c.max_concurrent = 1;
        c.max_queue = 0;
    });
    for attempt in 1..=2 {
        let (s, _, b) = send(
            app.clone(),
            post_json("/api/levels/02-plain-sum/run", code_body("fn main() {}")),
        )
        .await;
        // 第二次若是 JUDGE_BUSY，代表第一次的 permit 沒有歸還
        assert_error_body(s, StatusCode::SERVICE_UNAVAILABLE, &b, "JUDGE_UNAVAILABLE");
        let _ = attempt;
    }
}

#[tokio::test]
async fn level_endpoints_never_read_files_after_startup() {
    let cd = ContentDir::new();
    let starter = cd.read("levels/01-walk-grid/starter.rs");
    let store = rust_quest_backend::content::loader::load(cd.root()).expect("載入");
    std::fs::remove_dir_all(cd.root()).expect("刪除 content");
    let app = app(AppState::new(
        store,
        Judge::new(JudgeConfig {
            docker_bin: MISSING_DOCKER,
            max_concurrent: 0,
            max_queue: 0,
            ..JudgeConfig::default()
        }),
    ));

    let (s, _, b) = send(app.clone(), get("/api/levels/01-walk-grid")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(as_json(&b)["starter"], starter.as_str());

    let (s, _, _) = send(app.clone(), get("/api/levels")).await;
    assert_eq!(s, StatusCode::OK);

    let (s, _, b) = send(
        app,
        post_json("/api/levels/99-unknown/submit", code_body("fn main() {}")),
    )
    .await;
    assert_error_body(s, StatusCode::NOT_FOUND, &b, "LEVEL_NOT_FOUND");
}

#[tokio::test]
async fn chapters_sorted_ascending() {
    let cd = ContentDir::new();
    cd.write(
        "chapters.toml",
        "[[chapters]]\nchapter = 3\ntitle = \"所有權之海\"\n\n[[chapters]]\nchapter = 2\ntitle = \"潮汐羅盤\"\n\n[[chapters]]\nchapter = 1\ntitle = \"沙灘甦醒\"\n",
    );
    let store = cd.load().expect("倒序 chapters.toml 應可載入");
    let app = app(AppState::new(store, Judge::new(JudgeConfig::default())));
    let (s, _, b) = send(app, get("/api/levels")).await;
    assert_eq!(s, StatusCode::OK);
    let nums: Vec<i64> = as_json(&b)["chapters"]
        .as_array()
        .expect("chapters 陣列")
        .iter()
        .map(|c| c["chapter"].as_i64().unwrap())
        .collect();
    assert_eq!(nums, vec![1, 2, 3]);
}

// ───────── Content-Type 帶 charset ─────────

#[tokio::test]
async fn json_responses_declare_utf8_charset() {
    let cases = [
        get("/api/levels"),
        get("/api/levels/02-plain-sum"),
        get("/api/levels/99-unknown"),
        get("/api/health"),
        post_json("/api/levels/02-plain-sum/run", "{}"),
    ];
    for req in cases {
        let uri = req.uri().to_string();
        let app = app_with(|c| c.docker_bin = MISSING_DOCKER);
        let (_, h, _) = send(app, req).await;
        assert_eq!(
            h.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
            Some("application/json; charset=utf-8"),
            "{uri}"
        );
    }
}
