// 測試自己畫幾張航道
fn charts() -> Vec<String> {
    let places = ["碼頭", "海龜島", "珊瑚礁", "古碑", "燈塔灣", "沉船", "珊瑚礁東口"];
    let mut out = Vec::new();
    for len in 1..=places.len() {
        for start in 0..places.len() {
            let stops: Vec<&str> = (0..len).map(|k| places[(start + k) % places.len()]).collect();
            out.push(stops.join(" → "));
        }
    }
    out
}

#[test]
fn voyage_follows_the_new_chart() {
    for chart in charts() {
        let mut live = chart.clone();
        let stops = voyage(&mut live);
        let expected: Vec<String> = chart.replace("珊瑚礁", "燈塔灣").split(" → ").map(|s| s.to_string()).collect();
        ::std::assert_eq!(stops, expected, "航道「{}」蓋了堤防之後，Ferris 還照舊航道走", chart);
    }
}

#[test]
fn guide_reads_the_original_chart() {
    for chart in charts() {
        let guide = Guide::new(&chart);
        let start = chart.as_ptr() as usize;
        let end = start + chart.len();
        let mut n = 0;
        while let Some(stop) = guide.next_stop(n) {
            let at = stop.as_ptr() as usize;
            ::std::assert!(
                start <= at && at + stop.len() <= end,
                "導遊卷軸報出的「{}」不在原本的航道上：卷軸自己抄了一份，航道一改就過期了",
                stop
            );
            n += 1;
        }
        ::std::assert!(n > 0, "導遊卷軸一站都報不出來");
        ::std::assert!(
            std::ptr::eq(guide.next_stop(0).unwrap().as_ptr(), chart.as_ptr()),
            "導遊卷軸的第一站應該就是航道開頭那一段，不是另一份複本"
        );
    }
}

#[test]
fn guide_stops_after_the_last_place() {
    let chart = String::from("碼頭 → 珊瑚礁 → 古碑");
    let guide = Guide::new(&chart);
    ::std::assert_eq!(guide.next_stop(2), Some("古碑"), "第 3 站應該是古碑");
    ::std::assert_eq!(guide.next_stop(3), None, "古碑是終點，之後沒有下一站");
}
