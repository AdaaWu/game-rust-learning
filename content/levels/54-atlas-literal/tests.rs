// 測試自己算的最大值，用來對照 max_of!
fn biggest(values: &[i32]) -> i32 {
    let mut best = values[0];
    for &v in values {
        if v > best {
            best = v;
        }
    }
    best
}

#[test]
fn empty_atlas_has_no_isles() {
    let atlas: HashMap<String, (i32, i32)> = atlas! {};
    ::std::assert!(atlas.is_empty(), "atlas!{{}} 什麼都沒寫，海圖應該是空的");
}

#[test]
fn atlas_accepts_trailing_comma() {
    let with_comma: HashMap<String, (i32, i32)> = atlas! { "珍珠礁" => (2, 5), "藻林" => (-1, 4), };
    let without: HashMap<String, (i32, i32)> = atlas! { "珍珠礁" => (2, 5), "藻林" => (-1, 4) };
    ::std::assert_eq!(with_comma.len(), 2, "有結尾逗號的海圖應該有 2 座島");
    ::std::assert_eq!(with_comma, without, "有沒有結尾逗號，畫出來的海圖都該一樣");
    ::std::assert_eq!(with_comma.get("藻林"), Some(&(-1, 4)), "藻林的座標不對");
}

#[test]
fn atlas_single_isle() {
    let atlas: HashMap<String, (i32, i32)> = atlas! { "孤島" => (7, 7) };
    ::std::assert_eq!(atlas.len(), 1, "只寫一座島，海圖就只有一座島");
    ::std::assert_eq!(atlas.get("孤島"), Some(&(7, 7)), "孤島的座標不對");
}

#[test]
fn atlas_with_runtime_names_and_points() {
    for i in 0..5 {
        let name = ::std::format!("小島{}", i);
        let atlas: HashMap<String, (i32, i32)> = atlas! {
            name.as_str() => (i, -i),
            "燈塔" => (i * 2, i + 1),
            name.clone() + "北" => (0, i),
        };
        ::std::assert_eq!(atlas.len(), 3, "第 {} 張海圖應該有 3 座島", i);
        ::std::assert_eq!(atlas.get(&name), Some(&(i, -i)), "{} 的座標不對", name);
        ::std::assert_eq!(atlas.get("燈塔"), Some(&(i * 2, i + 1)), "燈塔的座標不對");
        ::std::assert_eq!(atlas.get(&::std::format!("{}北", name)), Some(&(0, i)), "{}北 的座標不對", name);
    }
}

#[test]
fn max_of_single_value() {
    let only: i32 = max_of!(3);
    ::std::assert_eq!(only, 3, "max_of!(3) 只有一個，它就是最大的");
    let with_comma: i32 = max_of!(-8,);
    ::std::assert_eq!(with_comma, -8, "max_of!(-8,) 也要接受結尾逗號");
}

#[test]
fn max_of_many_values() {
    let m: i32 = max_of!(1, 9, 4);
    ::std::assert_eq!(m, 9, "max_of!(1, 9, 4) 應該是 9");
    let m: i32 = max_of!(-5, -2, -9,);
    ::std::assert_eq!(m, -2, "max_of!(-5, -2, -9,) 應該是 -2");
    let m: i32 = max_of!(4, 4, 1, 0, 7, 2, 7, 3);
    ::std::assert_eq!(m, 7, "八個數字的最大值應該是 7");
}

#[test]
fn max_of_floats() {
    let m: f64 = max_of!(1.5, 3.25, 2.0);
    ::std::assert_eq!(m, 3.25, "max_of!(1.5, 3.25, 2.0) 應該是 3.25");
    let m: f64 = max_of!(-0.5, -2.75,);
    ::std::assert_eq!(m, -0.5, "max_of!(-0.5, -2.75,) 應該是 -0.5");
    let m: f64 = max_of!(0.125);
    ::std::assert_eq!(m, 0.125, "max_of!(0.125) 只有一個，它就是最大的");
}

#[test]
fn max_of_runtime_values() {
    for a in -3..4 {
        for b in -3..4 {
            for c in -3..4 {
                let m: i32 = max_of!(a, b * 2, c - 1);
                ::std::assert_eq!(m, biggest(&[a, b * 2, c - 1]), "max_of!({}, {}, {}) 算錯了", a, b * 2, c - 1);
                let two: i32 = max_of!(c, a);
                ::std::assert_eq!(two, biggest(&[c, a]), "max_of!({}, {}) 算錯了", c, a);
            }
        }
    }
}

#[test]
fn farthest_isle_route() {
    assert_cleared(&level_grid(), &route());
}
