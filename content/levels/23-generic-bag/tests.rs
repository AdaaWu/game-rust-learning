#[test]
fn largest_number() {
    ::std::assert_eq!(largest(&[3, 9, 2]), Some(&9), "數字袋裡最大的應該是 9");
}

#[test]
fn largest_word() {
    ::std::assert_eq!(largest(&["clam", "starfish", "crab"]), Some(&"starfish"), "字串依字典順序比大小");
}

#[test]
fn empty_bag() {
    let bag: [u8; 0] = [];
    ::std::assert_eq!(largest(&bag), None, "空袋子應該回傳 None，不能 panic");
}

#[test]
fn largest_shell() {
    let shells = [
        Shell { size: 4, name: "扇貝" },
        Shell { size: 7, name: "海螺" },
        Shell { size: 5, name: "寄居殼" },
    ];
    let best = largest(&shells).expect("貝殼袋不是空的");
    ::std::assert_eq!(best.name, "海螺", "貝殼應該先比 size，最大的是海螺");
}

#[test]
fn label_lists_items() {
    ::std::assert_eq!(label(&[1, 2]), "袋子裡有 [1, 2]", "label 要用 Debug 格式列出袋子內容");
}
