#[test]
fn gull_only_looks() {
    let bag = ::std::vec![String::from("藍寶石"), String::from("貝殼"), String::from("綠寶石")];
    ::std::assert_eq!(gull_count(&bag), 2, "海鷗數錯了：名稱以「寶石」結尾的應該有 2 樣");
    ::std::assert_eq!(bag.len(), 3, "海鷗只能看，背包裡的東西不該變少");
}

#[test]
fn gull_count_empty_bag() {
    ::std::assert_eq!(gull_count(&Vec::new()), 0, "空背包海鷗應該數到 0");
}

#[test]
fn put_in_adds_to_the_end() {
    let mut bag = ::std::vec![String::from("貝殼")];
    put_in(&mut bag, "紅寶石");
    ::std::assert_eq!(bag, ::std::vec![String::from("貝殼"), String::from("紅寶石")], "紅寶石應該放在背包最後面");
}

#[test]
fn bag_stays_with_ferris() {
    let (before, after, bag) = trip();
    ::std::assert_eq!(before, 1, "放入前海鷗應該數到 1 顆寶石");
    ::std::assert_eq!(after, 2, "放入紅寶石後海鷗應該數到 2 顆寶石");
    ::std::assert_eq!(
        bag,
        ::std::vec![String::from("藍寶石"), String::from("貝殼"), String::from("紅寶石")],
        "旅程結束時背包應該還在 Ferris 手上，而且多了紅寶石"
    );
}
