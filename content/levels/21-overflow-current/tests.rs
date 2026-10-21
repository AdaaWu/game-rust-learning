#[test]
fn back_off_inside_sea() {
    ::std::assert_eq!(back_off(5, 2), Some(3), "從第 5 格往左游 2 格應該到第 3 格");
    ::std::assert_eq!(back_off(2, 2), Some(0), "剛好游到岸邊第 0 格也算游得到");
}

#[test]
fn back_off_past_shore() {
    ::std::assert_eq!(back_off(1, 3), None, "從第 1 格往左游 3 格會被沖出岸邊，應該回傳 None");
    ::std::assert_eq!(back_off(0, 1), None, "已經在岸邊就不能再往左");
}

#[test]
fn strong_current_skipped() {
    let cmds = route(2, &[4]);
    ::std::assert!(!cmds.iter().any(|c| c == "MOVE LEFT"), "游不到的那段應該整段跳過，不能只游一半");
    ::std::assert_eq!(cmds.last().map(String::as_str), Some("MOVE DOWN"), "跳過之後還是要往下進港");
}

#[test]
fn route_reaches_harbor() {
    assert_cleared(&level_grid(), &route(START_X, &CHART));
}
