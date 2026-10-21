//! 測試輔助：解析 `@tl|` 事件與循環等待判定（architecture §3.11、ux.md §11.5），只供測試使用

use std::collections::HashMap;

pub const KINDS: [&str; 10] = [
    "start",
    "end",
    "lock_wait",
    "lock_got",
    "lock_free",
    "send",
    "recv_wait",
    "recv",
    "recv_closed",
    "note",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlEvent {
    pub kind: String,
    pub line: Option<u32>,
    pub thread: String,
    pub target: String,
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub events: Vec<TlEvent>,
    /// 以 `@tl|` 開頭但格式不符的原文
    pub malformed: Vec<String>,
}

fn parse_line(rest: &str) -> Option<TlEvent> {
    let parts: Vec<&str> = rest.split('|').collect();
    let [kind, line, thread, target] = parts[..] else {
        return None;
    };
    if !KINDS.contains(&kind) {
        return None;
    }
    let line = match line {
        "" => None,
        n => Some(n.parse().ok()?),
    };
    let thread = thread.trim();
    if thread.is_empty() {
        return None;
    }
    Some(TlEvent {
        kind: kind.to_string(),
        line,
        thread: thread.to_string(),
        target: target.to_string(),
    })
}

/// 逐行解析 stdout；非 `@tl|` 開頭的行（玩家自己的輸出）略過
pub fn parse_events(stdout: &str) -> Parsed {
    let mut out = Parsed::default();
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix("@tl|") else {
            continue;
        };
        match parse_line(rest) {
            Some(e) => out.events.push(e),
            None => out.malformed.push(line.to_string()),
        }
    }
    out
}

/// 泳道排序：`main` 置頂、其餘依首次出現
fn lane_order(events: &[TlEvent]) -> Vec<String> {
    let mut lanes: Vec<String> = Vec::new();
    for e in events {
        if !lanes.contains(&e.thread) {
            lanes.push(e.thread.clone());
        }
    }
    if let Some(i) = lanes.iter().position(|l| l == "main") {
        let main = lanes.remove(i);
        lanes.insert(0, main);
    }
    lanes
}

/// 前綴內的循環等待；回傳依循環順序排列的泳道，從排序最前者開始
fn cycle_in(events: &[TlEvent]) -> Option<Vec<String>> {
    // 鎖 → 未結束的持有區段（泳道, 開始步）
    let mut holds: HashMap<&str, Vec<(&str, usize)>> = HashMap::new();
    let mut last: HashMap<&str, &TlEvent> = HashMap::new();
    for (step, e) in events.iter().enumerate() {
        match e.kind.as_str() {
            "lock_got" => holds
                .entry(e.target.as_str())
                .or_default()
                .push((e.thread.as_str(), step)),
            "lock_free" => {
                if let Some(list) = holds.get_mut(e.target.as_str())
                    && let Some(i) = list.iter().rposition(|(t, _)| *t == e.thread)
                {
                    list.remove(i);
                }
            }
            _ => {}
        }
        last.insert(e.thread.as_str(), e);
    }
    let holder = |lock: &str| -> Option<&str> {
        holds
            .get(lock)?
            .iter()
            .max_by_key(|(_, step)| *step)
            .map(|(t, _)| *t)
    };
    let waits: HashMap<&str, &str> = last
        .iter()
        .filter(|(_, e)| e.kind == "lock_wait")
        .filter_map(|(t, e)| holder(&e.target).map(|h| (*t, h)))
        .collect();

    for start in lane_order(events) {
        let mut path: Vec<&str> = Vec::new();
        let mut cur = start.as_str();
        while let Some(next) = waits.get(cur) {
            if path.contains(&cur) {
                break;
            }
            path.push(cur);
            if *next == start {
                return Some(path.iter().map(|s| s.to_string()).collect());
            }
            cur = next;
        }
    }
    None
}

/// 逐步檢查每個前綴，回傳第一次出現循環的步數（1 起算）與循環
pub fn first_deadlock(events: &[TlEvent]) -> Option<(usize, Vec<String>)> {
    (1..=events.len()).find_map(|n| cycle_in(&events[..n]).map(|c| (n, c)))
}

