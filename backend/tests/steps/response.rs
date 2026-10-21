//! Then：檢查回應

use std::fs;
use std::process::Command;

use cucumber::gherkin::Step;
use cucumber::then;
use serde_json::Value;

use super::given::{judge_containers, wait_until};
use super::{
    ApiWorld, at, content_dir, docstring, level, literal, pairs, row_matches, sorted_keys,
    split_list, unquote,
};

fn table(step: &Step) -> (Vec<String>, Vec<Vec<String>>) {
    let rows = &step.table.as_ref().expect("步驟需要表格").rows;
    (rows[0].clone(), rows[1..].to_vec())
}

fn array(v: &Value) -> &Vec<Value> {
    v.as_array().unwrap_or_else(|| panic!("不是陣列：{v}"))
}

fn text(v: &Value) -> &str {
    v.as_str().unwrap_or_else(|| panic!("不是字串：{v}"))
}

fn find_key(v: &Value, key: &str) -> bool {
    match v {
        Value::Object(map) => map.iter().any(|(k, c)| k == key || find_key(c, key)),
        Value::Array(items) => items.iter().any(|c| find_key(c, key)),
        _ => false,
    }
}

/// 陣列與表格逐列比對，長度須相同
fn assert_rows_in_order(items: &[Value], step: &Step) {
    let (header, rows) = table(step);
    assert_eq!(items.len(), rows.len(), "筆數不同：{items:?}");
    for (i, (item, row)) in items.iter().zip(&rows).enumerate() {
        assert!(
            row_matches(item, &header, row),
            "第 {} 筆不符\n預期 {header:?} = {row:?}\n實際 {item}",
            i + 1
        );
    }
}

/// 找出符合每一列的項目，回傳最後一列對到的項目
fn assert_contains_rows(items: &[Value], step: &Step) -> Value {
    let (header, rows) = table(step);
    let mut found = Value::Null;
    for row in &rows {
        found = items
            .iter()
            .find(|item| row_matches(item, &header, row))
            .unwrap_or_else(|| panic!("找不到 {header:?} = {row:?}\n實際 {items:?}"))
            .clone();
    }
    found
}

// ---------- 狀態與標頭 ----------

#[then(regex = r"^回應狀態碼為 (\d+)$")]
fn status_is(w: &mut ApiWorld, status: u16) {
    assert_eq!(w.reply().status, status, "{}", w.body_text());
}

#[then(regex = r#"^回應標頭 "([^"]+)" 為 "(.*)"$"#)]
fn header_is(w: &mut ApiWorld, name: String, value: String) {
    let actual = w.reply().headers.get(&name).map(|v| v.to_str().unwrap());
    assert_eq!(actual, Some(value.as_str()), "標頭 {name}");
}

#[then(regex = r#"^回應沒有 "([^"]+)" 標頭$"#)]
fn header_absent(w: &mut ApiWorld, name: String) {
    assert!(w.reply().headers.get(&name).is_none(), "不應有標頭 {name}");
}

// ---------- 整體內容 ----------

#[then(regex = r"^回應 JSON 為：$")]
fn json_is(w: &mut ApiWorld, step: &Step) {
    assert_eq!(w.json(), literal(&docstring(step)));
}

#[then(regex = r#"^回應內容不含 "(.*)"$"#)]
fn body_lacks(w: &mut ApiWorld, needle: String) {
    let needle = unquote(&needle);
    assert!(!w.body_text().contains(&needle), "回應不應含 {needle}");
}

#[then(regex = r#"^回應內容不含關卡 "([^"]+)" 的 tests\.rs 中任一個測試函式名稱$"#)]
fn body_lacks_test_names(w: &mut ApiWorld, id: String) {
    let body = w.body_text();
    let names = &level(&id).test_names;
    assert!(!names.is_empty(), "{id} 應有測試名稱");
    for name in names {
        assert!(!body.contains(name.as_str()), "回應不應含測試名稱 {name}");
    }
}

#[then(regex = r#"^回應 JSON 任何層級都不含 key "([^"]+)"$"#)]
fn no_key_anywhere(w: &mut ApiWorld, key: String) {
    assert!(!find_key(&w.json(), &key), "不應出現 key {key}");
}

