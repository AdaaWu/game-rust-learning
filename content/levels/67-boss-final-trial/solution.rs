// 偵察隊：出海偵察，把回報寫上指揮部的回報板
mod scout {
    use std::sync::mpsc::Sender;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    // 一隊的任務：隊號，以及偵察回報（寶石的海圖座標，格式 "x,y;x,y"）
    pub struct Mission {
        pub team: usize,
        pub report: String,
    }

    // 最終試煉派出的三隊
    pub fn missions() -> Vec<Mission> {
        vec![
            Mission { team: 1, report: String::from("2,0;2,2") },
            Mission { team: 2, report: String::from("5,1") },
            Mission { team: 3, report: String::from("5,4;-1,3") },
        ]
    }

    // 出海偵察一會兒，把回報寫上回報板，再通知指揮部「這隊回來了」
    pub fn explore(mission: Mission, board: Arc<Mutex<Vec<(usize, String)>>>, done: Sender<usize>) {
        thread::sleep(Duration::from_millis(10));
        board.lock().unwrap().push((mission.team, mission.report));
        done.send(mission.team).unwrap();
    }
}

// 指揮部：等各隊回來，整理回報，排出最終路線
mod command {
    use super::scout::{self, Mission};
    use std::collections::BTreeMap;
    use std::num::ParseIntError;
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;

    // 海圖大小
    pub const WIDTH: i64 = 8;
    pub const HEIGHT: i64 = 5;
    // Ferris 的起點與終點
    pub const START: (usize, usize) = (0, 0);
    pub const GOAL: (usize, usize) = (7, 4);

    #[derive(Debug, PartialEq)]
    pub enum TrialError {
        // 第 team 隊的回報看不懂
        BadReport { team: usize },
    }

    // 把 "x,y;x,y" 解析成海圖座標
    pub fn parse_report(report: &str) -> Result<Vec<(i64, i64)>, ParseIntError> {
        let mut points = Vec::new();
        for pair in report.split(';') {
            let (x, y) = pair.split_once(',').unwrap_or((pair, ""));
            points.push((x.trim().parse()?, y.trim().parse()?));
        }
        Ok(points)
    }

    // 海圖座標換成地圖格子；在海圖外就回傳 None
    pub fn to_cell(x: i64, y: i64) -> Option<(usize, usize)> {
        let cell_x = usize::try_from(x).ok()?;
        let cell_y = usize::try_from(y).ok()?;
        if x < WIDTH && y < HEIGHT {
            Some((cell_x, cell_y))
        } else {
            None
        }
    }

    // 檢查 to_cell 寫得對不對，抓到錯就 panic
    pub fn check_to_cell(to_cell: fn(i64, i64) -> Option<(usize, usize)>) {
        assert_eq!(to_cell(0, 0), Some((0, 0)), "左上角在海圖內");
        assert_eq!(to_cell(2, 1), Some((2, 1)), "(2, 1) 在海圖內");
        assert_eq!(to_cell(WIDTH - 1, 0), Some((7, 0)), "最右一欄在海圖內");
        assert_eq!(to_cell(0, HEIGHT - 1), Some((0, 4)), "最下一列在海圖內");
        assert_eq!(to_cell(WIDTH - 1, HEIGHT - 1), Some((7, 4)), "右下角在海圖內");
        assert_eq!(to_cell(-1, 0), None, "x 是負的就在海圖外");
        assert_eq!(to_cell(0, -1), None, "y 是負的就在海圖外");
        assert_eq!(to_cell(WIDTH, 0), None, "x 等於寬度就超出海圖");
        assert_eq!(to_cell(0, HEIGHT), None, "y 等於高度就超出海圖");
        assert_eq!(to_cell(i64::MIN, i64::MIN), None, "離海圖最遠的負座標也在海圖外");
    }

    // 派出各隊，等全部回來後整理回報：依隊號排出寶石的格子，海圖外的座標略過
    pub fn gather(missions: Vec<Mission>) -> Result<Vec<(usize, usize)>, TrialError> {
        let board = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = mpsc::channel();
        for mission in missions {
            let board = Arc::clone(&board);
            let tx = tx.clone();
            thread::spawn(move || scout::explore(mission, board, tx));
        }
        drop(tx);

        println!("指揮部等各隊回來");
        for team in rx {
            println!("第 {} 隊回來了", team);
        }
        let notes = board.lock().unwrap();

        let mut by_team = BTreeMap::new();
        for (team, report) in notes.iter() {
            let points = parse_report(report).map_err(|_| TrialError::BadReport { team: *team })?;
            by_team.insert(*team, points);
        }

        let mut cells = Vec::new();
        for (team, points) in by_team {
            for (x, y) in points {
                match to_cell(x, y) {
                    Some(cell) => cells.push(cell),
                    None => println!("第 {} 隊回報的 ({}, {}) 在海圖外，略過", team, x, y),
                }
            }
        }
        Ok(cells)
    }

    // 從 from 走到 to：先水平、再垂直
    fn path(from: (usize, usize), to: (usize, usize), cmds: &mut Vec<String>) {
        let dx = to.0 as i64 - from.0 as i64;
        let dy = to.1 as i64 - from.1 as i64;
        let horizontal = if dx > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
        for _ in 0..dx.abs() {
            cmds.push(horizontal.to_string());
        }
        let vertical = if dy > 0 { "MOVE DOWN" } else { "MOVE UP" };
        for _ in 0..dy.abs() {
            cmds.push(vertical.to_string());
        }
    }

    // 從起點依序撿起每顆寶石，最後走到終點
    pub fn route(cells: &[(usize, usize)]) -> Vec<String> {
        let mut cmds = Vec::new();
        let mut at = START;
        for &cell in cells {
            path(at, cell, &mut cmds);
            cmds.push("PICK".to_string());
            at = cell;
        }
        path(at, GOAL, &mut cmds);
        cmds
    }
}

fn main() {
    match command::gather(scout::missions()) {
        Ok(cells) => {
            for c in command::route(&cells) {
                println!("{}", c);
            }
        }
        Err(e) => println!("試煉中斷：{:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::command;

    #[test]
    fn chart_cells_are_checked() {
        command::check_to_cell(command::to_cell);
    }
}
