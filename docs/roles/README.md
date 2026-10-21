# 角色分工主檔

> 進度看 `docs/PLAN.md`，本檔定義「誰做哪一項、怎麼交接」
> 派工時把對應角色檔的「派工單範本」填好交給 subagent

## 角色清單

- **主控（Orchestrator）**：`orchestrator.md`，與使用者討論、拆工、派工、更新 PLAN.md
- **課程設計師（Curriculum Designer）**：`curriculum-designer.md`，學習路徑與關卡設計
- **系統架構師（Architect）**：`architect.md`，架構、API、判題器規格與規格一致性
- **UX 設計師（UX Designer）**：`ux-designer.md`，介面、互動狀態、可視化規格
- **BDD 分析師（BDD Analyst）**：`bdd-analyst.md`，把規格轉成 Gherkin 情境
- **後端工程師（Backend Engineer）**：`backend-engineer.md`，Axum 與判題器 TDD
- **前端工程師（Frontend Engineer）**：`frontend-engineer.md`，Vue 元件與 e2e TDD
- **關卡作者（Content Author）**：`content-author.md`，把關卡設計落成可判題的檔案
- **DevOps**：`devops.md`，工具鏈、runner 映像、啟動腳本
- **驗收員（Verifier）**：`verifier.md`，獨立檢查成果與範圍紀律

## PLAN.md 項目負責人

- 階段 0 環境：DevOps
- 階段 1 SDD
  - curriculum.md：課程設計師
  - architecture.md、api.md、交叉核對、decisions.md：系統架構師
  - ux.md：UX 設計師
  - 規格一致性驗收：驗收員
- 階段 2 BDD：BDD 分析師；後端情境由系統架構師審、UI 情境由 UX 設計師審
- 階段 3a B1–B7：後端工程師
- 階段 3b F1–F8：前端工程師
- 階段 4 關卡內容：關卡作者；峰值記憶體量測由 DevOps 協助
- 階段 5 驗收：驗收員
- 階段 6 提交：主控產切分清單，使用者確認後執行

## 交接流程

```
① 課程／架構／UX 規格
  ↓
② 規格交叉核對
  ↓
③ BDD 情境
  ↓
④ 測試＋紅燈
  ↓
⑤ 使用者看測試
  ↓
⑥ 實作＋綠燈
  ↓
⑦ 驗收
```

- ① 三個設計角色可平行，產出放 `docs/specs/`
- ② 系統架構師統一欄位，決策寫入 `docs/specs/decisions.md` 與 PLAN.md「已定決策」
- ③ BDD 分析師只依定稿規格寫，不自行新增需求
- ④ 工程師第一棒：只寫測試與 stub，紅燈原因須為斷言不符
- ⑤ 主控把測試檔與紅燈結果給使用者看
- ⑥ 工程師第二棒：只實作到綠燈，不加測試沒要求的行為
- ⑦ 驗收員獨立檢查，異動 3 個檔案以下的小改免驗收

## 共通規則（每張派工單都要附）

- 先讀 `docs/PLAN.md`「已定決策」，不可推翻；有疑慮寫進回報
- 需求沒寫的欄位、參數、設定項不加，列在回報的「需求外建議」
- 只動派工單列出的檔案；多個 agent 可能改同檔時改為依序
- 不 commit、不 push（經主控指派提交時，訊息用 `type(scope) 中文摘要`（不加冒號），例 `feat(backend) 判題支援 timeline`）
- Rust 指令前先 `source $HOME/.cargo/env`
- 回報：結論、異動檔案、未解問題、需求外建議；不貼原始輸出
- 文件用繁體中文（台灣用語）；流程用垂直箭頭鏈；預設不用表格

## 模型配置

- 主控：opus、effort high
- 設計、工程、關卡、DevOps、驗收：opus、effort medium
- 純調查（搜尋程式碼、查文件）：sonnet 或 Explore
