//! S2-3 timeline 測試輔助的自我驗證，以及 fixture timeline 關卡的實際執行（後者需 Docker）

mod common;

use common::load_fixture_store;
use common::timeline::{
    TlEvent, clean_run_violations, duplicate_lane_names, first_deadlock, lanes_without_end,
    parse_events, stuck_lanes, unmatched_channels, unpaired_locks,
};
use rust_quest_backend::content::loader::attach_timeline;
use rust_quest_backend::judge::diagnostics::DiagnosticLevel;
use rust_quest_backend::judge::outcome::JudgeStatus;
use rust_quest_backend::judge::test_output::TestStatus;
use rust_quest_backend::judge::{Judge, JudgeConfig};

/// architecture §3.11 的 33 starter 範例（逾時前已印出的事件）
const DEADLOCK_SAMPLE: &str = "\
@tl|recv_wait|78|main|回報
@tl|start|42|分身 0|
@tl|start|42|分身 1|
@tl|lock_wait|45|分身 0|窄橋 0
@tl|lock_got|45|分身 0|窄橋 0
@tl|lock_wait|45|分身 1|窄橋 1
@tl|lock_got|45|分身 1|窄橋 1
@tl|lock_wait|45|分身 0|窄橋 1
@tl|lock_wait|45|分身 1|窄橋 0
";

fn ev(kind: &str, thread: &str, target: &str) -> TlEvent {
    TlEvent {
        kind: kind.into(),
        line: None,
        thread: thread.into(),
        target: target.into(),
    }
}

fn lanes(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// ───────── 解析 ─────────

#[test]
fn parses_sample_fields() {
    let p = parse_events(DEADLOCK_SAMPLE);
    assert!(p.malformed.is_empty());
    assert_eq!(p.events.len(), 9);
    assert_eq!(
        p.events[0],
        TlEvent {
            kind: "recv_wait".into(),
            line: Some(78),
            thread: "main".into(),
            target: "回報".into()
        }
    );
    assert_eq!(p.events[1].target, "", "start 的 target 留空");
}

#[test]
fn parse_skips_player_output_and_reports_malformed() {
    let stdout = "過橋 2 次\n\
        @tl|lock_free||分身 0|窄橋 0\n\
        @tl|explode|1|main|x\n\
        @tl|note|abc|main|x\n\
        @tl|note|3|main\n\
        @tl|note|3|  |x\n\
        @tl|end|| 分身 0 |\n";
    let p = parse_events(stdout);
    assert_eq!(
        p.events,
        vec![ev("lock_free", "分身 0", "窄橋 0"), ev("end", "分身 0", "")]
    );
    assert_eq!(p.malformed.len(), 4, "{:?}", p.malformed);
}

// ───────── 循環等待 ─────────

#[test]
fn sample_deadlocks_at_last_step_starting_from_first_lane() {
    let p = parse_events(DEADLOCK_SAMPLE);
    assert_eq!(
        first_deadlock(&p.events),
        Some((9, lanes(&["分身 0", "分身 1"])))
    );
}

#[test]
fn ordered_locking_has_no_deadlock() {
    let events = vec![
        ev("start", "A", ""),
        ev("start", "B", ""),
        ev("lock_wait", "A", "L0"),
        ev("lock_got", "A", "L0"),
        ev("lock_wait", "B", "L0"),
        ev("lock_wait", "A", "L1"),
        ev("lock_got", "A", "L1"),
        ev("lock_free", "A", "L1"),
        ev("lock_free", "A", "L0"),
        ev("lock_got", "B", "L0"),
        ev("lock_free", "B", "L0"),
        ev("end", "A", ""),
        ev("end", "B", ""),
    ];
    assert_eq!(first_deadlock(&events), None);
}

#[test]
fn relocking_own_mutex_is_cycle_of_one() {
    let events = vec![
        ev("lock_wait", "A", "L"),
        ev("lock_got", "A", "L"),
        ev("lock_wait", "A", "L"),
    ];
    assert_eq!(first_deadlock(&events), Some((3, lanes(&["A"]))));
}

#[test]
fn recv_wait_never_forms_cycle() {
    let events = vec![
        ev("lock_got", "A", "L"),
        ev("recv_wait", "A", "ch"),
        ev("lock_wait", "main", "L"),
    ];
    assert_eq!(first_deadlock(&events), None);
}

#[test]
fn waiting_on_free_lock_is_not_cycle() {
    let events = vec![
        ev("lock_got", "A", "L0"),
        ev("lock_free", "A", "L0"),
        ev("lock_wait", "B", "L0"),
        ev("lock_wait", "A", "L1"),
    ];
    assert_eq!(first_deadlock(&events), None);
}

#[test]
fn cycle_in_any_prefix_is_reported_even_if_later_events_follow() {
    let mut events = parse_events(DEADLOCK_SAMPLE).events;
    events.push(ev("lock_got", "分身 0", "窄橋 1"));
    assert_eq!(
        first_deadlock(&events).map(|(step, _)| step),
        Some(9),
        "前綴出現循環即為死鎖"
    );
}

#[test]
fn cycle_starts_from_main_when_main_is_in_it() {
    let events = vec![
        ev("lock_got", "分身 0", "L0"),
        ev("lock_got", "main", "L1"),
        ev("lock_wait", "分身 0", "L1"),
        ev("lock_wait", "main", "L0"),
    ];
    assert_eq!(
        first_deadlock(&events),
        Some((4, lanes(&["main", "分身 0"])))
    );
}

// ───────── 收尾與成對 ─────────

#[test]
fn lanes_without_end_ignores_lanes_without_start() {
    let events = vec![
        ev("recv_wait", "main", "ch"),
        ev("start", "A", ""),
        ev("start", "B", ""),
        ev("end", "A", ""),
    ];
    assert_eq!(lanes_without_end(&events), lanes(&["B"]));
}

#[test]
fn duplicate_lane_names_are_reported_once() {
    let events = vec![
        ev("start", "分身 0", ""),
        ev("start", "分身 0", ""),
        ev("start", "分身 0", ""),
        ev("start", "分身 1", ""),
    ];
    assert_eq!(duplicate_lane_names(&events), lanes(&["分身 0"]));
}

#[test]
fn stuck_lanes_end_on_wait() {
    let p = parse_events(DEADLOCK_SAMPLE);
    assert_eq!(stuck_lanes(&p.events), lanes(&["main", "分身 0", "分身 1"]));
}

#[test]
fn unpaired_locks_and_channels_are_reported() {
    let events = vec![
        ev("lock_got", "A", "L0"),
        ev("lock_free", "A", "L0"),
        ev("lock_got", "A", "L1"),
        ev("send", "A", "ch"),
        ev("send", "A", "ch"),
        ev("recv", "main", "ch"),
    ];
    assert_eq!(unpaired_locks(&events), vec![("L1".to_string(), 1, 0)]);
    assert_eq!(unmatched_channels(&events), vec![("ch".to_string(), 2, 1)]);
}

#[test]
fn clean_run_violations_flags_empty_stdout_and_deadlock_sample() {
    assert_eq!(clean_run_violations("過橋 2 次\n").len(), 1);
    let v = clean_run_violations(DEADLOCK_SAMPLE);
    assert!(v.iter().any(|s| s.contains("循環等待")), "{v:?}");
    assert!(v.iter().any(|s| s.contains("停在等待")), "{v:?}");
}

// ───────── fixture timeline 關卡實際執行（需要 Docker）─────────

const TL: &str = "04-bridge-timeline";

#[tokio::test]
#[ignore = "需要 Docker 與 rust-quest-runner:dev"]
async fn fixture_timeline_solution_run_has_clean_events_and_no_warning() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    let judge = Judge::new(JudgeConfig::default());
    let run = judge.run(level, &level.solution).await.expect("判題成功");
    assert_eq!(run.status, JudgeStatus::Ok, "{:?}", run.compile.diagnostics);
    let warnings: Vec<_> = run
        .compile
        .diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();
    assert!(warnings.is_empty(), "{warnings:?}");
    let stdout = run.execution.expect("執行結果").stdout;
    let violations = clean_run_violations(&stdout);
    assert!(violations.is_empty(), "{violations:?}\n{stdout}");
}

