// 測試自己產生一批超出刻度的讀數
fn too_high_readings() -> Vec<u32> {
    let mut raws = ::std::vec![256, 257, 300, 511, 512, 1000, 65_535, 65_536, u32::MAX - 1, u32::MAX];
    raws.extend((256..5000).step_by(97));
    raws
}

#[test]
fn tide_within_scale() {
    for raw in 0..=255u32 {
        ::std::assert_eq!(tide_level(raw), Ok(raw as u8), "讀數 {} 在刻度內，應該原樣換成潮位", raw);
    }
}

#[test]
fn tide_above_scale_is_error() {
    for raw in too_high_readings() {
        match tide_level(raw) {
            Err(TideError { raw: reported }) => {
                ::std::assert_eq!(reported, raw, "回報錯誤時要帶上原始讀數 {}", raw)
            }
            Ok(level) => ::std::panic!("讀數 {} 超出刻度，卻被換成潮位 {}：超出刻度要回報 TideError，不能硬塞進 u8", raw, level),
        }
    }
}

#[test]
fn route_crosses_the_sandbar() {
    assert_cleared(&level_grid(), &route());
}
