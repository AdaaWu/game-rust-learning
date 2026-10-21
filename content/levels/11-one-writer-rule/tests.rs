#[test]
fn remembers_first_and_adds_ruby() {
    let mut bag = vec![String::from("珍珠")];
    let first = first_then_add(&mut bag);
    assert_eq!(first, "珍珠");
    assert_eq!(bag, vec![String::from("珍珠"), String::from("紅寶石")]);
}

#[test]
fn works_with_longer_bag() {
    let mut bag = vec![String::from("海星"), String::from("貝殼")];
    assert_eq!(first_then_add(&mut bag), "海星");
    assert_eq!(bag.len(), 3, "紅寶石沒有放進背包");
}
