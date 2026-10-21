//! B5 judge 執行器（不需 Docker）：設定預設值、暫存目錄權限、輸出上限、計時、§3.6 狀態判定

mod common;

use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use common::{names, read_fixture};
use rust_quest_backend::judge::diagnostics::{Diagnostic, DiagnosticLevel};
use rust_quest_backend::judge::exec::{CappedRun, Overflow, cap_output, run_capped};
use rust_quest_backend::judge::outcome::{
    JudgeStatus, build_run_response, build_submit_response, determine_status,
};
use rust_quest_backend::judge::test_output::{TestResult, TestStatus};
use rust_quest_backend::judge::{
    JudgeConfig, JudgeHealth, Mode, UnavailableReason, health_from_checks, prepare_workdir,
};

// ───────── JudgeConfig 預設值 ─────────

#[test]
fn judge_config_defaults_match_spec() {
    let cfg = JudgeConfig::default();
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let expected_concurrent = std::cmp::max(1, cpus / 2);
    assert_eq!(cfg.docker_bin, "docker");
    assert_eq!(cfg.image, "rust-quest-runner:dev");
    assert_eq!(cfg.max_concurrent, expected_concurrent);
    assert_eq!(cfg.max_queue, expected_concurrent * 4);
    assert_eq!(cfg.queue_wait_timeout, Duration::from_secs(10));
    assert_eq!(cfg.container_start_timeout, Duration::from_secs(10));
    assert_eq!(cfg.compile_timeout, Duration::from_secs(15));
    assert_eq!(cfg.run_timeout, Duration::from_secs(5));
    assert_eq!(cfg.container_max_lifetime_secs, 60);
    assert_eq!(cfg.output_limit_bytes, 64 * 1024);
    assert_eq!(cfg.max_code_bytes, 64 * 1024);
}

// ───────── health 判定 ─────────

#[test]
fn health_from_checks_maps_reasons() {
    assert_eq!(
        health_from_checks(true, true),
        JudgeHealth {
            available: true,
            reason: None
        }
    );
    assert_eq!(
        health_from_checks(false, false),
        JudgeHealth {
            available: false,
            reason: Some(UnavailableReason::DockerUnreachable)
        }
    );
    assert_eq!(
        health_from_checks(false, true),
        JudgeHealth {
            available: false,
            reason: Some(UnavailableReason::DockerUnreachable)
        }
    );
    assert_eq!(
        health_from_checks(true, false),
        JudgeHealth {
            available: false,
            reason: Some(UnavailableReason::RunnerImageMissing)
        }
    );
}

// ───────── 暫存目錄 ─────────

#[test]
fn workdir_has_0755_dir_and_0644_main_rs() {
    let dir = prepare_workdir("fn main() {}\n").expect("建立暫存目錄");
    let dir_mode = std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
    let file = dir.path().join("main.rs");
    let file_mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
    assert_eq!(dir_mode, 0o755);
    assert_eq!(file_mode, 0o644);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "fn main() {}\n");
}

#[test]
fn workdir_is_prefixed_unique_and_removed_on_drop() {
    let a = prepare_workdir("a").expect("a");
    let b = prepare_workdir("b").expect("b");
    let name = a.path().file_name().unwrap().to_string_lossy().to_string();
    assert!(name.starts_with("rq-judge-"), "{name}");
    assert_ne!(a.path(), b.path(), "每次判題要用不同目錄");
    let path = a.path().to_path_buf();
    drop(a);
    assert!(!path.exists(), "Drop 後應刪除");
}

// ───────── 輸出截斷 ─────────

#[test]
fn cap_output_under_and_at_limit_is_untouched() {
    assert_eq!(cap_output(b"abc", 10), ("abc".to_string(), false));
    assert_eq!(cap_output(b"abcd", 4), ("abcd".to_string(), false));
}

#[test]
fn cap_output_over_limit_keeps_prefix() {
    assert_eq!(cap_output(b"abcdef", 4), ("abcd".to_string(), true));
}

#[test]
fn cap_output_backs_off_to_utf8_boundary() {
    // 「海星」各 3 位元組，切在第 4 位元組時退回「海」
    assert_eq!(cap_output("海星".as_bytes(), 4), ("海".to_string(), true));
}

