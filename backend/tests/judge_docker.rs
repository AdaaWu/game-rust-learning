//! B5 judge 執行器 Docker 整合測試（cargo test -- --ignored）
//! 需要 Docker 與 rust-quest-runner:dev；每次判題使用不同暫存目錄

mod common;

use std::process::Command;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use common::load_fixture_store;
use rust_quest_backend::content::LevelStore;
use rust_quest_backend::content::model::Level;
use rust_quest_backend::judge::diagnostics::DiagnosticLevel;
use rust_quest_backend::judge::outcome::JudgeStatus;
use rust_quest_backend::judge::test_output::TestStatus;
use rust_quest_backend::judge::{Judge, JudgeConfig, JudgeError, UnavailableReason};

const IMAGE: &str = "rust-quest-runner:dev";
const LABEL_FILTER: &str = "label=rust-quest.judge=1";

/// 容器計數會互相干擾，Docker 測試依序執行
static DOCKER_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

fn ensure_docker() {
    let ok = Command::new("docker")
        .args(["image", "inspect", IMAGE])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(
        ok,
        "Docker 不可用或找不到 {IMAGE}：請先 docker build -t {IMAGE} runner/"
    );
}

fn judge_with(f: impl FnOnce(&mut JudgeConfig)) -> Judge {
    let mut cfg = JudgeConfig::default();
    f(&mut cfg);
    Judge::new(cfg)
}

fn judge() -> Judge {
    judge_with(|_| {})
}

fn store() -> LevelStore {
    load_fixture_store()
}

fn level<'a>(store: &'a LevelStore, id: &str) -> &'a Level {
    store.get(id).unwrap_or_else(|| panic!("fixture 應有 {id}"))
}

fn labeled_containers() -> Vec<String> {
    let out = Command::new("docker")
        .args(["ps", "-aq", "--filter", LABEL_FILTER])
        .output()
        .expect("docker ps");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect()
}

