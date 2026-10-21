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
    for i in 0..net.len() {
        if net[i].rotten {
            net.remove(i);
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
