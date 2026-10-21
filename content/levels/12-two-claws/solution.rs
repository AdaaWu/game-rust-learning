// 左鉗擦亮寶石
fn polish(gem: &mut String) {
    gem.push_str("✨");
}

// 右鉗刻上主人的名字
fn engrave(gem: &mut String, name: &str) {
    gem.push_str("（");
    gem.push_str(name);
    gem.push_str("）");
}

// 兩隻鉗子一起加工寶石：先擦亮再刻字，回傳成品
fn two_claws(raw: &str, name: &str) -> String {
    let mut gem = raw.to_string();
    let left = &mut gem;
    polish(left);
    let right = &mut gem;
    engrave(right, name);
    gem
}

fn main() {
    println!("{}", two_claws("珍珠", "Ferris"));
}
