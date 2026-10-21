// 在另一條執行緒跑 job，最多等 limit_ms 毫秒
fn within<T: Send + 'static>(limit_ms: u64, late: &str, job: impl FnOnce() -> T + Send + 'static) -> T {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(job());
    });
    match done_rx.recv_timeout(std::time::Duration::from_millis(limit_ms)) {
        Ok(value) => value,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => ::std::panic!("{}", late),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            ::std::panic!("指揮部在半路倒下了（panic），看看主控台的訊息")
        }
    }
}

const STILL_WAITING: &str = "螃蟹都回報完了，指揮部還在乾等：relay 手上的 tx 放下（drop）了嗎？";

// 測試自己扮演螃蟹，把 (編號, 指令) 依序送進 channel
fn fake_reports(tx: std::sync::mpsc::Sender<(usize, Vec<String>)>, reports: Vec<(usize, &'static str)>) {
    for (crab, cmd) in reports {
        tx.send((crab, ::std::vec![cmd.to_string()])).unwrap();
    }
}

#[test]
fn relay_reaches_the_goal() {
    let route = within(500, STILL_WAITING, || relay(3));
    assert_cleared(&level_grid(), &route);
}

#[test]
fn scout_reports_its_segment() {
    let (tx, rx) = std::sync::mpsc::channel();
    scout(2, tx);
    let got = rx.recv_timeout(std::time::Duration::from_millis(200)).ok();
    ::std::assert_eq!(got, Some((2, segment(2))), "螃蟹 2 沒有把自己的路段送回指揮部");
}

#[test]
fn collect_sorts_by_crab() {
    let (tx, rx) = std::sync::mpsc::channel();
    fake_reports(tx, ::std::vec![(2, "PICK"), (0, "MOVE UP"), (1, "MOVE LEFT")]);
    let route = within(500, "所有螃蟹都放下 Sender 了，指揮部還在等", move || collect(rx));
    ::std::assert_eq!(route, ::std::vec!["MOVE UP", "MOVE LEFT", "PICK"], "路線要照螃蟹編號接起來，不是照回報先後");
}

#[test]
fn lost_crab_does_not_block() {
    let (tx, rx) = std::sync::mpsc::channel();
    let lost_crab = tx.clone();
    fake_reports(tx, ::std::vec![(0, "MOVE RIGHT")]);
    let route = within(3000, "有一隻螃蟹迷路了，指揮部卻一直等下去：每份回報最多等 PATIENCE 就好", move || {
        collect(rx)
    });
    drop(lost_crab);
    ::std::assert_eq!(route, ::std::vec!["MOVE RIGHT"], "迷路的螃蟹沒回報，其他螃蟹的路段還是要交出來");
}

#[test]
fn slow_crab_still_counted() {
    let (tx, rx) = std::sync::mpsc::channel();
    // 這隻螃蟹走得慢，比 relay 的時限還久才回報，但沒有超過 PATIENCE
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(700));
        fake_reports(tx, ::std::vec![(0, "MOVE DOWN")]);
    });
    let route = within(3000, "慢螃蟹回報完也放下 Sender 了，指揮部還在等", move || collect(rx));
    ::std::assert_eq!(route, ::std::vec!["MOVE DOWN"], "慢一點的螃蟹還在 PATIENCE 內，指揮部卻沒收到牠的回報：要用 recv_timeout 等，不是只看一眼");
}

#[test]
fn no_crabs_no_wait() {
    ::std::assert!(within(500, STILL_WAITING, || relay(0)).is_empty(), "沒派出螃蟹，指揮部應該馬上回報空路線");
}
