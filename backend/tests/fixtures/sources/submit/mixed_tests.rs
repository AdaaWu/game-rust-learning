#[test]
fn zeta_assert_msg() {
    assert_eq!(1 + 1, 3, "海星數量不對");
}

#[test]
fn alpha_passes() {}

#[test]
fn mid_plain_panic() {
    panic!("背包破了");
}

#[test]
#[ignore]
fn beta_ignored() {}

#[test]
fn gamma_multiline() {
    panic!("第一行\n第二行");
}
