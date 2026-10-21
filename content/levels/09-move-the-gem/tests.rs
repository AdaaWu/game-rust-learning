#[test]
fn report_counts_and_names_first() {
    let (n, first) = report();
    ::std::assert_eq!(n, 3, "背包數量不對");
    ::std::assert_eq!(first, "珍珠", "第一樣寶物不對");
}
