// 取巧：不標生命週期，用 unsafe transmute 把回傳值硬轉成 'static
// expect: compile_error unsafe_code
fn clean(name: &str) -> &str {
    name.trim()
}
fn longer(a: &str, b: &str) -> &'static str {
    let r = if a.chars().count() >= b.chars().count() { a } else { b };
    unsafe { std::mem::transmute::<&str, &'static str>(r) }
}
fn read_tablet(left: &str, right: &str) -> String {
    longer(clean(left), clean(right)).to_string()
}
fn main() {}
