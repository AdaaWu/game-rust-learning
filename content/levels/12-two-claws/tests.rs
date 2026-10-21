#[test]
fn polished_then_engraved() {
    ::std::assert_eq!(
        two_claws("珍珠", "Ferris"),
        "珍珠✨（Ferris）",
        "成品不對：應該先擦亮，再刻上名字"
    );
}

#[test]
fn works_for_any_gem() {
    ::std::assert_eq!(two_claws("藍寶石", "海鷗"), "藍寶石✨（海鷗）", "換一顆寶石，兩隻鉗子的加工結果不對");
}
