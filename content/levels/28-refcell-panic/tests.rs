fn chest_with(items: &[&str]) -> Rc<RefCell<Chest>> {
    let items = items.iter().map(|s| s.to_string()).collect();
    Rc::new(RefCell::new(Chest { items }))
}

#[test]
fn deposit_new_item() {
    let chest = chest_with(&["珍珠"]);
    ::std::assert_eq!(deposit(&chest, "海星"), 2);
    ::std::assert_eq!(chest.borrow().items, ::std::vec![String::from("珍珠"), String::from("海星")]);
}

#[test]
fn duplicate_is_skipped() {
    let chest = chest_with(&["珍珠"]);
    ::std::assert_eq!(deposit(&chest, "珍珠"), 1, "重複的寶物不應該放入");
}

#[test]
fn gull_shares_the_chest() {
    let chest = chest_with(&[]);
    let gull = Rc::clone(&chest);
    deposit(&gull, "海星");
    deposit(&chest, "貝殼");
    ::std::assert_eq!(Rc::strong_count(&chest), 2);
    ::std::assert_eq!(gull.borrow().items.len(), 2, "海鷗看到的箱子內容不對");
}
