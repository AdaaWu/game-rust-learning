#[test]
fn islands_know_each_other() {
    let turtle = new_island("海龜島");
    let coral = new_island("珊瑚島");
    link(&turtle, &coral);
    assert_eq!(neighbor_name(&turtle), Some(String::from("珊瑚島")), "海龜島認不得隔壁的珊瑚島");
    assert_eq!(neighbor_name(&coral), Some(String::from("海龜島")), "珊瑚島認不得隔壁的海龜島");
}

#[test]
fn link_adds_no_owner() {
    let turtle = new_island("海龜島");
    let coral = new_island("珊瑚島");
    link(&turtle, &coral);
    assert_eq!(Rc::strong_count(&turtle), 1, "互相連結後海龜島多了一個擁有者，漩渦就是這樣開始的");
    assert_eq!(Rc::strong_count(&coral), 1, "互相連結後珊瑚島多了一個擁有者，漩渦就是這樣開始的");
}

#[test]
fn whirlpool_disappears() {
    let (turtle_trace, coral_trace) = {
        let turtle = new_island("海龜島");
        let coral = new_island("珊瑚島");
        link(&turtle, &coral);
        (Rc::downgrade(&turtle), Rc::downgrade(&coral))
    };
    assert!(turtle_trace.upgrade().is_none(), "離開作用域後海龜島還浮在海上：記憶體洩漏了");
    assert!(coral_trace.upgrade().is_none(), "離開作用域後珊瑚島還浮在海上：記憶體洩漏了");
}

#[test]
fn sunken_neighbor_is_gone() {
    let turtle = new_island("海龜島");
    let coral = new_island("珊瑚島");
    link(&turtle, &coral);
    drop(coral);
    assert_eq!(neighbor_name(&turtle), None, "珊瑚島已經沒有主人了，卻被海龜島硬拉著不讓它沉");
}
