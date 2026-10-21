//! 內部 Level 與對外 DTO

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LevelKind {
    Build,
    FixCompile,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LevelVisual {
    Grid,
    Memory,
    Timeline,
    None,
}

/// 座標 [x, y]
pub type Point = [i64; 2];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridDef {
    pub width: i64,
    pub height: i64,
    pub start: Point,
    pub goal: Point,
    #[serde(default)]
    pub walls: Vec<Point>,
    #[serde(default)]
    pub gems: Vec<Point>,
    #[serde(default)]
    pub doors: Vec<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarState {
    Live,
    Moved,
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeapState {
    Live,
    Freed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PointerKind {
    Own,
    Shared,
    Mut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackVar {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(default)]
    pub value: Option<String>,
    pub state: VarState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackFrame {
    pub id: String,
    #[serde(rename = "fn")]
    pub fn_name: String,
    #[serde(default)]
    pub vars: Vec<StackVar>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeapBlock {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub value: String,
    pub state: HeapState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pointer {
    pub from: String,
    pub to: String,
    pub kind: PointerKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotError {
    #[serde(default)]
    pub code: Option<String>,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemorySnapshot {
    pub line: u32,
    pub note: String,
    #[serde(default)]
    pub frames: Vec<StackFrame>,
    #[serde(default)]
    pub heap: Vec<HeapBlock>,
    #[serde(default)]
    pub pointers: Vec<Pointer>,
    #[serde(default)]
    pub error: Option<SnapshotError>,
}

/// 內部關卡資料，含 solution 與 tests，不直接序列化給前端
#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub id: String,
    pub chapter: u32,
    pub order: u32,
    pub title: String,
    pub kind: LevelKind,
    pub visual: LevelVisual,
    pub story: String,
    pub concept: String,
    pub hints: Vec<String>,
    pub starter: String,
    pub solution: String,
    pub tests: String,
    pub grid: Option<GridDef>,
    pub memory_snapshots: Option<Vec<MemorySnapshot>>,
    /// tests.rs 依宣告順序抽出的測試函式名稱
    pub test_names: Vec<String>,
    /// grid 關卡依 [grid] 產生的 level_grid() 原始碼
    pub level_grid_src: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChapterSummary {
    pub chapter: u32,
    pub title: String,
}

/// GET /api/levels 的關卡項目
#[derive(Debug, Serialize)]
pub struct LevelSummary<'a> {
    pub id: &'a str,
    pub chapter: u32,
    pub order: u32,
    pub title: &'a str,
    pub kind: LevelKind,
    pub visual: LevelVisual,
}

impl<'a> From<&'a Level> for LevelSummary<'a> {
    fn from(l: &'a Level) -> Self {
        Self {
            id: &l.id,
            chapter: l.chapter,
            order: l.order,
            title: &l.title,
            kind: l.kind,
            visual: l.visual,
        }
    }
}

/// GET /api/levels/:id 的回應
#[derive(Debug, Serialize)]
pub struct LevelDetail<'a> {
    pub id: &'a str,
    pub chapter: u32,
    pub order: u32,
    pub title: &'a str,
    pub kind: LevelKind,
    pub visual: LevelVisual,
    pub story: &'a str,
    pub concept: &'a str,
    pub hints: &'a [String],
    pub starter: &'a str,
    pub grid: Option<&'a GridDef>,
    pub memory_snapshots: Option<&'a [MemorySnapshot]>,
}

impl<'a> From<&'a Level> for LevelDetail<'a> {
    fn from(l: &'a Level) -> Self {
        Self {
            id: &l.id,
            chapter: l.chapter,
            order: l.order,
            title: &l.title,
            kind: l.kind,
            visual: l.visual,
            story: &l.story,
            concept: &l.concept,
            hints: &l.hints,
            starter: &l.starter,
            grid: l.grid.as_ref(),
            memory_snapshots: l.memory_snapshots.as_deref(),
        }
    }
}
