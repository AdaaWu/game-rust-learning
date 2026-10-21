mod tower {
    // 每層樓：先橫著走 walk 格（負數往左），再往上爬 up 格
    pub struct Floor {
        pub walk: i32,
        pub up: u32,
    }

    // 塔頂暗門的開法，只有塔裡的人知道
    fn door_code() -> String {
        "OPEN UP".to_string()
    }

    // 守門員給的整份路線：爬完三層樓，打開暗門走到塔頂
    pub(crate) fn plan() -> Vec<String> {
        let floors = [
            Floor { walk: 4, up: 2 },
            Floor { walk: -3, up: 2 },
            Floor { walk: 3, up: 0 },
        ];
        let mut cmds = stairs::climb(&floors);
        cmds.push(door_code());
        cmds.push("MOVE UP".to_string());
        cmds.push("MOVE UP".to_string());
        cmds
    }

    pub(crate) mod stairs {
        use super::Floor;

        // 依序爬每一層樓
        pub fn climb(floors: &[Floor]) -> Vec<String> {
            let mut cmds = Vec::new();
            for floor in floors {
                let side = if floor.walk > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
                for _ in 0..floor.walk.abs() {
                    cmds.push(side.to_string());
                }
                for _ in 0..floor.up {
                    cmds.push("MOVE UP".to_string());
                }
            }
            cmds
        }
    }
}

fn main() {
    let trial = tower::stairs::climb(&[tower::Floor { walk: 1, up: 1 }]);
    println!("試爬一層：{} 步", trial.len());
    for c in tower::plan() {
        println!("{}", c);
    }
}
