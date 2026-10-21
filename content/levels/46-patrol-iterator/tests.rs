fn off(dx: i32, dy: i32) -> Offset {
    Offset { dx, dy }
}

fn at(x: i32, y: i32) -> Pos {
    Pos { x, y }
}

// 比對欄位值，不用玩家型別的 ==
fn xy(p: &Pos) -> (i32, i32) {
    (p.x, p.y)
}

fn d(o: &Offset) -> (i32, i32) {
    (o.dx, o.dy)
}

// 測試自己產生的節拍：長度 1–5，含負值與 y 方向
fn sample_patterns() -> Vec<Vec<Offset>> {
    let mut seed: u32 = 2024;
    let mut next = move || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((seed >> 16) % 7) as i32 - 3
    };
    let mut out = Vec::new();
    for len in 1..=5 {
        for _ in 0..4 {
            out.push((0..len).map(|_| off(next(), next())).collect());
        }
    }
    out
}

#[test]
fn add_moves_both_axes() {
    ::std::assert_eq!(xy(&(at(3, 4) + off(2, -5))), (5, -1), "(3, 4) 加上 (2, -5) 應該到 (5, -1)");
    ::std::assert_eq!(xy(&(at(0, 0) + off(0, 1))), (0, 1), "往下一格時 y 應該加 1");
    for p in sample_patterns() {
        for o in p {
            let (dx, dy) = d(&o);
            ::std::assert_eq!(xy(&(at(7, -2) + o)), (7 + dx, -2 + dy), "位置加位移時 x、y 要各自相加");
        }
    }
}

#[test]
fn patrol_cycles_the_pattern() {
    for p in sample_patterns() {
        let got: Vec<(i32, i32)> = Patrol::new(p.clone()).take(p.len() * 3 + 1).map(|o| d(&o)).collect();
        let expected: Vec<(i32, i32)> = (0..p.len() * 3 + 1).map(|i| d(&p[i % p.len()])).collect();
        ::std::assert_eq!(got, expected, "巡邏隊應該從第一拍開始，照節拍一圈一圈循環");
    }
}

#[test]
fn patrol_never_stops() {
    let p = ::std::vec![off(1, 0), off(0, 1), off(-1, 0)];
    ::std::assert_eq!(
        Patrol::new(p.clone()).nth(1_000_000).map(|o| d(&o)),
        Some(d(&p[1_000_000 % 3])),
        "巡邏隊永遠不停：第 1,000,001 拍也要照節拍算出來"
    );
    let mut patrol = Patrol::new(::std::vec![off(0, -1)]);
    for _ in 0..10_000 {
        ::std::assert_eq!(patrol.next().map(|o| d(&o)), Some((0, -1)), "只有一拍的節拍也要一直循環下去");
    }
}

#[test]
fn empty_patrol_has_no_beats() {
    ::std::assert!(Patrol::new(Vec::new()).next().is_none(), "沒有節拍的巡邏隊不該走任何一步");
    ::std::assert!(waypoints(at(1, 1), Patrol::new(Vec::new()), 5).is_empty(), "沒有節拍時不會經過任何位置");
}

#[test]
fn waypoints_follow_n_beats() {
    for p in sample_patterns() {
        for n in [0, 1, 4, 11] {
            let got: Vec<(i32, i32)> = waypoints(at(2, 3), Patrol::new(p.clone()), n).iter().map(xy).collect();
            let mut pos = (2, 3);
            let mut expected = Vec::new();
            for i in 0..n {
                let (dx, dy) = d(&p[i % p.len()]);
                pos = (pos.0 + dx, pos.1 + dy);
                expected.push(pos);
            }
            ::std::assert_eq!(got, expected, "走 {} 拍應該剛好經過 {} 個位置，而且一路累加", n, n);
        }
    }
}

#[test]
fn route_follows_the_patrol() {
    assert_cleared(&level_grid(), &route());
}
