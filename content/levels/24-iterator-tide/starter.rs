// 退潮後沙灘上的記號：R 右、L 左、U 上、D 下、* 寶石，其他都是貝殼與海草
const SAND: &str = "R~R*D~~D R*D";

// 把沙灘記號翻成 Ferris 指令，雜訊一律略過
fn decode(marks: &str) -> Vec<String> {
    todo!("用 chars、filter、map、collect 串成一條 iterator 鏈")
}

// 數一數沙灘上有幾個 target 記號
fn count_marks(marks: &str, target: char) -> usize {
    todo!("用 filter 搭配會捕捉 target 的 closure，再 count")
}

fn route() -> Vec<String> {
    decode(SAND)
}

fn main() {
    println!("沙灘上有 {} 顆寶石", count_marks(SAND, '*'));
    for c in route() {
        println!("{}", c);
    }
}
