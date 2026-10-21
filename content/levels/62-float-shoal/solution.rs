// 每往下潛一格，深度增加多少公尺
const DIVE: [f64; 6] = [0.1, 0.1, 0.1, 0.2, 0.2, 0.1];
// 寶石沉在這幾個深度（公尺）
const GEM_DEPTHS: [f64; 3] = [0.3, 0.7, 0.8];

// 深度計的精度（公尺）：差距小於這個值就算到了
const DEPTH_TOLERANCE: f64 = 1e-9;

// 潛到的深度是不是剛好在目標深度
fn reached(depth: f64, target: f64) -> bool {
    (depth - target).abs() < DEPTH_TOLERANCE
}

// 一格一格往下潛，邊潛邊報深度；到了寶石的深度就撿起來
fn dive_route(steps: &[f64]) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut depth = 0.0;
    for step in steps {
        cmds.push("MOVE DOWN".to_string());
        depth += step;
        cmds.push(format!("深度 {} 公尺", depth));
        if GEM_DEPTHS.iter().any(|target| reached(depth, *target)) {
            cmds.push("PICK".to_string());
        }
    }
    cmds
}

// 潛到底之後往右游上岸
fn route() -> Vec<String> {
    let mut cmds = dive_route(&DIVE);
    cmds.push("MOVE RIGHT".to_string());
    cmds.push("MOVE RIGHT".to_string());
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
