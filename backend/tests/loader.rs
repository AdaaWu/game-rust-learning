//! B1 關卡 loader：啟動一次載入、欄位驗證、[grid] 驗證、memory 驗證、測試名稱抽取

mod common;

use common::{ContentDir, load_fixture_store, names, read_fixture};
use rust_quest_backend::content::loader::extract_test_names;
use rust_quest_backend::content::model::{
    HeapState, LevelKind, LevelVisual, PointerKind, VarState,
};

const GRID: &str = "01-walk-grid";
const GRID2: &str = "09-late-grid";
const PLAIN: &str = "02-plain-sum";
const MEMORY: &str = "03-move-memory";

fn toml(id: &str) -> String {
    format!("levels/{id}/level.toml")
}

/// 把 01-walk-grid 的 [grid] 換成指定內容
fn set_grid(cd: &ContentDir, body: &str) {
    let rel = toml(GRID);
    let s = cd.read(&rel);
    let idx = s.find("[grid]").expect("fixture 應有 [grid]");
    cd.write(&rel, &format!("{}[grid]\n{}\n", &s[..idx], body));
}

// ───────── 成功載入 ─────────

#[test]
fn loads_all_fixture_levels_sorted_by_chapter_then_order() {
    let store = load_fixture_store();
    let ids: Vec<&str> = store.levels().iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids, vec![GRID, GRID2, PLAIN, MEMORY]);
}

#[test]
fn get_returns_level_by_id_and_none_for_unknown() {
    let store = load_fixture_store();
    assert_eq!(store.get(PLAIN).map(|l| l.id.as_str()), Some(PLAIN));
    assert!(store.get("99-unknown").is_none());
}

#[test]
fn skips_non_directory_entries_in_levels() {
    let cd = ContentDir::new();
    assert!(cd.path("levels/.gitkeep").exists(), "fixture 應含 .gitkeep");
    cd.write("levels/notes.txt", "不是關卡");
    let store = cd.load().expect(".gitkeep 與一般檔案應被略過");
    assert_eq!(store.levels().len(), 4);
}

#[test]
fn parses_required_fields_and_reads_three_source_files() {
    let store = load_fixture_store();
    let level = store.get(GRID).expect("應載入 01-walk-grid");
    assert_eq!(level.chapter, 1);
    assert_eq!(level.order, 1);
    assert_eq!(level.title, "第一步");
    assert_eq!(level.kind, LevelKind::Build);
    assert_eq!(level.visual, LevelVisual::Grid);
    assert_eq!(level.story, "我在沙灘上醒來。");
    assert_eq!(level.concept, "## 函式\n\n- 回傳指令清單");
    assert_eq!(level.hints, names(&["先往下走", "再往右走"]));
    assert_eq!(
        level.starter,
        read_fixture("content/levels/01-walk-grid/starter.rs")
    );
    assert_eq!(
        level.solution,
        read_fixture("content/levels/01-walk-grid/solution.rs")
    );
    assert_eq!(
        level.tests,
        read_fixture("content/levels/01-walk-grid/tests.rs")
    );
}

#[test]
fn parses_every_kind_and_visual_variant() {
    let store = load_fixture_store();
    assert_eq!(
        store.get(GRID2).map(|l| l.kind),
        Some(LevelKind::FixCompile)
    );
    assert_eq!(store.get(PLAIN).map(|l| l.kind), Some(LevelKind::Debug));
    assert_eq!(store.get(PLAIN).map(|l| l.visual), Some(LevelVisual::None));
    assert_eq!(
        store.get(MEMORY).map(|l| l.visual),
        Some(LevelVisual::Memory)
    );
}

#[test]
fn omitted_hints_become_empty_vec() {
    let store = load_fixture_store();
    assert_eq!(store.get(GRID2).map(|l| l.hints.clone()), Some(vec![]));
}

#[test]
fn grid_omitted_arrays_become_empty_vec() {
    let store = load_fixture_store();
    let grid = store
        .get(GRID)
        .and_then(|l| l.grid.clone())
        .expect("01 應有 grid");
    assert_eq!(grid.width, 5);
    assert_eq!(grid.height, 3);
    assert_eq!(grid.start, [0, 0]);
    assert_eq!(grid.goal, [4, 1]);
    assert_eq!(grid.walls, vec![[2, 0], [2, 1]]);
    assert!(grid.gems.is_empty());
    assert!(grid.doors.is_empty());
}

