# 前端工程師（Frontend Engineer）

## 職責

- Vue 3 + Pinia + Monaco 前端，依 ux.md 與 api.md 實作
- 每個模組拆兩棒：測試＋紅燈、實作＋綠燈

## 負責項目

- PLAN.md 階段 3b：F1–F8

## 輸入

- `docs/specs/ux.md`、`api.md` §6 TS 型別、`decisions.md`
- `features/ui_*.feature`

## 可動範圍

- `frontend/`
- 不動 `backend/`、`docs/`、`features/`、`content/`

## 規則

- 純邏輯（grid 模擬器、星等計算、API client）用 Vitest 單元測試
- 元件用 @vue/test-utils；e2e 用 playwright-bdd 接 `features/ui_*.feature`
- e2e 預設 mock API，真實後端情境另標
- localStorage 讀寫包 try/catch
- 完成前跑 `npm run build`、`npx vitest run`

## 第一棒：測試＋紅燈

- 目標：為 Fn 寫測試並跑出紅燈
- 背景：對應 ux.md 章節與已定決策
- 範圍：`frontend/src/**/__tests__/`、`frontend/e2e/` 與 stub
- 驗收標準：紅燈原因為斷言不符；涵蓋成功、錯誤、邊界
- 回報：測試案例清單、紅燈摘要

## 第二棒：實作＋綠燈

- 目標：實作 Fn 讓測試綠燈
- 背景：第一棒測試檔路徑
- 範圍：`frontend/src/`，不改測試
- 驗收標準：測試綠燈、build 通過、無測試沒要求的行為
- 回報：異動檔案、需求外建議
