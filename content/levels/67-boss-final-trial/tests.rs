type CellFn = fn(i64, i64) -> Option<(usize, usize)>;

// 指揮部的耐心：在另一條執行緒跑 job，超過時限就判定試煉卡住了
fn within<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static) -> T {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(job());
    });
    match done_rx.recv_timeout(std::time::Duration::from_millis(500)) {
        Ok(value) => value,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            ::std::panic!("指揮部等不到回報，整個試煉卡住了：偵察隊回來時，回報板寫得上去嗎？")
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            ::std::panic!("指揮部在半路倒下了（panic），看看主控台的訊息")
        }
    }
}

fn mission(team: usize, report: &str) -> scout::Mission {
    scout::Mission { team, report: report.to_string() }
}

fn gather_in_time(missions: Vec<scout::Mission>) -> Result<Vec<(usize, usize)>, command::TrialError> {
    within(move || command::gather(missions))
}

#[test]
fn final_trial_reaches_the_goal() {
    let cells = gather_in_time(scout::missions()).expect("這次試煉的回報都看得懂，不該回報錯誤");
    assert_cleared(&level_grid(), &command::route(&cells));
}

#[test]
fn every_team_is_heard() {
    for n in 1..=4usize {
        let missions: Vec<scout::Mission> = (1..=n).map(|team| mission(team, &::std::format!("{},{}", team, team % 5))).collect();
        let cells = gather_in_time(missions).expect("回報都看得懂，不該回報錯誤");
        let expected: Vec<(usize, usize)> = (1..=n).map(|team| (team, team % 5)).collect();
        ::std::assert_eq!(cells, expected, "派出 {} 隊，指揮部整理出的寶石格子不對", n);
    }
}

#[test]
fn bad_report_is_not_swallowed() {
    let broken = ["4,x", "4", "", "a,b;1,1", "1,1;2"];
    for (i, bad) in broken.iter().enumerate() {
        let bad_team = i % 3 + 1;
        let missions: Vec<scout::Mission> = (1..=3)
            .map(|team| if team == bad_team { mission(team, bad) } else { mission(team, "1,1") })
            .collect();
        let got = gather_in_time(missions);
        ::std::assert!(
            ::std::matches!(got, Err(command::TrialError::BadReport { team }) if team == bad_team),
            "第 {} 隊回報「{}」看不懂，應回報第 {} 隊的錯誤，不能讓那一隊就這樣消失，實際 {:?}",
            bad_team, bad, bad_team, got
        );
    }
}

#[test]
fn route_order_follows_team_numbers() {
    let order = [4, 2, 5, 1, 3];
    for round in 1..=20 {
        let missions: Vec<scout::Mission> = order.iter().map(|&team| mission(team, &::std::format!("{},0", team))).collect();
        let cells = gather_in_time(missions).expect("回報都看得懂，不該回報錯誤");
        ::std::assert_eq!(
            cells,
            ::std::vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)],
            "第 {} 次整理的順序跟隊號對不上：路線要照隊號排，每次都一樣",
            round
        );
    }
}

#[test]
fn off_chart_points_are_skipped() {
    let missions = ::std::vec![
        mission(1, "-1,0;1,1"),
        mission(2, "8,0;0,-2;6,3"),
        mission(3, "0,5;3,2;-9223372036854775808,0"),
    ];
    let cells = gather_in_time(missions).expect("回報都看得懂，不該回報錯誤");
    ::std::assert_eq!(cells, ::std::vec![(1, 1), (6, 3), (3, 2)], "海圖外的座標要略過，海圖內的照隊號留下");
    for x in -3..=10i64 {
        for y in -3..=7i64 {
            let inside = (0..8).contains(&x) && (0..5).contains(&y);
            let expected = if inside { Some((x as usize, y as usize)) } else { None };
            ::std::assert_eq!(command::to_cell(x, y), expected, "海圖座標 ({}, {}) 換成格子的結果不對", x, y);
        }
    }
}