#[test]
fn cap_output_replaces_invalid_utf8() {
    assert_eq!(
        cap_output(&[b'f', 0xff, b'g'], 10),
        ("f\u{FFFD}g".to_string(), false)
    );
}

// ───────── run_capped（用 sh，不需 Docker）─────────

fn sv(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[tokio::test]
async fn run_capped_returns_exit_code_and_output() {
    let r = run_capped(
        "sh",
        &sv(&["-c", "echo hi; echo oops >&2; exit 3"]),
        Duration::from_secs(5),
        1024,
        Overflow::Kill,
    )
    .await
    .expect("執行 sh");
    assert_eq!(r.exit_code, Some(3));
    assert_eq!(r.stdout, "hi\n");
    assert_eq!(r.stderr, "oops\n");
    assert!(!r.timed_out && !r.stdout_truncated && !r.stderr_truncated);
}

#[tokio::test]
async fn run_capped_passes_args_without_shell() {
    let r = run_capped(
        "printf",
        &sv(&["%s", "$(whoami); echo x"]),
        Duration::from_secs(5),
        1024,
        Overflow::Kill,
    )
    .await
    .expect("執行 printf");
    assert_eq!(r.stdout, "$(whoami); echo x");
}

#[tokio::test]
async fn run_capped_timeout_keeps_partial_stdout() {
    let started = Instant::now();
    let r = run_capped(
        "sh",
        &sv(&["-c", "echo started; sleep 5"]),
        Duration::from_millis(300),
        1024,
        Overflow::Kill,
    )
    .await
    .expect("執行 sh");
    assert!(r.timed_out);
    assert_eq!(r.exit_code, None, "被終止時 exit_code 為 None");
    assert_eq!(r.stdout, "started\n");
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "應在 timeout 後立即返回"
    );
}

#[tokio::test]
async fn run_capped_kills_on_stdout_overflow() {
    let started = Instant::now();
    let r = run_capped(
        "sh",
        &sv(&["-c", "yes"]),
        Duration::from_secs(10),
        1000,
        Overflow::Kill,
    )
    .await
    .expect("執行 yes");
    assert!(r.stdout_truncated);
    assert_eq!(r.stdout.len(), 1000);
    assert_eq!(r.exit_code, None);
    assert!(!r.timed_out);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "讀到上限就終止，不等 timeout"
    );
}

#[tokio::test]
async fn run_capped_kills_on_stderr_overflow() {
    let r = run_capped(
        "sh",
        &sv(&["-c", "yes >&2"]),
        Duration::from_secs(10),
        1000,
        Overflow::Kill,
    )
    .await
    .expect("執行 yes");
    assert!(r.stderr_truncated);
    assert_eq!(r.stderr.len(), 1000);
    assert!(!r.stdout_truncated);
    assert_eq!(r.exit_code, None);
}

#[tokio::test]
async fn run_capped_discard_keeps_running_to_completion() {
    let r = run_capped(
        "sh",
        &sv(&["-c", "head -c 5000 /dev/zero | tr '\\0' a >&2; echo done"]),
        Duration::from_secs(5),
        1000,
        Overflow::Discard,
    )
    .await
    .expect("執行 sh");
    assert_eq!(r.exit_code, Some(0), "丟棄模式不終止行程");
    assert_eq!(r.stderr.len(), 1000);
    assert!(r.stderr_truncated);
    assert_eq!(r.stdout, "done\n");
}

#[tokio::test]
async fn run_capped_truncation_respects_utf8_boundary() {
    let r = run_capped(
        "sh",
        &sv(&["-c", "printf '海星海星'; sleep 5"]),
        Duration::from_secs(5),
        4,
        Overflow::Kill,
    )
    .await
    .expect("執行 sh");
    assert!(r.stdout_truncated);
    assert_eq!(r.stdout, "海");
}

// ───────── §3.6 狀態判定 ─────────

fn raw(exit_code: Option<i32>) -> CappedRun {
    CappedRun {
        exit_code,
        ..CappedRun::default()
    }
}

