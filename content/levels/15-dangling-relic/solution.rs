// 在遺跡裡用名字鑄造寶冠，帶回地面
fn forge_relic(name: &str) -> String {
    let relic = format!("{}之冠", name);
    relic
}

// 進遺跡拿寶物，遺跡關門後把寶物帶出來
fn escape(name: &str) -> String {
    let carried: String;
    {
        let relic = format!("{}的寶冠", name);
        carried = relic;
    } // 遺跡關門
    carried
}

fn main() {
    println!("{}", forge_relic("蟹王"));
    println!("{}", escape("海鷗"));
}
