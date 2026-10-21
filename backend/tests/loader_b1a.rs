//! B1a loader：依 [grid] 產生 level_grid()、驗證 memory_snapshots 的 error

mod common;

use std::fs;
use std::process::Command;

use common::{ContentDir, load_fixture_store};
use rust_quest_backend::content::loader::generate_level_grid;
use rust_quest_backend::content::model::{GridDef, SnapshotError};
use rust_quest_backend::judge::merge::merge_submit;

const GRID: &str = "01-walk-grid";
const GRID2: &str = "09-late-grid";
const PLAIN: &str = "02-plain-sum";
const MEMORY: &str = "03-move-memory";

fn toml(id: &str) -> String {
    format!("levels/{id}/level.toml")
}

// ───────── level_grid() 產生 ─────────

#[test]
fn generates_level_grid_with_empty_vecs_for_omitted_arrays() {
    let grid = GridDef {
        width: 5,
        height: 3,
        start: [0, 0],
        goal: [4, 1],
        walls: vec![[2, 0], [2, 1]],
        gems: vec![],
        doors: vec![],
    };
    assert_eq!(
        generate_level_grid(&grid),
        "fn level_grid() -> Grid { Grid { width: 5, height: 3, start: (0, 0), goal: (4, 1), \
         walls: ::std::vec![(2, 0), (2, 1)], gems: ::std::vec![], doors: ::std::vec![] } }"
    );
}

#[test]
fn generates_level_grid_with_gems_and_doors() {
    let grid = GridDef {
        width: 12,
        height: 2,
        start: [11, 1],
        goal: [0, 0],
        walls: vec![[1, 0]],
        gems: vec![[0, 1], [5, 1]],
        doors: vec![[3, 0]],
    };
    assert_eq!(
        generate_level_grid(&grid),
        "fn level_grid() -> Grid { Grid { width: 12, height: 2, start: (11, 1), goal: (0, 0), \
         walls: ::std::vec![(1, 0)], gems: ::std::vec![(0, 1), (5, 1)], doors: ::std::vec![(3, 0)] } }"
    );
}

#[test]
fn loader_stores_generated_level_grid_for_grid_levels() {
    let store = load_fixture_store();
    for id in [GRID, GRID2] {
        let level = store.get(id).unwrap_or_else(|| panic!("應載入 {id}"));
        let grid = level.grid.as_ref().expect("grid 關卡應有 grid");
        assert_eq!(
            level.level_grid_src.as_deref(),
            Some(generate_level_grid(grid).as_str())
        );
        assert!(
            level
                .level_grid_src
                .as_deref()
                .is_some_and(|s| !s.is_empty()),
            "{id} 的 level_grid 原始碼不可為空"
        );
    }
}

#[test]
fn loader_leaves_level_grid_none_for_other_visuals() {
    let store = load_fixture_store();
    for id in [PLAIN, MEMORY] {
        let level = store.get(id).unwrap_or_else(|| panic!("應載入 {id}"));
        assert!(level.level_grid_src.is_none(), "{id} 不應有 level_grid");
    }
}

/// 產生的 level_grid() 經 merge_submit 併入 grid_sim.rs 後可編譯，solution 通過、starter 不通過（09-late-grid 含 gems 與 doors）
/// 用 host rustc，不需 Docker
#[test]
#[ignore = "需要 host rustc；驗證產生格式與 grid_sim.rs 的 Grid 對齊"]
fn generated_level_grid_compiles_with_grid_sim() {
    let store = load_fixture_store();
    let level = store.get(GRID2).expect("應載入 09-late-grid");
    let level_grid = level.level_grid_src.clone().expect("應有 level_grid");

    // starter 刻意少 mut 編譯失敗，補上 mut 只驗證路線不通關
    let starter = level.starter.replacen("let cmds", "let mut cmds", 1);
    for (code, should_pass) in [(level.solution.clone(), true), (starter, false)] {
        let src = merge_submit(&code, &level.tests, Some((store.grid_sim(), &level_grid))).source;
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main.rs");
        let bin = dir.path().join("main");
        fs::write(&main, src).unwrap();
        let compile = Command::new("rustc")
            .args(["--edition", "2024", "--test", "-A", "dead_code", "-o"])
            .arg(&bin)
            .arg(&main)
            .output()
            .expect("執行 rustc");
        assert!(
            compile.status.success(),
            "合併後應可編譯：{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new(&bin).arg("--test-threads=1").output().unwrap();
        assert_eq!(
            run.status.success(),
            should_pass,
            "{}",
            String::from_utf8_lossy(&run.stdout)
        );
    }
}

// ───────── memory_snapshots 的 error ─────────

#[test]
fn snapshot_error_is_parsed() {
    let store = load_fixture_store();
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03 應有 memory_snapshots");
    assert_eq!(
        snaps[1].error,
        Some(SnapshotError {
            code: Some("E0382".into()),
            target: "report.bag".into()
        })
    );
    assert_eq!(snaps[0].error, None);
}

#[test]
fn snapshot_error_code_omitted_becomes_none() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "error = { code = \"E0382\", target = \"report.bag\" }",
        "error = { target = \"report.bag\" }",
    );
    let store = cd.load().expect("省略 error.code 應可載入");
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03");
    assert_eq!(
        snaps[1].error,
        Some(SnapshotError {
            code: None,
            target: "report.bag".into()
        })
    );
}

#[test]
fn snapshot_error_target_may_be_heap_block() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "error = { code = \"E0382\", target = \"report.bag\" }",
        "error = { code = \"E0382\", target = \"h2\" }\nheap = [\n  { id = \"h2\", type = \"String\", value = \"珍珠\", state = \"freed\" },\n]",
    );
    let store = cd.load().expect("error.target 指向 heap 應可載入");
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03");
    assert_eq!(
        snaps[1].error.as_ref().map(|e| e.target.as_str()),
        Some("h2")
    );
}

#[test]
fn fails_when_error_target_unknown() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "target = \"report.bag\"",
        "target = \"ghost\"",
    );
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_error_target_only_exists_in_other_step() {
    // h1 只存在第 1 步，錯誤在第 2 步
    let cd = ContentDir::new();
    cd.replace(&toml(MEMORY), "target = \"report.bag\"", "target = \"h1\"");
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_error_target_missing() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "error = { code = \"E0382\", target = \"report.bag\" }",
        "error = { code = \"E0382\" }",
    );
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_error_code_format_invalid() {
    for bad in ["E382", "e0382", "E03820", "0382", "", "E038a", "W0382"] {
        let cd = ContentDir::new();
        cd.replace(
            &toml(MEMORY),
            "code = \"E0382\"",
            &format!("code = \"{bad}\""),
        );
        cd.assert_load_fails(MEMORY);
    }
}

#[test]
fn fails_on_unknown_field_in_snapshot_error() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "target = \"report.bag\"",
        "target = \"report.bag\", message = \"x\"",
    );
    cd.assert_load_fails(MEMORY);
}