#[tokio::test]
#[ignore = "需要 Docker 與 rust-quest-runner:dev"]
async fn fixture_timeline_starter_run_times_out_with_circular_wait() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    let judge = Judge::new(JudgeConfig::default());
    let run = judge.run(level, &level.starter).await.expect("判題成功");
    assert_eq!(
        run.status,
        JudgeStatus::Timeout,
        "{:?}",
        run.compile.diagnostics
    );
    let stdout = run.execution.expect("執行結果").stdout;
    let (_, mut cycle) = first_deadlock(&parse_events(&stdout).events)
        .unwrap_or_else(|| panic!("應出現循環等待：\n{stdout}"));
    // 循環起點依泳道首次出現順序，兩隻分身誰先啟動不固定
    cycle.sort();
    assert_eq!(cycle, lanes(&["分身 0", "分身 1"]));
}

#[tokio::test]
#[ignore = "需要 Docker 與 rust-quest-runner:dev"]
async fn fixture_timeline_failed_submit_message_has_no_events() {
    let store = load_fixture_store();
    let level = store.get(TL).expect("timeline 關卡");
    let raw = common::read_fixture(&format!("content/levels/{TL}/solution.rs"))
        .replace("    total\n}", "    total + 1\n}");
    let helper = common::read_fixture("content/shared/timeline.rs");
    let code = attach_timeline(&raw, &helper);
    let judge = Judge::new(JudgeConfig::default());
    let res = judge
        .submit(level, store.grid_sim(), &code)
        .await
        .expect("判題成功");
    assert_eq!(
        res.status,
        JudgeStatus::TestFailed,
        "{:?}",
        res.compile.diagnostics
    );
    let failed: Vec<_> = res
        .tests
        .iter()
        .filter(|t| t.status == TestStatus::Failed)
        .collect();
    assert_eq!(failed.len(), 1, "{:?}", res.tests);
    let message = failed[0].message.as_deref().unwrap_or_default();
    assert!(
        !message.contains("@tl|"),
        "Submit 時小幫手應靜音：{message}"
    );
}
