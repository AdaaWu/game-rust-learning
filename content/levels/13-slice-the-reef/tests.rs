#[test]
fn segment_is_a_view_not_a_copy() {
    let map = String::from("~~##~~**~~");
    let part = segment(&map, 2, 4);
    assert_eq!(part, "##~~", "切給隊友的那一段不對");
    assert_eq!(part.as_ptr(), map[2..].as_ptr(), "切片應該指向原本的地圖，而不是另外複製一份");
}

#[test]
fn segment_whole_map() {
    let map = String::from("**~~");
    assert_eq!(segment(&map, 0, 4), "**~~", "從頭切到尾應該就是整張地圖");
}

#[test]
fn deepest_reads_copy_values() {
    assert_eq!(deepest(&[3, 9, 4]), 9, "最深的水深應該是 9");
    let depths = vec![12, 3, 9, 4];
    assert_eq!(deepest(&depths[1..]), 9, "只看後段時，最深的應該是 9，不是第一格的 12");
    assert_eq!(depths.len(), 4, "看完水深，原本的 Vec 應該還在");
}

#[test]
fn first_mate_is_a_clone() {
    let crew = vec![String::from("海鷗"), String::from("寄居蟹")];
    let first = first_mate(&crew);
    assert_eq!(first, "海鷗", "第一位隊友應該是海鷗");
    assert_eq!(crew.len(), 2, "隊伍名單不該少人");
    assert_ne!(first.as_ptr(), crew[0].as_ptr(), "回傳的名字應該是複製出來的新字串");
}
