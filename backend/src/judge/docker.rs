//! docker 指令參數組裝（不含 "docker" 本身，不經 shell）、容器 RAII guard

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{JudgeConfig, Mode};

pub const JUDGE_LABEL: &str = "rust-quest.judge=1";

const HIDDEN_MODULE: &str = "__quest_hidden_tests";

fn sv(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// rq-judge-<唯一值>
pub fn new_container_name() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("rq-judge-{}-{nanos}-{seq}", std::process::id())
}

pub fn run_args(config: &JudgeConfig, name: &str, workdir: &Path) -> Vec<String> {
    let mut args = sv(&[
        "run",
        "-d",
        "--rm",
        "--name",
        name,
        "--label",
        JUDGE_LABEL,
        "--network",
        "none",
        "--memory",
        "256m",
        "--memory-swap",
        "256m",
        "--cpus",
        "1",
        "--pids-limit",
        "64",
        "--read-only",
        "--tmpfs",
        "/tmp:rw,exec,nosuid,size=64m",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--user",
        "1000:1000",
        "--init",
        "--log-driver",
        "none",
        "--ulimit",
        "nofile=256:256",
        "--ulimit",
        "core=0",
        "-v",
    ]);
    args.push(format!("{}:/work:ro", workdir.display()));
    args.push(config.image.to_string());
    args.push("sleep".into());
    args.push(config.container_max_lifetime_secs.to_string());
    args
}

pub fn compile_args(name: &str, mode: Mode) -> Vec<String> {
    let test_flag = match mode {
        Mode::Run => "",
        Mode::Submit => " --test",
    };
    let script = format!(
        "cp /work/main.rs /tmp/main.rs && rustc --edition 2024 --error-format=json{test_flag} -A dead_code -F unsafe_code -o /tmp/main /tmp/main.rs"
    );
    vec!["exec".into(), name.into(), "sh".into(), "-c".into(), script]
}

pub fn exec_args(name: &str, mode: Mode, test_names: &[String]) -> Vec<String> {
    let mut args = sv(&["exec", name, "/tmp/main"]);
    if mode == Mode::Submit {
        args.push("--test-threads=1".into());
        args.push("--exact".into());
        args.extend(test_names.iter().map(|t| format!("{HIDDEN_MODULE}::{t}")));
    }
    args
}

pub fn kill_args(name: &str) -> Vec<String> {
    sv(&["kill", name])
}

pub fn rm_args(name: &str) -> Vec<String> {
    sv(&["rm", "-f", name])
}

pub fn list_stale_args() -> Vec<String> {
    vec![
        "ps".into(),
        "-aq".into(),
        "--filter".into(),
        format!("label={JUDGE_LABEL}"),
    ]
}

pub fn version_args() -> Vec<String> {
    sv(&["version", "--format", "{{.Server.Version}}"])
}

pub fn image_inspect_args(image: &str) -> Vec<String> {
    sv(&["image", "inspect", image])
}

/// Drop 時 docker rm -f 容器；請求 future 被 drop 也會執行
pub struct ContainerGuard {
    docker_bin: &'static str,
    name: String,
}

impl ContainerGuard {
    pub fn new(docker_bin: &'static str, name: String) -> Self {
        Self { docker_bin, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        // 用獨立執行緒與 std 行程，不受 tokio runtime 關閉影響
        let bin = self.docker_bin;
        let args = rm_args(&self.name);
        std::thread::spawn(move || {
            let _ = std::process::Command::new(bin)
                .args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        });
    }
}
