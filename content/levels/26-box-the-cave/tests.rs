#[test]
fn three_layer_cave() {
    let cave = dig(&[3, 1, 4]);
    assert_eq!(depth(&cave), 3, "洞窟應該有三層");
    assert_eq!(total_gems(&cave), 8, "三層的寶石加起來應該是 8 顆");
}

#[test]
fn dead_end_only() {
    let cave = dig(&[]);
    assert_eq!(depth(&cave), 0, "一開始就是死路，深度應該是 0");
    assert_eq!(total_gems(&cave), 0, "死路裡沒有寶石");
}

#[test]
fn hundred_layer_cave() {
    let cave = dig(&[1; 100]);
    assert_eq!(depth(&cave), 100, "一百層的洞窟算錯層數了");
    assert_eq!(total_gems(&cave), 100, "一百層、每層一顆，總數應該是 100");
}
