fn new_shell() -> Rc<Shell> {
    Rc::new(Shell { name: String::from("粉紅螺殼"), rooms: 3 })
}

#[test]
fn three_crabs_share_one_shell() {
    let shell = new_shell();
    let ferris = move_in("Ferris", &shell);
    let hermit = move_in("阿殼", &shell);
    let fiddler = move_in("小紅", &shell);
    ::std::assert!(Rc::ptr_eq(&ferris.home, &shell), "Ferris 住進了另一間房子，不是同一間貝殼屋");
    ::std::assert!(Rc::ptr_eq(&hermit.home, &shell), "阿殼住進了另一間房子，不是同一間貝殼屋");
    ::std::assert!(Rc::ptr_eq(&fiddler.home, &shell), "小紅住進了另一間房子，不是同一間貝殼屋");
    ::std::assert_eq!(fiddler.name, "小紅", "螃蟹的名字被寫錯了");
}

#[test]
fn every_resident_holds_a_key() {
    let shell = new_shell();
    ::std::assert_eq!(keys(&shell), 1, "還沒有螃蟹搬進來時，只有屋主手上一把鑰匙");
    let _ferris = move_in("Ferris", &shell);
    let _hermit = move_in("阿殼", &shell);
    let _fiddler = move_in("小紅", &shell);
    ::std::assert_eq!(keys(&shell), 4, "三隻螃蟹搬進來後，加上屋主應該有 4 把鑰匙");
}

#[test]
fn move_out_returns_the_key() {
    let shell = new_shell();
    let _ferris = move_in("Ferris", &shell);
    let hermit = move_in("阿殼", &shell);
    let _fiddler = move_in("小紅", &shell);
    ::std::assert_eq!(move_out(hermit), 3, "阿殼搬走後應該剩 3 把鑰匙");
    ::std::assert_eq!(keys(&shell), 3, "阿殼搬走了，鑰匙卻沒有交回來");
}

#[test]
fn last_crab_leaves() {
    let crab = move_in("Ferris", &new_shell());
    ::std::assert_eq!(move_out(crab), 0, "最後一隻螃蟹搬走後，貝殼屋應該沒有鑰匙了");
}
