//! Submit 合併檔組成

#[derive(Debug, Clone, PartialEq)]
pub struct MergedSource {
    pub source: String,
    pub user_line_count: usize,
}

const HIDDEN_HEADER: &str = "#[cfg(test)]\n#[allow(dead_code)]\n#[allow(unused_imports)]\nmod __quest_hidden_tests {\n    use super::*;\n";

fn push_with_newline(out: &mut String, s: &str) {
    out.push_str(s);
    if !s.ends_with('\n') {
        out.push('\n');
    }
}

/// grid：grid 關卡傳 Some((grid_sim.rs 全文, level_grid 原始碼))
pub fn merge_submit(user_code: &str, tests_rs: &str, grid: Option<(&str, &str)>) -> MergedSource {
    let mut source = String::new();
    push_with_newline(&mut source, user_code);
    source.push_str(HIDDEN_HEADER);
    if let Some((grid_sim, level_grid)) = grid {
        source.push_str(grid_sim);
        source.push('\n');
        source.push_str(level_grid);
        source.push('\n');
    }
    push_with_newline(&mut source, tests_rs);
    source.push_str("}\n");
    MergedSource {
        source,
        user_line_count: user_code.lines().count(),
    }
}
