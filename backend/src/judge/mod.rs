//! Judge：permit、流程編排

pub mod diagnostics;
pub mod docker;
pub mod exec;
pub mod merge;
pub mod outcome;
pub mod test_output;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tokio::process::Command;
use tokio::sync::Semaphore;

use crate::content::model::Level;
use docker::ContainerGuard;
use exec::{CappedRun, Overflow};
use outcome::{RunResponse, SubmitResponse, build_run_response, build_submit_response};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Run,
    Submit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JudgeConfig {
    /// docker CLI 路徑
    pub docker_bin: &'static str,
    pub image: &'static str,
    pub max_concurrent: usize,
    pub max_queue: usize,
    pub queue_wait_timeout: Duration,
    pub container_start_timeout: Duration,
    pub compile_timeout: Duration,
    pub run_timeout: Duration,
    pub container_max_lifetime_secs: u64,
    pub output_limit_bytes: usize,
    pub max_code_bytes: usize,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let max_concurrent = std::cmp::max(1, cpus / 2);
        Self {
            docker_bin: "docker",
            image: "rust-quest-runner:dev",
            max_concurrent,
            max_queue: max_concurrent * 4,
            queue_wait_timeout: Duration::from_secs(10),
            container_start_timeout: Duration::from_secs(10),
            compile_timeout: Duration::from_secs(15),
            run_timeout: Duration::from_secs(5),
            container_max_lifetime_secs: 60,
            output_limit_bytes: 64 * 1024,
            max_code_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum JudgeError {
    #[error("判題排隊已滿")]
    Busy,
    #[error("判題服務未就緒：{0}")]
    Unavailable(String),
    #[error("內部錯誤：{0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableReason {
    DockerUnreachable,
    RunnerImageMissing,
}

impl UnavailableReason {
    fn message(self) -> &'static str {
        match self {
            Self::DockerUnreachable => "Docker 無法連線",
            Self::RunnerImageMissing => "找不到 runner 映像",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JudgeHealth {
    pub available: bool,
    pub reason: Option<UnavailableReason>,
}

/// 依 docker version 與 image inspect 的結果判定可用性
pub fn health_from_checks(docker_ok: bool, image_ok: bool) -> JudgeHealth {
    let reason = if !docker_ok {
        Some(UnavailableReason::DockerUnreachable)
    } else if !image_ok {
        Some(UnavailableReason::RunnerImageMissing)
    } else {
        None
    };
    JudgeHealth {
        available: reason.is_none(),
        reason,
    }
}

/// 建立暫存目錄並寫入 main.rs，目錄 0755、檔案 0644
pub fn prepare_workdir(source: &str) -> std::io::Result<TempDir> {
    let dir = tempfile::Builder::new().prefix("rq-judge-").tempdir()?;
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755))?;
    let file = dir.path().join("main.rs");
    fs::write(&file, source)?;
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644))?;
    Ok(dir)
}

/// health 結果快取秒數
const HEALTH_CACHE_TTL: Duration = Duration::from_secs(10);
/// docker 管理指令（version、image inspect、ps、rm、kill）逾時
const DOCKER_CHECK_TIMEOUT: Duration = Duration::from_secs(10);
/// 編譯 stderr 上限，超過丟棄
const COMPILE_OUTPUT_LIMIT: usize = 1024 * 1024;

/// 一次判題的原始結果
struct RawJudge {
    compile: CappedRun,
    diagnostics: Vec<diagnostics::Diagnostic>,
    exec: Option<CappedRun>,
}

#[derive(Debug)]
pub struct Judge {
    config: JudgeConfig,
    admission: Semaphore,
    exec_permits: Semaphore,
    health_cache: Mutex<Option<(Instant, JudgeHealth)>>,
}

impl Judge {
    pub fn new(config: JudgeConfig) -> Self {
        Self {
            admission: Semaphore::new(config.max_concurrent + config.max_queue),
            exec_permits: Semaphore::new(config.max_concurrent),
            health_cache: Mutex::new(None),
            config,
        }
    }

    pub fn config(&self) -> &JudgeConfig {
        &self.config
    }

    pub async fn run(&self, _level: &Level, code: &str) -> Result<RunResponse, JudgeError> {
        let raw = self.judge(Mode::Run, code, None, &[]).await?;
        Ok(build_run_response(&raw.compile, raw.diagnostics, raw.exec))
    }

    pub async fn submit(
        &self,
        level: &Level,
        grid_sim: &str,
        code: &str,
    ) -> Result<SubmitResponse, JudgeError> {
        let grid = level.level_grid_src.as_deref().map(|g| (grid_sim, g));
        let merged = merge::merge_submit(code, &level.tests, grid);
        let raw = self
            .judge(
                Mode::Submit,
                &merged.source,
                Some(merged.user_line_count),
                &level.test_names,
            )
            .await?;
        Ok(build_submit_response(
            &raw.compile,
            raw.diagnostics,
            raw.exec,
            &level.test_names,
        ))
    }