#[then(regex = r"^回應 JSON 不含 key (.+)$")]
fn top_level_lacks(w: &mut ApiWorld, keys: String) {
    let v = w.json();
    for key in keys.split('、') {
        let key = key.trim().trim_matches('"');
        assert!(v.get(key).is_none(), "頂層不應有 key {key}：{v}");
    }
}

#[then(regex = r#"^回應 JSON 的 key 恰好為 "([^"]*)"$"#)]
fn top_keys_exact(w: &mut ApiWorld, keys: String) {
    let mut expected = split_list(&keys);
    expected.sort();
    assert_eq!(sorted_keys(&w.json()), expected);
}

#[then(regex = r#"^回應 JSON 含字串欄位 "([^"]+)" 與 "([^"]+)"$"#)]
fn has_string_fields(w: &mut ApiWorld, a: String, b: String) {
    let v = w.json();
    for key in [a, b] {
        assert!(v[&key].is_string(), "{key} 應為字串：{v}");
    }
}

// ---------- 指定路徑 ----------

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 為 (.+)$"#)]
fn path_is(w: &mut ApiWorld, path: String, value: String) {
    let v = w.json();
    assert_eq!(at(&v, &path), &literal(&value), "{path}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 為：$"#)]
fn path_is_doc(w: &mut ApiWorld, path: String, step: &Step) {
    let v = w.json();
    assert_eq!(at(&v, &path), &literal(&docstring(step)), "{path}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 為空陣列$"#)]
fn path_is_empty_array(w: &mut ApiWorld, path: String) {
    let v = w.json();
    assert!(array(at(&v, &path)).is_empty(), "{path} 應為空陣列");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 長度為 (\d+)$"#)]
fn path_len(w: &mut ApiWorld, path: String, len: usize) {
    let v = w.json();
    assert_eq!(array(at(&v, &path)).len(), len, "{path}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 長度等於關卡 "([^"]+)" 的 tests\.rs 測試數$"#)]
fn path_len_is_test_count(w: &mut ApiWorld, path: String, id: String) {
    let v = w.json();
    assert_eq!(array(at(&v, &path)).len(), level(&id).test_names.len());
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 是長度 (\d+) 的字串陣列$"#)]
fn path_string_array(w: &mut ApiWorld, path: String, len: usize) {
    let v = w.json();
    let items = array(at(&v, &path));
    assert_eq!(items.len(), len, "{path}");
    assert!(items.iter().all(Value::is_string), "{path} 應全為字串");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 等於關卡 "([^"]+)" 的 (\w+)\.rs 全文$"#)]
fn path_equals_file(w: &mut ApiWorld, path: String, id: String, file: String) {
    let expected = fs::read_to_string(content_dir().join(format!("levels/{id}/{file}.rs")))
        .expect("讀關卡檔案");
    let v = w.json();
    assert_eq!(text(at(&v, &path)), expected);
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 的 key 恰好為 "([^"]*)"$"#)]
fn path_keys_exact(w: &mut ApiWorld, path: String, keys: String) {
    let v = w.json();
    let mut expected = split_list(&keys);
    expected.sort();
    assert_eq!(sorted_keys(at(&v, &path)), expected, "{path}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 每一筆的 key 恰好為 "([^"]*)"$"#)]
fn each_keys_exact(w: &mut ApiWorld, path: String, keys: String) {
    let v = w.json();
    let mut expected = split_list(&keys);
    expected.sort();
    for item in array(at(&v, &path)) {
        assert_eq!(sorted_keys(item), expected, "{item}");
    }
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 每一筆都沒有 key "([^"]+)"$"#)]
fn each_lacks_key(w: &mut ApiWorld, path: String, key: String) {
    let v = w.json();
    for item in array(at(&v, &path)) {
        assert!(item.get(&key).is_none(), "不應有 {key}：{item}");
    }
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 每一筆的 (".+)$"#)]
fn each_field_is(w: &mut ApiWorld, path: String, conditions: String) {
    let v = w.json();
    let items = array(at(&v, &path));
    assert!(!items.is_empty(), "{path} 不應為空");
    for (key, expected) in pairs(&conditions, "都是") {
        for item in items {
            assert_eq!(item[&key], expected, "{item}");
        }
    }
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 依序為：$"#)]
fn path_rows_in_order(w: &mut ApiWorld, path: String, step: &Step) {
    let v = w.json();
    assert_rows_in_order(array(at(&v, &path)), step);
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 的 name 依序為 "([^"]*)"$"#)]
fn names_in_order(w: &mut ApiWorld, path: String, names: String) {
    let v = w.json();
    let actual: Vec<&str> = array(at(&v, &path))
        .iter()
        .map(|t| text(&t["name"]))
        .collect();
    assert_eq!(actual, split_list(&names));
}

// ---------- 字串內容 ----------

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 包含 "(.*)"$"#)]
fn path_contains(w: &mut ApiWorld, path: String, needle: String) {
    let v = w.json();
    let s = text(at(&v, &path));
    assert!(s.contains(&unquote(&needle)), "{path} 應含 {needle}：{s}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 不含 "(.*)"$"#)]
fn path_lacks(w: &mut ApiWorld, path: String, needle: String) {
    let v = w.json();
    let s = text(at(&v, &path));
    assert!(!s.contains(&unquote(&needle)), "{path} 不應含 {needle}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 以 "(.*)" 開頭$"#)]
fn path_starts_with(w: &mut ApiWorld, path: String, prefix: String) {
    let v = w.json();
    let s = text(at(&v, &path));
    assert!(
        s.starts_with(&unquote(&prefix)),
        "{path} 應以 {prefix} 開頭"
    );
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 的 UTF-8 位元組長度為 (\d+)$"#)]
fn path_byte_len(w: &mut ApiWorld, path: String, len: usize) {
    let v = w.json();
    assert_eq!(text(at(&v, &path)).len(), len, "{path}");
}

#[then(regex = r#"^回應 JSON 的 "([^"]+)" 由 (\d+) 個 "(.*)" 組成$"#)]
fn path_repeats(w: &mut ApiWorld, path: String, n: usize, unit: String) {
    let v = w.json();
    assert!(
        text(at(&v, &path)) == unit.repeat(n),
        "{path} 應為 {n} 個 {unit}"
    );
}

// ---------- 陣列項目（diagnostics、tests）----------

#[then(regex = r#"^"([^"]+)" 含一筆：$"#)]
fn contains_row(w: &mut ApiWorld, path: String, step: &Step) {
    let v = w.json();
    w.picked = Some(assert_contains_rows(array(at(&v, &path)), step));
}

#[then(regex = r#"^"([^"]+)" 含一筆 (.+) 的診斷$"#)]
fn contains_matching(w: &mut ApiWorld, path: String, conditions: String) {
    let v = w.json();
    let conds = pairs(&conditions, "為");
    let found = array(at(&v, &path))
        .iter()
        .find(|d| conds.iter().all(|(k, e)| &d[k] == e))
        .unwrap_or_else(|| panic!("{path} 找不到 {conds:?}：{v}"));
    w.picked = Some(found.clone());
}

#[then(regex = r#"^該筆診斷的 "([^"]+)" 以 "(.*)" 開頭$"#)]
fn picked_starts_with(w: &mut ApiWorld, key: String, prefix: String) {
    let d = w.picked.as_ref().expect("先用「含一筆」選診斷");
    let s = text(&d[&key]);
    assert!(
        s.starts_with(&unquote(&prefix)),
        "{key} 應以 {prefix} 開頭：{s}"
    );
}

#[then(regex = r#"^該筆診斷的 "([^"]+)" 包含 "(.*)"$"#)]
fn picked_contains(w: &mut ApiWorld, key: String, needle: String) {
    let d = w.picked.as_ref().expect("先用「含一筆」選診斷");
    let s = text(&d[&key]);
    assert!(s.contains(&unquote(&needle)), "{key} 應含 {needle}：{s}");
}

#[then(regex = r#"^"([^"]+)" 依序為：$"#)]
fn rows_in_order(w: &mut ApiWorld, path: String, step: &Step) {
    let v = w.json();
    assert_rows_in_order(array(at(&v, &path)), step);
}

#[then(regex = r#"^"([^"]+)" 共 (\d+) 筆，每一筆都是：$"#)]
fn all_rows_are(w: &mut ApiWorld, path: String, n: usize, step: &Step) {
    let v = w.json();
    let items = array(at(&v, &path));
    assert_eq!(items.len(), n, "{path}：{v}");
    let (header, rows) = table(step);
    for item in items {
        assert!(row_matches(item, &header, &rows[0]), "不符：{item}");
    }
}

#[then(regex = r#"^"([^"]+)" 第一筆的 "([^"]+)" 為 (.+)$"#)]
fn first_field_is(w: &mut ApiWorld, path: String, key: String, value: String) {
    let v = w.json();
    let first = array(at(&v, &path)).first().expect("至少一筆").clone();
    assert_eq!(first[&key], literal(&value), "{first}");
}

#[then(regex = r#"^"([^"]+)" 第 (\d+) 筆的 "([^"]+)" 以 "(.*)" 開頭$"#)]
fn nth_starts_with(w: &mut ApiWorld, path: String, n: usize, key: String, prefix: String) {
    let v = w.json();
    let item = &array(at(&v, &path))[n - 1];
    assert!(
        text(&item[&key]).starts_with(&unquote(&prefix)),
        "{key} 應以 {prefix} 開頭：{item}"
    );
}

#[then(regex = r#"^"([^"]+)" 沒有任何一筆的 "([^"]+)" 以 "(.*)" 開頭$"#)]
fn none_starts_with(w: &mut ApiWorld, path: String, key: String, prefix: String) {
    let v = w.json();
    let prefix = unquote(&prefix);
    for item in array(at(&v, &path)) {
        assert!(!text(&item[&key]).starts_with(&prefix), "{item}");
    }
}

#[then(regex = r#"^"([^"]+)" 沒有任何一筆的 "([^"]+)" 包含 "(.*)" 或 "(.*)"$"#)]
fn none_contains_either(w: &mut ApiWorld, path: String, key: String, a: String, b: String) {
    let v = w.json();
    for item in array(at(&v, &path)) {
        let s = text(&item[&key]);
        assert!(
            !s.contains(&unquote(&a)) && !s.contains(&unquote(&b)),
            "{item}"
        );
    }
}

#[then(regex = r#"^"([^"]+)" 每一筆的 "([^"]+)" 都屬於 "([^"]*)"$"#)]
fn each_in_set(w: &mut ApiWorld, path: String, key: String, set: String) {
    let v = w.json();
    let allowed = split_list(&set);
    for item in array(at(&v, &path)) {
        let s = text(&item[&key]).to_string();
        assert!(allowed.contains(&s), "{key} = {s} 不在 {allowed:?}");
    }
}

#[then(regex = r#"^"([^"]+)" 每一筆的 "([^"]+)" 都不含 "(.*)"$"#)]
fn each_lacks_text(w: &mut ApiWorld, path: String, key: String, needle: String) {
    let v = w.json();
    let needle = unquote(&needle);
    for item in array(at(&v, &path)) {
        if let Some(s) = item[&key].as_str() {
            assert!(!s.contains(&needle), "{key} 不應含 {needle}：{s}");
        }
    }
}

#[then(regex = r#"^"([^"]+)" 中 "([^"]+)" 的 "([^"]+)" 包含 "(.*)"$"#)]
fn named_item_contains(w: &mut ApiWorld, path: String, name: String, key: String, needle: String) {
    let v = w.json();
    let item = array(at(&v, &path))
        .iter()
        .find(|t| t["name"] == name.as_str())
        .unwrap_or_else(|| panic!("找不到 {name}"));
    let s = text(&item[&key]);
    assert!(s.contains(&unquote(&needle)), "{key} 應含 {needle}：{s}");
}

// ---------- Docker 狀態 ----------

#[then(regex = r#"^Docker 本機映像清單中沒有 "([^"]+)"$"#)]
fn image_absent(_w: &mut ApiWorld, image: String) {
    let status = Command::new("docker")
        .args(["image", "inspect", &image])
        .output()
        .expect("執行 docker image inspect")
        .status;
    assert!(!status.success(), "不應存在映像 {image}");
}

/// 容器在背景移除，等到清空為止
#[then(regex = r#"^Docker 中沒有帶 label "([^"]+)" 的容器$"#)]
async fn no_labelled_containers(_w: &mut ApiWorld, label: String) {
    assert_eq!(label, rust_quest_backend::judge::docker::JUDGE_LABEL);
    wait_until(|| judge_containers().is_empty(), "仍有判題容器").await;
}
