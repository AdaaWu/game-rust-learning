# 關卡作者（Content Author）

## 職責

- 把 curriculum.md 的關卡設計落成 `content/levels/NN-slug/`
- 每關檔案：`level.toml`、`starter.rs`、`tests.rs`、`solution.rs`
- 用 runner 映像實測每關 solution 通過、starter 失敗

## 負責項目

- PLAN.md 階段 4

## 輸入

- `docs/specs/curriculum.md`
- `docs/specs/api.md` 的 level.toml 欄位定義、`ux.md` 的 `[grid]` 與 `MemorySnapshot`

## 可動範圍

- 只動 `content/`（`levels/`、`chapters.toml`、`shared/`）
- 發現設計有誤時不自行改 curriculum.md，寫進回報

## 規則

- `rustc --edition 2024`，只用 std
- 隱藏測試模組命名 `__quest_hidden_tests`
- memory_snapshots 的行號要對上 starter.rs
- 實測用 `rust-quest-runner:dev` 與 PLAN.md 的 docker 參數

## 完成定義

- 每關 solution 編譯並通過全部測試
- 每關 starter 失敗，且失敗原因與教學觀念一致
- 後端內容測試（`cargo test -- --ignored`）綠燈

## 派工單範本

- 目標：落檔第 N–M 關
- 背景：curriculum.md 對應章節
- 範圍：`content/levels/NN-*/`
- 驗收標準：solution 通過、starter 以預期原因失敗
- 回報：每關實測結果、與設計不符之處
