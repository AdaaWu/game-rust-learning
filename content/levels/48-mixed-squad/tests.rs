// 測試專用夥伴：只實作必要的方法，steps_with 用預設的
struct TestCrab {
    path: Vec<String>,
}

impl Walker for TestCrab {
    fn name(&self) -> String {
        ::std::format!("測試蟹{}", self.path.len())
    }

    fn steps(&self) -> Vec<String> {
        self.path.clone()
    }
}

fn crab(dirs: &[&str]) -> TestCrab {
    TestCrab {
        path: dirs.iter().map(|d| ::std::format!("MOVE {}", d)).collect(),
    }
}

#[test]
fn relay_takes_any_walker() {
    let dirs = ["UP", "LEFT", "DOWN", "RIGHT"];
    for len in 0..4 {
        let a = crab(&dirs[..len]);
        let b = crab(&dirs[len..]);
        let mut expected = ::std::vec![::std::format!("{} 接棒", a.name())];
        expected.extend(a.steps());
        expected.push(::std::format!("{} 接棒", b.name()));
        expected.extend(b.steps());
        expected.push("海鷗 接棒".to_string());
        expected.extend(::std::vec!["MOVE UP".to_string(); 2]);
        let squad: Vec<Box<dyn Walker>> = ::std::vec![Box::new(a), Box::new(b), Box::new(Seagull { dir: "UP" })];
        ::std::assert_eq!(relay(&squad), expected, "不同型別的夥伴放進同一支隊伍，每位都要依序喊接棒再走自己那段");
    }
    let nobody: Vec<Box<dyn Walker>> = Vec::new();
    ::std::assert!(relay(&nobody).is_empty(), "隊伍沒有人時不該有任何指令");
}

#[test]
fn steps_with_rewrites_each_step() {
    let c = crab(&["UP", "RIGHT", "RIGHT"]);
    ::std::assert_eq!(
        c.steps_with(&|s: &str| s.to_lowercase()),
        ::std::vec!["move up", "move right", "move right"],
        "steps_with 要把每一步交給 f 改寫"
    );
    ::std::assert_eq!(
        Seagull { dir: "LEFT" }.steps_with(&|s: &str| ::std::format!("{}!", s)),
        ::std::vec!["MOVE LEFT!", "MOVE LEFT!"],
        "海鷗的兩步都要交給 f 改寫"
    );
    ::std::assert_eq!(
        Hermit { dir: "UP" }.steps_with(&flip),
        ::std::vec!["MOVE DOWN", "PICK"],
        "寄居蟹反過來走時，PICK 不受影響"
    );
}

#[test]
fn lead_goes_and_comes_back() {
    let c = crab(&["RIGHT", "DOWN", "DOWN"]);
    ::std::assert_eq!(
        lead(&c),
        ::std::vec!["MOVE RIGHT", "MOVE DOWN", "MOVE DOWN", "PICK", "MOVE UP", "MOVE UP", "MOVE LEFT"],
        "隊長探路要走過去、撿寶石、再照相反順序退回原點"
    );
    ::std::assert_eq!(
        lead(&Seagull { dir: "LEFT" }),
        ::std::vec!["MOVE LEFT", "MOVE LEFT", "PICK", "MOVE RIGHT", "MOVE RIGHT"],
        "海鷗探路飛兩格過去、再飛兩格回來"
    );
}

#[test]
fn route_crosses_the_maze() {
    assert_cleared(&level_grid(), &route());
}
