//! B2 rustc JSON 診斷解析：欄位對應、過濾、隱藏區段行號、上限

mod common;

use common::read_fixture;
use rust_quest_backend::judge::diagnostics::{Diagnostic, DiagnosticLevel, parse_diagnostics};

fn fixture(name: &str) -> String {
    read_fixture(&format!("rustc/{name}"))
}

/// 取 fixture 第 n 行 JSON 的 rendered 原文
fn rendered_of(name: &str, n: usize) -> String {
    let line = fixture(name).lines().nth(n).unwrap().to_string();
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    v["rendered"].as_str().unwrap().to_string()
}

fn positions(d: &Diagnostic) -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    (d.line, d.column, d.end_line, d.end_column)
}

// ───────── 欄位對應 ─────────

#[test]
fn e0382_maps_every_field_from_primary_span() {
    let diags = parse_diagnostics(&fixture("run_e0382_move.jsonl"), None);
    assert_eq!(diags.len(), 1, "aborting 與 failure-note 應被過濾");
    assert_eq!(
        diags[0],
        Diagnostic {
            code: Some("E0382".into()),
            message: "borrow of moved value: `bag`".into(),
            line: Some(10),
            column: Some(17),
            end_line: Some(10),
            end_column: Some(20),
            level: DiagnosticLevel::Error,
            explanation_url: Some("https://doc.rust-lang.org/error_codes/E0382.html".into()),
            rendered: Some(rendered_of("run_e0382_move.jsonl", 0)),
        }
    );
}

#[test]
fn syntax_error_without_code_has_null_code_and_url() {
    let diags = parse_diagnostics(&fixture("run_syntax_error.jsonl"), None);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, None);
    assert_eq!(diags[0].message, "expected expression, found `;`");
    assert_eq!(positions(&diags[0]), (Some(2), Some(13), Some(2), Some(14)));
    assert_eq!(diags[0].explanation_url, None);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn warnings_are_kept_with_lint_name_as_code_in_order() {
    let diags = parse_diagnostics(&fixture("run_warnings_only.jsonl"), None);
    let codes: Vec<Option<&str>> = diags.iter().map(|d| d.code.as_deref()).collect();
    assert_eq!(
        codes,
        vec![
            Some("unused_mut"),
            Some("unused_variables"),
            Some("unused_variables")
        ],
        "warning 依 rustc 順序保留，`3 warnings emitted` 被過濾"
    );
    assert!(diags.iter().all(|d| d.level == DiagnosticLevel::Warning));
    assert!(
        diags.iter().all(|d| d.explanation_url.is_none()),
        "lint 名不產生說明連結"
    );
    assert_eq!(diags[1].message, "unused variable: `unused`");
    assert_eq!(positions(&diags[1]), (Some(2), Some(9), Some(2), Some(15)));
}

#[test]
fn e0601_zero_width_span_is_kept() {
    let diags = parse_diagnostics(&fixture("run_e0601_no_main.jsonl"), None);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E0601"));
    assert_eq!(positions(&diags[0]), (Some(1), Some(15), Some(1), Some(15)));
}

#[test]
fn multiline_primary_span_keeps_end_position() {
    let diags = parse_diagnostics(&fixture("run_multiline_span.jsonl"), None);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E0308"));
    assert_eq!(positions(&diags[0]), (Some(2), Some(19), Some(4), Some(6)));
}

#[test]
fn internal_compiler_error_level_maps_to_error() {
    let diags = parse_diagnostics(&fixture("synthetic_ice_level.jsonl"), None);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

// ───────── 過濾與非 JSON ─────────

#[test]
fn summary_and_failure_note_are_filtered() {
    let diags = parse_diagnostics(&fixture("submit_hidden_e0425.jsonl"), Some(1));
    assert!(
        diags
            .iter()
            .all(|d| !d.message.starts_with("aborting due to")
                && !d.message.contains("warning emitted")),
        "摘要訊息不應回傳"
    );
    assert!(
        diags
            .iter()
            .all(|d| !d.message.starts_with("For more information")),
        "failure-note 不應回傳"
    );
    assert_eq!(diags.len(), 3);
}

#[test]
fn non_json_line_becomes_error_with_raw_text() {
    let diags = parse_diagnostics(&fixture("synthetic_non_json_line.jsonl"), None);
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].code.as_deref(), Some("E0382"));
    let raw = "error: linking with `cc` failed: exit status: 1";
    assert_eq!(
        diags[1],
        Diagnostic {
            code: None,
            message: raw.into(),
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            level: DiagnosticLevel::Error,
            explanation_url: None,
            rendered: Some(raw.into()),
        }
    );
}

#[test]
fn empty_or_blank_stderr_gives_no_diagnostics() {
    assert_eq!(parse_diagnostics("", None), vec![]);
    assert_eq!(parse_diagnostics("\n\n", Some(3)), vec![]);
}

#[test]
fn at_most_fifty_diagnostics_are_returned() {
    let diags = parse_diagnostics(&fixture("run_sixty_errors.jsonl"), None);
    assert_eq!(diags.len(), 50);
    assert!(diags.iter().all(|d| d.code.as_deref() == Some("E0308")));
    assert_eq!(diags[0].line, Some(2), "保留 rustc 輸出順序的前 50 筆");
    assert_eq!(diags[49].line, Some(51));
}

