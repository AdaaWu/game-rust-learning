// 導遊卷軸：借用航道，依序報出每一站
struct Guide<'a> {
    notes: &'a str,
}

impl<'a> Guide<'a> {
    // 照著航道做一份導遊卷軸
    fn new(notes: &'a str) -> Guide<'a> {
        Guide { notes }
    }

    // 第 n 站（從 0 起算）的地名；超過終點回傳 None
    fn next_stop(&self, n: usize) -> Option<&str> {
        self.notes.split(" → ").nth(n)
    }
}

// 碼頭公告：珊瑚礁蓋了堤防，航道改走燈塔灣
fn build_dike(chart: &mut String) {
    *chart = chart.replace("珊瑚礁", "燈塔灣");
}

// 出航：先聽碼頭公告，再領導遊卷軸，照卷軸走完每一站
fn voyage(chart: &mut String) -> Vec<String> {
    build_dike(chart);
    let guide = Guide::new(chart);
    let mut stops = Vec::new();
    let mut n = 0;
    while let Some(stop) = guide.next_stop(n) {
        stops.push(stop.to_string());
        n += 1;
    }
    stops
}

fn main() {
    let mut chart = String::from("碼頭 → 珊瑚礁 → 古碑");
    let stops = voyage(&mut chart);
    println!("最新航道：{}", chart);
    for stop in &stops {
        if chart.contains(stop.as_str()) {
            println!("抵達 {}", stop);
        } else {
            println!("前往 {}……砰！撞上新蓋的堤防", stop);
        }
    }
}
