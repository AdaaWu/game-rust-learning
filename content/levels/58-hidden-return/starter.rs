// 唸完一卷咒語的結果
struct Chant {
    cmds: Vec<String>,    // 唸得出來的指令，依序排好
    skipped: Vec<String>, // 唸不出來的壞咒語原文
}

// 解讀一道咒語：認得就回傳指令，認不得就把原文交回
fn parse_spell(spell: &str) -> Result<String, String> {
    let words: Vec<&str> = spell.split_whitespace().collect();
    match words.as_slice() {
        ["PICK"] => Ok("PICK".to_string()),
        ["MOVE", dir] if ["UP", "DOWN", "LEFT", "RIGHT"].contains(dir) => Ok(format!("MOVE {}", dir)),
        _ => Err(spell.to_string()),
    }
}

// 唸一道咒語：認得就記進路線，唸不出來就收工
macro_rules! step_or_bail {
    ($chant:ident, $spell:expr) => {
        match parse_spell($spell) {
            Ok(cmd) => $chant.cmds.push(cmd),
            Err(_) => return $chant,
        }
    };
}

// 唸完整卷咒語：壞咒語略過並記進 skipped，其餘照順序唸完
fn chant(spells: &[&str]) -> Chant {
    let mut chant = Chant { cmds: Vec::new(), skipped: Vec::new() };
    for spell in spells {
        step_or_bail!(chant, spell);
    }
    chant
}

fn route() -> Vec<String> {
    let scroll = [
        "MOVE RIGHT", "MOVE RIGHT", "PICK", "MOVE UP", "SWIM RIGHT", "MOVE RIGHT",
        "MOVE RIGHT", "PICK", "MOVE DOWN", "MOVE RIGHT", "MOVE RIGHT",
    ];
    chant(&scroll).cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
