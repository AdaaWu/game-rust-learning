#[test]
fn decode_all_marks() {
    assert_eq!(
        decode("ULDR*"),
        vec!["MOVE UP", "MOVE LEFT", "MOVE DOWN", "MOVE RIGHT", "PICK"],
        "五種記號都要翻成對應的指令，順序不能亂"
    );
}

#[test]
fn decode_skips_noise() {
    assert!(decode("~ ~xr@").is_empty(), "貝殼、海草、小寫字母都是雜訊，不該變成指令");
    assert_eq!(decode("~R~"), vec!["MOVE RIGHT"], "雜訊中間的記號還是要留下");
}

#[test]
fn count_marks_by_target() {
    assert_eq!(count_marks(SAND, '*'), 2, "沙灘上應該有 2 顆寶石");
    assert_eq!(count_marks(SAND, 'D'), 3, "沙灘上應該有 3 個往下的記號");
    assert_eq!(count_marks("", '*'), 0, "空沙灘沒有任何記號");
}

#[test]
fn route_follows_sand() {
    assert_cleared(&level_grid(), &route());
}