fn timed_out(exit_code: Option<i32>) -> CappedRun {
    CappedRun {
        exit_code,
        timed_out: true,
        ..CappedRun::default()
    }
}

fn truncated(exit_code: Option<i32>) -> CappedRun {
    CappedRun {
        exit_code,
        stdout_truncated: true,
        ..CappedRun::default()
    }
}

fn test(name: &str, status: TestStatus) -> TestResult {
    let message = (status == TestStatus::Failed).then(|| "boom".to_string());
    TestResult {
        name: name.into(),
        status,
        message,
    }
}

#[test]
fn status_1_compile_timeout() {
    for c in [timed_out(None), timed_out(Some(137))] {
        assert_eq!(
            determine_status(Mode::Run, &c, None, &[]),
            JudgeStatus::Timeout
        );
        assert_eq!(
            determine_status(Mode::Submit, &c, None, &[]),
            JudgeStatus::Timeout
        );
    }
}

#[test]
fn status_2_compile_oom_is_memory_limit_not_compile_error() {
    let c = raw(Some(137));
    assert_eq!(
        determine_status(Mode::Run, &c, None, &[]),
        JudgeStatus::MemoryLimit
    );
    assert_eq!(
        determine_status(Mode::Submit, &c, None, &[]),
        JudgeStatus::MemoryLimit
    );
}

#[test]
fn status_3_compile_error() {
    let c = raw(Some(1));
    assert_eq!(
        determine_status(Mode::Run, &c, None, &[]),
        JudgeStatus::CompileError
    );
    assert_eq!(
        determine_status(Mode::Submit, &c, None, &[]),
        JudgeStatus::CompileError
    );
}

#[test]
fn status_4_exec_timeout_wins_over_later_rules() {
    let c = raw(Some(0));
    for e in [timed_out(None), timed_out(Some(137))] {
        assert_eq!(
            determine_status(Mode::Run, &c, Some(&e), &[]),
            JudgeStatus::Timeout
        );
    }
    let both = CappedRun {
        timed_out: true,
        stdout_truncated: true,
        ..CappedRun::default()
    };
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&both), &[]),
        JudgeStatus::Timeout
    );
    let tests = [test("a", TestStatus::Failed), test("b", TestStatus::NotRun)];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&timed_out(None)), &tests),
        JudgeStatus::Timeout,
        "已有失敗測試時仍以 timeout 為準"
    );
}

#[test]
fn status_5_exec_oom_not_killed_by_host() {
    let c = raw(Some(0));
    let e = raw(Some(137));
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&e), &[]),
        JudgeStatus::MemoryLimit
    );
    let tests = [test("a", TestStatus::Passed), test("b", TestStatus::NotRun)];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&e), &tests),
        JudgeStatus::MemoryLimit
    );
}

#[test]
fn status_6_output_limit_for_stdout_or_stderr() {
    let c = raw(Some(0));
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&truncated(None)), &[]),
        JudgeStatus::OutputLimit
    );
    let stderr = CappedRun {
        stderr_truncated: true,
        ..CappedRun::default()
    };
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&stderr), &[]),
        JudgeStatus::OutputLimit
    );
    // host 因超限 kill 後回 137，不可誤判為 OOM
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&truncated(Some(137))), &[]),
        JudgeStatus::OutputLimit
    );
    assert_eq!(
        determine_status(
            Mode::Submit,
            &c,
            Some(&truncated(None)),
            &[test("a", TestStatus::NotRun)]
        ),
        JudgeStatus::OutputLimit
    );
}

#[test]
fn status_7_run_nonzero_exit_is_runtime_panic() {
    let c = raw(Some(0));
    for code in [101, 3, 1] {
        assert_eq!(
            determine_status(Mode::Run, &c, Some(&raw(Some(code))), &[]),
            JudgeStatus::RuntimePanic,
            "exit {code}"
        );
    }
}

#[test]
fn status_7_submit_any_failed_test_is_test_failed() {
    let c = raw(Some(0));
    let tests = [test("a", TestStatus::Passed), test("b", TestStatus::Failed)];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&raw(Some(101))), &tests),
        JudgeStatus::TestFailed
    );
}

