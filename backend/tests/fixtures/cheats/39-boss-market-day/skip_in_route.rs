// 取巧：清除售完攤位的迴圈不修，改在 route 走訪時跳過售完的攤位
use std::collections::HashMap;
use std::str::FromStr;

// 今天市集的交易紀錄：攤位、進貨或賣出、件數
const MARKET_LOG: &str = "
1號攤 進 3
3號攤 進 2
2號攤 進 1
4號攤 進 4
3號攤 出 2
5號攤 進 2
4號攤 出 4
6號攤 進 5
1號攤 出 1
5號攤 出 1
";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Flow {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq)]
struct Trade {
    stall: String,
    flow: Flow,
    qty: u32,
}

#[derive(Debug, PartialEq)]
enum TradeError {
    BadLine(String),
}

// 一行紀錄 → Trade，例如 "3號攤 出 2"
impl FromStr for Trade {
    type Err = TradeError;

    fn from_str(line: &str) -> Result<Trade, TradeError> {
        let bad = || TradeError::BadLine(line.to_string());
        let parts: Vec<&str> = line.split_whitespace().collect();
        let [stall, flow, qty] = parts.as_slice() else {
            return Err(bad());
        };
        let flow = match *flow {
            "進" => Flow::In,
            "出" => Flow::Out,
            _ => return Err(bad()),
        };
        let qty = qty.parse::<u32>().map_err(|_| bad())?;
        Ok(Trade { stall: stall.to_string(), flow, qty })
    }
}

// 整份紀錄 → Vec<Trade>，空白行略過
fn parse_trades(log: &str) -> Result<Vec<Trade>, TradeError> {
    let mut trades = Vec::new();
    for line in log.lines() {
        if line.trim().is_empty() {
            continue;
        }
        trades.push(line.parse::<Trade>()?);
    }
    Ok(trades)
}

// 每個攤位收市時還剩幾件
fn stock(trades: &[Trade]) -> HashMap<String, u32> {
    let mut left = HashMap::new();
    for t in trades {
        let e = left.entry(t.stall.clone()).or_insert(0);
        match t.flow {
            Flow::In => *e += t.qty,
            Flow::Out => *e -= t.qty,
        }
    }
    left
}

// 要拜訪的攤位：還有貨的才去，照攤位名稱排序（1號攤、2號攤……）
fn visit_order(stock: &HashMap<String, u32>) -> Vec<String> {
    let mut order: Vec<String> = stock.keys().cloned().collect();
    order.sort();
    // 清掉售完的攤位
    let mut i = 0;
    while i < order.len() {
        if stock[&order[i]] == 0 {
            order.remove(i);
        }
        i += 1;
    }
    order
}

// 每個攤位在地圖上的位置
fn stall_spot(stall: &str) -> (i32, i32) {
    match stall {
        "1號攤" => (2, 0),
        "2號攤" => (5, 0),
        "3號攤" => (5, 2),
        "4號攤" => (3, 2),
        "5號攤" => (2, 4),
        _ => (6, 4),
    }
}

// 從 from 走到 to：先左右、再上下
fn walk(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let mut cmds = Vec::new();
    let x_dir = if to.0 > from.0 { "RIGHT" } else { "LEFT" };
    for _ in 0..(to.0 - from.0).abs() {
        cmds.push(format!("MOVE {}", x_dir));
    }
    let y_dir = if to.1 > from.1 { "DOWN" } else { "UP" };
    for _ in 0..(to.1 - from.1).abs() {
        cmds.push(format!("MOVE {}", y_dir));
    }
    cmds
}

fn route() -> Vec<String> {
    let trades = parse_trades(MARKET_LOG).unwrap();
    let left = stock(&trades);
    let mut cmds = Vec::new();
    let mut here = (0, 0);
    for stall in visit_order(&left) {
        if left[&stall] == 0 {
            continue;
        }
        let spot = stall_spot(&stall);
        cmds.extend(walk(here, spot));
        cmds.push("PICK".to_string());
        here = spot;
    }
    cmds.extend(walk(here, (7, 4)));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
