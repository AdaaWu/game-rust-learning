//! 關卡內容：啟動時一次載入，之後唯讀

pub mod loader;
pub mod model;

use std::collections::HashMap;
use std::sync::Arc;

use model::{ChapterSummary, Level};

#[derive(Debug, Default)]
pub struct LevelStore {
    by_id: HashMap<String, Arc<Level>>,
    ordered: Vec<Arc<Level>>,
    chapters: Vec<ChapterSummary>,
    grid_sim: String,
}

impl LevelStore {
    pub fn get(&self, id: &str) -> Option<&Arc<Level>> {
        self.by_id.get(id)
    }

    /// 依 (chapter, order) 排序的全部關卡
    pub fn levels(&self) -> &[Arc<Level>] {
        &self.ordered
    }

    /// 至少有一關的章節，依 chapter 升冪
    pub fn chapters(&self) -> &[ChapterSummary] {
        &self.chapters
    }

    /// content/shared/grid_sim.rs 全文
    pub fn grid_sim(&self) -> &str {
        &self.grid_sim
    }
}
