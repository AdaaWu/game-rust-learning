// 測試自己排一串寶石：第 k 顆在 (2k + 1, k)
fn gem_row(n: usize) -> Vec<(usize, usize)> {
    (0..n).map(|k| (2 * k + 1, k)).collect()
}

#[test]
fn empty_island_has_no_last_gem() {
    ::std::assert_eq!(last_gem(&[]), None, "空無一物的島沒有最後一顆寶石，應該回傳 None，不是讓 Ferris 掉進海裡");
}

#[test]
fn single_gem_is_the_last() {
    for x in 0..5 {
        ::std::assert_eq!(last_gem(&[(x, 3)]), Some((x, 3)), "島上只有 ({}, 3) 一顆寶石，它就是最後一顆", x);
    }
}

#[test]
fn last_gem_of_many() {
    for n in 0..8 {
        let gems = gem_row(n);
        let expected = if n == 0 { None } else { Some((2 * n - 1, n - 1)) };
        ::std::assert_eq!(last_gem(&gems), expected, "有 {} 顆寶石的島，最後一顆找錯了", n);
    }
}

#[test]
fn scan_row_walks_to_the_edge() {
    ::std::assert!(scan_row(0).is_empty(), "寬度 0 的長堤不用走");
    ::std::assert!(scan_row(1).is_empty(), "寬度 1 的長堤，Ferris 已經站在最右格");
    for width in 2..10 {
        ::std::assert_eq!(scan_row(width).len(), width - 1, "寬 {} 格的長堤只要往右走 {} 步，多走一步就掉出地圖", width, width - 1);
    }
}

#[test]
fn route_clears_the_islands() {
    assert_cleared(&level_grid(), &route());
}
