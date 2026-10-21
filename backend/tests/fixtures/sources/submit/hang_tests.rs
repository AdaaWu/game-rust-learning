#[test]
fn a_first() {}

#[test]
fn b_second() {}

#[test]
fn c_hangs() {
    loop { std::hint::spin_loop(); }
}

#[test]
fn d_never() {}