/// 容器在 guard Drop 後非同步刪除，最多等 10 秒
async fn assert_no_judge_containers_left() {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let left = labeled_containers();
        if left.is_empty() {
            return;
        }
        assert!(Instant::now() < deadline, "判題容器沒有被移除：{left:?}");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

// ───────── Run ─────────

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_hello_world_ok_and_container_removed() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let r = judge()
        .run(
            level(&s, "02-plain-sum"),
            "fn main() {\n    println!(\"Hello, Ferris!\");\n}\n",
        )
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Ok);
    assert!(r.compile.success);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, Some(0));
    assert_eq!(e.stdout, "Hello, Ferris!\n");
    assert_eq!(e.stderr, "");
    assert_no_judge_containers_left().await;
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_grid_solution_prints_commands() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "01-walk-grid");
    let r = judge().run(lv, &lv.solution).await.expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Ok);
    assert_eq!(
        r.execution.expect("執行結果").stdout,
        "MOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE UP\n"
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_compile_error_returns_e0382_without_execution() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "03-move-memory");
    let r = judge().run(lv, &lv.starter).await.expect("判題成功");
    assert_eq!(r.status, JudgeStatus::CompileError);
    assert!(!r.compile.success);
    assert!(r.execution.is_none());
    let d = &r.compile.diagnostics[0];
    assert_eq!(d.code.as_deref(), Some("E0382"));
    assert_eq!(
        (d.line, d.column, d.end_line, d.end_column),
        (Some(10), Some(17), Some(10), Some(20))
    );
    assert_eq!(d.level, DiagnosticLevel::Error);
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_missing_main_is_e0601_compile_error() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let r = judge()
        .run(level(&s, "02-plain-sum"), "fn helper() {}\n")
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::CompileError);
    assert_eq!(r.compile.diagnostics[0].code.as_deref(), Some("E0601"));
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_panic_is_runtime_panic_with_exit_101() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let code = "fn main() {\n    let v: Vec<i32> = Vec::new();\n    println!(\"{}\", v[0]);\n}\n";
    let r = judge()
        .run(level(&s, "02-plain-sum"), code)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::RuntimePanic);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, Some(101));
    assert!(
        e.stderr.contains("panicked at /tmp/main.rs:3:"),
        "{}",
        e.stderr
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_process_exit_nonzero_is_runtime_panic() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let r = judge()
        .run(
            level(&s, "02-plain-sum"),
            "fn main() {\n    std::process::exit(3);\n}\n",
        )
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::RuntimePanic);
    assert_eq!(r.execution.expect("執行結果").exit_code, Some(3));
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_timeout_keeps_printed_lines_and_removes_container() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let code = "use std::io::Write;\nfn main() {\n    for _ in 0..3 {\n        println!(\"MOVE DOWN\");\n    }\n    std::io::stdout().flush().unwrap();\n    loop {\n        std::hint::spin_loop();\n    }\n}\n";
    let j = judge_with(|c| c.run_timeout = Duration::from_secs(2));
    let started = Instant::now();
    let r = j
        .run(level(&s, "01-walk-grid"), code)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Timeout);
    assert!(!r.compile.timed_out);
    let e = r.execution.expect("執行結果");
    assert!(e.timed_out);
    assert_eq!(e.exit_code, None);
    assert_eq!(e.stdout, "MOVE DOWN\nMOVE DOWN\nMOVE DOWN\n");
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "應在執行 timeout 後結束"
    );
    assert_no_judge_containers_left().await;
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_endless_println_is_output_limit() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let r = judge()
        .run(
            level(&s, "01-walk-grid"),
            "fn main() {\n    loop {\n        println!(\"MOVE UP\");\n    }\n}\n",
        )
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::OutputLimit);
    let e = r.execution.expect("執行結果");
    assert!(e.stdout_truncated);
    assert_eq!(e.exit_code, None);
    assert_eq!(e.stdout.len(), 65536, "保留恰好 64 KiB");
    assert!(e.stdout.starts_with("MOVE UP\n"));
    assert_no_judge_containers_left().await;
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_out_of_memory_is_memory_limit() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let code = "fn main() {\n    println!(\"start\");\n    let v = vec![1u8; 400 * 1024 * 1024];\n    println!(\"{}\", v[v.len() - 1]);\n}\n";
    let r = judge()
        .run(level(&s, "02-plain-sum"), code)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::MemoryLimit);
    let e = r.execution.expect("執行階段 OOM 應有 execution");
    assert_eq!(e.exit_code, None, "OOM 不原樣回 137");
    assert!(!e.timed_out);
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_fork_bomb_is_contained() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let code = "fn main() {\n    loop {\n        let _ = std::process::Command::new(\"sh\").arg(\"-c\").arg(\"sleep 100\").spawn();\n    }\n}\n";
    let j = judge_with(|c| c.run_timeout = Duration::from_secs(2));
    let r = j
        .run(level(&s, "02-plain-sum"), code)
        .await
        .expect("判題成功");
    assert_ne!(r.status, JudgeStatus::Ok);
    assert_no_judge_containers_left().await;
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn run_has_no_network_and_readonly_work_dir() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let code = "fn main() {\n    let net = std::net::TcpStream::connect(\"1.1.1.1:80\").is_err();\n    let work = std::fs::write(\"/work/x\", \"a\").is_err();\n    let root = std::fs::write(\"/x\", \"a\").is_err();\n    println!(\"{net} {work} {root}\");\n}\n";
    let r = judge()
        .run(level(&s, "02-plain-sum"), code)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Ok);
    assert_eq!(r.execution.expect("執行結果").stdout, "true true true\n");
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn compile_timeout_sets_compile_timed_out() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let j = judge_with(|c| c.compile_timeout = Duration::from_millis(50));
    let r = j
        .run(level(&s, "02-plain-sum"), "fn main() {}\n")
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Timeout);
    assert!(r.compile.timed_out);
    assert!(!r.compile.success);
    assert!(r.execution.is_none());
    assert_no_judge_containers_left().await;
}

