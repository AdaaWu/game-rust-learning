// 測試自己算出每層樓該有的指令
fn expected_climb(floors: &[(i32, u32)]) -> Vec<String> {
    let mut cmds = Vec::new();
    for &(walk, up) in floors {
        let side = if walk > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
        for _ in 0..walk.abs() {
            cmds.push(side.to_string());
        }
        for _ in 0..up {
            cmds.push("MOVE UP".to_string());
        }
    }
    cmds
}

fn build_floors(spec: &[(i32, u32)]) -> Vec<tower::Floor> {
    spec.iter().map(|&(walk, up)| tower::Floor { walk, up }).collect()
}

#[test]
fn plan_reaches_tower_top() {
    assert_cleared(&level_grid(), &tower::plan());
}

#[test]
fn climb_follows_each_floor() {
    let mut seed: u32 = 17;
    for round in 0..40 {
        let count = (round % 5) + 1;
        let mut spec = Vec::new();
        for _ in 0..count {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let walk = (seed >> 8) as i32 % 9 - 4;
            let up = (seed >> 20) % 4;
            spec.push((walk, up));
        }
        ::std::assert_eq!(
            tower::stairs::climb(&build_floors(&spec)),
            expected_climb(&spec),
            "樓層 {:?} 的爬法不對",
            spec
        );
    }
}

#[test]
fn climb_handles_edge_floors() {
    ::std::assert!(tower::stairs::climb(&[]).is_empty(), "沒有樓層時不該有任何指令");
    let spec = [(0, 2), (-2, 0), (0, 0)];
    ::std::assert_eq!(
        tower::stairs::climb(&build_floors(&spec)),
        expected_climb(&spec),
        "只往上、只橫走、原地不動的樓層都要照實爬"
    );
}