#[test]
fn grid_with_all_arrays_is_parsed() {
    let store = load_fixture_store();
    let grid = store
        .get(GRID2)
        .and_then(|l| l.grid.clone())
        .expect("09 應有 grid");
    assert_eq!(grid.walls, vec![[1, 0]]);
    assert_eq!(grid.gems, vec![[0, 2]]);
    assert_eq!(grid.doors, vec![[3, 1]]);
}

#[test]
fn grid_and_memory_are_none_for_other_visuals() {
    let store = load_fixture_store();
    let grid_level = store.get(GRID).expect("01");
    assert!(grid_level.memory_snapshots.is_none());
    let plain = store.get(PLAIN).expect("02");
    assert!(plain.grid.is_none());
    assert!(plain.memory_snapshots.is_none());
    let memory = store.get(MEMORY).expect("03");
    assert!(memory.grid.is_none());
}

#[test]
fn loads_shared_grid_sim_source() {
    let store = load_fixture_store();
    assert_eq!(store.grid_sim(), read_fixture("content/shared/grid_sim.rs"));
}

#[test]
fn level_test_names_follow_declaration_order() {
    let store = load_fixture_store();
    assert_eq!(
        store.get(PLAIN).map(|l| l.test_names.clone()),
        Some(names(&["sums_three", "empty_is_zero", "all_ones"]))
    );
}

// ───────── memory_snapshots 解析與預設值 ─────────

#[test]
fn memory_snapshots_are_parsed_with_defaults() {
    let store = load_fixture_store();
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03 應有 memory_snapshots");
    assert_eq!(snaps.len(), 2);

    let s1 = &snaps[0];
    assert_eq!(s1.line, 8);
    assert_eq!(s1.note, "bag 擁有 heap 上的三個字串");
    assert_eq!(s1.frames.len(), 1);
    assert_eq!(s1.frames[0].id, "report");
    assert_eq!(s1.frames[0].fn_name, "report");
    assert_eq!(s1.frames[0].vars[0].id, "report.bag");
    assert_eq!(s1.frames[0].vars[0].ty, "Vec<String>");
    assert_eq!(s1.frames[0].vars[0].value.as_deref(), Some("len=3 cap=3"));
    assert_eq!(s1.frames[0].vars[0].state, VarState::Live);
    assert_eq!(s1.heap[0].id, "h1");
    assert_eq!(s1.heap[0].ty, "[String; 3]");
    assert_eq!(s1.heap[0].state, HeapState::Live);
    assert_eq!(s1.pointers[0].from, "report.bag");
    assert_eq!(s1.pointers[0].to, "h1");
    assert_eq!(s1.pointers[0].kind, PointerKind::Own);
    assert!(s1.error.is_none(), "省略 error 應為 None");

    let s2 = &snaps[1];
    assert_eq!(s2.heap, vec![], "省略 heap 應為空陣列");
    assert_eq!(s2.pointers, vec![], "省略 pointers 應為空陣列");
    assert_eq!(s2.frames[0].vars[0].value, None, "省略 value 應為 None");
    assert_eq!(s2.frames[0].vars[0].state, VarState::Moved);
}

#[test]
fn memory_omitted_frames_and_vars_become_empty_vec() {
    let cd = ContentDir::new();
    let rel = toml(MEMORY);
    let s = cd.read(&rel);
    // 第三步只寫 line、note；另加一個沒有 vars 的 frame
    cd.write(
        &rel,
        &format!("{s}\n[[memory_snapshots]]\nline = 11\nnote = \"結束\"\n"),
    );
    let store = cd.load().expect("省略 frames 應可載入");
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03");
    assert_eq!(snaps.len(), 3);
    assert_eq!(snaps[2].frames, vec![]);

    let cd = ContentDir::new();
    let s = cd.read(&rel);
    cd.write(
        &rel,
        &format!("{s}\n[[memory_snapshots.frames]]\nid = \"main\"\nfn = \"main\"\n"),
    );
    let store = cd.load().expect("省略 vars 應可載入");
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03");
    let main = snaps[1]
        .frames
        .iter()
        .find(|f| f.id == "main")
        .expect("main frame");
    assert_eq!(main.vars, vec![]);
}

