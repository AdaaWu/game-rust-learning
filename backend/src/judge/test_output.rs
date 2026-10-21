//! libtest 文字輸出解析

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Passed,
    Failed,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
}

const HIDDEN_PREFIX: &str = "__quest_hidden_tests::";

/// 去掉 panicked at 前綴行與 backtrace 提示行，再去掉頭尾空行
fn clean_message(lines: &[&str]) -> String {
    let kept: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| !(l.starts_with("thread '") && l.contains(" panicked at ")))
        .filter(|l| !l.starts_with("note: run with `RUST_BACKTRACE=1`"))
        .collect();
    let start = kept.iter().position(|l| !l.trim().is_empty());
    let end = kept.iter().rposition(|l| !l.trim().is_empty());
    match (start, end) {
        (Some(s), Some(e)) => kept[s..=e].join("\n"),
        _ => String::new(),
    }
}

/// 依 ---- <name> stdout ---- 區段收集失敗訊息
fn failure_messages(stdout: &str) -> HashMap<String, String> {
    let mut messages = HashMap::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in stdout.lines() {
        let header = line
            .strip_prefix("---- ")
            .and_then(|r| r.strip_suffix(" stdout ----"));
        if header.is_some() || line == "failures:" {
            if let Some((name, body)) = current.take() {
                messages.insert(name, clean_message(&body));
            }
            current = header.map(|name| (name.to_string(), Vec::new()));
            continue;
        }
        if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((name, body)) = current {
        messages.insert(name, clean_message(&body));
    }
    messages
}

/// test_names 為 tests.rs 宣告順序，結果依此排序
pub fn parse_test_output(stdout: &str, test_names: &[String]) -> Vec<TestResult> {
    let mut results: HashMap<&str, &str> = HashMap::new();
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix("test ") else {
            continue;
        };
        let Some((name, outcome)) = rest.split_once(" ... ") else {
            continue;
        };
        if let Some(name) = name.strip_prefix(HIDDEN_PREFIX) {
            results.insert(name, outcome);
        }
    }
    let messages = failure_messages(stdout);
    test_names
        .iter()
        .map(|name| {
            let (status, message) = match results.get(name.as_str()) {
                Some(&"ok") => (TestStatus::Passed, None),
                Some(&"FAILED") => (
                    TestStatus::Failed,
                    Some(
                        messages
                            .get(&format!("{HIDDEN_PREFIX}{name}"))
                            .cloned()
                            .unwrap_or_default(),
                    ),
                ),
                _ => (TestStatus::NotRun, None),
            };
            TestResult {
                name: name.clone(),
                status,
                message,
            }
        })
        .collect()
}

pub fn all_not_run(test_names: &[String]) -> Vec<TestResult> {
    test_names
        .iter()
        .map(|name| TestResult {
            name: name.clone(),
            status: TestStatus::NotRun,
            message: None,
        })
        .collect()
}
