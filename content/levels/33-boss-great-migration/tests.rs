// 測試用的一段路：只有一道指令
fn short_leg(index: usize, cmd: &'static str, bridges: Vec<usize>) -> Leg {
    Leg { index, moves: ::std::vec![cmd], bridges }
}

// 指揮官的耐心：在另一條執行緒跑 job，超過時限就判定隊伍卡住了
fn within<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static) -> T {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(job());
    });
    match done_rx.recv_timeout(std::time::Duration::from_millis(700)) {
        Ok(value) => value,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            ::std::panic!("隊伍停在海上不動了：是分身在窄橋上互等，還是指揮官在等永遠不會來的回報？")
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            ::std::panic!("指揮官在半路倒下了（panic），看看主控台的訊息")
        }
    }
}

#[test]
fn great_migration_reaches_the_tablet() {
    let route = within(|| plan(legs()));
    let expected = ::std::vec![
        "MOVE DOWN", "MOVE DOWN", "MOVE RIGHT", "MOVE RIGHT", "PICK", "MOVE RIGHT", "MOVE UP", "MOVE UP",
        "MOVE RIGHT", "PICK", "MOVE RIGHT", "OPEN DOWN", "MOVE DOWN", "MOVE DOWN",
        "MOVE RIGHT", "PICK", "MOVE DOWN",
    ];
    ::std::assert_eq!(route, expected, "大遷徙的路線要照分身 0、1、2 的段落順序接起來");
}

#[test]
fn opposite_scouts_both_report() {
    let bridges = std::sync::Arc::new(::std::vec![std::sync::Mutex::new(()), std::sync::Mutex::new(())]);
    let (tx, rx) = std::sync::mpsc::channel();
    for (index, order) in [(0, ::std::vec![0, 1]), (1, ::std::vec![1, 0])] {
        let bridges = std::sync::Arc::clone(&bridges);
        let tx = tx.clone();
        std::thread::spawn(move || scout(short_leg(index, "PICK", order), bridges, tx));
    }
    drop(tx);
    let mut seen = Vec::new();
    for _ in 0..2 {
        match rx.recv_timeout(std::time::Duration::from_millis(700)) {
            Ok((index, _)) => seen.push(index),
            Err(_) => ::std::panic!("兩隻分身從相反方向踏上窄橋，結果誰也不讓誰，一直沒有回報"),
        }
    }
    seen.sort();
    ::std::assert_eq!(seen, ::std::vec![0, 1], "兩隻分身都該回報");
}

#[test]
fn scout_waits_for_occupied_bridge() {
    let bridges = std::sync::Arc::new(::std::vec![std::sync::Mutex::new(()), std::sync::Mutex::new(())]);
    let occupied = bridges[0].lock().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let shared = std::sync::Arc::clone(&bridges);
    std::thread::spawn(move || scout(short_leg(0, "PICK", ::std::vec![0]), shared, tx));
    ::std::assert!(
        rx.recv_timeout(std::time::Duration::from_millis(150)).is_err(),
        "窄橋 0 上明明有人，分身卻沒等就回報了：分身要真的站上（lock）窄橋"
    );
    drop(occupied);
    ::std::assert!(
        rx.recv_timeout(std::time::Duration::from_millis(700)).is_ok(),
        "窄橋 0 讓出來了，分身還是沒有回報"
    );
}

#[test]
fn other_bridge_stays_free() {
    let bridges = std::sync::Arc::new(::std::vec![std::sync::Mutex::new(()), std::sync::Mutex::new(())]);
    let occupied = bridges[1].lock().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let (shared, waiting_tx) = (std::sync::Arc::clone(&bridges), tx.clone());
    std::thread::spawn(move || scout(short_leg(1, "PICK", ::std::vec![1]), shared, waiting_tx));
    std::thread::sleep(std::time::Duration::from_millis(100));
    let shared = std::sync::Arc::clone(&bridges);
    std::thread::spawn(move || scout(short_leg(0, "PICK", ::std::vec![0]), shared, tx));
    match rx.recv_timeout(std::time::Duration::from_millis(700)) {
        Ok((index, _)) => ::std::assert_eq!(index, 0, "窄橋 1 還被佔著，走窄橋 1 的分身不該先回報"),
        Err(_) => ::std::panic!("有分身在等窄橋 1，走窄橋 0 的分身也被擋住了：不同的窄橋不該互相等"),
    }
    drop(occupied);
    ::std::assert!(
        rx.recv_timeout(std::time::Duration::from_millis(700)).is_ok(),
        "窄橋 1 讓出來了，等它的分身還是沒有回報"
    );
}

#[test]
fn scout_holds_every_bridge_until_report() {
    let bridges = std::sync::Arc::new(::std::vec![std::sync::Mutex::new(()), std::sync::Mutex::new(())]);
    let occupied = bridges[1].lock().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let shared = std::sync::Arc::clone(&bridges);
    std::thread::spawn(move || scout(short_leg(0, "PICK", ::std::vec![0, 1]), shared, tx));
    std::thread::sleep(std::time::Duration::from_millis(300));
    ::std::assert!(rx.try_recv().is_err(), "窄橋 1 上明明有人，分身卻沒等就回報了：路上的每座窄橋都要站上（lock）");
    ::std::assert!(
        bridges[0].try_lock().is_err(),
        "分身在等窄橋 1 時放開了窄橋 0：要同時站穩路上所有窄橋，回報完才讓出"
    );
    drop(occupied);
    ::std::assert!(
        rx.recv_timeout(std::time::Duration::from_millis(700)).is_ok(),
        "窄橋 1 讓出來了，分身還是沒有回報"
    );
}

#[test]
fn opposite_bridges_no_deadlock() {
    let route = within(|| {
        plan(::std::vec![
            short_leg(0, "MOVE RIGHT", ::std::vec![0, 1]),
            short_leg(1, "MOVE DOWN", ::std::vec![1, 0]),
        ])
    });
    ::std::assert_eq!(route, ::std::vec!["MOVE RIGHT", "MOVE DOWN"], "兩隻分身過橋後回報的路線不對");
}

#[test]
fn reports_follow_leg_order() {
    let route = within(|| plan(::std::vec![short_leg(1, "PICK", ::std::vec![]), short_leg(0, "MOVE LEFT", ::std::vec![0])]));
    ::std::assert_eq!(route, ::std::vec!["MOVE LEFT", "PICK"], "路線要照段落編號接起來，不是照回報先後");
}

#[test]
fn no_scouts_no_route() {
    ::std::assert!(within(|| plan(Vec::new())).is_empty(), "沒派出分身，指揮官應該馬上回報空路線");
}
