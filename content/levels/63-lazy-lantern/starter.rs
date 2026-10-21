// 洞窟裡的一盞燈籠：照亮往某個方向的下一格，那一格可能有寶石
struct Lantern {
    dir: &'static str,
    gem: bool,
    lit: bool,
}

// 掛一盞還沒點亮的燈籠
fn lantern(dir: &'static str, gem: bool) -> Lantern {
    Lantern { dir, gem, lit: false }
}

// 洞窟的通道：照順序每一格掛一盞燈籠
fn cave_path() -> Vec<Lantern> {
    vec![
        lantern("RIGHT", false),
        lantern("RIGHT", false),
        lantern("DOWN", true),
        lantern("DOWN", false),
        lantern("RIGHT", false),
        lantern("RIGHT", true),
        lantern("DOWN", false),
        lantern("RIGHT", false),
    ]
}

// 把所有燈籠點亮
fn light_all(lanterns: &mut [Lantern]) {
    lanterns.iter_mut().map(|l| l.lit = true);
}

// 點亮通道後出發：只走燈籠照亮的路，遇到暗的就停下來
fn route() -> Vec<String> {
    let mut lanterns = cave_path();
    light_all(&mut lanterns);
    let mut cmds = Vec::new();
    for l in &lanterns {
        if !l.lit {
            cmds.push("前面一片漆黑，Ferris 不敢往前".to_string());
            break;
        }
        cmds.push(format!("MOVE {}", l.dir));
        if l.gem {
            cmds.push("PICK".to_string());
        }
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