#[test]
fn status_8_ok() {
    let c = raw(Some(0));
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&raw(Some(0))), &[]),
        JudgeStatus::Ok
    );
    let tests = [test("a", TestStatus::Passed), test("b", TestStatus::Passed)];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&raw(Some(0))), &tests),
        JudgeStatus::Ok
    );
}

// ───────── 回應組裝：Run ─────────

fn warning() -> Diagnostic {
    Diagnostic {
        code: Some("unused_variables".into()),
        message: "unused variable: `x`".into(),
        line: Some(2),
        column: Some(9),
        end_line: Some(2),
        end_column: Some(10),
        level: DiagnosticLevel::Warning,
        explanation_url: None,
        rendered: Some("warning: unused variable".into()),
    }
}

#[test]
fn run_response_ok_keeps_warnings_and_output() {
    let exec = CappedRun {
        exit_code: Some(0),
        stdout: "MOVE DOWN\n".into(),
        ..CappedRun::default()
    };
    let r = build_run_response(&raw(Some(0)), vec![warning()], Some(exec));
    assert_eq!(r.status, JudgeStatus::Ok);
    assert!(r.compile.success);
    assert!(!r.compile.timed_out);
    assert_eq!(r.compile.diagnostics, vec![warning()]);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, Some(0));
    assert_eq!(e.stdout, "MOVE DOWN\n");
    assert!(!e.timed_out && !e.stdout_truncated && !e.stderr_truncated);
}

#[test]
fn run_response_compile_error_has_null_execution() {
    let r = build_run_response(&raw(Some(1)), vec![], None);
    assert_eq!(r.status, JudgeStatus::CompileError);
    assert!(!r.compile.success);
    assert!(!r.compile.timed_out);
    assert!(r.execution.is_none());
}

#[test]
fn run_response_compile_timeout() {
    let r = build_run_response(&timed_out(None), vec![], None);
    assert_eq!(r.status, JudgeStatus::Timeout);
    assert!(r.compile.timed_out);
    assert!(!r.compile.success);
    assert!(r.execution.is_none());
}

#[test]
fn run_response_compile_oom() {
    let r = build_run_response(&raw(Some(137)), vec![], None);
    assert_eq!(r.status, JudgeStatus::MemoryLimit);
    assert!(!r.compile.success);
    assert!(!r.compile.timed_out);
    assert!(r.execution.is_none());
}

#[test]
fn run_response_exec_timeout_nulls_exit_code_and_keeps_stdout() {
    let exec = CappedRun {
        exit_code: Some(137),
        stdout: "MOVE DOWN\nMOVE DOWN\n".into(),
        timed_out: true,
        ..CappedRun::default()
    };
    let r = build_run_response(&raw(Some(0)), vec![], Some(exec));
    assert_eq!(r.status, JudgeStatus::Timeout);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, None);
    assert!(e.timed_out);
    assert_eq!(e.stdout, "MOVE DOWN\nMOVE DOWN\n");
}

#[test]
fn run_response_exec_oom_nulls_exit_code_and_keeps_stdout() {
    let exec = CappedRun {
        exit_code: Some(137),
        stdout: "before\n".into(),
        ..CappedRun::default()
    };
    let r = build_run_response(&raw(Some(0)), vec![], Some(exec));
    assert_eq!(r.status, JudgeStatus::MemoryLimit);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, None, "OOM 不原樣回 137");
    assert!(!e.timed_out);
    assert_eq!(e.stdout, "before\n");
}

#[test]
fn run_response_output_limit() {
    let exec = CappedRun {
        exit_code: None,
        stdout: "x".repeat(10),
        stdout_truncated: true,
        ..CappedRun::default()
    };
    let r = build_run_response(&raw(Some(0)), vec![], Some(exec));
    assert_eq!(r.status, JudgeStatus::OutputLimit);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, None);
    assert!(e.stdout_truncated);
    assert_eq!(e.stdout, "x".repeat(10));
}

