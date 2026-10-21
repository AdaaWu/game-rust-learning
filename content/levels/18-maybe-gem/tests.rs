#[test]
fn describe_gem_chest() {
    assert_eq!(describe(Some("珍珠")), "寶箱裡有珍珠", "有寶石的箱子要說出寶石名字");
}

#[test]
fn describe_empty_chest() {
    assert_eq!(describe(None), "空箱子", "空箱子要老實說是空的");
}

#[test]
fn empty_chests_no_pick() {
    let cmds = route(&[None, None, None]);
    assert!(!cmds.iter().any(|c| c == "PICK"), "全是空箱時 Ferris 不該伸手撿");
    assert_eq!(cmds.len(), 4, "三個空箱應該只有三步往右加一步往下");
}

#[test]
fn route_clears_island() {
    assert_cleared(&level_grid(), &route(&CHESTS));
}
