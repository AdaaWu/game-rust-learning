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

// 測試專用夥伴：一拍走三格
struct Turtle;

impl Walker for Turtle {
    fn name(&self) -> String {
        "海龜".to_string()
    }

    fn stride(&self) -> usize {
        3
    }
}

// 測試自己產生的節拍：長度 2–5，只有上下左右
fn sample_patterns() -> Vec<Vec<Offset>> {
    let dirs = [off(1, 0), off(0, 1), off(-1, 0), off(0, -1)];
    let mut seed: u32 = 52;
    let mut out = Vec::new();
    for len in 2..=5 {
        for _ in 0..4 {
            out.push(
                (0..len)
                    .map(|_| {
                        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                        dirs[((seed >> 16) % 4) as usize]
                    })
                    .collect(),
            );
        }
    }
    out
}

// 第一道關卡：心跳要從第一拍開始
fn check_heartbeat() {
    let got: Vec<(i32, i32)> = Heartbeat::new(::std::vec![off(1, 0), off(0, 1), off(-1, 0)]).take(4).map(|o| d(&o)).collect();
    ::std::assert_eq!(got.first(), Some(&(1, 0)), "心跳的第一拍被跳過了：第一步應該是 MOVE RIGHT");
    ::std::assert_eq!(got, ::std::vec![(1, 0), (0, 1), (-1, 0), (1, 0)], "心跳應該照節拍一圈一圈循環");
    for p in sample_patterns() {
        let got: Vec<(i32, i32)> = Heartbeat::new(p.clone()).take(p.len() * 2 + 1).map(|o| d(&o)).collect();
        let expected: Vec<(i32, i32)> = (0..p.len() * 2 + 1).map(|i| d(&p[i % p.len()])).collect();
        ::std::assert_eq!(got, expected, "心跳應該從 pattern 的第一拍開始循環");
    }
    ::std::assert!(Heartbeat::new(Vec::new()).next().is_none(), "沒有節拍的心跳不該有任何一拍");
}

// 第二道關卡：同一隻海鷗，不管怎麼呼叫步幅都一樣
fn check_stride() {
    let gull = Seagull;
    let as_dyn: &dyn Walker = &gull;
    ::std::assert_eq!(
        gull.stride(),
        as_dyn.stride(),
        "海鷗直接呼叫 stride() 走 {} 格，經過 dyn Walker 卻走 {} 格：兩段路線接不起來",
        gull.stride(),
        as_dyn.stride()
    );
    ::std::assert_eq!(as_dyn.stride(), 2, "海鷗一拍應該飛 2 格");
    let crab: &dyn Walker = &Crab;
    ::std::assert_eq!((Crab.stride(), crab.stride()), (1, 1), "Ferris 一拍走 1 格");
}

// 第三道關卡：位置加位移，x、y 各自相加
fn check_add() {
    ::std::assert_eq!(xy(&(at(3, 5) + off(2, -4))), (5, 1), "(3, 5) 加上 (2, -4) 應該到 (5, 1)");
    ::std::assert_eq!(xy(&(at(0, 0) + off(0, 1))), (0, 1), "往下一格，y 應該加 1");
    for p in sample_patterns() {
        for o in p {
            let (dx, dy) = d(&o);
            ::std::assert_eq!(xy(&(at(-7, 11) + o)), (-7 + dx, 11 + dy), "位置加位移時 x、y 要各自相加");
        }
    }
}

#[test]
fn heart_beats_from_first() {
    check_heartbeat();
    ::std::assert_eq!(route().first().map(|s| s.as_str()), Some("MOVE RIGHT"), "第一步應該是 MOVE RIGHT");
}

#[test]
fn seagull_stride_matches_everywhere() {
    check_heartbeat();
    check_stride();
}

#[test]
fn add_moves_both_axes() {
    check_heartbeat();
    check_stride();
    check_add();
}

#[test]
fn march_with_any_squad() {
    check_heartbeat();
    check_stride();
    check_add();
    // 從遠離寶石的地方出發，只看移動
    for p in sample_patterns() {
        let squad: Vec<Box<dyn Walker>> = ::std::vec![Box::new(Turtle), Box::new(Seagull), Box::new(Crab)];
        let (cmds, end) = march(&squad, &p, 3, at(100, 100));
        let mut expected = Vec::new();
        let mut pos = at(100, 100);
        for (name, stride) in [("海龜", 3), ("海鷗", 2), ("Ferris", 1)] {
            expected.push(::std::format!("{} 帶隊", name));
            for i in 0..3 {
                let o = p[i % p.len()];
                for _ in 0..stride {
                    pos = at(pos.x + o.dx, pos.y + o.dy);
                    expected.push(o.to_string());
                }
            }
        }
        ::std::assert_eq!(cmds, expected, "每位夥伴都要從心跳的第一拍開始，依自己的步幅走");
        ::std::assert_eq!(xy(&end), xy(&pos), "接力結束的位置不對");
    }
}

#[test]
fn route_reaches_the_heart() {
    assert_cleared(&level_grid(), &route());
}
