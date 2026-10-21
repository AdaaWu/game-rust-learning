//! 內容測試（architecture §6.2 ④）：讀 repo 根目錄 content/，每關 solution 通過、starter 失敗
//! 需要 Docker 與 rust-quest-runner:dev（cargo test -- --ignored）

mod common;

use std::path::Path;
use std::sync::Arc;

use common::timeline::{clean_run_violations, first_deadlock, parse_events};
use rust_quest_backend::content::LevelStore;
use rust_quest_backend::content::loader::{attach_timeline, load};
use rust_quest_backend::content::model::{Level, LevelKind, LevelVisual};
use rust_quest_backend::judge::diagnostics::DiagnosticLevel;
use rust_quest_backend::judge::outcome::JudgeStatus;
use rust_quest_backend::judge::{Judge, JudgeConfig};

fn content_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../content")
}

#[test]
#[ignore = "讀真實 content/"]
fn real_content_loads() {
    let store = load(&content_root()).expect("content/ 應可載入");
    assert!(!store.levels().is_empty(), "content/ 至少要有一關");
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn every_level_solution_passes_and_starter_fails() {
    let store = load(&content_root()).expect("content/ 應可載入");
    assert!(!store.levels().is_empty());
    let judge = Judge::new(JudgeConfig::default());
    for level in store.levels() {
        let id = &level.id;

        let sol = judge
            .submit(level, store.grid_sim(), &level.solution)
            .await
            .expect("判題成功");
        assert_eq!(
            sol.status,
            JudgeStatus::Ok,
            "{id} solution 應通過：{:?}",
            sol.tests
        );

        let run = judge.run(level, &level.solution).await.expect("判題成功");
        assert_eq!(run.status, JudgeStatus::Ok, "{id} solution 用 Run 應可執行");

        let st = judge
            .submit(level, store.grid_sim(), &level.starter)
            .await
            .expect("判題成功");
        assert_ne!(st.status, JudgeStatus::Ok, "{id} starter 不應通過");
        if level.kind == LevelKind::FixCompile {
            assert_eq!(
                st.status,
                JudgeStatus::CompileError,
                "{id} fix-compile 的 starter 應編譯失敗"
            );
        }
    }
}

/// 反例第二行寫這個標記，表示預期被 `-F unsafe_code` 擋在編譯階段
const EXPECT_UNSAFE_COMPILE_ERROR: &str = "// expect: compile_error unsafe_code";

fn cheats_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cheats")
}

