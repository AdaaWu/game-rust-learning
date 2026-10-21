// &Mutex 能同時交給很多條執行緒，因為 Mutex<u32> 是 Sync
fn assert_sync<T: Sync>(_: &T) {}

#[test]
fn one_crab_one_haul() {
    let total = std::sync::Mutex::new(10u32);
    add_haul(&total, 5);
    assert_eq!(*total.lock().unwrap(), 15, "螃蟹交了 5 顆寶石，計數器卻沒有變");
}

#[test]
fn hundred_crabs_share_one_counter() {
    let total = std::sync::Mutex::new(0u32);
    assert_sync(&total);
    // 分四波出發，每波 25 隻螃蟹同時把寶石加進同一個計數器
    for wave in 0..4u32 {
        std::thread::scope(|s| {
            for crab in 1..=25u32 {
                let total = &total;
                s.spawn(move || add_haul(total, wave * 25 + crab));
            }
        });
    }
    assert_eq!(*total.lock().unwrap(), 5050, "一百隻螃蟹同時交寶石，有寶石在途中消失了");
}

#[test]
fn three_crabs() {
    assert_eq!(gather(vec![3, 5, 7]), 15, "三隻螃蟹的寶石沒有全部算進去");
}

#[test]
fn hundred_crabs() {
    let beaches: Vec<u32> = (1..=100).collect();
    assert_eq!(gather(beaches), 5050, "有寶石在途中消失了");
}

#[test]
fn no_crab() {
    assert_eq!(gather(Vec::new()), 0, "沒有螃蟹出門，計數器卻不是 0");
}
