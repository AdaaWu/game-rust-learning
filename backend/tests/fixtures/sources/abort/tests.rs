#[test]
fn a_first() {
    assert_eq!(Rune(1).0, 1);
}

#[test]
fn b_overflows() {
    let _ = Rune(2).to_string();
}

#[test]
fn c_never() {}
