// 唸一道咒語之後，測試再接著記一步：巨集不該偷偷離開這個函式
fn chant_then_walk(spell: &str) -> Chant {
    let mut chant = Chant { cmds: Vec::new(), skipped: Vec::new() };
    step_or_bail!(chant, spell);
    chant.cmds.push("MOVE UP".to_string());
    chant
}

#[test]
fn macro_does_not_leave_the_caller() {
    let after_bad = chant_then_walk("FLY UP");
    ::std::assert_eq!(
        after_bad.cmds,
        ::std::vec!["MOVE UP"],
        "壞咒語之後的程式沒有執行：巨集裡的 return 讓呼叫它的函式提早結束了"
    );
    ::std::assert_eq!(after_bad.skipped, ::std::vec!["FLY UP"], "唸不出來的 FLY UP 要記進 skipped");
    let after_good = chant_then_walk("PICK");
    ::std::assert_eq!(after_good.cmds, ::std::vec!["PICK", "MOVE UP"], "好咒語唸完也要接著往下做");
}

#[test]
fn bad_spells_anywhere() {
    let good = ["MOVE LEFT", "PICK", "MOVE DOWN", "MOVE RIGHT", "MOVE UP"];
    let bad = ["SWIM LEFT", "MOVE NORTH", "pick", "JUMP", "MOVE"];
    for len in 1..7 {
        for mask in 0..(1usize << len) {
            let mut spells = Vec::new();
            let mut want_cmds = Vec::new();
            let mut want_skipped = Vec::new();
            for i in 0..len {
                if mask & (1 << i) != 0 {
                    let spell = bad[(i + len) % bad.len()];
                    spells.push(spell);
                    want_skipped.push(spell.to_string());
                } else {
                    let spell = good[(i + mask) % good.len()];
                    spells.push(spell);
                    want_cmds.push(spell.to_string());
                }
            }
            let result = chant(&spells);
            ::std::assert_eq!(result.cmds, want_cmds, "咒語 {:?}：壞咒語以外的指令要全部照順序唸完", spells);
            ::std::assert_eq!(result.skipped, want_skipped, "咒語 {:?}：每一道壞咒語都要記進 skipped", spells);
        }
    }
}

#[test]
fn empty_scroll() {
    let result = chant(&[]);
    ::std::assert!(result.cmds.is_empty() && result.skipped.is_empty(), "空白捲軸什麼都不用唸");
}

#[test]
fn scroll_route_reaches_exit() {
    assert_cleared(&level_grid(), &route());
}