#[test]
fn run_response_panic_keeps_exit_101_and_stderr() {
    let stderr = "\nthread 'main' (12) panicked at /tmp/main.rs:15:11:\nRefCell already borrowed\n";
    let exec = CappedRun {
        exit_code: Some(101),
        stderr: stderr.into(),
        ..CappedRun::default()
    };
    let r = build_run_response(&raw(Some(0)), vec![], Some(exec));
    assert_eq!(r.status, JudgeStatus::RuntimePanic);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, Some(101));
    assert_eq!(e.stderr, stderr, "Run 的 stderr 原樣回傳");
}

// ───────── 回應組裝：Submit ─────────

fn sum_names() -> Vec<String> {
    names(&["sums_three", "empty_is_zero", "all_ones"])
}

fn statuses(tests: &[TestResult]) -> Vec<TestStatus> {
    tests.iter().map(|t| t.status).collect()
}

#[test]
fn submit_response_all_pass() {
    let exec = CappedRun {
        exit_code: Some(0),
        stdout: read_fixture("libtest/all_pass.txt"),
        ..CappedRun::default()
    };
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &sum_names());
    assert_eq!(r.status, JudgeStatus::Ok);
    assert!(r.passed);
    assert_eq!(statuses(&r.tests), vec![TestStatus::Passed; 3]);
    assert_eq!(r.tests[0].name, "sums_three");
}

#[test]
fn submit_response_partial_fail() {
    let exec = CappedRun {
        exit_code: Some(101),
        stdout: read_fixture("libtest/partial_fail.txt"),
        ..CappedRun::default()
    };
    let r = build_submit_response(&raw(Some(0)), vec![warning()], Some(exec), &sum_names());
    assert_eq!(r.status, JudgeStatus::TestFailed);
    assert!(!r.passed);
    assert_eq!(r.compile.diagnostics, vec![warning()]);
    assert_eq!(
        statuses(&r.tests),
        vec![TestStatus::Failed, TestStatus::Passed, TestStatus::Passed]
    );
    assert_eq!(r.execution.expect("執行結果").exit_code, Some(101));
}

#[test]
fn submit_response_compile_failures_mark_all_not_run() {
    for (compile, status) in [
        (raw(Some(1)), JudgeStatus::CompileError),
        (timed_out(None), JudgeStatus::Timeout),
        (raw(Some(137)), JudgeStatus::MemoryLimit),
    ] {
        let r = build_submit_response(&compile, vec![], None, &sum_names());
        assert_eq!(r.status, status);
        assert!(!r.passed);
        assert!(r.execution.is_none());
        assert_eq!(
            statuses(&r.tests),
            vec![TestStatus::NotRun; 3],
            "{status:?}"
        );
        assert_eq!(r.tests.len(), 3, "永遠列出全部隱藏測試");
    }
}

#[test]
fn submit_response_exec_timeout_keeps_finished_results() {
    let exec = CappedRun {
        exit_code: None,
        stdout: read_fixture("libtest/killed_midway.txt"),
        timed_out: true,
        ..CappedRun::default()
    };
    let declared = names(&["a_first", "b_second", "c_hangs", "d_never"]);
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &declared);
    assert_eq!(r.status, JudgeStatus::Timeout);
    assert!(!r.passed);
    assert_eq!(
        statuses(&r.tests),
        vec![
            TestStatus::Passed,
            TestStatus::Passed,
            TestStatus::NotRun,
            TestStatus::NotRun
        ]
    );
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, None);
    assert!(e.timed_out);
}

#[test]
fn submit_response_output_limit() {
    let exec = CappedRun {
        exit_code: None,
        stdout_truncated: true,
        ..CappedRun::default()
    };
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &sum_names());
    assert_eq!(r.status, JudgeStatus::OutputLimit);
    assert!(!r.passed);
    assert_eq!(statuses(&r.tests), vec![TestStatus::NotRun; 3]);
}

// ───────── docker 不可用（docker_bin 指向不存在的路徑，不需 Docker）─────────

fn unreachable_judge() -> rust_quest_backend::judge::Judge {
    rust_quest_backend::judge::Judge::new(JudgeConfig {
        docker_bin: "/nonexistent/docker",
        ..JudgeConfig::default()
    })
}

