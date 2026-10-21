// 測試自己編的潮汐：長度 len，指令依序輪替
fn made_up_chant(len: usize) -> Vec<String> {
    let words = ["MOVE UP", "PICK", "MOVE LEFT", "MOVE DOWN", "OPEN RIGHT"];
    (0..len).map(|i| words[(i * 3 + len) % words.len()].to_string()).collect()
}

#[test]
fn argument_runs_once() {
    let mut calls = 0;
    let mut bag: Vec<String> = Vec::new();
    log_step!(bag, {
        calls += 1;
        ::std::format!("MOVE {}", "UP")
    });
    ::std::assert_eq!(calls, 1, "log_step! 只呼叫一次，參數卻被執行了 {} 次：$e 是程式碼，不是值", calls);
    ::std::assert_eq!(bag, ::std::vec!["MOVE UP"], "行囊裡應該只有一道 MOVE UP");
}

#[test]
fn logs_what_it_reads() {
    let mut tide = ::std::vec!["MOVE LEFT".to_string(), "PICK".to_string()].into_iter();
    let mut bag: Vec<String> = Vec::new();
    log_step!(bag, tide.next().unwrap());
    ::std::assert_eq!(bag, ::std::vec!["MOVE LEFT"], "記進行囊的應該是潮汐剛唸出的第一道");
    ::std::assert_eq!(tide.next(), Some("PICK".to_string()), "只記一步，潮汐應該只往前一道");
}

#[test]
fn follow_keeps_every_step() {
    for len in 0..10 {
        let chant = made_up_chant(len);
        ::std::assert_eq!(follow(chant.clone()), chant, "潮汐唸了 {} 道，Ferris 記下的指令要一模一樣", len);
    }
}

#[test]
fn tide_route_reaches_shore() {
    assert_cleared(&level_grid(), &route());
}