fn timeline_helper() -> String {
    let path = content_root().join("shared/timeline.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("讀不到 {}：{e}", path.display()))
}

/// 逐一 Submit cheats/<id>/*.rs；`only` 有值時只檢查該 visual；回傳（已檢查的反例數, 失敗訊息）
async fn check_cheats(
    store: &LevelStore,
    judge: &Judge,
    only: Option<LevelVisual>,
) -> (usize, Vec<String>) {
    let mut level_dirs: Vec<_> = std::fs::read_dir(cheats_root())
        .expect("cheats/ 應存在")
        .map(|e| e.expect("讀取目錄項目").path())
        .filter(|p| p.is_dir())
        .collect();
    level_dirs.sort();
    assert!(!level_dirs.is_empty(), "cheats/ 至少要有一個關卡目錄");

    let mut checked = 0;
    let mut failures = Vec::new();
    for dir in level_dirs {
        let id = dir.file_name().unwrap().to_string_lossy().into_owned();
        let Some(level) = store.get(&id) else {
            if only.is_none() {
                failures.push(format!("cheats/{id}：content/ 找不到同 id 的關卡"));
            }
            continue;
        };
        if only.is_some_and(|v| v != level.visual) {
            continue;
        }
        let mut files: Vec<_> = std::fs::read_dir(&dir)
            .expect("讀取關卡目錄")
            .map(|e| e.expect("讀取檔案項目").path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .collect();
        files.sort();
        assert!(!files.is_empty(), "cheats/{id} 至少要有一個 .rs");
        for file in files {
            let raw = std::fs::read_to_string(&file).expect("讀取取巧檔");
            // timeline 關卡比照 loader 組法接上小幫手，反例檔不自帶
            let code = if level.visual == LevelVisual::Timeline {
                attach_timeline(&raw, &timeline_helper())
            } else {
                raw.clone()
            };
            let res = judge
                .submit(level, store.grid_sim(), &code)
                .await
                .expect("判題成功");
            checked += 1;
            let name = file.file_name().unwrap().to_string_lossy();
            if level.visual == LevelVisual::Timeline
                && res
                    .tests
                    .iter()
                    .any(|t| t.message.as_deref().is_some_and(|m| m.contains("@tl|")))
            {
                failures.push(format!("cheats/{id}/{name}：Submit 的測試訊息含 @tl| 事件"));
            }
            // 第二行標記：僅允許因禁用 unsafe 而編譯失敗
            if raw.lines().nth(1) == Some(EXPECT_UNSAFE_COMPILE_ERROR) {
                let has_unsafe_diag = res
                    .compile
                    .diagnostics
                    .iter()
                    .any(|d| d.code.as_deref() == Some("unsafe_code"));
                if res.status != JudgeStatus::CompileError || !has_unsafe_diag {
                    failures.push(format!(
                        "cheats/{id}/{name}：預期因 unsafe_code 編譯失敗，實際 {:?}",
                        res.status
                    ));
                }
                continue;
            }
            // 反例須可編譯，才能證明是測試擋下取巧
            if res.status == JudgeStatus::CompileError {
                failures.push(format!("cheats/{id}/{name}：反例編譯失敗，屬無效反例"));
            } else if res.passed {
                failures.push(format!("cheats/{id}/{name}：取巧解答 Submit 通過"));
            }
        }
    }
    (checked, failures)
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn cheats_never_pass_submit() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let judge = Judge::new(JudgeConfig::default());
    let (_, failures) = check_cheats(&store, &judge, None).await;
    assert!(
        failures.is_empty(),
        "取巧反例檢查失敗：\n{}",
        failures.join("\n")
    );
}

/// 與判題相同的共用模擬器（architecture §6.2 ④）
#[allow(dead_code, clippy::all)]
mod grid_sim {
    use rust_quest_backend::content::model::GridDef;

    include!("../../content/shared/grid_sim.rs");

    /// 用 [grid] 建出 Grid，把 stdout 逐行交給 assert_cleared
    pub fn assert_stdout_clears(def: &GridDef, stdout: &str) {
        let point = |p: &[i64; 2]| (p[0] as i32, p[1] as i32);
        let grid = Grid {
            width: def.width as i32,
            height: def.height as i32,
            start: point(&def.start),
            goal: point(&def.goal),
            walls: def.walls.iter().map(point).collect(),
            gems: def.gems.iter().map(point).collect(),
            doors: def.doors.iter().map(point).collect(),
        };
        let cmds: Vec<String> = stdout.lines().map(String::from).collect();
        assert_cleared(&grid, &cmds);
    }
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn grid_solution_run_stdout_clears_grid() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let judge = Judge::new(JudgeConfig::default());
    let grid_levels: Vec<_> = store.levels().iter().filter(|l| l.grid.is_some()).collect();
    assert!(!grid_levels.is_empty(), "content/ 應有 grid 關卡");
    for level in grid_levels {
        let run = judge.run(level, &level.solution).await.expect("判題成功");
        assert_eq!(
            run.status,
            JudgeStatus::Ok,
            "{} solution 用 Run 應可執行",
            level.id
        );
        let stdout = run.execution.expect("執行結果").stdout;
        let grid = level.grid.as_ref().expect("grid 關卡");
        grid_sim::assert_stdout_clears(grid, &stdout);
    }
}

// ───────── timeline 關卡（architecture §6.2 ④）─────────

/// 找出 visual = timeline 的關卡；一關都沒有時失敗，避免默默通過
fn timeline_levels(store: &LevelStore) -> Vec<Arc<Level>> {
    let found: Vec<Arc<Level>> = store
        .levels()
        .iter()
        .filter(|l| l.visual == LevelVisual::Timeline)
        .cloned()
        .collect();
    assert!(
        !found.is_empty(),
        "content/ 沒有任何 visual = \"timeline\" 的關卡（第 8 章 30／31／33 應改用 timeline，decisions §15 ⑭）"
    );
    found
}

/// 指定 id 的關卡必須存在且為 timeline
fn timeline_level(store: &LevelStore, id: &str) -> Arc<Level> {
    let level = store
        .get(id)
        .unwrap_or_else(|| panic!("content/ 找不到 {id}"))
        .clone();
    assert_eq!(
        level.visual,
        LevelVisual::Timeline,
        "{id} 應為 visual = \"timeline\"（decisions §15 ⑬ 第 8 章套用）"
    );
    level
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn timeline_solution_run_events_have_no_circular_wait_and_all_lanes_end() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let judge = Judge::new(JudgeConfig::default());
    let mut failures = Vec::new();
    for level in timeline_levels(&store) {
        let run = judge.run(&level, &level.solution).await.expect("判題成功");
        if run.status != JudgeStatus::Ok {
            failures.push(format!(
                "{}：solution Run 應為 ok，實際 {:?}",
                level.id, run.status
            ));
            continue;
        }
        let stdout = run.execution.expect("執行結果").stdout;
        for v in clean_run_violations(&stdout) {
            failures.push(format!("{}：{v}", level.id));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn timeline_solution_compiles_without_warning() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let judge = Judge::new(JudgeConfig::default());
    let mut failures = Vec::new();
    for level in timeline_levels(&store) {
        let run = judge.run(&level, &level.solution).await.expect("判題成功");
        for d in &run.compile.diagnostics {
            if d.level == DiagnosticLevel::Warning {
                failures.push(format!("{}：{:?} {}", level.id, d.code, d.message));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn timeline_33_starter_run_times_out_with_circular_wait() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let level = timeline_level(&store, "33-boss-great-migration");
    let judge = Judge::new(JudgeConfig::default());
    let run = judge.run(&level, &level.starter).await.expect("判題成功");
    assert_eq!(
        run.status,
        JudgeStatus::Timeout,
        "{:?}",
        run.compile.diagnostics
    );
    let stdout = run.execution.expect("執行結果").stdout;
    assert!(
        first_deadlock(&parse_events(&stdout).events).is_some(),
        "33 starter 的事件應出現循環等待：\n{stdout}"
    );
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn timeline_31_starter_keeps_unused_assignments_warning() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let level = timeline_level(&store, "31-shared-counter");
    let judge = Judge::new(JudgeConfig::default());
    let run = judge.run(&level, &level.starter).await.expect("判題成功");
    let codes: Vec<_> = run
        .compile
        .diagnostics
        .iter()
        .filter_map(|d| d.code.as_deref())
        .collect();
    assert!(codes.contains(&"unused_assignments"), "{codes:?}");
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn timeline_30_starter_keeps_e0373_and_e0277() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let level = timeline_level(&store, "30-send-the-crabs");
    let judge = Judge::new(JudgeConfig::default());
    let run = judge.run(&level, &level.starter).await.expect("判題成功");
    assert_eq!(run.status, JudgeStatus::CompileError);
    let mut codes: Vec<_> = run
        .compile
        .diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .filter_map(|d| d.code.clone())
        .collect();
    codes.sort();
    codes.dedup();
    assert_eq!(codes, vec!["E0277".to_string(), "E0373".to_string()]);
}

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn timeline_cheats_submit_with_helper_never_pass() {
    let store = load(&content_root()).expect("content/ 應可載入");
    timeline_levels(&store);
    let judge = Judge::new(JudgeConfig::default());
    let (checked, failures) = check_cheats(&store, &judge, Some(LevelVisual::Timeline)).await;
    assert!(checked > 0, "timeline 關卡沒有任何取巧反例");
    assert!(
        failures.is_empty(),
        "timeline 取巧反例檢查失敗：\n{}",
        failures.join("\n")
    );
}
