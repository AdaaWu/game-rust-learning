#[test]
fn longer_picks_more_characters() {
    assert_eq!(longer("蟹王", "海鷗守衛"), "海鷗守衛", "海鷗守衛有 4 個字，比蟹王長");
    assert_eq!(longer("Ferris", "蟹"), "Ferris", "Ferris 有 6 個字，比蟹長");
}

#[test]
fn tie_returns_first() {
    assert_eq!(longer("海鷗", "蟹王"), "海鷗", "字數一樣時應該回傳第一個名字");
}

#[test]
fn result_used_while_both_names_alive() {
    let elder = String::from("古老的守護者");
    let picked;
    {
        let king = String::from("蟹王");
        picked = longer(&elder, &king).to_string();
    }
    assert_eq!(picked, "古老的守護者", "兩個名字都還在時挑出的名字不對");
}

#[test]
fn read_tablet_cleans_then_picks() {
    assert_eq!(read_tablet("  蟹王  ", " 海鷗守衛 "), "海鷗守衛", "要先擦掉空白，再挑較長的名字");
}

#[test]
fn longer_points_to_one_of_the_names() {
    let king = String::from("蟹王");
    let guard = String::from("海鷗守衛");
    assert_eq!(longer(&king, &guard).as_ptr(), guard.as_ptr(), "回傳的名字應該就是古碑上的海鷗守衛，而不是另外抄一份");
    assert_eq!(longer(&guard, &king).as_ptr(), guard.as_ptr(), "換個順序，回傳的仍應指向海鷗守衛本身");
    let gull = String::from("海鷗");
    assert_eq!(longer(&gull, &king).as_ptr(), gull.as_ptr(), "字數一樣時，回傳的應該指向第一個名字本身");
}
