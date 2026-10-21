use std::cell::RefCell;
use std::rc::{Rc, Weak};

// 貝殼：名字、珍珠數量，以及相鄰的貝殼
struct Shell {
    name: String,
    pearls: RefCell<u32>,
    neighbors: RefCell<Vec<Weak<Shell>>>,
}

impl Drop for Shell {
    fn drop(&mut self) {
        println!("{} 回到海底", self.name);
    }
}

// 撿起一個新貝殼
fn new_shell(name: &str, pearls: u32) -> Rc<Shell> {
    Rc::new(Shell { name: name.to_string(), pearls: RefCell::new(pearls), neighbors: RefCell::new(Vec::new()) })
}

// 兩個貝殼互相成為鄰居
fn link(a: &Rc<Shell>, b: &Rc<Shell>) {
    a.neighbors.borrow_mut().push(Rc::downgrade(b));
    b.neighbors.borrow_mut().push(Rc::downgrade(a));
}

// 第一個鄰居；沒有鄰居或鄰居已經回到海底就回傳 None
fn first_neighbor(shell: &Shell) -> Option<Rc<Shell>> {
    shell.neighbors.borrow().first().and_then(|n| n.upgrade())
}

// 目前有幾顆珍珠
fn pearls(shell: &Shell) -> u32 {
    *shell.pearls.borrow()
}

// 分珍珠：每個鄰居分到一顆，分完為止
fn share(shell: &Shell) {
    for n in shell.neighbors.borrow().iter() {
        let Some(n) = n.upgrade() else { continue };
        if *shell.pearls.borrow() > 0 {
            *shell.pearls.borrow_mut() -= 1;
            *n.pearls.borrow_mut() += 1;
        }
    }
}

fn main() {
    {
        let clam = new_shell("蛤蜊殼", 3);
        let conch = new_shell("海螺殼", 0);
        link(&clam, &conch);
        share(&clam);
        println!("蛤蜊殼剩 {} 顆，海螺殼有 {} 顆", pearls(&clam), pearls(&conch));
    }
    println!("試煉結束，貝殼都該回到海底了");
}
