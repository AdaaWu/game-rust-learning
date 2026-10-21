#[test]
fn low_tide_walks_out() {
    ::std::assert_eq!(steps_out(20), 3, "水位 20 公分時應該能往外走 3 步");
    ::std::assert_eq!(steps_out(0), 5, "完全退潮時應該能往外走 5 步");
    ::std::assert_eq!(steps_out(50), 0, "水位 50 公分時應該原地不動");
}

#[test]
fn high_tide_steps_back() {
    ::std::assert_eq!(
        steps_out(60),
        -1,
        "水位 60 公分時應該回傳 -1 往岸上退；u8 裝不下負數，要先轉成 i32 再相減"
    );
    ::std::assert_eq!(steps_out(255), -20, "潮汐表最高水位 255 公分時應該回傳 -20");
}

#[test]
fn ferris_reaches_reef_exit() {
    assert_cleared(&level_grid(), &route());
}
