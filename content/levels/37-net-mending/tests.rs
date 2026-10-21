// 測試用的簡易亂數（線性同餘）
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

// 依旗標建一張網，魚名帶編號方便看出順序
fn net_from(flags: &[bool]) -> Vec<Fish> {
    flags
        .iter()
        .enumerate()
        .map(|(i, r)| Fish { name: ::std::format!("魚{}", i), rotten: *r })
        .collect()
}

fn check(flags: &[bool]) {
    let mut net = net_from(flags);
    let want: Vec<String> = net.iter().filter(|f| !f.rotten).map(|f| f.name.clone()).collect();
    mend(&mut net);
    let left: Vec<&str> = net.iter().map(|f| f.name.as_str()).collect();
    ::std::assert!(
        !net.iter().any(|f| f.rotten),
        "網子 {:?}（true 是爛魚）補完還留著爛魚：{:?}",
        flags,
        left
    );
    ::std::assert_eq!(left, want, "網子 {:?}（true 是爛魚）補完後，好魚要全部留下且順序不變", flags);
}

#[test]
fn mends_the_sample_net() {
    check(&[true, true, false, true, false]);
}

#[test]
fn four_rotten_in_a_row() {
    check(&[false, true, true, true, true, false]);
}

#[test]
fn all_rotten_or_empty() {
    check(&[true, true, true]);
    check(&[]);
}

#[test]
fn last_one_rotten_or_none_rotten() {
    check(&[false, false, true]);
    check(&[false, false, false, false]);
}

#[test]
fn random_nets() {
    for round in 0..30u64 {
        let mut seed = round * 23 + 19;
        let len = (next_rand(&mut seed) % 12) as usize;
        let flags: Vec<bool> = (0..len).map(|_| next_rand(&mut seed) % 2 == 0).collect();
        check(&flags);
    }
}
