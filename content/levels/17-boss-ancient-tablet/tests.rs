#[test]
fn counts_characters_not_bytes() {
    assert_eq!(Tablet::new("蟹王之墓").char_count(), 4, "古碑上明明只有 4 個字");
    assert_eq!(Tablet::new("Ferris").char_count(), 6, "英文名字的字數也要對");
}

#[test]
fn excerpt_cuts_whole_characters() {
    assert_eq!(Tablet::new("蟹王之墓").excerpt(1, 2), "王之", "從第 1 個字取 2 個字應該是「王之」");
}

#[test]
fn excerpt_handles_mixed_text() {
    assert_eq!(Tablet::new("Ferris到此一遊").excerpt(4, 4), "is到此", "中英混寫的古碑切錯了");
}

#[test]
fn excerpt_stops_at_the_end() {
    let tablet = Tablet::new("蟹王之墓");
    assert_eq!(tablet.excerpt(2, 10), "之墓", "超過結尾時應該取到最後一個字為止");
    assert_eq!(tablet.excerpt(9, 1), "", "起點超過結尾時應該什麼都取不到");
}

#[test]
fn excerpt_borrows_the_original() {
    let text = String::from("蟹王之墓");
    let part = Tablet::new(&text).excerpt(1, 2);
    assert_eq!(part.as_ptr(), text["蟹".len()..].as_ptr(), "解讀結果應該是古碑原文的切片，不是抄一份");
}

#[test]
fn decode_joins_heads() {
    assert_eq!(decode(&["蟹王之墓", "海鷗守門"], 2), "蟹王·海鷗", "古碑解讀結果不對");
    assert_eq!(decode(&["Ferris", "Gull"], 3), "Fer·Gul", "英文古碑解讀結果不對");
}
