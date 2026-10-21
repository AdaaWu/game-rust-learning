fn off(dx: i32, dy: i32) -> Offset {
    Offset { dx, dy }
}

// 測試自己產生的節拍：約三分之一是原地踏步
fn sample_beats(seed: u32, len: usize) -> Vec<Offset> {
    let mut s = seed;
    (0..len)
        .map(|_| {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            match (s >> 16) % 6 {
                0 | 1 => off(0, 0),
                2 => off(1, 0),
                3 => off(-1, 0),
                4 => off(0, 1),
                _ => off(0, -1),
            }
        })
        .collect()
}

fn moving(beats: &[Offset]) -> u32 {
    beats.iter().filter(|b| b.dx != 0 || b.dy != 0).count() as u32
}

#[test]
fn patrol_rings_on_each_move() {
    for seed in 0..10 {
        let beats = sample_beats(seed, 12);
        let mut rings = 0;
        let cmds = patrol(&beats, || rings += 1);
        ::std::assert_eq!(cmds.len(), beats.len(), "每一拍都要留下一行紀錄");
        ::std::assert_eq!(rings, moving(&beats), "只有真的移動時才呼叫 on_beat");
    }
}

#[test]
fn count_beats_counts_moves() {
    for seed in 0..30 {
        let beats = sample_beats(seed * 7 + 1, seed as usize % 9 + 1);
        ::std::assert_eq!(
            count_beats(&beats),
            moving(&beats),
            "計步器數錯了：{:?} 實際移動 {} 格",
            beats,
            moving(&beats)
        );
    }
}

#[test]
fn count_beats_skips_standing_still() {
    ::std::assert_eq!(count_beats(&[off(0, 0), off(1, 0), off(0, 0)]), 1, "原地踏步不算步數，這段只移動 1 格");
    ::std::assert_eq!(count_beats(&[off(0, 0); 5]), 0, "一直原地踏步，步數應該是 0");
    ::std::assert_eq!(count_beats(&[]), 0, "沒有節拍，步數應該是 0");
}

#[test]
fn route_follows_the_signs() {
    assert_cleared(&level_grid(), &route());
}