#[test]
fn memory_same_ids_across_steps_are_allowed() {
    // fixture 兩步都有 report.bag，載入成功即代表跨步可重複
    let store = load_fixture_store();
    assert!(store.get(MEMORY).is_some());
}

#[test]
fn memory_pointer_to_variable_is_allowed() {
    let cd = ContentDir::new();
    let rel = toml(MEMORY);
    cd.replace(
        &rel,
        "{ from = \"report.bag\", to = \"h1\", kind = \"own\" },",
        "{ from = \"report.bag\", to = \"h1\", kind = \"own\" },\n  { from = \"report.r\", to = \"report.bag\", kind = \"shared\" },",
    );
    cd.replace(
        &rel,
        "value = \"len=3 cap=3\", state = \"live\" },",
        "value = \"len=3 cap=3\", state = \"live\" },\n  { id = \"report.r\", name = \"r\", type = \"&Vec<String>\", state = \"live\" },",
    );
    let store = cd.load().expect("指向變數的指標應可載入");
    let snaps = store
        .get(MEMORY)
        .and_then(|l| l.memory_snapshots.clone())
        .expect("03");
    assert_eq!(snaps[0].pointers.len(), 2);
    assert_eq!(snaps[0].pointers[1].kind, PointerKind::Shared);
}

// ───────── chapters.toml ─────────

#[test]
fn fails_when_chapters_toml_missing() {
    let cd = ContentDir::new();
    cd.remove("chapters.toml");
    cd.assert_load_fails("chapters.toml");
}

#[test]
fn fails_when_chapter_number_duplicated() {
    let cd = ContentDir::new();
    let s = cd.read("chapters.toml");
    cd.write(
        "chapters.toml",
        &format!("{s}\n[[chapters]]\nchapter = 2\ntitle = \"重複\"\n"),
    );
    cd.assert_load_fails("chapters.toml");
}

#[test]
fn fails_when_chapters_toml_has_unknown_field() {
    let cd = ContentDir::new();
    cd.replace(
        "chapters.toml",
        "title = \"沙灘甦醒\"",
        "title = \"沙灘甦醒\"\nsubtitle = \"基礎語法\"",
    );
    cd.assert_load_fails("chapters.toml");
}

#[test]
fn fails_when_chapter_title_is_blank() {
    let cd = ContentDir::new();
    cd.replace("chapters.toml", "title = \"潮汐羅盤\"", "title = \"  \"");
    cd.assert_load_fails("chapters.toml");
}

#[test]
fn fails_when_chapter_number_not_positive() {
    let cd = ContentDir::new();
    cd.replace("chapters.toml", "chapter = 4", "chapter = 0");
    cd.assert_load_fails("chapters.toml");
}

#[test]
fn fails_when_level_chapter_not_in_chapters_toml() {
    let cd = ContentDir::new();
    cd.replace(&toml(PLAIN), "chapter = 2", "chapter = 7");
    cd.assert_load_fails(PLAIN);
}

#[test]
fn fails_when_shared_grid_sim_missing() {
    let cd = ContentDir::new();
    cd.remove("shared/grid_sim.rs");
    cd.assert_load_fails("grid_sim.rs");
}

// ───────── level.toml 必填與格式 ─────────

#[test]
fn fails_when_id_differs_from_directory_name() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(PLAIN),
        "id = \"02-plain-sum\"",
        "id = \"02-plain-total\"",
    );
    cd.assert_load_fails(PLAIN);
}

#[test]
fn fails_when_id_format_invalid() {
    for bad in ["2-bad", "05-Bad", "05_bad", "05-", "005-bad", "ab-bad"] {
        let cd = ContentDir::new();
        cd.clone_level(PLAIN, bad, bad, 50);
        cd.assert_load_fails(bad);
    }
}

#[test]
fn fails_when_order_duplicated() {
    let cd = ContentDir::new();
    cd.replace(&toml(PLAIN), "order = 2", "order = 1");
    let result = cd.load();
    assert!(result.is_err(), "order 重複應載入失敗");
    let msg = result.err().unwrap().to_string();
    assert!(
        msg.contains(PLAIN) || msg.contains(GRID),
        "錯誤訊息應含重複的關卡目錄，實際：{msg}"
    );
}

#[test]
fn fails_when_order_not_positive() {
    for bad in ["0", "-3"] {
        let cd = ContentDir::new();
        cd.replace(&toml(PLAIN), "order = 2", &format!("order = {bad}"));
        cd.assert_load_fails(PLAIN);
    }
}

