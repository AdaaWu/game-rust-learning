use std::cell::RefCell;
use std::rc::Rc;

// Ferris 與海鷗共用的寶物箱
struct Chest {
    items: Vec<String>,
}

// 放入寶物（重複就不放），回傳箱內數量
fn deposit(chest: &Rc<RefCell<Chest>>, item: &str) -> usize {
    let exists = chest.borrow().items.iter().any(|i| i.as_str() == item);
    if !exists {
        chest.borrow_mut().items.push(item.to_string());
    }
    chest.borrow().items.len()
}

fn main() {
    let chest = Rc::new(RefCell::new(Chest { items: vec![String::from("珍珠")] }));
    let gull = Rc::clone(&chest);
    println!("箱內 {} 樣", deposit(&chest, "海星"));
    println!("箱內 {} 樣", deposit(&gull, "海星"));
}