#[tokio::test]
async fn health_reports_docker_unreachable_when_docker_bin_missing() {
    assert_eq!(
        unreachable_judge().health().await,
        JudgeHealth {
            available: false,
            reason: Some(UnavailableReason::DockerUnreachable)
        }
    );
}

#[tokio::test]
async fn run_and_submit_are_unavailable_when_docker_bin_missing() {
    let store = common::load_fixture_store();
    let level = store.get("02-plain-sum").expect("02-plain-sum");
    let judge = unreachable_judge();
    let run = judge.run(level, "fn main() {}\n").await;
    assert!(
        matches!(
            run,
            Err(rust_quest_backend::judge::JudgeError::Unavailable(_))
        ),
        "{run:?}"
    );
    let submit = judge
        .submit(level, store.grid_sim(), "fn main() {}\n")
        .await;
    assert!(
        matches!(
            submit,
            Err(rust_quest_backend::judge::JudgeError::Unavailable(_))
        ),
        "{submit:?}"
    );
}

// ───────── v1.3 ④ Submit 測試缺結果 ─────────

#[test]
fn status_submit_exit_0_with_missing_result_is_test_failed() {
    let c = raw(Some(0));
    let tests = [test("a", TestStatus::Passed), test("b", TestStatus::NotRun)];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&raw(Some(0))), &tests),
        JudgeStatus::TestFailed
    );
}

#[test]
fn status_submit_nonzero_exit_without_failed_test_is_test_failed() {
    let c = raw(Some(0));
    for tests in [
        vec![test("a", TestStatus::Passed), test("b", TestStatus::NotRun)],
        vec![test("a", TestStatus::Passed), test("b", TestStatus::Passed)],
    ] {
        assert_eq!(
            determine_status(Mode::Submit, &c, Some(&raw(Some(101))), &tests),
            JudgeStatus::TestFailed,
            "{tests:?}"
        );
    }
}

#[test]
fn status_submit_missing_results_keep_kill_precedence() {
    let c = raw(Some(0));
    let tests = [test("a", TestStatus::Passed), test("b", TestStatus::NotRun)];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&timed_out(None)), &tests),
        JudgeStatus::Timeout
    );
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&raw(Some(137))), &tests),
        JudgeStatus::MemoryLimit
    );
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&truncated(None)), &tests),
        JudgeStatus::OutputLimit
    );
}

#[test]
fn submit_response_exit_0_with_missing_test_marks_not_run_and_fails() {
    let exec = CappedRun {
        exit_code: Some(0),
        stdout: read_fixture("libtest/all_pass.txt"),
        ..CappedRun::default()
    };
    let declared = names(&["sums_three", "empty_is_zero", "all_ones", "extra_case"]);
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &declared);
    assert_eq!(r.status, JudgeStatus::TestFailed);
    assert!(!r.passed, "只有全部宣告測試都 passed 才為真");
    assert_eq!(
        statuses(&r.tests),
        vec![
            TestStatus::Passed,
            TestStatus::Passed,
            TestStatus::Passed,
            TestStatus::NotRun
        ]
    );
}

#[test]
fn submit_response_nonzero_exit_without_failure_marks_unfinished_not_run() {
    let exec = CappedRun {
        exit_code: Some(101),
        stdout: read_fixture("libtest/killed_midway.txt"),
        ..CappedRun::default()
    };
    let declared = names(&["a_first", "b_second", "c_hangs", "d_never"]);
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &declared);
    assert_eq!(r.status, JudgeStatus::TestFailed);
    assert!(!r.passed);
    assert_eq!(
        statuses(&r.tests),
        vec![
            TestStatus::Passed,
            TestStatus::Passed,
            TestStatus::NotRun,
            TestStatus::NotRun
        ]
    );
}

// ───────── 驗收補測：序列化、編譯 exit、執行 OOM ─────────

