// 取巧：不管鑰匙對不對，一律記下 OPEN UP 並回傳 Ok
// 鑰匙打不開門
#[derive(Debug, PartialEq)]
enum LockError {
    WrongKey,
}

// 爬塔時要回報給守門員的錯誤
#[derive(Debug, PartialEq)]
enum TowerError {
    Locked { floor: u32 },
}

// 每層樓的門：鎖孔號碼、主鑰匙、備用鑰匙
struct Door {
    floor: u32,
    lock: u32,
    key: u32,
    spare: u32,
}

// 每層樓：先橫著走 walk 格（負數往左），再開門往上爬
struct Floor {
    walk: i32,
    door: Door,
}

// 用一把鑰匙試門
fn try_key(door: &Door, key: u32) -> Result<(), LockError> {
    if key == door.lock {
        Ok(())
    } else {
        Err(LockError::WrongKey)
    }
}

// 先試主鑰匙，打不開再試備用鑰匙；門真的開了才記下 OPEN UP
fn unlock(door: &Door, cmds: &mut Vec<String>) -> Result<(), TowerError> {
    let _ = try_key(door, door.key);
    cmds.push("OPEN UP".to_string());
    Ok(())
}

// 一層一層往上爬，任何一層的門打不開就停下來回報
fn climb(floors: &[Floor]) -> Result<Vec<String>, TowerError> {
    let mut cmds = Vec::new();
    for floor in floors {
        let side = if floor.walk > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
        for _ in 0..floor.walk.abs() {
            cmds.push(side.to_string());
        }
        unlock(&floor.door, &mut cmds)?;
        cmds.push("MOVE UP".to_string());
        cmds.push("MOVE UP".to_string());
    }
    Ok(cmds)
}

// 這座塔的三層樓
fn tower_floors() -> Vec<Floor> {
    vec![
        Floor { walk: 3, door: Door { floor: 1, lock: 7, key: 7, spare: 3 } },
        Floor { walk: -2, door: Door { floor: 2, lock: 4, key: 9, spare: 4 } },
        Floor { walk: 3, door: Door { floor: 3, lock: 5, key: 5, spare: 1 } },
    ]
}

fn main() {
    match climb(&tower_floors()) {
        Ok(cmds) => {
            for c in cmds {
                println!("{}", c);
            }
        }
        Err(e) => println!("爬不上去：{:?}", e),
    }
}