// ───────── Submit 行號對應 ─────────

#[test]
fn submit_hidden_section_nulls_positions_and_rendered() {
    // 使用者程式碼 1 行，錯誤全在隱藏測試
    let diags = parse_diagnostics(&fixture("submit_hidden_e0425.jsonl"), Some(1));
    assert_eq!(diags.len(), 3, "隱藏區段的 warning 被過濾，只留 3 筆 error");
    for d in &diags {
        assert_eq!(positions(d), (None, None, None, None), "{}", d.message);
        assert_eq!(d.rendered, None, "隱藏區段不回傳 rendered");
        assert_eq!(d.level, DiagnosticLevel::Error);
    }
    for d in &diags {
        assert_eq!(d.code.as_deref(), Some("E0425"));
        assert_eq!(d.message, "cannot find function `total` in this scope");
        assert_eq!(
            d.explanation_url.as_deref(),
            Some("https://doc.rust-lang.org/error_codes/E0425.html")
        );
    }
}

#[test]
fn run_mode_never_remaps_lines() {
    let diags = parse_diagnostics(&fixture("submit_hidden_e0425.jsonl"), None);
    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].line, Some(9));
    assert!(diags[0].rendered.is_some());
}

#[test]
fn submit_user_section_is_returned_unchanged() {
    // 使用者程式碼 6 行；第 14 行 `spare` 的 warning 在隱藏區段被過濾
    let diags = parse_diagnostics(&fixture("submit_user_and_hidden_warning.jsonl"), Some(6));
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("unused_variables"));
    assert_eq!(positions(&diags[0]), (Some(2), Some(9), Some(2), Some(15)));
    assert_eq!(
        diags[0].rendered,
        Some(rendered_of("submit_user_and_hidden_warning.jsonl", 0))
    );
}

#[test]
fn submit_span_crossing_into_hidden_section_is_clamped() {
    // span 2:19–4:6；使用者程式碼只有 3 行
    let diags = parse_diagnostics(&fixture("run_multiline_span.jsonl"), Some(3));
    assert_eq!(diags.len(), 1);
    assert_eq!(positions(&diags[0]), (Some(2), Some(19), Some(3), None));
    assert!(
        diags[0].rendered.is_some(),
        "起點在使用者區段時保留 rendered"
    );
}

#[test]
fn submit_span_starting_on_last_user_line_is_kept() {
    let diags = parse_diagnostics(&fixture("run_multiline_span.jsonl"), Some(2));
    assert_eq!(diags.len(), 1);
    assert_eq!(positions(&diags[0]), (Some(2), Some(19), Some(2), None));
}

#[test]
fn submit_span_ending_on_last_user_line_is_unchanged() {
    let diags = parse_diagnostics(&fixture("run_multiline_span.jsonl"), Some(4));
    assert_eq!(diags.len(), 1);
    assert_eq!(positions(&diags[0]), (Some(2), Some(19), Some(4), Some(6)));
}

#[test]
fn submit_span_starting_after_user_code_is_hidden() {
    let diags = parse_diagnostics(&fixture("run_multiline_span.jsonl"), Some(1));
    assert_eq!(diags.len(), 1);
    assert_eq!(positions(&diags[0]), (None, None, None, None));
    assert_eq!(diags[0].rendered, None);
    assert_eq!(diags[0].message, "mismatched types");
}

#[test]
fn submit_warning_in_hidden_section_is_dropped() {
    let diags = parse_diagnostics(&fixture("submit_user_and_hidden_warning.jsonl"), Some(6));
    assert!(
        diags.iter().all(|d| !d.message.contains("`spare`")),
        "隱藏區段的 warning 不回傳：{diags:?}"
    );
    assert_eq!(diags.len(), 1);
}

#[test]
fn submit_warning_on_last_user_line_is_kept() {
    // 使用者區段延伸到第 14 行時，兩筆 warning 都屬使用者區段
    let diags = parse_diagnostics(&fixture("submit_user_and_hidden_warning.jsonl"), Some(14));
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[1].message, "unused variable: `spare`");
    assert_eq!(diags[1].line, Some(14));
    assert!(diags[1].rendered.is_some());
}

#[test]
fn cap_of_fifty_applies_after_filtering() {
    // 在 60 筆錯誤前插入會被過濾的摘要與 failure-note，上限仍取 50 筆真正的診斷
    let e0382 = fixture("run_e0382_move.jsonl");
    let noise: Vec<&str> = e0382.lines().skip(1).collect();
    assert_eq!(noise.len(), 2, "fixture 應有 aborting 與 failure-note");
    let stderr = format!(
        "{}\n{}",
        noise.join("\n"),
        fixture("run_sixty_errors.jsonl")
    );
    let diags = parse_diagnostics(&stderr, None);
    assert_eq!(diags.len(), 50);
    assert_eq!(diags[49].line, Some(51));
    assert!(diags.iter().all(|d| d.code.as_deref() == Some("E0308")));
}
