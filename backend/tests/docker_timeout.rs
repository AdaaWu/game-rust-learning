//! B7a docker 可用性檢查逾時：daemon 不回應時 health 與判題不會卡住
//! 以會長時間 sleep 的假 docker 腳本模擬，不需 Docker

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::load_fixture_store;
use http_body_util::BodyExt;
use rust_quest_backend::judge::{Judge, JudgeConfig};
use rust_quest_backend::{AppState, app};
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

/// 規格的檢查逾時 10 秒，外層再留餘裕
const OUTER_LIMIT: Duration = Duration::from_secs(15);

/// 產生假 docker 腳本；回傳 (暫存目錄, 腳本路徑)
fn fake_docker(body: &str) -> (TempDir, &'static str) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("docker");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let path: &'static str = Box::leak(path.to_string_lossy().into_owned().into_boxed_str());
    (dir, path)
}

fn app_with_docker(bin: &'static str) -> Router {
    let cfg = JudgeConfig {
        docker_bin: bin,
        ..JudgeConfig::default()
    };
    app(AppState::new(load_fixture_store(), Judge::new(cfg)))
}

async fn send(app: Router, req: Request<Body>) -> (StatusCode, Option<String>, Value) {
    let started = Instant::now();
    let resp = tokio::time::timeout(OUTER_LIMIT, app.oneshot(req))
        .await
        .unwrap_or_else(|_| panic!("請求超過 {OUTER_LIMIT:?} 仍未回應"))
        .expect("oneshot");
    assert!(started.elapsed() < OUTER_LIMIT);
    let status = resp.status();
    let retry_after = resp
        .headers()
        .get(header::RETRY_AFTER)
        .map(|v| v.to_str().unwrap().to_string());
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, retry_after, serde_json::from_slice(&body).unwrap())
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post_code(uri: &str) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "code": "fn main() {}" }).to_string()))
        .unwrap()
}

#[tokio::test]
async fn health_is_docker_unreachable_when_docker_version_hangs() {
    let (_dir, bin) = fake_docker("exec sleep 120");
    let app = app_with_docker(bin);

    let (s, _, v) = send(app.clone(), get("/api/health")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        v,
        json!({ "status": "degraded", "judge": { "available": false, "reason": "docker_unreachable" } })
    );

    let (s, retry_after, v) = send(app, post_code("/api/levels/02-plain-sum/run")).await;
    assert_eq!(s, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(v["error"]["code"], "JUDGE_UNAVAILABLE");
    assert_eq!(retry_after, None);
}

#[tokio::test]
async fn submit_is_judge_unavailable_when_docker_version_hangs() {
    let (_dir, bin) = fake_docker("exec sleep 120");
    let app = app_with_docker(bin);

    let (s, retry_after, v) = send(app, post_code("/api/levels/02-plain-sum/submit")).await;
    assert_eq!(s, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(v["error"]["code"], "JUDGE_UNAVAILABLE");
    assert_eq!(retry_after, None);
}

#[tokio::test]
async fn health_is_degraded_when_image_inspect_hangs() {
    let (_dir, bin) = fake_docker("[ \"$1\" = version ] && exit 0\nexec sleep 120");
    let app = app_with_docker(bin);

    let (s, _, v) = send(app, get("/api/health")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["status"], "degraded");
    assert_eq!(v["judge"]["available"], false);
}

// B7b：其餘 docker 指令逾時、image inspect 逾時判 docker_unreachable

/// 檢查逾時 10 秒加上編譯逾時 1 秒，外層再留餘裕
const KILL_OUTER_LIMIT: Duration = Duration::from_secs(15);

fn judge_with_docker(bin: &'static str) -> Judge {
    Judge::new(JudgeConfig {
        docker_bin: bin,
        ..JudgeConfig::default()
    })
}

#[tokio::test]
async fn health_is_docker_unreachable_when_image_inspect_hangs() {
    let (_dir, bin) = fake_docker("[ \"$1\" = version ] && exit 0\nexec sleep 120");
    let app = app_with_docker(bin);

    let (s, _, v) = send(app, get("/api/health")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        v,
        json!({ "status": "degraded", "judge": { "available": false, "reason": "docker_unreachable" } })
    );
}

#[tokio::test]
async fn health_is_runner_image_missing_when_image_inspect_fails() {
    let (_dir, bin) = fake_docker("[ \"$1\" = version ] && exit 0\nexit 1");
    let app = app_with_docker(bin);

    let (s, _, v) = send(app, get("/api/health")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        v,
        json!({ "status": "degraded", "judge": { "available": false, "reason": "runner_image_missing" } })
    );
}

#[tokio::test]
async fn cleanup_returns_error_when_docker_ps_hangs() {
    let (_dir, bin) = fake_docker("exec sleep 120");
    let judge = judge_with_docker(bin);

    let result = tokio::time::timeout(OUTER_LIMIT, judge.cleanup_stale_containers())
        .await
        .unwrap_or_else(|_| panic!("清理超過 {OUTER_LIMIT:?} 仍未結束"));
    assert!(result.is_err());
}

#[tokio::test]
async fn cleanup_returns_error_when_docker_rm_hangs() {
    let (_dir, bin) = fake_docker("[ \"$1\" = ps ] && echo stale-id && exit 0\nexec sleep 120");
    let judge = judge_with_docker(bin);

    let result = tokio::time::timeout(OUTER_LIMIT, judge.cleanup_stale_containers())
        .await
        .unwrap_or_else(|_| panic!("清理超過 {OUTER_LIMIT:?} 仍未結束"));
    assert!(result.is_err());
}

#[tokio::test]
async fn run_responds_timeout_when_docker_kill_hangs() {
    let (_dir, bin) = fake_docker(
        "case \"$1\" in\n  version|image|run|rm) exit 0 ;;\n  *) exec sleep 120 ;;\nesac",
    );
    let cfg = JudgeConfig {
        docker_bin: bin,
        compile_timeout: Duration::from_secs(1),
        ..JudgeConfig::default()
    };
    let app = app(AppState::new(load_fixture_store(), Judge::new(cfg)));

    let started = Instant::now();
    let resp = tokio::time::timeout(
        KILL_OUTER_LIMIT,
        app.oneshot(post_code("/api/levels/02-plain-sum/run")),
    )
    .await
    .unwrap_or_else(|_| panic!("請求超過 {KILL_OUTER_LIMIT:?} 仍未回應"))
    .expect("oneshot");
    assert!(started.elapsed() < KILL_OUTER_LIMIT);
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "timeout");
}
