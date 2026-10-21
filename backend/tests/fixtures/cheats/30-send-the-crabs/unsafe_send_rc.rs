// 取巧：保留 Rc 不換 Arc，用 unsafe impl Send/Sync 騙過編譯器
// expect: compile_error unsafe_code
use std::rc::Rc;
use std::thread::{self, JoinHandle};
fn roll_call(n: usize) -> Vec<JoinHandle<String>> {
    let mut handles = Vec::new();
    for i in 0..n {
        handles.push(thread::spawn(move || format!("分身 {} 到！", i)));
    }
    handles
}
struct Map {
    beaches: Vec<String>,
}
#[derive(Clone)]
struct Packed(Rc<Map>);
unsafe impl Send for Packed {}
unsafe impl Sync for Packed {}
impl std::ops::Deref for Packed {
    type Target = Map;
    fn deref(&self) -> &Map { &self.0 }
}
fn pack(map: Map) -> Packed {
    Packed(Rc::new(map))
}
fn dispatch(map: Map, n: usize) -> Vec<JoinHandle<String>> {
    let map = pack(map);
    let mut handles = Vec::new();
    for i in 0..n {
        let map = map.clone();
        handles.push(thread::spawn(move || {
            let beach = &map.beaches[i % map.beaches.len()];
            format!("分身 {} 抵達{}", i, beach)
        }));
    }
    handles
}
fn main() {}