#[test]
fn scout_writes_the_board_from_its_own_thread() {
    fn assert_send<T: Send>() {}
    assert_send::<scout::Mission>();
    let board = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (tx, rx) = std::sync::mpsc::channel();
    let shared = std::sync::Arc::clone(&board);
    std::thread::spawn(move || scout::explore(mission(7, "1,1"), shared, tx));
    let team = rx
        .recv_timeout(std::time::Duration::from_millis(500))
        .expect("第 7 隊出海後一直沒有回來");
    ::std::assert_eq!(team, 7, "回來的應該是第 7 隊");
    ::std::assert_eq!(*board.lock().unwrap(), ::std::vec![(7, String::from("1,1"))], "第 7 隊的回報沒有寫上回報板");
}

// 正確的換算
fn cell_reference(x: i64, y: i64) -> Option<(usize, usize)> {
    if (0..8).contains(&x) && (0..5).contains(&y) { Some((x as usize, y as usize)) } else { None }
}

// 正確結果，換一種寫法
fn cell_reference_try(x: i64, y: i64) -> Option<(usize, usize)> {
    let cell = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
    (cell.0 < 8 && cell.1 < 5).then_some(cell)
}

fn mutant_cast_only(x: i64, y: i64) -> Option<(usize, usize)> {
    Some((x as usize, y as usize))
}

fn mutant_upper_only(x: i64, y: i64) -> Option<(usize, usize)> {
    if x < 8 && y < 5 { Some((x as usize, y as usize)) } else { None }
}

fn mutant_y_unchecked(x: i64, y: i64) -> Option<(usize, usize)> {
    if (0..8).contains(&x) && y < 5 { Some((x as usize, y as usize)) } else { None }
}

fn mutant_edge_inclusive(x: i64, y: i64) -> Option<(usize, usize)> {
    if (0..=8).contains(&x) && (0..=5).contains(&y) { Some((x as usize, y as usize)) } else { None }
}

fn mutant_drops_last_column(x: i64, y: i64) -> Option<(usize, usize)> {
    if (0..7).contains(&x) && (0..5).contains(&y) { Some((x as usize, y as usize)) } else { None }
}

fn mutant_drops_last_row(x: i64, y: i64) -> Option<(usize, usize)> {
    if (0..8).contains(&x) && (0..4).contains(&y) { Some((x as usize, y as usize)) } else { None }
}

fn mutant_swapped(x: i64, y: i64) -> Option<(usize, usize)> {
    cell_reference(x, y).map(|(cx, cy)| (cy, cx))
}

fn mutant_rejects_origin(x: i64, y: i64) -> Option<(usize, usize)> {
    if x > 0 && y > 0 { cell_reference(x, y) } else { None }
}

// 執行玩家的 check_to_cell，暫時關掉 panic 訊息，回傳是否 panic 與訊息
fn run_check(f: CellFn) -> Result<(), String> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(move || command::check_to_cell(f));
    std::panic::set_hook(previous);
    outcome.map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

#[test]
fn checker_accepts_correct_cells() {
    for f in [cell_reference as CellFn, cell_reference_try as CellFn] {
        if let Err(msg) = run_check(f) {
            ::std::panic!("你的 check_to_cell 把正確的換算也判成錯：{}", msg);
        }
    }
}

#[test]
fn checker_catches_wrong_cells() {
    let mutants: [(&str, CellFn); 8] = [
        ("負座標直接 as usize，沒有任何檢查", mutant_cast_only),
        ("只檢查上界，負座標 as usize 變成極大值", mutant_upper_only),
        ("x 檢查完整，y 只檢查上界", mutant_y_unchecked),
        ("寬度與高度那一格也算在海圖內", mutant_edge_inclusive),
        ("最右一欄被當成海圖外", mutant_drops_last_column),
        ("最下一列被當成海圖外", mutant_drops_last_row),
        ("x 與 y 對調", mutant_swapped),
        ("第 0 欄與第 0 列被當成海圖外", mutant_rejects_origin),
    ];
    for (name, f) in mutants {
        ::std::assert!(run_check(f).is_err(), "你的 check_to_cell 沒抓到：{}（把這個錯誤版本傳進去時沒有 panic）", name);
    }
}
