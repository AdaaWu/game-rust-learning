// 取巧：回傳 String 繞過生命週期標註
fn clean(name: &str) -> &str {
    name.trim()
}
fn longer(a: &str, b: &str) -> String {
    if a.chars().count() >= b.chars().count() { a.to_string() } else { b.to_string() }
}
fn read_tablet(left: &str, right: &str) -> String {
    longer(clean(left), clean(right)).to_string()
}
fn main() {}
