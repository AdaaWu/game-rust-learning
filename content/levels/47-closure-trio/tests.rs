#[test]
fn repeat_calls_exactly_n_times() {
    for n in [0u32, 1, 2, 3, 7, 20] {
        let mut count = 0u32;
        repeat(n, || count += 1);
        ::std::assert_eq!(count, n, "repeat({}, ..) 應該剛好捏 {} 下，實際 {} 下", n, n, count);
    }
}

#[test]
fn repeat_keeps_closure_state() {
    let mut log = Vec::new();
    let mut beat = 0;
    repeat(5, || {
        beat += 1;
        log.push(beat * 10);
    });
    ::std::assert_eq!(log, ::std::vec![10, 20, 30, 40, 50], "同一個 closure 應該被連續呼叫，狀態一路累積");
}

#[test]
fn repeat_still_accepts_read_only_closure() {
    let shell = String::from("貝殼");
    let seen = std::cell::Cell::new(0usize);
    repeat(4, || seen.set(seen.get() + shell.chars().count()));
    ::std::assert_eq!(seen.get(), 8, "只讀的 closure（Fn）也應該能交給 repeat");
}

#[test]
fn deliver_hands_over_owned_string() {
    for name in ["珍珠", "海星", "古老的貝殼", ""] {
        let s = name.to_string();
        let got = deliver(move || s);
        ::std::assert_eq!(got, name, "deliver 應該把 closure 交出來的東西原封不動回傳");
    }
}

#[test]
fn deliver_calls_closure_once() {
    let mut calls = 0;
    let pearl = String::from("黑珍珠");
    let got = deliver(|| {
        calls += 1;
        pearl
    });
    ::std::assert_eq!(calls, 1, "東西只能交出去一次，closure 應該剛好被呼叫 1 次");
    ::std::assert_eq!(got, "黑珍珠", "deliver 應該回傳 closure 交出來的東西");
}
