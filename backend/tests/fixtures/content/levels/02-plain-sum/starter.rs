fn total(xs: &[i32]) -> i32 {
    xs.iter().map(|x| x * x).sum()
}

fn main() {
    println!("{}", total(&[1, 2, 3]));
}