    async fn judge(
        &self,
        mode: Mode,
        source: &str,
        user_line_count: Option<usize>,
        test_names: &[String],
    ) -> Result<RawJudge, JudgeError> {
        let _admission = self.admission.try_acquire().map_err(|_| JudgeError::Busy)?;
        let _permit =
            tokio::time::timeout(self.config.queue_wait_timeout, self.exec_permits.acquire())
                .await
                .map_err(|_| JudgeError::Busy)?
                .map_err(|e| JudgeError::Internal(e.to_string()))?;

        let health = self.health().await;
        if let Some(reason) = health.reason {
            return Err(JudgeError::Unavailable(reason.message().into()));
        }

        let workdir = prepare_workdir(source).map_err(|e| JudgeError::Internal(e.to_string()))?;
        let guard = ContainerGuard::new(self.config.docker_bin, docker::new_container_name());
        self.start_container(guard.name(), workdir.path()).await?;

        let compile = self
            .docker(
                docker::compile_args(guard.name(), mode),
                self.config.compile_timeout,
                COMPILE_OUTPUT_LIMIT,
                Overflow::Discard,
            )
            .await?;
        if compile.timed_out {
            self.kill(guard.name()).await;
        }
        let diagnostics = diagnostics::parse_diagnostics(&compile.stderr, user_line_count);

        let exec = if !compile.timed_out && compile.exit_code == Some(0) {
            let exec = self
                .docker(
                    docker::exec_args(guard.name(), mode, test_names),
                    self.config.run_timeout,
                    self.config.output_limit_bytes,
                    Overflow::Kill,
                )
                .await?;
            if exec.timed_out || exec.stdout_truncated || exec.stderr_truncated {
                self.kill(guard.name()).await;
            }
            Some(exec)
        } else {
            None
        };

        Ok(RawJudge {
            compile,
            diagnostics,
            exec,
        })
    }

    async fn docker(
        &self,
        args: Vec<String>,
        timeout: Duration,
        limit: usize,
        overflow: Overflow,
    ) -> Result<CappedRun, JudgeError> {
        exec::run_capped(self.config.docker_bin, &args, timeout, limit, overflow)
            .await
            .map_err(|e| JudgeError::Internal(e.to_string()))
    }

    /// docker run -d 殼容器；失敗或逾時回 Unavailable 並讓 health 快取失效
    async fn start_container(&self, name: &str, workdir: &Path) -> Result<(), JudgeError> {
        let result = exec::run_capped(
            self.config.docker_bin,
            &docker::run_args(&self.config, name, workdir),
            self.config.container_start_timeout,
            self.config.output_limit_bytes,
            Overflow::Discard,
        )
        .await;
        let reason = match result {
            Ok(r) if r.timed_out => "容器啟動逾時".to_string(),
            Ok(r) if r.exit_code == Some(0) => return Ok(()),
            Ok(r) => format!("容器啟動失敗：{}", r.stderr.trim()),
            Err(e) => format!("容器啟動失敗：{e}"),
        };
        tracing::warn!("{reason}");
        *self.health_cache.lock().expect("health cache") = None;
        Err(JudgeError::Unavailable(reason))
    }

    async fn kill(&self, name: &str) {
        let status = Command::new(self.config.docker_bin)
            .args(docker::kill_args(name))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status();
        let _ = tokio::time::timeout(DOCKER_CHECK_TIMEOUT, status).await;
    }

    /// 執行 docker 檢查指令；逾時回 None
    async fn check(&self, args: Vec<String>) -> Option<bool> {
        let status = Command::new(self.config.docker_bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status();
        let result = tokio::time::timeout(DOCKER_CHECK_TIMEOUT, status)
            .await
            .ok()?;
        Some(result.is_ok_and(|s| s.success()))
    }

    pub async fn health(&self) -> JudgeHealth {
        if let Some((at, health)) = *self.health_cache.lock().expect("health cache")
            && at.elapsed() < HEALTH_CACHE_TTL
        {
            return health;
        }
        let version = self.check(docker::version_args()).await;
        let inspect = if version == Some(true) {
            self.check(docker::image_inspect_args(self.config.image))
                .await
        } else {
            Some(false)
        };
        // image inspect 逾時代表 daemon 不回應
        let docker_ok = version == Some(true) && inspect.is_some();
        let health = health_from_checks(docker_ok, inspect == Some(true));
        *self.health_cache.lock().expect("health cache") = Some((Instant::now(), health));
        health
    }

    /// 清掉帶判題 label 的殘留容器，回傳清掉的數量
    pub async fn cleanup_stale_containers(&self) -> std::io::Result<usize> {
        let out = Command::new(self.config.docker_bin)
            .args(docker::list_stale_args())
            .stdin(Stdio::null())
            .kill_on_drop(true)
            .output();
        let out = with_check_timeout(out, "docker ps").await?;
        let ids: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        if ids.is_empty() {
            return Ok(0);
        }
        let mut args = vec!["rm".to_string(), "-f".to_string()];
        args.extend(ids.iter().cloned());
        let status = Command::new(self.config.docker_bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .kill_on_drop(true)
            .status();
        let status = with_check_timeout(status, "docker rm -f").await?;
        if !status.success() {
            return Err(std::io::Error::other("docker rm -f 失敗"));
        }
        Ok(ids.len())
    }
}

/// 套用 docker 管理指令逾時，逾時轉成 TimedOut 錯誤
async fn with_check_timeout<T>(
    fut: impl Future<Output = std::io::Result<T>>,
    what: &str,
) -> std::io::Result<T> {
    tokio::time::timeout(DOCKER_CHECK_TIMEOUT, fut)
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, format!("{what} 逾時")))?
}
