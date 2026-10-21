//! B3 libtest 文字輸出解析：passed/failed/not_run、panic message、排序

mod common;

use common::{names, read_fixture};
use rust_quest_backend::judge::test_output::{
    TestResult, TestStatus, all_not_run, parse_test_output,
};

fn fixture(name: &str) -> String {
    read_fixture(&format!("libtest/{name}"))
}

fn passed(name: &str) -> TestResult {
    TestResult {
        name: name.into(),
        status: TestStatus::Passed,
        message: None,
    }
}

fn not_run(name: &str) -> TestResult {
    TestResult {
        name: name.into(),
        status: TestStatus::NotRun,
        message: None,
    }
}

fn failed(name: &str, message: &str) -> TestResult {
    TestResult {
        name: name.into(),
        status: TestStatus::Failed,
        message: Some(message.into()),
    }
}

/// 02-plain-sum 的宣告順序（與 libtest 的字母序不同）
fn sum_names() -> Vec<String> {
    names(&["sums_three", "empty_is_zero", "all_ones"])
}

#[test]
fn all_pass_in_declaration_order_without_prefix() {
    let results = parse_test_output(&fixture("all_pass.txt"), &sum_names());
    assert_eq!(
        results,
        vec![
            passed("sums_three"),
            passed("empty_is_zero"),
            passed("all_ones")
        ]
    );
}

#[test]
fn failed_test_message_strips_thread_line_and_backtrace_note() {
    let results = parse_test_output(&fixture("partial_fail.txt"), &sum_names());
    assert_eq!(
        results,
        vec![
            failed(
                "sums_three",
                "assertion `left == right` failed: 總和不對\n  left: 14\n right: 6"
            ),
            passed("empty_is_zero"),
            passed("all_ones"),
        ]
    );
}

#[test]
fn thread_line_without_thread_id_is_also_stripped() {
    let results = parse_test_output(&fixture("partial_fail_no_thread_id.txt"), &sum_names());
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0],
        failed(
            "sums_three",
            "assertion `left == right` failed: 總和不對\n  left: 14\n right: 6"
        )
    );
}

#[test]
fn mixed_results_messages_ignored_and_player_tests() {
    let declared = names(&[
        "zeta_assert_msg",
        "alpha_passes",
        "mid_plain_panic",
        "beta_ignored",
        "gamma_multiline",
    ]);
    let results = parse_test_output(&fixture("mixed_with_player_test.txt"), &declared);
    assert_eq!(
        results,
        vec![
            failed(
                "zeta_assert_msg",
                "assertion `left == right` failed: 海星數量不對\n  left: 2\n right: 3"
            ),
            passed("alpha_passes"),
            failed("mid_plain_panic", "背包破了"),
            not_run("beta_ignored"),
            failed("gamma_multiline", "第一行\n第二行"),
        ],
        "ignored 視為 not_run；玩家的 tests::mine 不列入"
    );
}

#[test]
fn killed_midway_marks_running_and_remaining_as_not_run() {
    let declared = names(&["a_first", "b_second", "c_hangs", "d_never"]);
    let results = parse_test_output(&fixture("killed_midway.txt"), &declared);
    assert_eq!(
        results,
        vec![
            passed("a_first"),
            passed("b_second"),
            not_run("c_hangs"),
            not_run("d_never"),
        ]
    );
}

#[test]
fn declared_test_missing_from_output_is_not_run() {
    let declared = names(&["sums_three", "empty_is_zero", "all_ones", "extra_case"]);
    let results = parse_test_output(&fixture("all_pass.txt"), &declared);
    assert_eq!(results.len(), 4);
    assert_eq!(results[3], not_run("extra_case"));
}

#[test]
fn empty_stdout_gives_all_not_run() {
    let results = parse_test_output("", &sum_names());
    assert_eq!(
        results,
        vec![
            not_run("sums_three"),
            not_run("empty_is_zero"),
            not_run("all_ones")
        ]
    );
}

#[test]
fn all_not_run_lists_every_test_in_order() {
    assert_eq!(
        all_not_run(&sum_names()),
        vec![
            not_run("sums_three"),
            not_run("empty_is_zero"),
            not_run("all_ones")
        ]
    );
}

#[test]
fn all_not_run_with_no_tests_is_empty() {
    assert_eq!(all_not_run(&[]), vec![]);
}
