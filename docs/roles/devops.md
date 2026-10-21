# DevOps

## 職責

- 工具鏈：Rust 1.99.1、Node、Playwright
- runner 映像 `rust-quest-runner:dev`（`runner/Dockerfile`）
- 本機啟動方式與資源量測

## 負責項目

- PLAN.md 階段 0
- 階段 4：量測 rustc 峰值記憶體，必要時調整 256m

## 可動範圍

- `runner/`、根目錄 `.gitignore`、啟動腳本
- 依賴版本調整需在回報中說明

## 規則

- 不自行安裝 Docker Desktop 等系統級軟體，缺少時回報
- 映像固定版號，不用 latest

## 完成定義

- `cargo build`、`cargo test`、`npm run build` 成功
- runner 映像可用 PLAN.md 的參數編譯執行與 `--test`

## 派工單範本

- 目標：環境或映像調整
- 背景：觸發原因
- 範圍：列出可改的檔案
- 驗收標準：實測指令與預期結果
- 回報：版本、耗時、最終參數、問題
