fn two_beaches() -> Map {
    Map { beaches: ::std::vec![String::from("珊瑚灘"), String::from("海龜灣")] }
}

// 等每隻分身回來，收齊回報
fn wait_all(handles: Vec<std::thread::JoinHandle<String>>) -> Vec<String> {
    handles.into_iter().map(|h| h.join().expect("有分身在路上倒下了")).collect()
}

// 每隻分身都要是不同的執行緒，而且不是測試自己這條
fn assert_own_threads(handles: &[std::thread::JoinHandle<String>]) {
    let here = std::thread::current().id();
    let ids: std::collections::HashSet<_> = handles.iter().map(|h| h.thread().id()).collect();
    ::std::assert!(!ids.contains(&here), "有分身沒有出門，留在指揮官身邊");
    ::std::assert_eq!(ids.len(), handles.len(), "有兩隻分身擠在同一條執行緒");
}

// 只有能安全跨執行緒移動（Send）又能共享（Sync）的值才能通過
fn assert_send_sync<T: Send + Sync>(_: &T) {}

#[test]
fn three_clones_two_beaches() {
    let handles = dispatch(two_beaches(), 3);
    assert_own_threads(&handles);
    ::std::assert_eq!(
        wait_all(handles),
        ::std::vec![
            String::from("分身 0 抵達珊瑚灘"),
            String::from("分身 1 抵達海龜灣"),
            String::from("分身 2 抵達珊瑚灘"),
        ],
        "分身的回報內容或順序不對"
    );
}

#[test]
fn no_clone_sent() {
    ::std::assert!(dispatch(two_beaches(), 0).is_empty(), "沒有派出分身，卻有分身出門了");
}

#[test]
fn fifty_clones() {
    let handles = dispatch(two_beaches(), 50);
    assert_own_threads(&handles);
    let reports = wait_all(handles);
    ::std::assert_eq!(reports.len(), 50, "有分身沒有回報");
    ::std::assert_eq!(reports[49], "分身 49 抵達海龜灣", "最後一隻分身走錯海灘了");
}

#[test]
fn roll_call_in_order() {
    let handles = roll_call(3);
    assert_own_threads(&handles);
    ::std::assert_eq!(
        wait_all(handles),
        ::std::vec![String::from("分身 0 到！"), String::from("分身 1 到！"), String::from("分身 2 到！")],
        "點名時有分身喊錯了編號"
    );
}

#[test]
fn packed_map_travels_with_every_clone() {
    let shared = pack(two_beaches());
    assert_send_sync(&shared);
    let copy = shared.clone();
    ::std::assert_eq!(
        shared.beaches.as_ptr(),
        copy.beaches.as_ptr(),
        "每隻分身各抄了一整份地圖：pack 交回的東西 clone 時應該只多一份參照，大家共用同一張"
    );
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let map = shared.clone();
            std::thread::spawn(move || map.beaches[i % map.beaches.len()].clone())
        })
        .collect();
    ::std::assert_eq!(
        wait_all(handles),
        ::std::vec![
            String::from("珊瑚灘"),
            String::from("海龜灣"),
            String::from("珊瑚灘"),
            String::from("海龜灣"),
        ],
        "分身帶出門的地圖讀起來不對"
    );
}
