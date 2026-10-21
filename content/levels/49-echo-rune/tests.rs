// 會格式化 Rune 的測試名稱以 zz_ 開頭：libtest 依名稱字母序執行，Display 還在遞迴時 stack overflow 會讓整個測試程式中止

fn all_runes() -> Vec<(Rune, &'static str)> {
    ::std::vec![
        (Rune::Up, "MOVE UP"),
        (Rune::Down, "MOVE DOWN"),
        (Rune::Left, "MOVE LEFT"),
        (Rune::Right, "MOVE RIGHT"),
        (Rune::Pick, "PICK"),
    ]
}

#[test]
fn count_gems_in_spell() {
    let picks = spell().iter().filter(|r| ::std::matches!(r, Rune::Pick)).count();
    ::std::assert_eq!(picks, 3, "地板上的符文應該剛好有 3 個 PICK");
}

#[test]
fn zz_route_reads_the_spell() {
    let cmds = route();
    ::std::assert_eq!(cmds.len(), spell().len(), "每個符文都要唸成一行指令");
    assert_cleared(&level_grid(), &cmds);
}

#[test]
fn zz_rune_display_each() {
    for (rune, word) in all_runes() {
        ::std::assert_eq!(::std::format!("{}", rune), word, "符文 {:?} 應該唸成 {}", rune, word);
        ::std::assert_eq!(rune.to_string(), word, "to_string 會走 Display，結果要一樣");
    }
}

#[test]
fn zz_rune_display_inside_sentence() {
    for (rune, word) in all_runes() {
        ::std::assert_eq!(
            ::std::format!("Ferris 唸出「{}」！", rune),
            ::std::format!("Ferris 唸出「{}」！", word),
            "符文放在句子裡也要只唸出自己刻的字"
        );
    }
}
