use std::cell::RefCell;
use std::rc::{Rc, Weak};

// 島嶼：記得自己的名字，也記得隔壁的島
struct Island {
    name: String,
    neighbor: RefCell<Weak<Island>>,
}

impl Drop for Island {
    fn drop(&mut self) {
        println!("{} 沉入海底", self.name);
    }
}

// 浮出一座新島
fn new_island(name: &str) -> Rc<Island> {
    Rc::new(Island { name: name.to_string(), neighbor: RefCell::new(Weak::new()) })
}

// 兩座島互相記住對方
fn link(a: &Rc<Island>, b: &Rc<Island>) {
    *a.neighbor.borrow_mut() = Rc::downgrade(b);
    *b.neighbor.borrow_mut() = Rc::downgrade(a);
}

// 隔壁島的名字；沒有鄰居或鄰島已經沉了就回傳 None
fn neighbor_name(island: &Island) -> Option<String> {
    island.neighbor.borrow().upgrade().map(|n| n.name.clone())
}

fn main() {
    {
        let turtle = new_island("海龜島");
        let coral = new_island("珊瑚島");
        link(&turtle, &coral);
        println!("海龜島的鄰居：{:?}", neighbor_name(&turtle));
        println!("珊瑚島有 {} 個擁有者", Rc::strong_count(&coral));
    }
    println!("離開這片海域了，漩渦應該散了吧？");
}