#[test]
fn judge_status_and_enums_serialize_snake_case() {
    use serde_json::{Value, to_value};
    for (status, text) in [
        (JudgeStatus::Ok, "ok"),
        (JudgeStatus::CompileError, "compile_error"),
        (JudgeStatus::RuntimePanic, "runtime_panic"),
        (JudgeStatus::TestFailed, "test_failed"),
        (JudgeStatus::Timeout, "timeout"),
        (JudgeStatus::MemoryLimit, "memory_limit"),
        (JudgeStatus::OutputLimit, "output_limit"),
    ] {
        assert_eq!(to_value(status).unwrap(), Value::from(text));
    }
    assert_eq!(to_value(TestStatus::Passed).unwrap(), Value::from("passed"));
    assert_eq!(to_value(TestStatus::Failed).unwrap(), Value::from("failed"));
    assert_eq!(
        to_value(TestStatus::NotRun).unwrap(),
        Value::from("not_run")
    );
    assert_eq!(
        to_value(DiagnosticLevel::Error).unwrap(),
        Value::from("error")
    );
    assert_eq!(
        to_value(DiagnosticLevel::Warning).unwrap(),
        Value::from("warning")
    );
    assert_eq!(
        to_value(DiagnosticLevel::Note).unwrap(),
        Value::from("note")
    );
    assert_eq!(
        to_value(DiagnosticLevel::Help).unwrap(),
        Value::from("help")
    );
}

#[test]
fn run_response_compile_error_serializes_with_null_execution() {
    use rust_quest_backend::judge::outcome::{CompileResult, RunResponse};
    let linker = Diagnostic {
        code: None,
        message: "error: linking with `cc` failed".into(),
        line: None,
        column: None,
        end_line: None,
        end_column: None,
        level: DiagnosticLevel::Error,
        explanation_url: None,
        rendered: Some("error: linking with `cc` failed".into()),
    };
    let e0382 = Diagnostic {
        code: Some("E0382".into()),
        message: "borrow of moved value: `bag`".into(),
        line: Some(10),
        column: Some(17),
        end_line: Some(10),
        end_column: Some(20),
        level: DiagnosticLevel::Error,
        explanation_url: Some("https://doc.rust-lang.org/error_codes/E0382.html".into()),
        rendered: Some("error[E0382]: borrow of moved value".into()),
    };
    let resp = RunResponse {
        status: JudgeStatus::CompileError,
        compile: CompileResult {
            success: false,
            timed_out: false,
            diagnostics: vec![e0382, linker],
        },
        execution: None,
    };
    assert_eq!(
        serde_json::to_value(&resp).unwrap(),
        serde_json::json!({
            "status": "compile_error",
            "compile": {
                "success": false,
                "timed_out": false,
                "diagnostics": [
                    {
                        "code": "E0382",
                        "message": "borrow of moved value: `bag`",
                        "line": 10,
                        "column": 17,
                        "end_line": 10,
                        "end_column": 20,
                        "level": "error",
                        "explanation_url": "https://doc.rust-lang.org/error_codes/E0382.html",
                        "rendered": "error[E0382]: borrow of moved value"
                    },
                    {
                        "code": null,
                        "message": "error: linking with `cc` failed",
                        "line": null,
                        "column": null,
                        "end_line": null,
                        "end_column": null,
                        "level": "error",
                        "explanation_url": null,
                        "rendered": "error: linking with `cc` failed"
                    }
                ]
            },
            "execution": null
        })
    );
}

#[test]
fn status_compile_nonzero_other_than_1_is_compile_error() {
    for code in [101, 2] {
        assert_eq!(
            determine_status(Mode::Run, &raw(Some(code)), None, &[]),
            JudgeStatus::CompileError,
            "編譯 exit {code}"
        );
    }
}

#[test]
fn submit_response_exec_oom() {
    let exec = CappedRun {
        exit_code: Some(137),
        stdout: read_fixture("libtest/killed_midway.txt"),
        ..CappedRun::default()
    };
    let declared = names(&["a_first", "b_second", "c_hangs", "d_never"]);
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &declared);
    assert_eq!(r.status, JudgeStatus::MemoryLimit);
    assert!(!r.passed);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, None, "OOM 不原樣回 137");
    assert!(!e.timed_out);
    assert_eq!(
        statuses(&r.tests),
        vec![
            TestStatus::Passed,
            TestStatus::Passed,
            TestStatus::NotRun,
            TestStatus::NotRun
        ]
    );
}

