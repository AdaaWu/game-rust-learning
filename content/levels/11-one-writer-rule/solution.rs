// 先記下第一樣寶物，再放入紅寶石
fn first_then_add(bag: &mut Vec<String>) -> String {
    let first = bag[0].clone();
    bag.push(String::from("紅寶石"));
    first
}

fn main() {
    let mut bag = vec![String::from("珍珠")];
    let first = first_then_add(&mut bag);
    println!("{} {:?}", first, bag);
}
