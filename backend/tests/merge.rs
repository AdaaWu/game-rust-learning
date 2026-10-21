//! B4a 合併檔組成：使用者程式碼 →（缺結尾換行才補）換行 → 隱藏模組開頭 →（grid）模擬器與地圖 → tests.rs → 結尾

mod common;

use std::fs;
use std::process::Command;

use common::{load_fixture_store, read_fixture};
use rust_quest_backend::judge::merge::{MergedSource, merge_submit};

const HEADER: &str = "#[cfg(test)]\n#[allow(dead_code)]\n#[allow(unused_imports)]\nmod __quest_hidden_tests {\n    use super::*;\n";
const TESTS: &str = "#[test]\nfn t() {\n    assert_eq!(f(), 1);\n}\n";

#[test]
fn non_grid_level_merges_without_simulator() {
    let user = "fn f() -> i32 {\n    1\n}\n";
    let merged = merge_submit(user, TESTS, None);
    assert_eq!(
        merged,
        MergedSource {
            source: format!("{user}{HEADER}{TESTS}}}\n"),
            user_line_count: 3,
        }
    );
}

#[test]
fn grid_level_merges_simulator_then_level_grid_before_tests() {
    let user = "fn f() -> i32 {\n    1\n}\n";
    let sim = "struct Grid {}\nfn simulate() {}\n";
    let level_grid = "fn level_grid() -> Grid { Grid {} }";
    let merged = merge_submit(user, TESTS, Some((sim, level_grid)));
    assert_eq!(
        merged.source,
        format!("{user}{HEADER}{sim}\n{level_grid}\n{TESTS}}}\n")
    );
    assert_eq!(merged.user_line_count, 3);
}

#[test]
fn user_code_without_trailing_newline_does_not_glue_to_header() {
    let user = "fn f() -> i32 { 1 }";
    let merged = merge_submit(user, TESTS, None);
    assert!(
        merged
            .source
            .starts_with("fn f() -> i32 { 1 }\n#[cfg(test)]\n"),
        "{}",
        merged.source
    );
    assert_eq!(merged.user_line_count, 1);
}

#[test]
fn user_line_count_matches_editor_lines() {
    assert_eq!(merge_submit("a\nb\nc\n", TESTS, None).user_line_count, 3);
    assert_eq!(merge_submit("a\nb\nc", TESTS, None).user_line_count, 3);
    assert_eq!(merge_submit("only", TESTS, None).user_line_count, 1);
}

#[test]
fn user_code_is_placed_first_unchanged() {
    let user = "// 玩家自己的 mod tests\nmod tests {\n    #[test]\n    fn mine() {}\n}\nfn f() -> i32 { 1 }\n";
    let merged = merge_submit(user, TESTS, None);
    assert!(merged.source.starts_with(user));
    assert_eq!(merged.user_line_count, 6);
}

#[test]
fn hidden_module_closes_with_brace_and_newline() {
    let merged = merge_submit("fn f() -> i32 { 1 }\n", TESTS, None);
    assert!(
        merged.source.ends_with(&format!("{TESTS}}}\n")),
        "{}",
        merged.source
    );
}

/// 用 loader 產出的 grid 關卡實際合併、host rustc 編譯並跑測試：solution 通過
#[test]
#[ignore = "需要 host rustc"]
fn merged_grid_solution_compiles_and_passes_with_host_rustc() {
    let store = load_fixture_store();
    let level = store.get("01-walk-grid").expect("01-walk-grid");
    let merged = merge_submit(
        &level.solution,
        &level.tests,
        Some((
            store.grid_sim(),
            level.level_grid_src.as_deref().expect("level_grid"),
        )),
    );
    assert_eq!(merged.user_line_count, level.solution.lines().count());
    assert!(
        merged
            .source
            .contains(&read_fixture("content/shared/grid_sim.rs"))
    );

    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.rs");
    let bin = dir.path().join("main");
    fs::write(&main, &merged.source).unwrap();
    let out = Command::new("rustc")
        .args(["--edition", "2024", "--test", "-A", "dead_code", "-o"])
        .arg(&bin)
        .arg(&main)
        .output()
        .expect("執行 rustc");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin)
        .args([
            "--test-threads=1",
            "--exact",
            "__quest_hidden_tests::ferris_reaches_exit",
        ])
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stdout)
    );
}

#[test]
fn user_code_ending_with_newline_gets_no_extra_newline() {
    let merged = merge_submit("fn f() -> i32 { 1 }\n", TESTS, None);
    assert!(
        merged
            .source
            .starts_with(&format!("fn f() -> i32 {{ 1 }}\n{HEADER}")),
        "{}",
        merged.source
    );
}

#[test]
fn trailing_blank_lines_are_kept_as_is() {
    let user = "fn f() -> i32 { 1 }\n\n\n";
    let merged = merge_submit(user, TESTS, None);
    assert!(
        merged.source.starts_with(&format!("{user}{HEADER}")),
        "{}",
        merged.source
    );
    assert_eq!(merged.user_line_count, 3, "user_code.lines().count()");
}

#[test]
fn hidden_header_allows_unused_imports() {
    let merged = merge_submit("fn f() -> i32 { 1 }\n", TESTS, None);
    assert!(
        merged.source.contains(
            "#[allow(dead_code)]\n#[allow(unused_imports)]\nmod __quest_hidden_tests {\n"
        ),
        "{}",
        merged.source
    );
}

#[test]
fn tests_rs_without_trailing_newline_gets_newline_before_closing_brace() {
    let tests = "#[test]\nfn t() {}";
    let merged = merge_submit("fn f() -> i32 { 1 }\n", tests, None);
    assert!(
        merged.source.ends_with("fn t() {}\n}\n"),
        "{}",
        merged.source
    );
}