#[test]
fn fails_when_required_field_missing() {
    for field in [
        "id", "chapter", "order", "title", "kind", "visual", "story", "concept",
    ] {
        let cd = ContentDir::new();
        let rel = toml(PLAIN);
        let s = cd.read(&rel);
        let kept: Vec<&str> = s
            .lines()
            .filter(|l| !l.starts_with(&format!("{field} = ")))
            .collect();
        assert_ne!(kept.len(), s.lines().count(), "fixture 應有欄位 {field}");
        cd.write(&rel, &(kept.join("\n") + "\n"));
        let result = cd.load();
        assert!(result.is_err(), "缺 {field} 應載入失敗");
        let msg = result.err().unwrap().to_string();
        assert!(
            msg.contains(PLAIN),
            "缺 {field} 的錯誤訊息應含目錄名，實際：{msg}"
        );
    }
}

#[test]
fn fails_when_text_field_blank() {
    for (field, original) in [
        ("title", "title = \"數貝殼\""),
        ("story", "story = \"我想數清楚貝殼。\""),
        ("concept", "concept = \"## 迭代\\n\\n- iter().sum()\""),
    ] {
        for blank in ["\"\"", "\"  \\n \""] {
            let cd = ContentDir::new();
            cd.replace(&toml(PLAIN), original, &format!("{field} = {blank}"));
            cd.assert_load_fails(PLAIN);
        }
    }
}

#[test]
fn fails_when_kind_or_visual_invalid() {
    let cd = ContentDir::new();
    cd.replace(&toml(PLAIN), "kind = \"debug\"", "kind = \"puzzle\"");
    cd.assert_load_fails(PLAIN);

    let cd = ContentDir::new();
    cd.replace(&toml(PLAIN), "visual = \"none\"", "visual = \"graph\"");
    cd.assert_load_fails(PLAIN);
}

#[test]
fn fails_on_unknown_top_level_field() {
    let cd = ContentDir::new();
    cd.insert_top_level(PLAIN, "par = 3");
    cd.assert_load_fails(PLAIN);
}

#[test]
fn fails_on_unknown_grid_field() {
    let cd = ContentDir::new();
    cd.replace(&toml(GRID), "width = 5", "width = 5\nmap = \"S..G\"");
    cd.assert_load_fails(GRID);
}

#[test]
fn fails_when_hint_is_empty_string() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(PLAIN),
        "hints = [\"看看 map 做了什麼\"]",
        "hints = [\"看看 map 做了什麼\", \"\"]",
    );
    cd.assert_load_fails(PLAIN);
}

#[test]
fn empty_hints_array_is_allowed() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(PLAIN),
        "hints = [\"看看 map 做了什麼\"]",
        "hints = []",
    );
    let store = cd.load().expect("hints = [] 應可載入");
    assert_eq!(store.get(PLAIN).map(|l| l.hints.clone()), Some(vec![]));
}

// ───────── 三個 .rs 檔 ─────────

#[test]
fn fails_when_source_file_missing() {
    for file in ["starter.rs", "solution.rs", "tests.rs"] {
        let cd = ContentDir::new();
        cd.remove(&format!("levels/{PLAIN}/{file}"));
        cd.assert_load_fails(PLAIN);
    }
}

#[test]
fn fails_when_source_file_empty() {
    for file in ["starter.rs", "solution.rs", "tests.rs"] {
        let cd = ContentDir::new();
        cd.write(&format!("levels/{PLAIN}/{file}"), "");
        cd.assert_load_fails(PLAIN);
    }
}

#[test]
fn fails_when_tests_rs_has_no_test_attribute() {
    let cd = ContentDir::new();
    cd.write(&format!("levels/{PLAIN}/tests.rs"), "fn helper() {}\n");
    cd.assert_load_fails(PLAIN);
}

// ───────── [grid] 驗證 ─────────

#[test]
fn fails_when_grid_visual_has_no_grid_table() {
    let cd = ContentDir::new();
    let rel = toml(GRID);
    let s = cd.read(&rel);
    let idx = s.find("[grid]").unwrap();
    cd.write(&rel, &s[..idx]);
    cd.assert_load_fails(GRID);
}

