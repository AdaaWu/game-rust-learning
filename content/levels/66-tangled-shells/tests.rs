// 測試自己擺一圈貝殼：中間的貝殼有 pearls 顆珍珠，旁邊 n 個鄰居都是空的
fn ring(pearls: u32, n: usize) -> (Rc<Shell>, Vec<Rc<Shell>>) {
    let center = new_shell("中心殼", pearls);
    let around: Vec<Rc<Shell>> = (0..n).map(|i| new_shell(&::std::format!("鄰居殼 {}", i + 1), 0)).collect();
    for shell in &around {
        link(&center, shell);
    }
    (center, around)
}

#[test]
fn share_gives_each_neighbor_one() {
    for p in 0..5u32 {
        for n in 0..5usize {
            let (center, around) = ring(p, n);
            share(&center);
            let given = (p as usize).min(n);
            ::std::assert_eq!(pearls(&center) as usize, p as usize - given, "{} 顆珍珠分給 {} 個鄰居，中心殼剩下的數量不對", p, n);
            for (i, shell) in around.iter().enumerate() {
                let expected = if i < given { 1 } else { 0 };
                ::std::assert_eq!(pearls(shell), expected, "{} 顆珍珠分給 {} 個鄰居，第 {} 個鄰居應該有 {} 顆", p, n, i + 1, expected);
            }
        }
    }
}

#[test]
fn neighbor_is_the_same_shell() {
    let (center, around) = ring(2, 1);
    share(&center);
    let seen = first_neighbor(&center).expect("中心殼應該認得牠的鄰居");
    ::std::assert!(Rc::ptr_eq(&seen, &around[0]), "中心殼記得的鄰居，和真正的鄰居不是同一個貝殼");
    ::std::assert_eq!(pearls(&around[0]), 1, "分出去的珍珠沒有落在真正的鄰居身上");
    let back = first_neighbor(&around[0]).expect("鄰居也應該認得中心殼");
    ::std::assert!(Rc::ptr_eq(&back, &center), "鄰居記得的中心殼，和真正的中心殼不是同一個貝殼");
}

#[test]
fn link_adds_no_owner() {
    let (center, around) = ring(1, 2);
    share(&center);
    ::std::assert_eq!(Rc::strong_count(&center), 1, "成為鄰居之後，中心殼多了擁有者：互相擁有就是糾纏的開始");
    for shell in &around {
        ::std::assert_eq!(Rc::strong_count(shell), 1, "成為鄰居之後，{} 多了擁有者", shell.name);
        ::std::assert_eq!(Rc::weak_count(shell), 1, "{} 只有一個鄰居認得牠", shell.name);
    }
    ::std::assert_eq!(Rc::weak_count(&center), 2, "中心殼有兩個鄰居認得牠");
}

#[test]
fn shells_return_to_the_sea() {
    for p in 1..4u32 {
        let traces: Vec<std::rc::Weak<Shell>> = {
            let (center, around) = ring(p, 3);
            share(&center);
            for shell in &around {
                share(shell);
            }
            std::iter::once(&center).chain(around.iter()).map(Rc::downgrade).collect()
        };
        for trace in traces {
            ::std::assert!(trace.upgrade().is_none(), "離開作用域後，貝殼應該已經回到海底，卻還糾纏在一起（記憶體洩漏）");
        }
    }
}

#[test]
fn gone_neighbor_is_none() {
    let (center, mut around) = ring(1, 1);
    share(&center);
    drop(around.pop());
    ::std::assert!(first_neighbor(&center).is_none(), "鄰居已經沒有主人了，卻被中心殼硬拉著不讓牠回到海底");
}
