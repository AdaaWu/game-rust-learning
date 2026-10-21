//! B7 BDD：cucumber-rs 執行 features/ 的後端情境
//! - `cargo test`：略過 @docker 情境並列出略過數量
//! - `cargo test -- --ignored`：只跑 @docker 情境
//! - `cargo test -- --include-ignored`：全部情境
//! - `cargo test --test cucumber -- --tags '<表示式>'`：自訂 tag 篩選，優先於上面三種

mod steps;

use std::path::{Path, PathBuf};

use cucumber::gherkin::tagexpr::TagOperation;
use cucumber::gherkin::{Feature, GherkinEnv, Scenario};
use cucumber::{ScenarioType, World as _, cli, parser};
use steps::ApiWorld;

/// 後端負責的 feature 檔，ui_*.feature 由前端執行
const BACKEND_FEATURES: [&str; 4] = [
    "levels.feature",
    "run.feature",
    "submit.feature",
    "judge_busy.feature",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum DockerMode {
    Skip,
    Only,
    All,
}

fn features_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../features")
}

fn is_docker(scenario: &Scenario) -> bool {
    scenario.tags.iter().any(|t| t == "docker")
}

/// 情境展開後的數量（Scenario Outline 依 Examples 列數計）
fn expanded_count(scenario: &Scenario) -> usize {
    if scenario.examples.is_empty() {
        return 1;
    }
    scenario
        .examples
        .iter()
        .filter_map(|e| e.table.as_ref())
        .map(|t| t.rows.len().saturating_sub(1))
        .sum()
}

/// 解析 libtest 風格參數；其餘參數忽略
fn parse_args() -> (DockerMode, Option<TagOperation>) {
    let mut mode = DockerMode::Skip;
    let mut tags = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let expr = match arg.as_str() {
            "--ignored" => {
                mode = DockerMode::Only;
                None
            }
            "--include-ignored" => {
                mode = DockerMode::All;
                None
            }
            "--tags" | "-t" => args.next(),
            a => a.strip_prefix("--tags=").map(String::from),
        };
        if let Some(expr) = expr {
            let op = expr
                .parse::<TagOperation>()
                .unwrap_or_else(|e| panic!("無法解析 --tags {expr}：{e}"));
            tags = Some(op);
        }
    }
    (mode, tags)
}

/// 統計會被略過的 @docker 情境數
fn count_docker_scenarios() -> usize {
    BACKEND_FEATURES
        .iter()
        .map(|name| {
            let path = features_dir().join(name);
            let feature = Feature::parse_path(&path, GherkinEnv::default())
                .unwrap_or_else(|e| panic!("解析 {} 失敗：{e}", path.display()));
            feature
                .scenarios
                .iter()
                .filter(|s| is_docker(s))
                .map(expanded_count)
                .sum::<usize>()
        })
        .sum()
}

#[tokio::main]
async fn main() {
    let (mode, tags) = parse_args();
    if tags.is_none() && mode == DockerMode::Skip {
        println!(
            "略過 {} 個 @docker 情境（需要 Docker，以 `cargo test --test cucumber -- --ignored` 執行）",
            count_docker_scenarios()
        );
    }

    // 只解析後端 feature 檔
    let glob = format!(
        "{}/{{{}}}",
        features_dir().display(),
        BACKEND_FEATURES.join(",")
    );
    let opts = cli::Opts::<_, _, _, cli::Empty> {
        tags_filter: tags,
        parser: parser::basic::Cli {
            features: Some(glob.parse().expect("feature glob")),
        },
        ..Default::default()
    };

    ApiWorld::cucumber()
        .which_scenario(|_, _, sc| {
            // @docker 情境會檢查全域容器清單，逐一執行
            if is_docker(sc) {
                ScenarioType::Serial
            } else {
                ScenarioType::Concurrent
            }
        })
        .after(|_, _, _, _, world| {
            Box::pin(async move {
                if let Some(world) = world {
                    world.join_background().await;
                }
            })
        })
        .fail_on_skipped()
        .with_cli(opts)
        .filter_run_and_exit(features_dir(), move |_, _, sc| match mode {
            DockerMode::Skip => !is_docker(sc),
            DockerMode::Only => is_docker(sc),
            DockerMode::All => true,
        })
        .await;
}