#[test]
fn fails_when_non_grid_visual_has_grid_table() {
    for id in [PLAIN, MEMORY] {
        let cd = ContentDir::new();
        let rel = toml(id);
        let s = cd.read(&rel);
        cd.write(
            &rel,
            &format!("{s}\n[grid]\nwidth = 3\nheight = 3\nstart = [0, 0]\ngoal = [2, 2]\n"),
        );
        cd.assert_load_fails(id);
    }
}

#[test]
fn grid_size_boundaries_two_and_twelve_are_allowed() {
    let cd = ContentDir::new();
    set_grid(&cd, "width = 2\nheight = 2\nstart = [0, 0]\ngoal = [1, 1]");
    let store = cd.load().expect("2×2 應可載入");
    assert_eq!(
        store
            .get(GRID)
            .and_then(|l| l.grid.as_ref())
            .map(|g| (g.width, g.height)),
        Some((2, 2))
    );

    let cd = ContentDir::new();
    set_grid(
        &cd,
        "width = 12\nheight = 12\nstart = [0, 0]\ngoal = [11, 11]",
    );
    let store = cd.load().expect("12×12 應可載入");
    assert_eq!(
        store
            .get(GRID)
            .and_then(|l| l.grid.as_ref())
            .map(|g| (g.width, g.height)),
        Some((12, 12))
    );
}

#[test]
fn fails_when_grid_size_out_of_range() {
    for (w, h, goal) in [
        (1, 3, "[0, 1]"),
        (13, 3, "[1, 0]"),
        (3, 1, "[1, 0]"),
        (3, 13, "[1, 0]"),
    ] {
        let cd = ContentDir::new();
        set_grid(
            &cd,
            &format!("width = {w}\nheight = {h}\nstart = [0, 0]\ngoal = {goal}"),
        );
        cd.assert_load_fails(GRID);
    }
}

#[test]
fn fails_when_coordinate_out_of_bounds() {
    for body in [
        // x == width
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [5, 1]",
        // y == height
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\nwalls = [[0, 3]]",
        // 負數
        "width = 5\nheight = 3\nstart = [-1, 0]\ngoal = [4, 1]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\ngems = [[1, -1]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\ndoors = [[9, 0]]",
    ] {
        let cd = ContentDir::new();
        set_grid(&cd, body);
        cd.assert_load_fails(GRID);
    }
}