// ───────── Submit ─────────

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_solution_all_pass_in_declaration_order() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "02-plain-sum");
    let r = judge()
        .submit(lv, s.grid_sim(), &lv.solution)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Ok);
    assert!(r.passed);
    let names: Vec<&str> = r.tests.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["sums_three", "empty_is_zero", "all_ones"]);
    assert!(
        r.tests
            .iter()
            .all(|t| t.status == TestStatus::Passed && t.message.is_none())
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_starter_partial_fail_has_panic_message() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "02-plain-sum");
    let r = judge()
        .submit(lv, s.grid_sim(), &lv.starter)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::TestFailed);
    assert!(!r.passed);
    assert_eq!(r.tests[0].status, TestStatus::Failed);
    assert_eq!(
        r.tests[0].message.as_deref(),
        Some("assertion `left == right` failed: 總和不對\n  left: 14\n right: 6")
    );
    assert_eq!(r.tests[1].status, TestStatus::Passed);
    assert_eq!(r.tests[2].status, TestStatus::Passed);
    assert_eq!(r.execution.expect("執行結果").exit_code, Some(101));
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_compile_error_in_hidden_section_hides_positions() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "02-plain-sum");
    let r = judge()
        .submit(lv, s.grid_sim(), "fn main() {}\n")
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::CompileError);
    assert!(r.execution.is_none());
    let e0425: Vec<_> = r
        .compile
        .diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("E0425"))
        .collect();
    assert_eq!(e0425.len(), 3);
    assert!(
        r.compile
            .diagnostics
            .iter()
            .all(|d| d.level == DiagnosticLevel::Error),
        "隱藏區段的 warning 不回傳：{:?}",
        r.compile.diagnostics
    );
    for d in e0425 {
        assert_eq!(
            (d.line, d.column, d.end_line, d.end_column),
            (None, None, None, None)
        );
        assert_eq!(d.rendered, None);
    }
    assert!(r.tests.iter().all(|t| t.status == TestStatus::NotRun));
    assert_eq!(r.tests.len(), 3);
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_compile_error_in_user_code_keeps_editor_lines() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "03-move-memory");
    let r = judge()
        .submit(lv, s.grid_sim(), &lv.starter)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::CompileError);
    let d = r
        .compile
        .diagnostics
        .iter()
        .find(|d| d.code.as_deref() == Some("E0382"))
        .expect("E0382");
    assert_eq!(d.line, Some(10));
    assert!(d.rendered.is_some());
    assert_eq!(r.tests[0].status, TestStatus::NotRun);
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_does_not_run_player_tests() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let lv = level(&s, "02-plain-sum");
    let code = format!(
        "{}\n#[cfg(test)]\nmod tests {{\n    #[test]\n    fn mine() {{\n        panic!(\"玩家測試\");\n    }}\n}}\n",
        lv.solution
    );
    let r = judge()
        .submit(lv, s.grid_sim(), &code)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::Ok, "玩家自己的 #[test] 不應被執行");
    assert_eq!(r.tests.len(), 3);
    assert!(
        r.tests
            .iter()
            .all(|t| t.name != "mine" && t.name != "tests::mine")
    );
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn submit_grid_levels_use_generated_level_grid() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    for id in ["01-walk-grid", "09-late-grid"] {
        let lv = level(&s, id);
        let r = judge()
            .submit(lv, s.grid_sim(), &lv.solution)
            .await
            .expect("判題成功");
        assert_eq!(r.status, JudgeStatus::Ok, "{id} solution 應通過");
    }
    let lv = level(&s, "01-walk-grid");
    let r = judge()
        .submit(lv, s.grid_sim(), &lv.starter)
        .await
        .expect("判題成功");
    assert_eq!(r.status, JudgeStatus::TestFailed);
    assert!(
        r.tests[0]
            .message
            .as_deref()
            .is_some_and(|m| m.contains("Ferris 沒有停在終點")),
        "{:?}",
        r.tests[0].message
    );
}

// ───────── 併發、可用性、清理 ─────────

#[tokio::test]
#[ignore = "需要 Docker"]
async fn concurrent_request_beyond_capacity_is_busy() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let j = judge_with(|c| {
        c.max_concurrent = 1;
        c.max_queue = 0;
    });
    let lv = level(&s, "02-plain-sum");
    let slow = "fn main() {\n    std::thread::sleep(std::time::Duration::from_secs(2));\n}\n";
    let (a, b) = tokio::join!(j.run(lv, slow), async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        j.run(lv, "fn main() {}\n").await
    });
    assert!(a.is_ok(), "第一個請求應完成：{a:?}");
    assert_eq!(b.err(), Some(JudgeError::Busy));
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn health_ok_with_existing_image() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let h = judge().health().await;
    assert!(h.available);
    assert_eq!(h.reason, None);
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn missing_image_reports_runner_image_missing_and_run_unavailable() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let j = judge_with(|c| c.image = "rust-quest-runner:does-not-exist");
    let h = j.health().await;
    assert!(!h.available);
    assert_eq!(h.reason, Some(UnavailableReason::RunnerImageMissing));
    let r = j.run(level(&s, "02-plain-sum"), "fn main() {}\n").await;
    assert!(matches!(r, Err(JudgeError::Unavailable(_))), "{r:?}");
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn cleanup_removes_stale_labeled_containers() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let started = Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "--label",
            "rust-quest.judge=1",
            IMAGE,
            "sleep",
            "60",
        ])
        .output()
        .expect("docker run");
    assert!(started.status.success());
    assert!(!labeled_containers().is_empty());
    let removed = judge().cleanup_stale_containers().await.expect("清理");
    assert!(removed >= 1);
    assert_no_judge_containers_left().await;
}

#[tokio::test]
#[ignore = "需要 Docker"]
async fn dropped_request_removes_container() {
    let _g = DOCKER_LOCK.lock().await;
    ensure_docker();
    let s = store();
    let code = "fn main() {\n    loop {\n        std::hint::spin_loop();\n    }\n}\n";
    let j = judge();
    let dropped = tokio::time::timeout(
        Duration::from_secs(1),
        j.run(level(&s, "02-plain-sum"), code),
    )
    .await;
    assert!(dropped.is_err(), "1 秒內不應完成，future 會被 drop");
    assert_no_judge_containers_left().await;
}
