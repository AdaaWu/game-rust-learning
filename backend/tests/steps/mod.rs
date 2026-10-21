//! cucumber World 與共用輔助

mod given;
mod request;
mod response;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request};
use http_body_util::BodyExt;
use rust_quest_backend::content::LevelStore;
use rust_quest_backend::content::loader::load;
use rust_quest_backend::content::model::Level;
use rust_quest_backend::judge::{Judge, JudgeConfig};
use rust_quest_backend::{AppState, app};
use serde_json::Value;
use tokio::task::JoinHandle;
use tower::ServiceExt;

pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../content")
}

/// 正式關卡內容，整個測試行程只載入一次
fn store() -> Arc<LevelStore> {
    static STORE: OnceLock<Arc<LevelStore>> = OnceLock::new();
    STORE
        .get_or_init(|| Arc::new(load(&content_dir()).expect("content/ 應可載入")))
        .clone()
}

pub fn level(id: &str) -> Arc<Level> {
    store()
        .get(id)
        .unwrap_or_else(|| panic!("找不到關卡 {id}"))
        .clone()
}

pub struct Reply {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(cucumber::World)]
#[world(init = Self::new)]
pub struct ApiWorld {
    config: JudgeConfig,
    app: Option<Router>,
    reply: Option<Reply>,
    /// 「含一筆」選中的診斷，供「該筆診斷」使用
    picked: Option<Value>,
    background: Vec<JoinHandle<()>>,
}

impl fmt::Debug for ApiWorld {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reply = self.reply.as_ref().map(|r| {
            let body = String::from_utf8_lossy(&r.body);
            let body: String = body.chars().take(2000).collect();
            format!("{} {body}", r.status)
        });
        f.debug_struct("ApiWorld")
            .field("config", &self.config)
            .field("reply", &reply)
            .finish()
    }
}

impl ApiWorld {
    fn new() -> Self {
        Self {
            config: JudgeConfig::default(),
            app: None,
            reply: None,
            picked: None,
            background: Vec::new(),
        }
    }

    /// 判題設定只能在第一次送出請求前修改
    fn config_mut(&mut self) -> &mut JudgeConfig {
        assert!(self.app.is_none(), "判題設定須在送出請求前設定");
        &mut self.config
    }

    fn app(&mut self) -> Router {
        let config = &self.config;
        self.app
            .get_or_insert_with(|| {
                app(AppState {
                    levels: store(),
                    judge: Arc::new(Judge::new(config.clone())),
                })
            })
            .clone()
    }

    async fn send(&mut self, req: Request<Body>) {
        let resp = self.app().oneshot(req).await.expect("oneshot");
        let status = resp.status().as_u16();
        let headers = resp.headers().clone();
        let body = resp
            .into_body()
            .collect()
            .await
            .expect("讀 body")
            .to_bytes()
            .to_vec();
        self.reply = Some(Reply {
            status,
            headers,
            body,
        });
        self.picked = None;
    }

    /// 背景送出請求，情境結束時等它完成
    fn send_background(&mut self, req: Request<Body>) {
        let app = self.app();
        self.background.push(tokio::spawn(async move {
            let _ = app.oneshot(req).await;
        }));
    }

    pub async fn join_background(&mut self) {
        for handle in self.background.drain(..) {
            let _ = handle.await;
        }
    }

    fn reply(&self) -> &Reply {
        self.reply.as_ref().expect("尚未送出請求")
    }

    fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.reply().body).into_owned()
    }

    fn json(&self) -> Value {
        serde_json::from_slice(&self.reply().body)
            .unwrap_or_else(|e| panic!("回應不是 JSON（{e}）：{}", self.body_text()))
    }
}

/// 依 `a.b[0].c` 路徑取值，路徑不存在就失敗
pub fn at<'a>(root: &'a Value, path: &str) -> &'a Value {
    let mut cur = root;
    for seg in path.split('.') {
        let (key, rest) = seg.split_once('[').map_or((seg, ""), |(k, r)| (k, r));
        if !key.is_empty() {
            cur = cur
                .get(key)
                .unwrap_or_else(|| panic!("JSON 沒有 `{path}`（缺 `{key}`）：{root}"));
        }
        for idx in rest.split('[').filter(|s| !s.is_empty()) {
            let i: usize = idx.trim_end_matches(']').parse().expect("索引");
            cur = cur
                .get(i)
                .unwrap_or_else(|| panic!("JSON 沒有 `{path}`（索引 {i}）：{root}"));
        }
    }
    cur
}

/// 步驟中的雙引號字串依 JSON 跳脫規則還原（`\n` 為換行）
pub fn unquote(s: &str) -> String {
    serde_json::from_str(&format!("\"{s}\"")).unwrap_or_else(|_| s.to_string())
}

/// 步驟中的值：JSON 字面值（"字串"、null、true、數字）
pub fn literal(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|e| panic!("無法解析值 {s}：{e}"))
}

/// 表格儲存格與 JSON 值比對：null、布林、數字依型別，其餘比字串
pub fn cell_matches(actual: &Value, cell: &str) -> bool {
    match actual {
        Value::Null => cell == "null",
        Value::Bool(b) => cell == b.to_string(),
        Value::Number(n) => cell.parse::<f64>().ok() == n.as_f64(),
        Value::String(s) => s == cell,
        _ => false,
    }
}

/// 以表頭為欄位，比對一筆資料的每一欄
pub fn row_matches(item: &Value, header: &[String], row: &[String]) -> bool {
    header
        .iter()
        .zip(row)
        .all(|(k, cell)| item.get(k).is_some_and(|v| cell_matches(v, cell)))
}

/// 解析 `"k" 為 v、"k2" 為 v2` 這類成對條件
pub fn pairs(text: &str, sep: &str) -> Vec<(String, Value)> {
    text.split('、')
        .map(|part| {
            let part = part.trim();
            let rest = part.strip_prefix('"').expect("條件以 \"key\" 開頭");
            let (key, value) = rest.split_once('"').expect("key 結尾引號");
            let value = value
                .trim()
                .strip_prefix(sep)
                .unwrap_or_else(|| panic!("條件缺「{sep}」：{part}"));
            (key.to_string(), literal(value.trim()))
        })
        .collect()
}

pub fn split_list(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().to_string()).collect()
}

pub fn sorted_keys(v: &Value) -> Vec<String> {
    let mut keys: Vec<String> = v
        .as_object()
        .unwrap_or_else(|| panic!("不是物件：{v}"))
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

/// docstring 去掉前後空行
pub fn docstring(step: &cucumber::gherkin::Step) -> String {
    step.docstring
        .as_deref()
        .expect("步驟需要 docstring")
        .trim_matches('\n')
        .to_string()
}
