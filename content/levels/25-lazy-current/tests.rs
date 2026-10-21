#[test]
fn haul_total() {
    let (total, _) = haul(&[3, 5, 7]);
    ::std::assert_eq!(total, 15, "三片海灘共 15 顆寶石，計數器卻沒有動");
}

#[test]
fn haul_log_each_beach() {
    let (_, log) = haul(&[3, 5, 7]);
    ::std::assert_eq!(log.len(), 3, "每片海灘都要寫一行日誌");
    ::std::assert_eq!(log[1], "第 2 片海灘：5 顆", "第二行日誌內容不對");
}

#[test]
fn haul_empty() {
    let (total, log) = haul(&[]);
    ::std::assert_eq!(total, 0, "沒有海灘就沒有寶石");
    ::std::assert!(log.is_empty(), "沒有海灘就沒有日誌");
}

#[test]
fn rich_beaches_above_threshold() {
    ::std::assert_eq!(rich_beaches(&[3, 5, 7], 4), ::std::vec![2, 3], "超過 4 顆的是第 2、3 片海灘");
}
