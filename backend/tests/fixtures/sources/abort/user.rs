use std::fmt;

struct Rune(u32);

impl fmt::Display for Rune {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

fn main() {
    println!("開始唸咒");
    println!("{}", Rune(1));
}
