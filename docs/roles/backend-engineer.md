# 後端工程師（Backend Engineer）

## 職責

- Axum 後端與判題器，依 api.md 與 architecture.md 實作
- 每個模組拆兩棒：測試＋紅燈、實作＋綠燈

## 負責項目

- PLAN.md 階段 3a：B1–B7

## 輸入

- `docs/specs/api.md`、`architecture.md`、`decisions.md`
- `features/levels.feature`、`run.feature`、`submit.feature`、`judge_busy.feature`

## 可動範圍

- `backend/`（含 `backend/tests/`）
- 不動 `frontend/`、`docs/`、`features/`、`content/`

## 規則

- 純邏輯測試不依賴 Docker；需要 Docker 的標 `#[ignore]`
- 關卡內容啟動時一次載入，請求路徑不讀檔
- docker 參數用 `Vec<String>` 傳入，不經 shell
- 暫存目錄 755、檔案 644
- 完成前跑 `cargo fmt --check`、`cargo clippy -- -D warnings`

## 第一棒：測試＋紅燈

- 目標：為 Bn 寫測試並跑出紅燈
- 背景：對應規格章節與已定決策
- 範圍：`backend/` 下的測試檔與 stub
- 驗收標準：紅燈原因為斷言不符；測試涵蓋成功、錯誤碼、業務規則、邊界
- 回報：測試案例清單、紅燈摘要

## 第二棒：實作＋綠燈

- 目標：實作 Bn 讓測試綠燈
- 背景：第一棒測試檔路徑
- 範圍：`backend/src/`，不改測試
- 驗收標準：`cargo test` 綠燈、fmt 與 clippy 通過、無測試沒要求的行為
- 回報：異動檔案、需求外建議
