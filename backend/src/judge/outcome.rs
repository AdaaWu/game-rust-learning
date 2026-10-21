//! 判題結果型別與狀態判定（architecture §3.6）

use super::Mode;
use super::diagnostics::Diagnostic;
use super::exec::CappedRun;
use super::test_output::{TestResult, TestStatus, all_not_run, parse_test_output};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgeStatus {
    Ok,
    CompileError,
    RuntimePanic,
    TestFailed,
    Timeout,
    MemoryLimit,
    OutputLimit,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub timed_out: bool,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RunResponse {
    pub status: JudgeStatus,
    pub compile: CompileResult,
    pub execution: Option<ExecutionResult>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SubmitResponse {
    pub status: JudgeStatus,
    pub passed: bool,
    pub compile: CompileResult,
    pub execution: Option<ExecutionResult>,
    pub tests: Vec<TestResult>,
}

/// 執行階段被 host 終止或 OOM（exit 137 且非 host kill）
fn exec_killed(exec: &CappedRun) -> bool {
    exec.timed_out
        || exec.stdout_truncated
        || exec.stderr_truncated
        || exec.exit_code == Some(OOM_EXIT)
}

const OOM_EXIT: i32 = 137;

fn compile_succeeded(compile: &CappedRun) -> bool {
    !compile.timed_out && compile.exit_code == Some(0)
}

pub fn determine_status(
    mode: Mode,
    compile: &CappedRun,
    exec: Option<&CappedRun>,
    tests: &[TestResult],
) -> JudgeStatus {
    if compile.timed_out {
        return JudgeStatus::Timeout;
    }
    if compile.exit_code == Some(OOM_EXIT) {
        return JudgeStatus::MemoryLimit;
    }
    let Some(exec) = exec.filter(|_| compile_succeeded(compile)) else {
        return JudgeStatus::CompileError;
    };
    if exec.timed_out {
        return JudgeStatus::Timeout;
    }
    // host 因輸出超限 kill 後的 137 不算 OOM
    if exec.stdout_truncated || exec.stderr_truncated {
        return JudgeStatus::OutputLimit;
    }
    if exec.exit_code == Some(OOM_EXIT) {
        return JudgeStatus::MemoryLimit;
    }
    match mode {
        Mode::Run if exec.exit_code == Some(0) => JudgeStatus::Ok,
        Mode::Run => JudgeStatus::RuntimePanic,
        Mode::Submit
            if exec.exit_code == Some(0)
                && tests.iter().all(|t| t.status == TestStatus::Passed) =>
        {
            JudgeStatus::Ok
        }
        Mode::Submit => JudgeStatus::TestFailed,
    }
}

fn compile_result(compile: &CappedRun, diagnostics: Vec<Diagnostic>) -> CompileResult {
    CompileResult {
        success: compile_succeeded(compile),
        timed_out: compile.timed_out,
        diagnostics,
    }
}

fn execution_result(exec: CappedRun) -> ExecutionResult {
    let exit_code = if exec_killed(&exec) {
        None
    } else {
        exec.exit_code
    };
    ExecutionResult {
        exit_code,
        stdout: exec.stdout,
        stderr: exec.stderr,
        timed_out: exec.timed_out,
        stdout_truncated: exec.stdout_truncated,
        stderr_truncated: exec.stderr_truncated,
    }
}

pub fn build_run_response(
    compile: &CappedRun,
    diagnostics: Vec<Diagnostic>,
    exec: Option<CappedRun>,
) -> RunResponse {
    let exec = exec.filter(|_| compile_succeeded(compile));
    RunResponse {
        status: determine_status(Mode::Run, compile, exec.as_ref(), &[]),
        compile: compile_result(compile, diagnostics),
        execution: exec.map(execution_result),
    }
}

pub fn build_submit_response(
    compile: &CappedRun,
    diagnostics: Vec<Diagnostic>,
    exec: Option<CappedRun>,
    test_names: &[String],
) -> SubmitResponse {
    let exec = exec.filter(|_| compile_succeeded(compile));
    let tests = match &exec {
        Some(e) => parse_test_output(&e.stdout, test_names),
        None => all_not_run(test_names),
    };
    let status = determine_status(Mode::Submit, compile, exec.as_ref(), &tests);
    SubmitResponse {
        status,
        passed: status == JudgeStatus::Ok,
        compile: compile_result(compile, diagnostics),
        execution: exec.map(execution_result),
        tests,
    }
}
