// 測試自己累加 n 步，每步 step 公尺
fn sum_steps(step: f64, n: usize) -> f64 {
    let mut depth = 0.0;
    for _ in 0..n {
        depth += step;
    }
    depth
}

// 只留下 Ferris 看得懂的指令
fn only_commands(cmds: &[String]) -> Vec<String> {
    cmds.iter().filter(|c| c.as_str() == "MOVE DOWN" || c.as_str() == "PICK").cloned().collect()
}

#[test]
fn accumulated_depth_counts_as_reached() {
    for (step, hundredths) in [(0.1, 10), (0.05, 5), (0.2, 20), (0.3, 30), (0.15, 15), (0.01, 1)] {
        for n in 1..=20 {
            let depth = sum_steps(step, n);
            let target = (hundredths * n) as f64 / 100.0;
            ::std::assert!(
                reached(depth, target),
                "每次潛 {} 公尺、潛了 {} 次，深度計顯示 {}，應該算到達 {} 公尺",
                step, n, depth, target
            );
        }
    }
}

#[test]
fn far_depth_is_not_reached() {
    let cases = [(0.3, 0.31), (0.3, 0.29), (0.0, 0.3), (1.0, 0.3), (0.8, 0.7), (2.0, 1.99), (0.5, 0.55)];
    for (depth, target) in cases {
        ::std::assert!(!reached(depth, target), "深度 {} 公尺離 {} 公尺還差得遠，不能算到達", depth, target);
    }
    for n in 1..=20 {
        let depth = sum_steps(0.1, n);
        let target = (n + 1) as f64 / 10.0;
        ::std::assert!(!reached(depth, target), "才潛到 {} 公尺，還沒到 {} 公尺", depth, target);
    }
}

// 第幾次往下潛之後撿了寶石
fn pick_after(cmds: &[String]) -> Vec<usize> {
    let mut downs = 0;
    let mut picks = Vec::new();
    for c in only_commands(cmds) {
        if c == "MOVE DOWN" {
            downs += 1;
        } else {
            picks.push(downs);
        }
    }
    picks
}

#[test]
fn dive_picks_at_every_gem_depth() {
    let cases: [(&[f64], Vec<usize>); 5] = [
        (&[0.1; 3], ::std::vec![3]),
        (&[0.05; 16], ::std::vec![6, 14, 16]),
        (&[0.1; 9], ::std::vec![3, 7, 8]),
        (&[0.15, 0.15, 0.2, 0.2, 0.1], ::std::vec![2, 4, 5]),
        (&[0.25, 0.25, 0.25], ::std::vec![]),
    ];
    for (steps, expected) in cases {
        ::std::assert_eq!(
            pick_after(&dive_route(steps)),
            expected,
            "照 {:?} 往下潛，應該在第 {:?} 次潛完撿寶石",
            steps, expected
        );
    }
}

#[test]
fn route_reaches_the_shore() {
    assert_cleared(&level_grid(), &route());
}
