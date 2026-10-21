//! S2-3 loader timeline：visual 列舉、組合 starter／solution、互斥規則、timeline.rs 缺檔（architecture §2.2）

mod common;

use common::{ContentDir, load_fixture_store, names, read_fixture};
use rust_quest_backend::content::loader::attach_timeline;
use rust_quest_backend::content::model::{LevelKind, LevelVisual};

const TL: &str = "04-bridge-timeline";
const HELPER_REL: &str = "shared/timeline.rs";

fn helper() -> String {
    read_fixture("content/shared/timeline.rs")
}

fn raw(file: &str) -> String {
    read_fixture(&format!("content/levels/{TL}/{file}"))
}

/// 規格組法：原文、缺結尾換行才補、一個空行、小幫手全文
fn composed(src: &str, helper: &str) -> String {
    let mut out = src.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(helper);
    out
}

// ───────── 組合規則（純函式）─────────

#[test]
fn attach_appends_blank_line_and_helper_when_src_ends_with_newline() {
    assert_eq!(
        attach_timeline("fn main() {}\n", "mod timeline {}\n"),
        "fn main() {}\n\nmod timeline {}\n"
    );
}

#[test]
fn attach_adds_missing_trailing_newline_once() {
    assert_eq!(
        attach_timeline("fn main() {}", "mod timeline {}\n"),
        "fn main() {}\n\nmod timeline {}\n"
    );
}

#[test]
fn attach_does_not_trim_existing_blank_lines() {
    assert_eq!(
        attach_timeline("fn main() {}\n\n", "mod timeline {}\n"),
        "fn main() {}\n\n\nmod timeline {}\n"
    );
}

#[test]
fn attach_keeps_helper_verbatim_without_adding_newline() {
    assert_eq!(
        attach_timeline("fn main() {}\n", "mod timeline {}"),
        "fn main() {}\n\nmod timeline {}"
    );
}

// ───────── 載入 fixture 的 timeline 關卡 ─────────

#[test]
fn timeline_level_parses_visual_timeline_without_grid_or_snapshots() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("應載入 04-bridge-timeline");
    assert_eq!(level.visual, LevelVisual::Timeline);
    assert_eq!(level.kind, LevelKind::Debug);
    assert!(level.grid.is_none());
    assert!(level.memory_snapshots.is_none());
    assert!(
        level.level_grid_src.is_none(),
        "timeline 不產生 level_grid()"
    );
}

#[test]
fn timeline_starter_is_raw_starter_blank_line_and_helper() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    assert_eq!(level.starter, composed(&raw("starter.rs"), &helper()));
}

#[test]
fn timeline_solution_is_composed_with_same_rule() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    assert_eq!(level.solution, composed(&raw("solution.rs"), &helper()));
}

#[test]
fn timeline_starter_without_trailing_newline_gets_one() {
    let cd = ContentDir::new();
    let rel = format!("levels/{TL}/starter.rs");
    let trimmed = raw("starter.rs").trim_end().to_string();
    cd.write(&rel, &trimmed);
    let store = cd.load().expect("應可載入");
    let level = store.get(TL).expect("timeline 關卡");
    assert_eq!(level.starter, format!("{trimmed}\n\n{}", helper()));
}

#[test]
fn timeline_solution_without_trailing_newline_gets_one() {
    let cd = ContentDir::new();
    let rel = format!("levels/{TL}/solution.rs");
    let trimmed = raw("solution.rs").trim_end().to_string();
    cd.write(&rel, &trimmed);
    let store = cd.load().expect("應可載入");
    let level = store.get(TL).expect("timeline 關卡");
    assert_eq!(level.solution, format!("{trimmed}\n\n{}", helper()));
}

#[test]
fn timeline_player_line_numbers_unchanged_and_helper_after_blank_line() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    let raw_starter = raw("starter.rs");
    let raw_lines: Vec<&str> = raw_starter.lines().collect();
    let lines: Vec<&str> = level.starter.lines().collect();
    let n = raw_lines.len();
    assert!(lines.len() > n + 1, "組合後應比原文多出小幫手");
    assert_eq!(&lines[..n], &raw_lines[..], "玩家程式碼行號不變");
    assert_eq!(lines[n], "", "原文與小幫手之間隔一行");
    assert_eq!(
        lines[n + 1],
        helper().lines().next().expect("小幫手第一行"),
        "小幫手從第 n + 2 行開始"
    );
}

#[test]
fn timeline_helper_is_read_from_content_shared_at_load() {
    let cd = ContentDir::new();
    let custom = "// 改過的小幫手\nmod timeline {}\n";
    cd.write(HELPER_REL, custom);
    let store = cd.load().expect("應可載入");
    let level = store.get(TL).expect("timeline 關卡");
    assert_eq!(level.starter, composed(&raw("starter.rs"), custom));
    assert_eq!(level.solution, composed(&raw("solution.rs"), custom));
}

#[test]
fn timeline_tests_and_test_names_are_not_composed() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    assert_eq!(level.tests, raw("tests.rs"));
    assert_eq!(level.test_names, names(&["both_cross"]));
}

#[test]
fn non_timeline_levels_do_not_get_helper() {
    let store = load_fixture_store();
    for id in [
        "01-walk-grid",
        "09-late-grid",
        "02-plain-sum",
        "03-move-memory",
    ] {
        let level = store.get(id).expect("fixture 關卡");
        assert_eq!(
            level.starter,
            read_fixture(&format!("content/levels/{id}/starter.rs")),
            "{id} starter 不接小幫手"
        );
        assert_eq!(
            level.solution,
            read_fixture(&format!("content/levels/{id}/solution.rs")),
            "{id} solution 不接小幫手"
        );
    }
}

// ───────── 互斥規則與缺檔 ─────────

#[test]
fn timeline_with_grid_fails() {
    let cd = ContentDir::new();
    let rel = format!("levels/{TL}/level.toml");
    let s = cd.read(&rel);
    cd.write(
        &rel,
        &format!("{s}\n[grid]\nwidth = 3\nheight = 3\nstart = [0, 0]\ngoal = [2, 2]\n"),
    );
    cd.assert_load_fails(TL);
}

#[test]
fn timeline_with_memory_snapshots_fails() {
    let cd = ContentDir::new();
    let rel = format!("levels/{TL}/level.toml");
    let s = cd.read(&rel);
    cd.write(
        &rel,
        &format!("{s}\n[[memory_snapshots]]\nline = 1\nnote = \"不該出現\"\n"),
    );
    cd.assert_load_fails(TL);
}

#[test]
fn missing_timeline_rs_fails_startup() {
    let cd = ContentDir::new();
    cd.remove(HELPER_REL);
    cd.assert_load_fails("timeline.rs");
}

#[test]
fn missing_timeline_rs_fails_even_without_timeline_levels() {
    let cd = ContentDir::new();
    cd.remove(&format!("levels/{TL}"));
    cd.remove(HELPER_REL);
    cd.assert_load_fails("timeline.rs");
}
