use std::path::Path;

use rust_quest_backend::content::loader::load;
use rust_quest_backend::judge::{Judge, JudgeConfig};
use rust_quest_backend::{AppState, app};

const CONTENT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../content");
const BIND_ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let store = match load(Path::new(CONTENT_DIR)) {
        Ok(store) => store,
        Err(e) => {
            tracing::error!("關卡載入失敗：{e}");
            std::process::exit(1);
        }
    };
    tracing::info!("已載入 {} 關", store.levels().len());

    let judge = Judge::new(JudgeConfig::default());
    let health = judge.health().await;
    if !health.available {
        tracing::warn!("判題器不可用：{:?}", health.reason);
    }
    match judge.cleanup_stale_containers().await {
        Ok(n) if n > 0 => tracing::info!("清掉 {n} 個殘留判題容器"),
        Ok(_) => {}
        Err(e) => tracing::warn!("清理殘留判題容器失敗：{e}"),
    }

    let listener = tokio::net::TcpListener::bind(BIND_ADDR)
        .await
        .unwrap_or_else(|e| panic!("無法綁定 {BIND_ADDR}：{e}"));
    tracing::info!("listening on {BIND_ADDR}");
    axum::serve(listener, app(AppState::new(store, judge)))
        .await
        .expect("server 異常結束");
}
