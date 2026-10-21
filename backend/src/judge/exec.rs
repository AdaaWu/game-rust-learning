//! 執行外部指令：計時、輸出上限

use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// 輸出超過上限時的處理
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// 讀到上限即終止行程
    Kill,
    /// 超過部分丟棄，行程繼續
    Discard,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CappedRun {
    /// 被 host 終止時為 None
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

/// 累積到 limit + 1 位元組為止；回傳是否已超過上限
fn append_capped(buf: &mut Vec<u8>, chunk: &[u8], limit: usize) -> bool {
    let room = (limit + 1).saturating_sub(buf.len());
    buf.extend_from_slice(&chunk[..chunk.len().min(room)]);
    buf.len() > limit
}

pub async fn run_capped(
    program: &str,
    args: &[String],
    timeout: Duration,
    output_limit: usize,
    overflow: Overflow,
) -> std::io::Result<CappedRun> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut out = child.stdout.take().expect("stdout piped");
    let mut err = child.stderr.take().expect("stderr piped");

    let mut out_buf = Vec::new();
    let mut err_buf = Vec::new();
    let mut out_chunk = [0u8; 8192];
    let mut err_chunk = [0u8; 8192];
    let (mut out_done, mut err_done) = (false, false);
    let mut status = None;
    let mut killed = false;
    let mut timed_out = false;
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    while !(out_done && err_done && status.is_some()) {
        tokio::select! {
            r = out.read(&mut out_chunk), if !out_done => match r {
                Ok(0) | Err(_) => out_done = true,
                Ok(n) => {
                    if append_capped(&mut out_buf, &out_chunk[..n], output_limit)
                        && overflow == Overflow::Kill
                    {
                        killed = true;
                        break;
                    }
                }
            },
            r = err.read(&mut err_chunk), if !err_done => match r {
                Ok(0) | Err(_) => err_done = true,
                Ok(n) => {
                    if append_capped(&mut err_buf, &err_chunk[..n], output_limit)
                        && overflow == Overflow::Kill
                    {
                        killed = true;
                        break;
                    }
                }
            },
            s = child.wait(), if status.is_none() => status = Some(s?),
            _ = &mut deadline => {
                timed_out = true;
                killed = true;
                break;
            }
        }
    }

    if killed {
        let _ = child.start_kill();
        let _ = child.wait().await;
    }
    let (stdout, stdout_truncated) = cap_output(&out_buf, output_limit);
    let (stderr, stderr_truncated) = cap_output(&err_buf, output_limit);
    Ok(CappedRun {
        exit_code: if killed {
            None
        } else {
            status.and_then(|s| s.code())
        },
        stdout,
        stderr,
        timed_out,
        stdout_truncated,
        stderr_truncated,
    })
}

/// 截到 limit 位元組，切在 UTF-8 字元中間時往前退；回傳 (文字, 是否截斷)
pub fn cap_output(bytes: &[u8], limit: usize) -> (String, bool) {
    if bytes.len() <= limit {
        return (String::from_utf8_lossy(bytes).into_owned(), false);
    }
    let mut end = limit;
    // 從截斷點往前找最近的字元起始位元組，字元不完整就退到它之前
    for back in 1..=limit.min(4) {
        let b = bytes[limit - back];
        if b < 0x80 {
            break;
        }
        if b >= 0xC0 {
            let width = match b {
                0xF0.. => 4,
                0xE0.. => 3,
                _ => 2,
            };
            if back < width {
                end = limit - back;
            }
            break;
        }
    }
    (String::from_utf8_lossy(&bytes[..end]).into_owned(), true)
}
