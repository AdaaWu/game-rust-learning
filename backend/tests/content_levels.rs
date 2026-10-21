//! 內容測試（architecture §6.2 ④）：讀 repo 根目錄 content/，每關 solution 通過、starter 失敗
//! 需要 Docker 與 rust-quest-runner:dev（cargo test -- --ignored）

use std::path::Path;

use rust_quest_backend::content::loader::load;
use rust_quest_backend::content::model::LevelKind;
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

#[tokio::test]
#[ignore = "需要 Docker；讀真實 content/"]
async fn cheats_never_pass_submit() {
    let store = load(&content_root()).expect("content/ 應可載入");
    let judge = Judge::new(JudgeConfig::default());
    let cheats_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cheats");
    let mut level_dirs: Vec<_> = std::fs::read_dir(&cheats_root)
        .expect("cheats/ 應存在")
        .map(|e| e.expect("讀取目錄項目").path())
        .filter(|p| p.is_dir())
        .collect();
    level_dirs.sort();
    assert!(!level_dirs.is_empty(), "cheats/ 至少要有一個關卡目錄");

    let mut failures = Vec::new();
    for dir in level_dirs {
        let id = dir.file_name().unwrap().to_string_lossy().into_owned();
        let Some(level) = store.get(&id) else {
            failures.push(format!("cheats/{id}：content/ 找不到同 id 的關卡"));
            continue;
        };
        let mut files: Vec<_> = std::fs::read_dir(&dir)
            .expect("讀取關卡目錄")
            .map(|e| e.expect("讀取檔案項目").path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .collect();
        files.sort();
        assert!(!files.is_empty(), "cheats/{id} 至少要有一個 .rs");
        for file in files {
            let code = std::fs::read_to_string(&file).expect("讀取取巧檔");
            let res = judge
                .submit(level, store.grid_sim(), &code)
                .await
                .expect("判題成功");
            let name = file.file_name().unwrap().to_string_lossy();
            // 第二行標記：僅允許因禁用 unsafe 而編譯失敗
            if code.lines().nth(1) == Some(EXPECT_UNSAFE_COMPILE_ERROR) {
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