/// 有 `start` 但沒有 `end` 的泳道
pub fn lanes_without_end(events: &[TlEvent]) -> Vec<String> {
    let started = names_with_kind(events, "start");
    let ended = names_with_kind(events, "end");
    let mut out: Vec<String> = Vec::new();
    for lane in started {
        if !ended.contains(&lane) && !out.contains(&lane) {
            out.push(lane);
        }
    }
    out
}

/// 出現兩次以上 `start` 的泳道名稱
pub fn duplicate_lane_names(events: &[TlEvent]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut dups: Vec<String> = Vec::new();
    for name in names_with_kind(events, "start") {
        if seen.contains(&name) {
            if !dups.contains(&name) {
                dups.push(name);
            }
        } else {
            seen.push(name);
        }
    }
    dups
}

/// 最後一個事件停在 `lock_wait`／`recv_wait` 的泳道
pub fn stuck_lanes(events: &[TlEvent]) -> Vec<String> {
    lane_order(events)
        .into_iter()
        .filter(|lane| {
            events
                .iter()
                .rev()
                .find(|e| &e.thread == lane)
                .is_some_and(|e| e.kind == "lock_wait" || e.kind == "recv_wait")
        })
        .collect()
}

/// `lock_got` 與 `lock_free` 次數不同的鎖：(鎖名, got, free)
pub fn unpaired_locks(events: &[TlEvent]) -> Vec<(String, usize, usize)> {
    count_pairs(events, "lock_got", "lock_free")
}

/// `send` 與 `recv` 次數不同的 channel：(名稱, send, recv)
pub fn unmatched_channels(events: &[TlEvent]) -> Vec<(String, usize, usize)> {
    count_pairs(events, "send", "recv")
}

fn names_with_kind(events: &[TlEvent], kind: &str) -> Vec<String> {
    events
        .iter()
        .filter(|e| e.kind == kind)
        .map(|e| e.thread.clone())
        .collect()
}

fn count_pairs(events: &[TlEvent], open: &str, close: &str) -> Vec<(String, usize, usize)> {
    let mut targets: Vec<&str> = Vec::new();
    for e in events.iter().filter(|e| e.kind == open || e.kind == close) {
        if !targets.contains(&e.target.as_str()) {
            targets.push(&e.target);
        }
    }
    targets
        .into_iter()
        .map(|t| {
            let n = |k: &str| {
                events
                    .iter()
                    .filter(|e| e.kind == k && e.target == t)
                    .count()
            };
            (t.to_string(), n(open), n(close))
        })
        .filter(|(_, a, b)| a != b)
        .collect()
}

/// solution Run 的事件序列應滿足的規則（architecture §6.2 ④）；回傳違反項目
pub fn clean_run_violations(stdout: &str) -> Vec<String> {
    let parsed = parse_events(stdout);
    let ev = &parsed.events;
    let mut out = Vec::new();
    if ev.is_empty() {
        out.push("stdout 沒有任何合法 @tl| 事件".to_string());
    }
    if !parsed.malformed.is_empty() {
        out.push(format!("格式不符的事件行：{:?}", parsed.malformed));
    }
    let open = lanes_without_end(ev);
    if !open.is_empty() {
        out.push(format!("有 start 沒有 end 的泳道：{open:?}"));
    }
    let dups = duplicate_lane_names(ev);
    if !dups.is_empty() {
        out.push(format!("具名泳道名稱重複：{dups:?}"));
    }
    let stuck = stuck_lanes(ev);
    if !stuck.is_empty() {
        out.push(format!("停在等待的泳道：{stuck:?}"));
    }
    let locks = unpaired_locks(ev);
    if !locks.is_empty() {
        out.push(format!("lock_got／lock_free 不成對：{locks:?}"));
    }
    let chans = unmatched_channels(ev);
    if !chans.is_empty() {
        out.push(format!("send／recv 數量不同：{chans:?}"));
    }
    if let Some((step, cycle)) = first_deadlock(ev) {
        out.push(format!("第 {step} 步出現循環等待：{cycle:?}"));
    }
    out
}
