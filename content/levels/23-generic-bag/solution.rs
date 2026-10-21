// 工匠送的貝殼，先比大小、再比名字
#[derive(Debug, PartialEq, PartialOrd)]
struct Shell {
    size: u32,
    name: &'static str,
}

// 萬用寶物袋：找出最大的那一個，空袋子回傳 None
fn largest<T: PartialOrd>(items: &[T]) -> Option<&T> {
    let mut best = items.first()?;
    for item in items {
        if item > best {
            best = item;
        }
    }
    Some(best)
}

// 把袋子裡的東西列成一行
fn label<T: std::fmt::Debug>(items: &[T]) -> String {
    format!("袋子裡有 {:?}", items)
}

fn main() {
    let gems = [3, 9, 2];
    let shells = [
        Shell { size: 4, name: "扇貝" },
        Shell { size: 7, name: "海螺" },
        Shell { size: 5, name: "寄居殼" },
    ];
    println!("{}", label(&gems));
    println!("最大的寶石：{:?}", largest(&gems));
    println!("最大的貝殼：{:?}", largest(&shells));
}
