// 洞窟：要嘛是死路，要嘛這層有寶石、裡面還有更深的洞窟
enum Cave {
    DeadEnd,
    Tunnel(u32, Box<Cave>),
}

// 由外到內，依每層的寶石數挖出洞窟
fn dig(gems: &[u32]) -> Cave {
    match gems {
        [] => Cave::DeadEnd,
        [first, rest @ ..] => Cave::Tunnel(*first, Box::new(dig(rest))),
    }
}

// 洞窟一共有幾層
fn depth(cave: &Cave) -> u32 {
    match cave {
        Cave::DeadEnd => 0,
        Cave::Tunnel(_, inner) => 1 + depth(inner),
    }
}

// 全部寶石加起來有幾顆
fn total_gems(cave: &Cave) -> u32 {
    match cave {
        Cave::DeadEnd => 0,
        Cave::Tunnel(gems, inner) => gems + total_gems(inner),
    }
}

fn main() {
    let cave = dig(&[3, 1, 4]);
    println!("洞窟有 {} 層，共 {} 顆寶石", depth(&cave), total_gems(&cave));
}
