// 導遊卷軸：記下航道，依序報出每一站
struct Guide {
    notes: String,
}

impl Guide {
    // 照著航道做一份導遊卷軸
    fn new(notes: &str) -> Guide {
        Guide { notes: notes.to_string() }
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

// 出航：領導遊卷軸、聽碼頭公告，再照卷軸走完每一站
fn voyage(chart: &mut String) -> Vec<String> {
    let guide = Guide::new(chart);
    build_dike(chart);
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
