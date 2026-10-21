// 測試專用夥伴：海龜一次往右下各走一格，用來確認預設方法對新型別也有效
struct Turtle;

impl Mover for Turtle {
    fn name(&self) -> String {
        "海龜".to_string()
    }

    fn step(&self, _dir: &str) -> Vec<String> {
        ::std::vec!["MOVE RIGHT".to_string(), "MOVE DOWN".to_string()]
    }
}

fn moves(dir: &str, n: usize) -> Vec<String> {
    (0..n).map(|_| ::std::format!("MOVE {}", dir)).collect()
}

#[test]
fn seagull_flies_two_cells() {
    ::std::assert_eq!(Seagull.step("DOWN"), moves("DOWN", 2), "海鷗拍一下翅膀應該飛兩格");
}

#[test]
fn ferris_travel_repeats_step() {
    ::std::assert_eq!(Ferris.travel("UP", 3), moves("UP", 3), "Ferris 走 3 次應該是 3 個 MOVE UP");
    ::std::assert!(Ferris.travel("UP", 0).is_empty(), "走 0 次就不該有任何指令");
}

#[test]
fn seagull_travel_uses_its_own_step() {
    ::std::assert_eq!(Seagull.travel("RIGHT", 2), moves("RIGHT", 4), "海鷗飛 2 次應該是 4 格");
}

#[test]
fn default_travel_works_for_new_mover() {
    ::std::assert_eq!(
        Turtle.travel("任意", 2),
        ::std::vec!["MOVE RIGHT", "MOVE DOWN", "MOVE RIGHT", "MOVE DOWN"],
        "新夥伴沒有自己寫 travel，也應該拿到預設的 travel"
    );
    ::std::assert_eq!(announce(&Turtle), "海龜 出發！", "announce 應該接受任何夥伴");
}

#[test]
fn announce_takes_dyn_mover() {
    let partner: &dyn Mover = &Seagull;
    ::std::assert_eq!(announce(partner), "海鷗 出發！", "announce 應該接受 &dyn Mover");
}

#[test]
fn relay_mixed_team_in_one_vec() {
    let team: Vec<Box<dyn Mover>> = ::std::vec![Box::new(Ferris), Box::new(Seagull), Box::new(Turtle)];
    ::std::assert_eq!(
        relay(&team, "UP", 1),
        ::std::vec!["Ferris 出發！", "MOVE UP", "海鷗 出發！", "MOVE UP", "MOVE UP", "海龜 出發！", "MOVE RIGHT", "MOVE DOWN"],
        "隊伍裡每位夥伴要依序報名，再用自己的方式移動"
    );
}

#[test]
fn relay_repeats_travel_per_member() {
    let team: Vec<Box<dyn Mover>> = ::std::vec![Box::new(Seagull), Box::new(Ferris)];
    let mut expected = ::std::vec!["海鷗 出發！".to_string()];
    expected.extend(moves("LEFT", 4));
    expected.push("Ferris 出發！".to_string());
    expected.extend(moves("LEFT", 2));
    ::std::assert_eq!(relay(&team, "LEFT", 2), expected, "每位夥伴都要往同一方向移動 n 次");
    let nobody: Vec<Box<dyn Mover>> = Vec::new();
    ::std::assert!(relay(&nobody, "LEFT", 2).is_empty(), "隊伍沒有人時不該有任何指令");
}

#[test]
fn route_reaches_workshop() {
    assert_cleared(&level_grid(), &route());
}
