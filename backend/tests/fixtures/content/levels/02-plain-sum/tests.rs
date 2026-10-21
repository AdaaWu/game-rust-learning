#[test]
fn sums_three() {
    assert_eq!(total(&[1, 2, 3]), 6, "總和不對");
}

// 輔助函式不是測試
fn ones(n: usize) -> Vec<i32> {
    vec![1; n]
}

#[test]
fn empty_is_zero() {
    assert_eq!(total(&[]), 0);
}

#[test]
fn all_ones() {
    assert_eq!(total(&ones(2)), 2);
}
