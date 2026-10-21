#[test]
fn forged_relic_comes_home() {
    ::std::assert_eq!(forge_relic("蟹王"), "蟹王之冠", "帶回地面的寶冠名字不對");
}

#[test]
fn escaped_relic_survives() {
    ::std::assert_eq!(escape("海鷗"), "海鷗的寶冠", "遺跡關門後，寶冠應該被完整帶出來");
}
