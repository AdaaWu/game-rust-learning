#![allow(dead_code)]
//! 測試共用：fixture 路徑、複製關卡、改寫檔案

pub mod timeline;

use std::fs;
use std::path::{Path, PathBuf};

use rust_quest_backend::content::LevelStore;
use rust_quest_backend::content::loader::load;
use tempfile::TempDir;

pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn fixture_content() -> PathBuf {
    fixtures_dir().join("content")
}

pub fn read_fixture(rel: &str) -> String {
    let path = fixtures_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("讀不到 fixture {}: {e}", path.display()))
}

pub fn load_fixture_store() -> LevelStore {
    load(&fixture_content()).expect("fixture content 應可載入")
}

pub fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// 把 fixture content 複製到暫存目錄，供各測試改壞
pub struct ContentDir {
    pub dir: TempDir,
}

impl ContentDir {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        copy_dir(&fixture_content(), dir.path());
        Self { dir }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    pub fn write(&self, rel: &str, content: &str) {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path(rel)).unwrap()
    }

    pub fn remove(&self, rel: &str) {
        let p = self.path(rel);
        if p.is_dir() {
            fs::remove_dir_all(p).unwrap();
        } else {
            fs::remove_file(p).unwrap();
        }
    }

    /// 取代檔案中的一段文字，找不到原文就讓測試失敗
    pub fn replace(&self, rel: &str, from: &str, to: &str) {
        let s = self.read(rel);
        assert!(s.contains(from), "fixture {rel} 找不到要取代的文字：{from}");
        self.write(rel, &s.replacen(from, to, 1));
    }

    /// 在 level.toml 頂層欄位區塊（第一個 table 之前）插入一行
    pub fn insert_top_level(&self, level_id: &str, line: &str) {
        let rel = format!("levels/{level_id}/level.toml");
        let s = self.read(&rel);
        let idx = s.find("\n[").map(|i| i + 1).unwrap_or(s.len());
        let mut out = String::new();
        out.push_str(&s[..idx]);
        out.push_str(line);
        out.push('\n');
        out.push_str(&s[idx..]);
        self.write(&rel, &out);
    }

    /// 以 fixture 的 01-walk-grid 為底，複製出新關卡目錄並改 id、order
    pub fn clone_level(&self, from_id: &str, new_dir: &str, new_id: &str, new_order: u32) {
        copy_dir(
            &self.path(&format!("levels/{from_id}")),
            &self.path(&format!("levels/{new_dir}")),
        );
        let rel = format!("levels/{new_dir}/level.toml");
        let s = self.read(&rel);
        let s = s.replacen(
            &format!("id = \"{from_id}\""),
            &format!("id = \"{new_id}\""),
            1,
        );
        let s = replace_order(&s, new_order);
        self.write(&rel, &s);
    }

    pub fn load(&self) -> Result<LevelStore, rust_quest_backend::content::loader::LoadError> {
        load(self.root())
    }

    /// 斷言載入失敗，且錯誤訊息含 needle（檔案路徑或關卡目錄名）
    pub fn assert_load_fails(&self, needle: &str) {
        let result = self.load();
        assert!(result.is_err(), "預期載入失敗（{needle}），實際成功");
        let msg = result.err().unwrap().to_string();
        assert!(
            msg.contains(needle),
            "錯誤訊息應含「{needle}」，實際：{msg}"
        );
    }
}

fn replace_order(s: &str, new_order: u32) -> String {
    s.lines()
        .map(|l| {
            if l.starts_with("order = ") {
                format!("order = {new_order}")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
