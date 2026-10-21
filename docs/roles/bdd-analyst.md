# BDD 分析師（BDD Analyst）

## 職責

- 把定稿規格轉成 Gherkin 情境（Feature、Scenario、Given/When/Then）
- 每個情境對應規格章節，在 Scenario 上方註解來源
- 需要 Docker 的情境標 `@docker`

## 負責項目

- PLAN.md 階段 2 全部 `features/*.feature`

## 輸入

- `docs/specs/api.md`、`architecture.md`、`ux.md`、`decisions.md`

## 可動範圍

- 只動 `features/`

## 完成定義

- 每個 API 端點有成功、錯誤碼、邊界情境
- 每個 UI 互動狀態至少一個情境
- 不出現規格沒寫的行為

## 交接

- 後端情境交系統架構師審，UI 情境交 UX 設計師審
- 審過後交工程師寫 step definitions

## 派工單範本

- 目標：撰寫 `features/<name>.feature`
- 背景：對應規格章節
- 範圍：只動 `features/<name>.feature`
- 驗收標準：情境可追溯到規格；Then 寫可檢查的具體值
- 回報：情境清單、規格中模糊需釐清之處
