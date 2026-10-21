//! B4 docker 指令組裝：Vec<String> 參數、安全回歸

use std::path::Path;

use rust_quest_backend::judge::docker::{
    JUDGE_LABEL, compile_args, exec_args, image_inspect_args, kill_args, list_stale_args,
    new_container_name, rm_args, run_args, version_args,
};
use rust_quest_backend::judge::{JudgeConfig, Mode, prepare_workdir};

const NAME: &str = "rq-judge-test";
const WORKDIR: &str = "/var/folders/xx/rq-judge-abc";

fn config() -> JudgeConfig {
    JudgeConfig {
        image: "rust-quest-runner:dev",
        container_max_lifetime_secs: 60,
        ..JudgeConfig::default()
    }
}

fn default_run_args() -> Vec<String> {
    run_args(&config(), NAME, Path::new(WORKDIR))
}

/// 參數中有連續的 [flag, value]
fn has_pair(args: &[String], flag: &str, value: &str) -> bool {
    args.windows(2).any(|w| w[0] == flag && w[1] == value)
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn sv(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// ───────── docker run：安全回歸 ─────────

#[test]
fn run_args_contain_required_isolation_limits() {
    let args = default_run_args();
    for (flag, value) in [
        ("--network", "none"),
        ("--memory", "256m"),
        ("--memory-swap", "256m"),
        ("--cpus", "1"),
        ("--pids-limit", "64"),
        ("--cap-drop", "ALL"),
        ("--security-opt", "no-new-privileges"),
        ("--user", "1000:1000"),
    ] {
        assert!(
            has_pair(&args, flag, value),
            "缺少 {flag} {value}：{args:?}"
        );
    }
    assert!(has_flag(&args, "--read-only"), "缺少 --read-only：{args:?}");
}

#[test]
fn run_args_mount_tmpfs_with_exec_and_size_limit() {
    let args = default_run_args();
    assert!(
        has_pair(&args, "--tmpfs", "/tmp:rw,exec,nosuid,size=64m"),
        "tmpfs 參數不符：{args:?}"
    );
}

/// 建議加固參數（decisions.md §2）：整合測試驗證失敗而拿掉時，同步刪掉對應斷言並記錄原因
#[test]
fn run_args_contain_hardening_flags() {
    let args = default_run_args();
    assert!(has_flag(&args, "--init"), "缺少 --init：{args:?}");
    assert!(
        has_pair(&args, "--log-driver", "none"),
        "缺少 --log-driver none"
    );
    assert!(
        has_pair(&args, "--ulimit", "nofile=256:256"),
        "缺少 ulimit nofile"
    );
    assert!(has_pair(&args, "--ulimit", "core=0"), "缺少 ulimit core");
}

#[test]
fn run_args_never_contain_dangerous_options() {
    let args = default_run_args();
    assert!(!args.is_empty(), "run 參數不可為空");
    for a in &args {
        assert_ne!(a, "--privileged");
        assert!(!a.contains("docker.sock"), "不可掛 docker.sock：{a}");
        assert!(!a.contains("seccomp=unconfined"), "不可關 seccomp：{a}");
        assert!(
            !a.contains("--network=host") && a != "host",
            "不可用 host 網路：{a}"
        );
        assert!(!a.contains("--cap-add"), "不可加 capability：{a}");
    }
}

#[test]
fn run_args_shape_detached_shell_container() {
    let args = default_run_args();
    assert_eq!(args.first().map(String::as_str), Some("run"));
    assert!(has_flag(&args, "-d"), "缺少 -d");
    assert!(has_flag(&args, "--rm"), "缺少 --rm");
    assert!(has_pair(&args, "--name", NAME), "缺少 --name");
    assert!(has_pair(&args, "--label", JUDGE_LABEL), "缺少 label");
    assert!(
        has_pair(&args, "-v", &format!("{WORKDIR}:/work:ro")),
        "暫存目錄應唯讀掛到 /work：{args:?}"
    );
    let tail: Vec<&str> = args
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(String::as_str)
        .collect();
    assert_eq!(
        tail,
        vec!["rust-quest-runner:dev", "sleep", "60"],
        "映像後接 sleep N"
    );
}

#[test]
fn run_args_use_configured_image_and_lifetime() {
    let cfg = JudgeConfig {
        image: "rust-quest-runner:other",
        container_max_lifetime_secs: 30,
        ..JudgeConfig::default()
    };
    let args = run_args(&cfg, NAME, Path::new(WORKDIR));
    let tail: Vec<&str> = args
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(String::as_str)
        .collect();
    assert_eq!(tail, vec!["rust-quest-runner:other", "sleep", "30"]);
}

#[test]
fn judge_label_constant() {
    assert_eq!(JUDGE_LABEL, "rust-quest.judge=1");
}

#[test]
fn container_names_are_prefixed_and_unique() {
    let a = new_container_name();
    let b = new_container_name();
    assert!(
        a.starts_with("rq-judge-") && a.len() > "rq-judge-".len(),
        "{a}"
    );
    assert_ne!(a, b);
}

// ───────── docker exec：編譯 ─────────

#[test]
fn compile_args_for_run_mode() {
    assert_eq!(
        compile_args(NAME, Mode::Run),
        sv(&[
            "exec",
            NAME,
            "sh",
            "-c",
            "cp /work/main.rs /tmp/main.rs && rustc --edition 2024 --error-format=json -A dead_code -F unsafe_code -o /tmp/main /tmp/main.rs",
        ])
    );
}

#[test]
fn compile_args_for_submit_mode_add_test_and_allow_dead_code() {
    let args = compile_args(NAME, Mode::Submit);
    assert_eq!(args.len(), 5, "{args:?}");
    assert_eq!(&args[..4], &sv(&["exec", NAME, "sh", "-c"])[..]);
    let script = &args[4];
    assert!(
        script.starts_with("cp /work/main.rs /tmp/main.rs && rustc "),
        "{script}"
    );
    for part in [
        "--edition 2024",
        "--error-format=json",
        "--test",
        "-A dead_code",
        "-F unsafe_code",
        "-o /tmp/main",
        "/tmp/main.rs",
    ] {
        assert!(script.contains(part), "缺少 {part}：{script}");
    }
}

#[test]
fn compile_args_keep_default_optimization_and_edition() {
    for mode in [Mode::Run, Mode::Submit] {
        let args = compile_args(NAME, mode);
        let joined = args.join(" ");
        assert!(joined.contains("--edition 2024"), "{joined}");
        assert!(
            joined.contains("-A dead_code"),
            "Run 與 Submit 都關 dead_code：{joined}"
        );
        assert!(
            joined.contains("-F unsafe_code"),
            "Run 與 Submit 都禁用 unsafe：{joined}"
        );
        assert!(!joined.contains("opt-level"), "不覆寫最佳化等級：{joined}");
        assert!(
            !joined.contains("overflow-checks=off"),
            "保留溢位檢查：{joined}"
        );
    }
    assert!(!compile_args(NAME, Mode::Run).join(" ").contains("--test"));
}

// ───────── docker exec：執行 ─────────

#[test]
fn exec_args_for_run_mode() {
    assert_eq!(
        exec_args(NAME, Mode::Run, &[]),
        sv(&["exec", NAME, "/tmp/main"])
    );
}

#[test]
fn exec_args_for_submit_mode_use_exact_hidden_names() {
    let tests = sv(&["sums_three", "empty_is_zero"]);
    assert_eq!(
        exec_args(NAME, Mode::Submit, &tests),
        sv(&[
            "exec",
            NAME,
            "/tmp/main",
            "--test-threads=1",
            "--exact",
            "__quest_hidden_tests::sums_three",
            "__quest_hidden_tests::empty_is_zero",
        ])
    );
}

// ───────── 其他 docker 指令 ─────────

#[test]
fn lifecycle_and_probe_args() {
    assert_eq!(kill_args(NAME), sv(&["kill", NAME]));
    assert_eq!(rm_args(NAME), sv(&["rm", "-f", NAME]));
    assert_eq!(
        list_stale_args(),
        sv(&["ps", "-aq", "--filter", "label=rust-quest.judge=1"])
    );
    assert_eq!(
        version_args(),
        sv(&["version", "--format", "{{.Server.Version}}"])
    );
    assert_eq!(
        image_inspect_args("rust-quest-runner:dev"),
        sv(&["image", "inspect", "rust-quest-runner:dev"])
    );
}

// ───────── 使用者程式碼不進指令列 ─────────

#[test]
fn user_code_never_appears_in_any_docker_args() {
    let marker = "'; rm -rf / #USER_CODE_MARKER";
    let code = format!("fn main() {{ println!(\"{marker}\"); }}\n");
    let dir = prepare_workdir(&code).expect("建立暫存目錄");
    let name = new_container_name();
    let all = [
        run_args(&config(), &name, dir.path()),
        compile_args(&name, Mode::Run),
        compile_args(&name, Mode::Submit),
        exec_args(&name, Mode::Run, &[]),
        exec_args(&name, Mode::Submit, &sv(&["sums_three"])),
        kill_args(&name),
        rm_args(&name),
    ];
    assert!(all.iter().all(|a| !a.is_empty()), "每個指令都應有參數");
    for args in &all {
        for a in args {
            assert!(
                !a.contains("USER_CODE_MARKER"),
                "使用者程式碼出現在參數中：{a}"
            );
        }
    }
}