#[test]
fn fails_when_grid_objects_overlap() {
    for body in [
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [0, 0]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\nwalls = [[0, 0]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\ngems = [[4, 1]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\nwalls = [[2, 2]]\ndoors = [[2, 2]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\nwalls = [[2, 2]]\ngems = [[2, 2]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\ngems = [[3, 0]]\ndoors = [[3, 0]]",
    ] {
        let cd = ContentDir::new();
        set_grid(&cd, body);
        cd.assert_load_fails(GRID);
    }
}

#[test]
fn fails_when_array_has_duplicate_coordinate() {
    for body in [
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\nwalls = [[2, 0], [2, 0]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\ngems = [[1, 1], [1, 1]]",
        "width = 5\nheight = 3\nstart = [0, 0]\ngoal = [4, 1]\ndoors = [[3, 2], [3, 2]]",
    ] {
        let cd = ContentDir::new();
        set_grid(&cd, body);
        cd.assert_load_fails(GRID);
    }
}

// ───────── memory_snapshots 驗證 ─────────

#[test]
fn fails_when_memory_visual_has_no_snapshots() {
    let cd = ContentDir::new();
    let rel = toml(MEMORY);
    let s = cd.read(&rel);
    let idx = s.find("[[memory_snapshots]]").unwrap();
    cd.write(&rel, &s[..idx]);
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_memory_snapshots_empty_array() {
    let cd = ContentDir::new();
    let rel = toml(MEMORY);
    let s = cd.read(&rel);
    let idx = s.find("[[memory_snapshots]]").unwrap();
    cd.write(&rel, &format!("{}memory_snapshots = []\n", &s[..idx]));
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_non_memory_visual_has_snapshots() {
    for id in [GRID, PLAIN] {
        let cd = ContentDir::new();
        let rel = toml(id);
        let s = cd.read(&rel);
        cd.write(
            &rel,
            &format!("{s}\n[[memory_snapshots]]\nline = 1\nnote = \"多餘\"\n"),
        );
        cd.assert_load_fails(id);
    }
}

#[test]
fn fails_when_duplicate_frame_id_in_same_step() {
    let cd = ContentDir::new();
    let rel = toml(MEMORY);
    let s = cd.read(&rel);
    cd.write(
        &rel,
        &format!("{s}\n[[memory_snapshots.frames]]\nid = \"report\"\nfn = \"again\"\n"),
    );
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_duplicate_var_id_in_same_step() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "{ id = \"report.bag\", name = \"bag\", type = \"Vec<String>\", state = \"moved\" },",
        "{ id = \"report.bag\", name = \"bag\", type = \"Vec<String>\", state = \"moved\" },\n  { id = \"report.bag\", name = \"bag2\", type = \"u8\", state = \"live\" },",
    );
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_duplicate_heap_id_in_same_step() {
    let cd = ContentDir::new();
    cd.replace(
        &toml(MEMORY),
        "value = \"珍珠 | 貝殼 | 海星\", state = \"live\" },",
        "value = \"珍珠 | 貝殼 | 海星\", state = \"live\" },\n  { id = \"h1\", type = \"u8\", value = \"1\", state = \"live\" },",
    );
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_pointer_endpoints_invalid() {
    for replacement in [
        // from 不存在
        "{ from = \"ghost\", to = \"h1\", kind = \"own\" },",
        // to 不存在
        "{ from = \"report.bag\", to = \"h9\", kind = \"own\" },",
        // from 必須是變數 id，不可是 heap id
        "{ from = \"h1\", to = \"report.bag\", kind = \"own\" },",
    ] {
        let cd = ContentDir::new();
        cd.replace(
            &toml(MEMORY),
            "{ from = \"report.bag\", to = \"h1\", kind = \"own\" },",
            replacement,
        );
        cd.assert_load_fails(MEMORY);
    }
}

#[test]
fn fails_when_snapshot_line_is_zero() {
    let cd = ContentDir::new();
    cd.replace(&toml(MEMORY), "line = 8", "line = 0");
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_snapshot_note_missing() {
    let cd = ContentDir::new();
    cd.replace(&toml(MEMORY), "note = \"bag 擁有 heap 上的三個字串\"\n", "");
    cd.assert_load_fails(MEMORY);
}

#[test]
fn fails_when_memory_enum_value_invalid() {
    for (from, to) in [
        ("state = \"moved\"", "state = \"gone\""),
        (
            "value = \"珍珠 | 貝殼 | 海星\", state = \"live\"",
            "value = \"珍珠 | 貝殼 | 海星\", state = \"moved\"",
        ),
        ("kind = \"own\"", "kind = \"weak\""),
    ] {
        let cd = ContentDir::new();
        cd.replace(&toml(MEMORY), from, to);
        cd.assert_load_fails(MEMORY);
    }
}

#[test]
fn fails_on_unknown_field_in_snapshot() {
    let cd = ContentDir::new();
    cd.replace(&toml(MEMORY), "line = 8", "line = 8\nstep = 1");
    cd.assert_load_fails(MEMORY);
}

// ───────── 測試名稱抽取（純函式）─────────

#[test]
fn extract_test_names_keeps_declaration_order() {
    let src = "#[test]\nfn zeta() {}\n\n#[test]\nfn alpha() {}\n\n#[test]\nfn mid() {}\n";
    assert_eq!(extract_test_names(src), names(&["zeta", "alpha", "mid"]));
}

#[test]
fn extract_test_names_skips_helper_functions() {
    let src = "fn helper() -> u8 { 1 }\n\n#[test]\nfn only_test() {\n    assert_eq!(helper(), 1);\n}\n\nfn after() {}\n";
    assert_eq!(extract_test_names(src), names(&["only_test"]));
}

#[test]
fn extract_test_names_allows_attributes_and_comments_between() {
    let src = "#[test]\n#[should_panic]\nfn panics() {}\n\n#[test]\n// 說明\nfn commented() {}\n";
    assert_eq!(extract_test_names(src), names(&["panics", "commented"]));
}

#[test]
fn extract_test_names_from_fixture_file() {
    let src = read_fixture("content/levels/02-plain-sum/tests.rs");
    assert_eq!(
        extract_test_names(&src),
        names(&["sums_three", "empty_is_zero", "all_ones"])
    );
}

#[test]
fn fails_when_level_dir_has_no_level_toml() {
    let cd = ContentDir::new();
    cd.remove(&format!("levels/{PLAIN}/level.toml"));
    cd.assert_load_fails(PLAIN);
}
