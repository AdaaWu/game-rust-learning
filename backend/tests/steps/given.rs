//! Given：後端啟動、判題設定、預先佔用的判題請求

use std::process::Command;
use std::time::{Duration, Instant};

use cucumber::given;
use rust_quest_backend::judge::docker::JUDGE_LABEL;

use super::request::code_request;
use super::{ApiWorld, store};

const INFINITE_LOOP: &str = "fn main() {\n    loop {}\n}\n";

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// 帶判題 label 的容器 id
pub fn judge_containers() -> Vec<String> {
    let out = Command::new("docker")
        .args(["ps", "-aq", "--filter", &format!("label={JUDGE_LABEL}")])
        .output()
        .expect("執行 docker ps");
    assert!(out.status.success(), "docker ps 失敗");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

#[given(regex = r"^後端以 content/ 全部 67 關的關卡內容啟動$")]
fn backend_started(_w: &mut ApiWorld) {
    assert_eq!(store().levels().len(), 67, "content/ 應為 67 關");
}

#[given(regex = r"^判題設定 max_concurrent 為 (\d+)、max_queue 為 (\d+)$")]
fn set_capacity(w: &mut ApiWorld, concurrent: usize, queue: usize) {
    let c = w.config_mut();
    c.max_concurrent = concurrent;
    c.max_queue = queue;
}

#[given(
    regex = r"^判題設定 max_concurrent 為 (\d+)、max_queue 為 (\d+)、排隊等候 timeout 為 (\d+) 秒$"
)]
fn set_capacity_and_wait(w: &mut ApiWorld, concurrent: usize, queue: usize, secs: u64) {
    let c = w.config_mut();
    c.max_concurrent = concurrent;
    c.max_queue = queue;
    c.queue_wait_timeout = Duration::from_secs(secs);
}

#[given(regex = r#"^判題設定的 docker_bin 為不存在的路徑 "([^"]+)"$"#)]
fn set_docker_bin(w: &mut ApiWorld, path: String) {
    w.config_mut().docker_bin = leak(&path);
}

#[given(regex = r#"^判題設定的 runner 映像為不存在的 tag "([^"]+)"$"#)]
fn set_image(w: &mut ApiWorld, image: String) {
    w.config_mut().image = leak(&image);
}

#[given(regex = r"^判題設定的編譯 timeout 為 (\d+) 毫秒$")]
fn set_compile_timeout(w: &mut ApiWorld, ms: u64) {
    w.config_mut().compile_timeout = Duration::from_millis(ms);
}

/// 背景送出無窮迴圈 Run，等到判題容器出現代表已拿到執行名額
#[given(regex = r"^已有一個 Run 請求正在執行無窮迴圈程式$")]
async fn running_infinite_loop(w: &mut ApiWorld) {
    // 前一個情境的容器在背景移除，先等清空
    wait_until(
        || judge_containers().is_empty(),
        "前一個情境的判題容器未清除",
    )
    .await;
    w.send_background(code_request("01-hello-ferris", "run", INFINITE_LOOP));
    wait_until(|| !judge_containers().is_empty(), "沒有判題容器啟動").await;
}

/// 每 100 毫秒檢查一次，30 秒內未成立就失敗
pub async fn wait_until(cond: impl Fn() -> bool, msg: &str) {
    let started = Instant::now();
    while !cond() {
        assert!(started.elapsed() < Duration::from_secs(30), "{msg}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[given(regex = r#"^對關卡 "([^"]+)" 送出 Run 並收到 "([^"]+)" 結果$"#)]
async fn run_and_expect(w: &mut ApiWorld, id: String, status: String) {
    w.send(code_request(&id, "run", INFINITE_LOOP)).await;
    assert_eq!(w.reply().status, 200, "{}", w.body_text());
    assert_eq!(w.json()["status"], status.as_str(), "{}", w.body_text());
}
