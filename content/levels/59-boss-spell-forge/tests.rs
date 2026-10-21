// 測試自己算的答案：熔出來的字句與帳本
fn expected(scroll: &[Spell]) -> (Vec<String>, HashMap<&'static str, u32>) {
    let mut words = Vec::new();
    let mut tally = HashMap::new();
    for spell in scroll {
        match spell {
            Spell::Go(dir) => words.push(::std::format!("MOVE {}", dir)),
            Spell::Gather(kind) => {
                words.push("PICK".to_string());
                *tally.entry(*kind).or_insert(0) += 1;
            }
            Spell::Rest => {}
        }
    }
    (words, tally)
}

// 依序檢查熔爐：先看有沒有熔完、再看帳本、最後看字句
fn check_forge(scroll: &[Spell]) {
    let (want_words, want_tally) = expected(scroll);
    let mut tally = HashMap::new();
    let words = smelt(scroll, &mut tally);
    ::std::assert_eq!(
        words.len(),
        want_words.len(),
        "熔爐只熔出 {} 道就收工了，整卷應該熔出 {} 道：休息符之後的咒語也要熔",
        words.len(),
        want_words.len()
    );
    for kind in ["珍珠", "海玻璃", "珊瑚"] {
        let want = want_tally.get(kind).copied().unwrap_or(0);
        let got = tally.get(kind).copied().unwrap_or(0);
        ::std::assert_eq!(got, want, "{}應為 {} 顆，實際 {} 顆", kind, want, got);
    }
    ::std::assert_eq!(
        commands(words.clone()),
        want_words,
        "交給 Ferris 的指令對不上，Ferris 會經過寶石卻沒撿：用 {{:?}} 印出熔出來的字句看看"
    );
    ::std::assert_eq!(words, want_words, "熔出來的字句要和指令一模一樣，Display 不能多任何字元");
    check_display();
}

// 直接檢查 Display 本身，不經過 forge!
fn check_display() {
    let cases = [
        (Spell::Go("UP"), "MOVE UP"),
        (Spell::Go("RIGHT"), "MOVE RIGHT"),
        (Spell::Gather("珍珠"), "PICK"),
        (Spell::Gather("海玻璃"), "PICK"),
        (Spell::Rest, "（休息）"),
    ];
    for (spell, want) in cases {
        let got = spell.to_string();
        ::std::assert_eq!(got, want, "{:?} 的 to_string() 應為 {:?}，實際 {:?}：修 bug 要修在 Display 本身", spell, want, got);
    }
}

// 測試自己編的咒語卷：依 seed 排出走路、撿寶石與休息符，最後一段固定在休息符之後
fn made_up_scroll(seed: usize, len: usize) -> Vec<Spell> {
    let pool = [
        Spell::Go("UP"),
        Spell::Gather("珍珠"),
        Spell::Rest,
        Spell::Go("LEFT"),
        Spell::Gather("珊瑚"),
        Spell::Go("DOWN"),
        Spell::Rest,
        Spell::Gather("海玻璃"),
    ];
    let mut scroll: Vec<Spell> = (0..len).map(|i| pool[(i * (seed + 1) + seed * 3) % pool.len()]).collect();
    scroll.extend([Spell::Rest, Spell::Gather("珍珠"), Spell::Go("RIGHT")]);
    scroll
}

#[test]
fn forge_made_up_scrolls() {
    for seed in 0..6 {
        for len in 0..12 {
            check_forge(&made_up_scroll(seed, len));
        }
    }
}

#[test]
fn rest_at_edges() {
    use Spell::*;
    check_forge(&[Rest, Go("RIGHT"), Gather("珍珠")]);
    check_forge(&[Gather("珍珠"), Rest, Rest, Gather("珍珠"), Go("UP")]);
    check_forge(&[Go("DOWN"), Gather("海玻璃"), Rest]);
    check_forge(&[Rest]);
}

// 用帶副作用的運算式呼叫 forge!，數一數它被執行幾次
fn smelt_counting(scroll: &[Spell], calls: &mut usize) -> Vec<String> {
    let mut words = Vec::new();
    for &spell in scroll {
        forge!(words, {
            *calls += 1;
            spell
        });
    }
    words
}

#[test]
fn each_spell_smelted_once() {
    use Spell::*;
    let scroll = [Go("LEFT"), Rest, Gather("珍珠"), Go("UP"), Rest, Gather("珊瑚")];
    let mut calls = 0;
    let words = smelt_counting(&scroll, &mut calls);
    ::std::assert_eq!(words.len(), 4, "熔爐只熔出 {} 道就收工了，整卷應該熔出 4 道：休息符之後的咒語也要熔", words.len());
    ::std::assert_eq!(calls, scroll.len(), "{} 道咒語卻被熔了 {} 次：forge! 的 $e 只能求值一次", scroll.len(), calls);
}

#[test]
fn forge_route_clears_workshop() {
    check_forge(&great_scroll());
    assert_cleared(&level_grid(), &route());
}
