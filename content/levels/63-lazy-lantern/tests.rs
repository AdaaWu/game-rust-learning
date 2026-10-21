// 測試自己掛一排燈籠：第 i 盞一開始亮不亮由 lit_at 決定
fn row_of(n: usize, lit_at: fn(usize) -> bool) -> Vec<Lantern> {
    let dirs = ["RIGHT", "DOWN", "LEFT", "UP"];
    (0..n).map(|i| Lantern { dir: dirs[i % 4], gem: i % 3 == 0, lit: lit_at(i) }).collect()
}

#[test]
fn light_all_lights_every_lantern() {
    for n in 0..8 {
        let mut lanterns = row_of(n, |_| false);
        light_all(&mut lanterns);
        let dark = lanterns.iter().filter(|l| !l.lit).count();
        ::std::assert_eq!(dark, 0, "{} 盞燈籠點完之後，還有 {} 盞是暗的", n, dark);
    }
}

#[test]
fn lit_lanterns_stay_lit() {
    for n in 1..8 {
        let mut lanterns = row_of(n, |i| i % 2 == 0);
        light_all(&mut lanterns);
        ::std::assert!(lanterns.iter().all(|l| l.lit), "{} 盞燈籠裡有幾盞原本就亮著，點完之後每一盞都該是亮的", n);
    }
}

#[test]
fn lighting_keeps_the_path() {
    let mut lanterns = row_of(7, |_| false);
    light_all(&mut lanterns);
    let expected = row_of(7, |_| true);
    for (i, (got, want)) in lanterns.iter().zip(&expected).enumerate() {
        ::std::assert_eq!((got.dir, got.gem), (want.dir, want.gem), "第 {} 盞燈籠點亮後，指的方向或寶石變了", i + 1);
    }
}

#[test]
fn route_through_the_cave() {
    assert_cleared(&level_grid(), &route());
}
