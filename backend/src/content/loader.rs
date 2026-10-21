//! 掃描、解析、驗證 content/

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;

use super::LevelStore;
use super::model::{ChapterSummary, GridDef, Level, LevelKind, LevelVisual, MemorySnapshot, Point};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct LoadError(pub String);

fn fail<T>(path: &Path, reason: impl std::fmt::Display) -> Result<T, LoadError> {
    Err(LoadError(format!("{}: {reason}", path.display())))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChaptersToml {
    chapters: Vec<ChapterToml>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChapterToml {
    chapter: u32,
    title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelToml {
    id: String,
    chapter: u32,
    order: u32,
    title: String,
    kind: LevelKind,
    visual: LevelVisual,
    story: String,
    concept: String,
    #[serde(default)]
    hints: Vec<String>,
    #[serde(default)]
    grid: Option<GridDef>,
    #[serde(default)]
    memory_snapshots: Option<Vec<MemorySnapshot>>,
}

fn read_file(path: &Path) -> Result<String, LoadError> {
    fs::read_to_string(path).or_else(|e| fail(path, e))
}

/// 載入 content 根目錄（含 chapters.toml、shared/grid_sim.rs、levels/）
pub fn load(root: &Path) -> Result<LevelStore, LoadError> {
    let chapters = load_chapters(&root.join("chapters.toml"))?;
    let grid_sim = read_file(&root.join("shared").join("grid_sim.rs"))?;

    let levels_dir = root.join("levels");
    let entries = fs::read_dir(&levels_dir).or_else(|e| fail(&levels_dir, e))?;
    let mut levels: Vec<Level> = Vec::new();
    for entry in entries {
        let entry = entry.or_else(|e| fail(&levels_dir, e))?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let level = load_level(&dir)?;
        if !chapters.iter().any(|c| c.chapter == level.chapter) {
            return fail(
                &dir,
                format_args!("chapter {} 不在 chapters.toml", level.chapter),
            );
        }
        levels.push(level);
    }

    let mut orders: HashMap<u32, &str> = HashMap::new();
    for level in &levels {
        if let Some(other) = orders.insert(level.order, &level.id) {
            return fail(
                &levels_dir.join(&level.id),
                format_args!("order {} 與 {other} 重複", level.order),
            );
        }
    }

    levels.sort_by_key(|l| (l.chapter, l.order));
    let mut chapters: Vec<ChapterSummary> = chapters
        .into_iter()
        .filter(|c| levels.iter().any(|l| l.chapter == c.chapter))
        .collect();
    chapters.sort_by_key(|c| c.chapter);

    let ordered: Vec<Arc<Level>> = levels.into_iter().map(Arc::new).collect();
    let by_id = ordered
        .iter()
        .map(|l| (l.id.clone(), Arc::clone(l)))
        .collect();
    Ok(LevelStore {
        by_id,
        ordered,
        chapters,
        grid_sim,
    })
}

fn load_chapters(path: &Path) -> Result<Vec<ChapterSummary>, LoadError> {
    let raw: ChaptersToml = toml::from_str(&read_file(path)?).or_else(|e| fail(path, e))?;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for c in raw.chapters {
        if c.chapter == 0 {
            return fail(path, "chapter 必須是正整數");
        }
        if c.title.trim().is_empty() {
            return fail(path, format_args!("第 {} 章 title 不可為空", c.chapter));
        }
        if !seen.insert(c.chapter) {
            return fail(path, format_args!("chapter {} 重複", c.chapter));
        }
        out.push(ChapterSummary {
            chapter: c.chapter,
            title: c.title,
        });
    }
    Ok(out)
}

fn read_source(dir: &Path, file: &str) -> Result<String, LoadError> {
    let path = dir.join(file);
    let src = read_file(&path)?;
    if src.trim().is_empty() {
        return fail(&path, "檔案不可為空");
    }
    Ok(src)
}

fn load_level(dir: &Path) -> Result<Level, LoadError> {
    let toml_path = dir.join("level.toml");
    let raw: LevelToml =
        toml::from_str(&read_file(&toml_path)?).or_else(|e| fail(&toml_path, e))?;

    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    if raw.id != dir_name {
        return fail(&toml_path, format_args!("id {} 與目錄名不同", raw.id));
    }
    if !is_valid_id(&raw.id) {
        return fail(&toml_path, format_args!("id {} 格式不符", raw.id));
    }
    if raw.order == 0 {
        return fail(&toml_path, "order 必須是正整數");
    }
    for (field, value) in [
        ("title", &raw.title),
        ("story", &raw.story),
        ("concept", &raw.concept),
    ] {
        if value.trim().is_empty() {
            return fail(&toml_path, format_args!("{field} 不可為空"));
        }
    }
    if raw.hints.iter().any(|h| h.is_empty()) {
        return fail(&toml_path, "hints 不可含空字串");
    }

    match (raw.visual, &raw.grid) {
        (LevelVisual::Grid, None) => return fail(&toml_path, "visual = grid 必須有 [grid]"),
        (LevelVisual::Grid, Some(grid)) => {
            validate_grid(grid).or_else(|e| fail(&toml_path, format_args!("[grid] {e}")))?
        }
        (_, Some(_)) => return fail(&toml_path, "只有 visual = grid 可以有 [grid]"),
        (_, None) => {}
    }
    match (raw.visual, &raw.memory_snapshots) {
        (LevelVisual::Memory, None) => {
            return fail(&toml_path, "visual = memory 必須有 memory_snapshots");
        }
        (LevelVisual::Memory, Some(snaps)) => validate_snapshots(snaps)
            .or_else(|e| fail(&toml_path, format_args!("memory_snapshots {e}")))?,
        (_, Some(_)) => return fail(&toml_path, "只有 visual = memory 可以有 memory_snapshots"),
        (_, None) => {}
    }

    let starter = read_source(dir, "starter.rs")?;
    let solution = read_source(dir, "solution.rs")?;
    let tests = read_source(dir, "tests.rs")?;
    let test_names = extract_test_names(&tests);
    if test_names.is_empty() {
        return fail(&dir.join("tests.rs"), "至少要有一個 #[test]");
    }
    let level_grid_src = raw.grid.as_ref().map(generate_level_grid);

    Ok(Level {
        id: raw.id,
        chapter: raw.chapter,
        order: raw.order,
        title: raw.title,
        kind: raw.kind,
        visual: raw.visual,
        story: raw.story,
        concept: raw.concept,
        hints: raw.hints,
        starter,
        solution,
        tests,
        grid: raw.grid,
        memory_snapshots: raw.memory_snapshots,
        test_names,
        level_grid_src,
    })
}

/// ^[0-9]{2}-[a-z0-9-]+$
fn is_valid_id(id: &str) -> bool {
    let b = id.as_bytes();
    b.len() > 3
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b'-'
        && b[3..]
            .iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn validate_grid(g: &GridDef) -> Result<(), String> {
    for (name, v) in [("width", g.width), ("height", g.height)] {
        if !(2..=12).contains(&v) {
            return Err(format!("{name} 必須在 2–12"));
        }
    }
    let objects: Vec<(&str, &Point)> = [("start", &g.start), ("goal", &g.goal)]
        .into_iter()
        .chain(g.walls.iter().map(|p| ("walls", p)))
        .chain(g.gems.iter().map(|p| ("gems", p)))
        .chain(g.doors.iter().map(|p| ("doors", p)))
        .collect();
    let mut seen: HashSet<&Point> = HashSet::new();
    for (name, p) in objects {
        if !(0..g.width).contains(&p[0]) || !(0..g.height).contains(&p[1]) {
            return Err(format!("{name} 座標 {p:?} 超出地圖"));
        }
        if !seen.insert(p) {
            return Err(format!("{name} 座標 {p:?} 與其他物件重疊"));
        }
    }
    Ok(())
}

fn is_valid_error_code(code: &str) -> bool {
    let b = code.as_bytes();
    b.len() == 5 && b[0] == b'E' && b[1..].iter().all(u8::is_ascii_digit)
}

fn validate_snapshots(snaps: &[MemorySnapshot]) -> Result<(), String> {
    if snaps.is_empty() {
        return Err("至少要有一步".into());
    }
    for (i, s) in snaps.iter().enumerate() {
        let step = i + 1;
        if s.line == 0 {
            return Err(format!("第 {step} 步 line 必須 ≥ 1"));
        }
        let mut frame_ids = HashSet::new();
        let mut var_ids = HashSet::new();
        for f in &s.frames {
            if !frame_ids.insert(f.id.as_str()) {
                return Err(format!("第 {step} 步 frame id {} 重複", f.id));
            }
            for v in &f.vars {
                if !var_ids.insert(v.id.as_str()) {
                    return Err(format!("第 {step} 步變數 id {} 重複", v.id));
                }
            }
        }
        let mut heap_ids = HashSet::new();
        for h in &s.heap {
            if !heap_ids.insert(h.id.as_str()) {
                return Err(format!("第 {step} 步 heap id {} 重複", h.id));
            }
        }
        let exists = |id: &str| var_ids.contains(id) || heap_ids.contains(id);
        for p in &s.pointers {
            if !var_ids.contains(p.from.as_str()) {
                return Err(format!("第 {step} 步指標 from {} 不是變數", p.from));
            }
            if !exists(&p.to) {
                return Err(format!("第 {step} 步指標 to {} 不存在", p.to));
            }
        }
        if let Some(err) = &s.error {
            if !exists(&err.target) {
                return Err(format!("第 {step} 步 error.target {} 不存在", err.target));
            }
            if let Some(code) = &err.code
                && !is_valid_error_code(code)
            {
                return Err(format!("第 {step} 步 error.code {code} 格式不符"));
            }
        }
    }
    Ok(())
}

/// 依宣告順序抽出 #[test] 函式名稱
pub fn extract_test_names(tests_src: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut pending = false;
    for line in tests_src.lines() {
        let line = line.trim();
        if line == "#[test]" {
            pending = true;
            continue;
        }
        if pending && let Some(rest) = line.strip_prefix("fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                names.push(name);
            }
            pending = false;
        }
    }
    names
}

fn tuple(p: &Point) -> String {
    format!("({}, {})", p[0], p[1])
}

fn tuple_vec(points: &[Point]) -> String {
    let items: Vec<String> = points.iter().map(tuple).collect();
    format!("vec![{}]", items.join(", "))
}

/// 依 [grid] 產生 fn level_grid() -> Grid 原始碼
pub fn generate_level_grid(grid: &GridDef) -> String {
    format!(
        "fn level_grid() -> Grid {{ Grid {{ width: {}, height: {}, start: {}, goal: {}, walls: {}, gems: {}, doors: {} }} }}",
        grid.width,
        grid.height,
        tuple(&grid.start),
        tuple(&grid.goal),
        tuple_vec(&grid.walls),
        tuple_vec(&grid.gems),
        tuple_vec(&grid.doors),
    )
}
