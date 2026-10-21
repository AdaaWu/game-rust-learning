//! Rust Quest 後端：關卡內容、判題器、HTTP 路由

pub mod content;
pub mod error;
pub mod judge;
pub mod routes;

use std::sync::Arc;

use content::LevelStore;
use judge::Judge;

pub use routes::app;

#[derive(Clone)]
pub struct AppState {
    pub levels: Arc<LevelStore>,
    pub judge: Arc<Judge>,
}

impl AppState {
    pub fn new(levels: LevelStore, judge: Judge) -> Self {
        Self {
            levels: Arc::new(levels),
            judge: Arc::new(judge),
        }
    }
}
