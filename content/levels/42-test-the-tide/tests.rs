type TideFn = fn((i32, i32), (i32, i32)) -> Vec<String>;

fn repeat_cmd(cmds: &mut Vec<String>, word: &str, n: i32) {
    for _ in 0..n {
        cmds.push(word.to_string());
    }
}

fn horizontal_word(dx: i32) -> &'static str {
    if dx > 0 { "MOVE RIGHT" } else { "MOVE LEFT" }
}

fn vertical_word(dy: i32) -> &'static str {
    if dy > 0 { "MOVE DOWN" } else { "MOVE UP" }
}

// 正確的潮汐表
fn tide_reference(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, horizontal_word(dx), dx.abs());
    repeat_cmd(&mut cmds, vertical_word(dy), dy.abs());
    cmds
}

// 正確結果，但用不同寫法實作
fn tide_reference_iter(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    std::iter::repeat_n(horizontal_word(dx).to_string(), dx.unsigned_abs() as usize)
        .chain(std::iter::repeat_n(vertical_word(dy).to_string(), dy.unsigned_abs() as usize))
        .collect()
}

fn mutant_left_becomes_right(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, "MOVE RIGHT", dx.abs());
    repeat_cmd(&mut cmds, vertical_word(dy), dy.abs());
    cmds
}

fn mutant_up_becomes_down(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, horizontal_word(dx), dx.abs());
    repeat_cmd(&mut cmds, "MOVE DOWN", dy.abs());
    cmds
}

fn mutant_ignores_y(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let dx = to.0 - from.0;
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, horizontal_word(dx), dx.abs());
    cmds
}

fn mutant_one_step_too_far(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let extra = if dx != 0 { 1 } else { 0 };
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, horizontal_word(dx), dx.abs() + extra);
    repeat_cmd(&mut cmds, vertical_word(dy), dy.abs());
    cmds
}

fn mutant_moves_when_already_there(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    if from == to {
        return ::std::vec!["MOVE UP".to_string()];
    }
    tide_reference(from, to)
}

fn mutant_vertical_first(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, vertical_word(dy), dy.abs());
    repeat_cmd(&mut cmds, horizontal_word(dx), dx.abs());
    cmds
}

fn mutant_skips_negative(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut cmds = Vec::new();
    repeat_cmd(&mut cmds, horizontal_word(dx), dx);
    repeat_cmd(&mut cmds, vertical_word(dy), dy);
    cmds
}

// 執行玩家的 check_steps，暫時關掉 panic 訊息，回傳是否 panic 與訊息
fn run_check(tide: TideFn) -> Result<(), String> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(move || check_steps(tide));
    std::panic::set_hook(previous);
    outcome.map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

fn assert_caught(name: &str, mutant: TideFn) {
    ::std::assert!(
        run_check(mutant).is_err(),
        "你的 check_steps 沒抓到：{}（把這個錯誤版本傳進去時沒有 panic）",
        name
    );
}

#[test]
fn check_accepts_correct_tide() {
    for tide in [tide_reference as TideFn, tide_reference_iter as TideFn] {
        if let Err(msg) = run_check(tide) {
            ::std::panic!("你的 check_steps 把正確的潮汐表也判成錯：{}", msg);
        }
    }
}

#[test]
fn check_catches_left_becomes_right() {
    assert_caught("往左走成往右", mutant_left_becomes_right);
}

#[test]
fn check_catches_up_becomes_down() {
    assert_caught("往上走成往下", mutant_up_becomes_down);
}

#[test]
fn check_catches_ignored_y() {
    assert_caught("完全忽略上下方向", mutant_ignores_y);
}

#[test]
fn check_catches_one_step_too_far() {
    assert_caught("水平方向多走一步", mutant_one_step_too_far);
}

#[test]
fn check_catches_moving_in_place() {
    assert_caught("已經在終點還多走一步", mutant_moves_when_already_there);
}

#[test]
fn check_catches_vertical_first() {
    assert_caught("先垂直後水平", mutant_vertical_first);
}

#[test]
fn check_catches_negative_skipped() {
    assert_caught("往左與往上時原地不動", mutant_skips_negative);
}

#[test]
fn steps_to_matches_tide_table() {
    for fx in -3..=3 {
        for fy in -3..=3 {
            for tx in -3..=3 {
                for ty in -3..=3 {
                    let (from, to) = ((fx, fy), (tx, ty));
                    ::std::assert_eq!(
                        steps_to(from, to),
                        tide_reference(from, to),
                        "從 {:?} 走到 {:?} 的指令不對",
                        from,
                        to
                    );
                }
            }
        }
    }
}

#[test]
fn route_reaches_tower_top() {
    assert_cleared(&level_grid(), &route());
}
