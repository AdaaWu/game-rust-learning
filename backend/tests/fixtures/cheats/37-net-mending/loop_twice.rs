// 取巧：保留索引 remove 的寫法，加上邊界檢查再多跑一遍，想把漏網的補回來
// 網裡的一條魚
#[derive(Debug, Clone, PartialEq)]
struct Fish {
    name: String,
    rotten: bool,
}

fn fish(name: &str, rotten: bool) -> Fish {
    Fish { name: name.to_string(), rotten }
}

// 把腐壞的魚從網裡拿掉，其他魚保持原本的順序
fn mend(net: &mut Vec<Fish>) {
    // 跑兩遍，第一遍漏掉的第二遍再補
    for _ in 0..2 {
        let mut i = 0;
        while i < net.len() {
            if net[i].rotten {
                net.remove(i);
            }
            i += 1;
        }
    }
}

fn main() {
    let mut net = vec![
        fish("鯖魚", true),
        fish("沙丁魚", true),
        fish("鮪魚", false),
        fish("秋刀魚", true),
        fish("鯛魚", false),
    ];
    mend(&mut net);
    for f in &net {
        println!("留下 {}", f.name);
    }
}