// ───────── v1.4 程式 abort（stack overflow，exit 134）─────────

const ABORT_EXIT: i32 = 134;

fn abort_exit(file: &str) -> i32 {
    read_fixture(&format!("abort/{file}"))
        .trim()
        .parse()
        .expect("exit code fixture")
}

#[test]
fn abort_fixtures_are_real_exit_134() {
    assert_eq!(abort_exit("run_exit.txt"), ABORT_EXIT);
    assert_eq!(abort_exit("submit_exit.txt"), ABORT_EXIT);
}

#[test]
fn status_run_abort_134_is_runtime_panic() {
    let c = raw(Some(0));
    assert_eq!(
        determine_status(Mode::Run, &c, Some(&raw(Some(ABORT_EXIT))), &[]),
        JudgeStatus::RuntimePanic
    );
}

#[test]
fn status_submit_abort_134_with_passed_and_not_run_is_test_failed() {
    let c = raw(Some(0));
    let tests = [
        test("a", TestStatus::Passed),
        test("b", TestStatus::Passed),
        test("c", TestStatus::NotRun),
    ];
    assert_eq!(
        determine_status(Mode::Submit, &c, Some(&raw(Some(ABORT_EXIT))), &tests),
        JudgeStatus::TestFailed
    );
}

#[test]
fn status_abort_134_is_not_memory_limit() {
    let c = raw(Some(0));
    let tests = [test("a", TestStatus::NotRun)];
    for (mode, tests) in [(Mode::Run, &[][..]), (Mode::Submit, &tests[..])] {
        assert_ne!(
            determine_status(mode, &c, Some(&raw(Some(ABORT_EXIT))), tests),
            JudgeStatus::MemoryLimit,
            "{mode:?}"
        );
    }
}

#[test]
fn run_response_abort_keeps_exit_134_and_overflow_stderr() {
    let exec = CappedRun {
        exit_code: Some(abort_exit("run_exit.txt")),
        stdout: read_fixture("abort/run_stdout.txt"),
        stderr: read_fixture("abort/run_stderr.txt"),
        ..CappedRun::default()
    };
    let r = build_run_response(&raw(Some(0)), vec![], Some(exec));
    assert_eq!(r.status, JudgeStatus::RuntimePanic);
    let e = r.execution.expect("執行結果");
    assert_eq!(
        e.exit_code,
        Some(ABORT_EXIT),
        "非 host kill，exit_code 照實回傳"
    );
    assert_eq!(
        e.stdout,
        read_fixture("abort/run_stdout.txt"),
        "abort 前的輸出保留"
    );
    assert_eq!(e.stderr, read_fixture("abort/run_stderr.txt"));
    assert!(e.stderr.contains("has overflowed its stack"));
}

#[test]
fn submit_response_abort_marks_finished_passed_rest_not_run_and_keeps_stderr() {
    let exec = CappedRun {
        exit_code: Some(abort_exit("submit_exit.txt")),
        stdout: read_fixture("abort/submit_stdout.txt"),
        stderr: read_fixture("abort/submit_stderr.txt"),
        ..CappedRun::default()
    };
    let declared = names(&["a_first", "b_overflows", "c_never"]);
    let r = build_submit_response(&raw(Some(0)), vec![], Some(exec), &declared);
    assert_eq!(r.status, JudgeStatus::TestFailed);
    assert!(!r.passed);
    assert_eq!(
        statuses(&r.tests),
        vec![TestStatus::Passed, TestStatus::NotRun, TestStatus::NotRun]
    );
    assert!(r.tests.iter().all(|t| t.message.is_none()), "{:?}", r.tests);
    let e = r.execution.expect("執行結果");
    assert_eq!(e.exit_code, Some(ABORT_EXIT));
    assert!(!e.timed_out);
    assert_eq!(
        e.stderr,
        read_fixture("abort/submit_stderr.txt"),
        "stderr 保留 overflow 訊息"
    );
    assert!(e.stderr.contains("fatal runtime error: stack overflow"));
}
