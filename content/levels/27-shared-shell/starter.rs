use std::rc::Rc;

// 貝殼屋：只有一間，大家共住
struct Shell {
    name: String,
    rooms: u32,
}

// 住在貝殼屋裡的螃蟹，手上拿著屋子的鑰匙
struct Crab {
    name: String,
    home: Rc<Shell>,
}

// 讓一隻螃蟹搬進同一間貝殼屋（不是再蓋一間一模一樣的）
fn move_in(name: &str, home: &Rc<Shell>) -> Crab {
    todo!("讓 {} 拿到 {} 的鑰匙", name, home.name)
}

// 現在一共有幾把鑰匙指著這間貝殼屋
fn keys(home: &Rc<Shell>) -> usize {
    todo!("數一數 {} 有幾把鑰匙", home.name)
}

// 螃蟹搬走並交回鑰匙，回傳屋子還剩幾把鑰匙
fn move_out(crab: Crab) -> usize {
    todo!("{} 搬走後，還剩幾把鑰匙？", crab.name)
}

fn main() {
    let shell = Rc::new(Shell { name: String::from("粉紅螺殼"), rooms: 3 });
    let ferris = move_in("Ferris", &shell);
    let hermit = move_in("阿殼", &shell);
    let fiddler = move_in("小紅", &shell);
    println!("{} 有 {} 間房，{} 把鑰匙", shell.name, shell.rooms, keys(&shell));
    println!("{} 搬走了，還剩 {} 把鑰匙", "阿殼", move_out(hermit));
    println!("{} 和 {} 還住在 {}", ferris.name, fiddler.name, fiddler.home.name);
}
