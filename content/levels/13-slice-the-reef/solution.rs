// 只切出地圖的一段給隊友看（這張地圖只有 ASCII 字元）
fn segment(map: &str, start: usize, len: usize) -> &str {
    &map[start..start + len]
}

// 找出最深的水深；u32 是 Copy，讀出來就是一份複本
fn deepest(depths: &[u32]) -> u32 {
    let mut max = depths[0];
    for d in depths {
        if *d > max { max = *d; }
    }
    max
}

// 回傳第一位隊友的名字；String 不是 Copy，要自己複製一份
fn first_mate(crew: &[String]) -> String {
    crew[0].clone()
}

fn main() {
    let map = String::from("~~##~~**~~");
    let depths = vec![12, 3, 9, 4];
    let crew = vec![String::from("海鷗"), String::from("寄居蟹")];
    println!("給隊友看：{}", segment(&map, 2, 4));
    println!("後段最深：{}", deepest(&depths[1..]));
    println!("第一位：{}，隊伍還在：{:?}", first_mate(&crew), crew);
}
