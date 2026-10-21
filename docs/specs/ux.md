# Rust 闖關遊戲 UX 與可視化設計規格

> 狀態：v1.5（§11：結論只看有 start 的泳道、死鎖第一條泳道定義、泳道名稱規則；v1.4 新增 TimelineView §11；決策見 decisions.md §15 ⑬）
> 範圍：前端 UX、版面、互動狀態、可視化元件（GridWorld、MemoryView、TimelineView）、進度規則、可及性、視覺風格
> 技術前提：Vue 3 + Vite + TypeScript + Pinia；編輯器用 Monaco；進度存 localStorage
> 不在本文件範圍：後端 API 細節（見 `api.md`）、關卡內容撰寫（見 `curriculum.md`）、Rust 編譯沙箱（見 `architecture.md`）
> 決策與理由：`docs/specs/decisions.md`

---

## 0. 名詞定義

本節先統一術語，後面直接使用。

- **Level（關卡）**：一份 `level.toml` 定義的題目，含故事、起始程式碼、測試、可視化設定
- **Chapter（章節）**：一組依序排列的 Level，對應一個 Rust 主題；章節名稱來自 `content/chapters.toml`，經 `GET /api/levels` 的 `chapters` 傳給前端（api.md §1）
- **本次遊玩（attempt）**：計算 ★2、★3 用的區間，從上次通關（或第一次進關）開始，到下一次 Submit 通過為止（§6.1）
- **kind**：關卡題型，三選一
  - `build`：從骨架寫出功能
  - `fix-compile`：起始程式碼編譯不過，讀 rustc 錯誤後修好
  - `debug`：能編譯也能跑，但行為錯誤，要找出 bug
- **visual**：右欄可視化類型，四選一：`grid`、`memory`、`timeline`、`none`；一關只有一種
- **泳道（lane）**：TimelineView 裡一條執行緒佔一列，橫向是事件順序（§11）
- **Run（執行）**：編譯並執行玩家程式，顯示 stdout 與可視化，**不判定通關**
- **Submit（送出）**：編譯並跑隱藏測試，**唯一的通關判定入口**
- **Diagnostic（診斷）**：後端解析 rustc 輸出後的單筆錯誤或警告，格式為 `{code, message, line, column, end_line, end_column, level, explanation_url, rendered}`（定義見 api.md §0.3）
- **JudgeStatus（判題狀態）**：Run／Submit 回傳的 `status`，七種：`ok`、`compile_error`、`runtime_panic`、`test_failed`、`timeout`、`memory_limit`、`output_limit`
- **TestResult（測試結果）**：Submit 回傳的單一測試 `{name, status, message}`，`status` 為 `passed`、`failed`、`not_run`
- **Marker（標記）**：Monaco 在編輯器上畫的波浪線，對應一筆 Diagnostic

---

## 1. 頁面與流程

整個產品只有三種畫面：章節地圖、關卡頁、通關結算（關卡頁上的 Modal）。

### 1.1 主流程

```
① 首頁／章節地圖
  ↓
② 選擇關卡
  ↓
③ 關卡頁：Ferris 說故事
  ↓
④ 撰寫程式＋Run 試跑
  ↓
⑤ Submit 送出
  ↓
⑥ 通關動畫
  ↓
⑦ 通關結算
  ↓
⑧ 下一關
```

① **首頁／章節地圖**
  - 進站直接顯示章節地圖，自動捲到「目前所在」節點（§6.4）
② **選擇關卡**
  - 點已解鎖節點進入關卡頁
  - 點鎖定節點：不跳頁，節點旁顯示 tooltip「先完成〈前一關名稱〉」
    - 同章：前一關＝依 `(chapter, order)` 排序的前一關
    - 章節第 1 關（跨章鎖定）：前一關＝前一個章節中第一個未通過（沒有 ★1）的關卡
③ **關卡頁：Ferris 說故事**
  - 左欄頂部 Ferris 對話泡泡顯示該關 `story`（見 §1.4）
  - 依 kind 顯示一行任務橫幅（見 §1.3）
④ **撰寫程式＋Run 試跑**
  - Run 可無限次，不影響星等
⑤ **Submit 送出**
  - 失敗：留在關卡頁，主控台依結果狀態切分頁（見 §3.3）
  - 成功：進入 ⑥
⑥ **通關動畫**
  - Ferris 跳舞＋彩帶，約 1.5 秒
  - 點任意處或按 `Esc` 可跳過
  - 系統設定 `prefers-reduced-motion: reduce` 時直接略過，進 ⑦
⑦ **通關結算**（Modal）
  - 逐顆亮星，每顆星旁寫達成條件與是否達成（見 §6）
  - 未達成的星寫一句怎麼拿到（例：「不看提示再挑戰一次就能拿到」）
  - 按鈕：「下一關」（主要）、「再玩一次」、「回地圖」
⑧ **下一關**
  - 依 `(chapter, order)` 排序的下一關已解鎖：直接進該關卡頁
  - 沒有下一關，或下一關仍鎖定（例：章節最後一關，但同章還有關卡沒過）：回章節地圖
  - 章節完成不另做過場動畫與徽章

### 1.2 路由

- `/`：章節地圖
- `/level/:levelId`：關卡頁
- 通關結算是關卡頁上的 Modal，不另開路由
- 直接輸入鎖定關卡的 URL：導回章節地圖，並 toast「這關還沒解鎖」
- 直接輸入關卡 URL、重新整理、瀏覽器上一頁／下一頁進入關卡：與從地圖進入相同，都**不會**重置本次遊玩計數（§6.1）
- 關卡頁不論從哪裡進入，都先取得 `GET /api/levels`（已快取則不再請求），解鎖判斷與頂部列章節名稱都需要它
- `GET /api/levels/:id` 回 404：顯示「找不到這關」頁，附回地圖連結

### 1.3 依 kind 的關卡頁差異

三種 kind 共用同一版面，只差在任務橫幅與 `debug` 的預期／實際並排。

- `build`：橫幅「任務：完成下面的程式」
- `fix-compile`
  - 橫幅：「這段程式編不過，先按執行看錯誤訊息，再修好它」
  - 關卡開啟時**不自動編譯**（避免每次進關都打後端），由玩家按 Run
- `debug`
  - 橫幅：「程式能跑，但結果不對，找出 bug」
  - 測試失敗時，「預期 vs 實際」並排顯示，差異處反白（拆解規則見 §3.3 test_failed）
- 主控台預設分頁只看結果狀態，不看 kind：`compile_error` 一律切「編譯錯誤」、`test_failed` 一律切「測試結果」（§3.3）

### 1.4 Ferris 對話泡泡

泡泡內容就是關卡的 `story`，不新增 `level.toml` 欄位，也沒有其他對話來源。

- 左欄頂部一個泡泡，顯示 `story` 全文（Markdown 渲染，經 DOMPurify 過濾），不分頁、沒有「下一句」
- 泡泡左側是 Ferris 頭像（一般表情，§8.5）
- 判題結果不改泡泡內容；結果只在主控台與可視化區呈現（§3.3）

---

## 2. 關卡頁版面

桌機三欄、平板兩欄、手機分頁；斷點採 `≥1200px`、`768–1199px`、`<768px`。

### 2.1 桌機（≥1200px）三欄

欄寬比例 `28% / 40% / 32%`，頂部列固定高度 56px。

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ← 地圖 │ 第 3 章 所有權之海 ▸ 3-1 交出去的背包   ☆☆☆      [☀/☾]  [⌨ 快捷鍵] │
├──────────────────────┬────────────────────────────────┬──────────────────────┤
│ ┌──────────────────┐ │ main.rs                        │ ┌ 可視化 ───────────┐ │
│ │ (Ferris) Ferris  │ │ ─────────────────────────────  │ │                   │ │
│ │ 把背包交給海鷗…  │ │  1 fn main() {                 │ │   GridWorld       │ │
│ └──────────────────┘ │  2     let gem = Gem::new();   │ │   或 MemoryView   │ │
│ ▌任務：完成下面的程式 │  3     take(gem);              │ │                   │ │
│                      │  4     println!("{:?}", gem);  │ │                   │ │
│ ## 觀念（concept）   │         ~~~ (波浪線)            │ ├───────────────────┤ │
│ markdown 教學內容…   │  5 }                           │ │ ⏮ ▶/⏸ ⏭  速度 1x ▾ │ │
│                      │                                │ │ 第 3 / 12 步       │ │
│ ```rust              │                                │ └───────────────────┘ │
│ 範例程式             │                                │ ┌ 主控台 ───────────┐ │
│ ```                  │                                │ │[輸出][編譯錯誤 2]  │ │
│                      │                                │ │[測試結果]          │ │
│ ┌ 提示 ────────────┐ │                                │ │                   │ │
│ │ 提示 1：…         │ │ ─────────────────────────────  │ │ > MOVE RIGHT      │ │
│ │ [看下一個提示 2/3]│ │ [↺ 重設]      [▶ 執行] [✓ 送出]│ │ > PICK            │ │
│ └──────────────────┘ │                                │ └───────────────────┘ │
└──────────────────────┴────────────────────────────────┴──────────────────────┘
```

- **頂部列**：返回地圖、「第 N 章 章節名稱 ▸ 關卡編號 關卡名」（章節名稱來自 `chapters`，編號格式見 §6.4）、本關星等（歷史聯集）、深淺色切換、快捷鍵說明按鈕（§7.2）
- **左欄**：可獨立捲動
  - 由上而下：Ferris 對話泡泡（`story`）→ kind 任務橫幅 → `concept`（Markdown）→ 提示區
  - 提示區固定在左欄底部（sticky），不隨內容捲走；`hints` 為空陣列時不顯示提示區
- **中欄**：Monaco 編輯器佔滿高度，底部固定操作列
  - 左側「重設」（次要、危險樣式）
  - 右側「執行」（次要）、「送出」（主要）
- **右欄**：上下兩區，預設 55% / 45%
  - `visual = "none"` 時可視化區隱藏，主控台佔滿右欄
  - 可視化區底部是播放控制列（GridWorld、MemoryView、TimelineView 共用，見 §4.4）
  - 主控台三個分頁：「輸出」「編譯錯誤 (n)」「測試結果」；數字 badge 顯示筆數

### 2.2 平板（768–1199px）兩欄

左欄收成抽屜，中右兩欄並排 `55% / 45%`。

```
┌───────────────────────────────────────────────────────┐
│ ☰ 故事 │ 3-1 交出去的背包       ☆☆☆     [☀/☾]        │
├─────────────────────────────┬─────────────────────────┤
│ main.rs                     │ ┌ 可視化 ─────────────┐ │
│  1 fn main() {              │ │                     │ │
│  2     let gem = Gem::new();│ │                     │ │
│  3     take(gem);           │ │ ⏮ ▶ ⏭  1x ▾         │ │
│  4     println!(...);       │ └─────────────────────┘ │
│        ~~~                  │ ┌ 主控台 ─────────────┐ │
│  5 }                        │ │[輸出][錯誤][測試]   │ │
│                             │ │                     │ │
│ [↺]          [▶ 執行][✓ 送出]│ └─────────────────────┘ │
└─────────────────────────────┴─────────────────────────┘
```

- 「☰ 故事」開啟左側抽屜（寬 360px，覆蓋在編輯器上），內容同桌機左欄
- 第一次進入關卡時抽屜預設展開，關閉後同一關卡不再自動展開
  - 「已自動展開過」只記在記憶體（Pinia，關卡 id 集合），不寫 localStorage；重新整理後重置，該關會再自動展開一次
- 提示按鈕在抽屜內；抽屜關閉時，「☰ 故事」旁顯示未讀提示數 badge

### 2.3 手機（<768px）分頁

三欄改成三個分頁，操作列固定在畫面底部。

```
┌─────────────────────────────┐
│ ←  3-1 交出去的背包  ☆☆☆    │
├─────────────────────────────┤
│ [ 故事 ] [ 程式 ] [ 結果 •] │
├─────────────────────────────┤
│                             │
│   （目前分頁的內容）        │
│                             │
│   故事：左欄全部內容        │
│   程式：Monaco 編輯器       │
│   結果：可視化 + 主控台     │
│         （上下堆疊）        │
│                             │
├─────────────────────────────┤
│ [↺]      [▶ 執行] [✓ 送出]  │
└─────────────────────────────┘
```

- 預設分頁：首次進關為「故事」，之後為「程式」
  - 「首次進關」指本次頁面載入後第一次進入該關；只記在記憶體（同平板抽屜），重新整理後重置
- 按 Run／Submit 後**自動切到「結果」分頁**
- 「結果」分頁有新內容且使用者不在該分頁時，分頁名旁顯示小圓點 `•`
- 主控台在「結果」分頁內仍保留三個子分頁
- Monaco 在手機上關閉 minimap、字級 14px、行號寬度縮小
- 底部操作列高度 56px，按鈕觸控區至少 44×44px

---

## 3. 互動狀態

Run 與 Submit 共用同一套狀態機，差別只在成功後的行為。

### 3.1 狀態機

```
① idle
  ↓
② running／submitting（請求中）
  ↓
③ 結果狀態
  ↓
④ 回到 idle
```

① **idle**：三個按鈕皆可按
② **running**（Run 請求中）／**submitting**（Submit 請求中）：見 §3.2；名稱與 architecture.md §7.2 `phase` 相同
③ **結果狀態**：見 §3.3
  - 判題結果（HTTP 200）：對應 JudgeStatus 七種，`ok` 在 Run 與 Submit 各有一種畫面
    - Run 可能出現：`ok`、`compile_error`、`runtime_panic`、`timeout`、`memory_limit`、`output_limit`
    - Submit 可能出現：`ok`、`compile_error`、`test_failed`、`timeout`、`memory_limit`、`output_limit`
  - 請求失敗（非 200），依 HTTP 狀態與 `error.code` 分四種：
    - `server_busy`：`503 JUDGE_BUSY`
    - `judge_unavailable`：`503 JUDGE_UNAVAILABLE`
    - `payload_too_large`：`413 PAYLOAD_TOO_LARGE`
    - `network_error`：斷線、逾時，以及上面以外的所有 4xx／5xx（含 400、404、500）
④ **回到 idle**：結果畫面保留到下一次 Run／Submit 或玩家修改程式碼（修改後只清除編輯器 marker，主控台內容保留並淡化顯示「程式已修改」）

### 3.2 Loading（請求中）

- 被按下的按鈕：文字換成 spinner＋「判題中…」（後端一次回傳結果，不分段回報進度）
- 另一顆按鈕與「重設」：disabled
- 編輯器：**維持可編輯**（不鎖定）；但若送出期間程式碼被改，結果回來後主控台頂部顯示「這是修改前那份程式碼的結果」
- 主控台：清空舊內容，顯示骨架列（skeleton）
- 可視化區
  - GridWorld：停止播放，回到第 0 步並半透明（`pending` prop，§4.1）
    - Run 回來：依新結果播放（§4.4）
    - Submit 回來：還原到上次 Run 播放結束的那一步，不重播；從未 Run 過則顯示初始地圖
  - TimelineView：規則同 GridWorld（§11.7）
  - MemoryView：不受影響，照常可播放（§5.5）
- 超過 3 秒仍未回應：主控台補一行「第一次編譯比較久，請稍候」
- 同一時間只允許一個請求；loading 中再按快捷鍵不重送
- 前端請求逾時 60 秒：判題最壞約 40 秒（排隊 10＋容器啟動 10＋編譯 15＋執行 5），再留餘裕；逾時即中止請求，歸為 `network_error`

### 3.3 結果狀態

每個狀態都定義三個位置：主控台、編輯器、可視化；按鈕與失敗計數另外註明。

- 「可視化」指 GridWorld；TimelineView 依同一組狀態另有對應畫面（§11.6），下方各狀態的 GridWorld 規則（自動播放、播到中斷點、Submit 不重播）同樣適用；MemoryView 播放手寫快照，不受任何 Run／Submit 結果影響（§5.5）
- 「算一次失敗嘗試」指本次遊玩的 `failedSubmits + 1`（§6.1）

**ok（Run 成功）**
- 主控台：切到「輸出」分頁，顯示 stdout，頂部綠色列「✓ 執行完成」
- 編輯器：清除 error marker；有 warning 時照 compile_error 的規則畫黃色波浪線
- 可視化：GridWorld 自動從第 1 步開始播放
- 提示：Run 成功不等於通關，頂部列下方顯示一行「看起來可以了，按『送出』驗收」

**ok（Submit 通關）**
- 主控台：切到「測試結果」分頁，全部測試列打勾
- 進入通關動畫 → 通關結算（§1.1 ⑥ ⑦）
- 寫入進度（§6）

**compile_error（編譯失敗）**
- 主控台：不論 kind、不論 Run 或 Submit，一律切到「編譯錯誤 (n)」分頁，每筆 Diagnostic 一張卡片（格式見 §3.4）
- 編輯器：每筆有位置的 Diagnostic 畫一條 marker
  - `level = "error"`：紅色波浪線
  - `level = "warning"`：黃色波浪線
  - marker 範圍：`line`／`column` 到 `end_line`／`end_column`；`end_column = null` 時標到 `end_line` 行尾
  - 零寬度範圍（`line = end_line` 且 `column = end_column`，例：缺分號）：擴到該位置所在 token 結尾；沒有 token 時至少標 1 個字元；位於非空行的行尾時改標前一個字元（規則見 architecture.md §7.4）
  - `line = null`：不畫 marker，卡片位置欄依 §3.4 規則顯示「隱藏測試」或「無位置」
  - 滑鼠移到波浪線上：hover 顯示人話（§3.4）＋錯誤碼連結
- 點錯誤卡片：編輯器捲到該行、游標移到該欄、該行背景閃一下；`line = null` 的卡片點了不動作
- 可視化：GridWorld 維持初始地圖並顯示「程式還沒編譯成功」
- 只有 warning、沒有 error 時視為編譯成功，繼續執行；warning 卡片放在「輸出」分頁頂部，可收合
- Submit 時算一次失敗嘗試
- Submit 的 `tests` 全部為 `not_run`，「測試結果」分頁照常列出（灰色「未執行」），但不自動切過去

**test_failed（測試失敗，僅 Submit）**
- 主控台：不論 kind，一律切到「測試結果」分頁
  - 每個測試一列，依 `tests` 陣列順序；分頁標題顯示「通過 x / 共 n」，由 `tests` 計數
  - `passed`：✓ 圖示＋「通過」
  - `failed`：✗ 圖示＋「失敗」，可展開顯示 `message`
  - `not_run`：— 圖示＋「未執行」，灰色；不可展開
  - 第一個失敗測試預設展開
- 失敗列的「預期 vs 實際」
  - `message` 含 `left:`、`right:` 兩行（`assert_eq!` 格式）時，拆成「實際（left）」與「預期（right）」並排，差異處反白
  - 斷言自訂訊息（`failed:` 之後的文字）放在最上面當標題
  - 拆不出來時直接顯示 `message` 原文
- **測試程式中途當掉**（`tests` 沒有任何 `failed`、至少一個 `not_run`，且 `execution.stderr` 非空；例：stack overflow 讓整個測試程式 abort）
  - 「測試結果」分頁頂部加一張紅色卡片「測試程式在執行途中整個當掉了，後面的測試沒有機會跑」，下方等寬字型顯示 `execution.stderr` 原文
  - 不自動展開任何測試列（沒有失敗列可展開）
- 可視化：不重播；維持最後一次 Run 的畫面（Submit 的 stdout 是測試輸出，不是指令）
- 算一次失敗嘗試（影響「一次就過」星，見 §6）

**runtime_panic（執行期 panic，僅 Run）**
- 主控台：「輸出」分頁先顯示 panic 前的 stdout，最後一張紅色卡片「程式在執行途中當掉了（panic）」＋ stderr 原始 panic 訊息
- 若 panic 訊息含 `main.rs:行:欄`，編輯器在該行**整行**畫紅色波浪線（欄位只用於卡片文字）
- 可視化：GridWorld 播放到 panic 前最後一個指令，停在該步並標示「程式在這裡結束」

**timeout（逾時）**
- 主控台：切到「輸出」分頁，顯示橘色卡片（Run 與 Submit 相同；Submit 的測試結果由玩家自行切到「測試結果」分頁查看）
  - `compile.timed_out = true`：「編譯太久被停下來了（超過 15 秒）」
  - `execution.timed_out = true`：「程式跑太久被停下來了（超過 5 秒）」，補一行常見原因：「可能有無窮迴圈或死鎖，檢查 `loop`、`while` 的結束條件與鎖的順序」
  - 有部分 stdout 就照常顯示在卡片上方
- 可視化：GridWorld 用已收到的 stdout 播放，結尾顯示「時間到」
- Submit 時算一次失敗嘗試；已有結果的測試照常顯示，其餘為「未執行」

**memory_limit（記憶體超過上限）**
- 主控台：橘色卡片「程式用太多記憶體被停下來了（超過 256 MB）」，補一行常見原因：「可能有無限遞迴，或一次配置了超大的 `Vec`」
  - 編譯階段 OOM（`execution = null`）時卡片改為「編譯時用太多記憶體被停下來了（超過 256 MB）」，不顯示常見原因
  - 有部分 stdout 就照常顯示在卡片上方
- 可視化：GridWorld 用已收到的 stdout 播放，結尾顯示「記憶體不足」；`execution = null` 時同 compile_error 維持初始地圖
- Submit 時算一次失敗嘗試

**output_limit（輸出超過上限）**
- 主控台：橘色卡片「輸出太多被停下來了（超過 64 KiB）」，補一行常見原因：「可能在無窮迴圈裡 `println!`」
  - 顯示已收到的 stdout；最後一行可能不完整，結尾標「（已截斷）」
- 可視化：GridWorld 丟棄最後一行後播放，結尾顯示「輸出過長已截斷」
- Submit 時算一次失敗嘗試（Submit 實務上極少出現，見 api.md §0.3）

**server_busy（503 `JUDGE_BUSY`）**
- 這是系統問題，不是玩家的錯，**不算失敗嘗試**
- 主控台：灰色卡片「伺服器現在有點忙，等一下再試」
- 按鈕：「執行」與「送出」**都** disabled 並各自顯示倒數（例「執行（2 秒後可再試）」「送出（2 秒後可再試）」）；「重設」不受影響
  - 秒數用 `Retry-After` header，只接受正整數秒（例 `2`）
  - 沒有 header、非整數（例 `1.5`、`abc`）或 HTTP 日期格式都用 10 秒
- 倒數結束後按鈕恢復，**不自動重送**

**judge_unavailable（503 `JUDGE_UNAVAILABLE`）**
- 不算失敗嘗試
- 主控台：灰色卡片「判題服務還沒準備好，關卡內容仍可瀏覽」
- 按鈕不倒數、不 disabled，玩家可自行重試
- 進入關卡頁時 `/api/health` 回 `degraded`：Run／Submit 直接 disabled，旁邊顯示同一句說明；重新整理頁面才會重新檢查

**payload_too_large（413 `PAYLOAD_TOO_LARGE`）**
- 只看 HTTP 413 判定，不讀 response body（body 可能不是 JSON）
- 不算失敗嘗試
- 主控台：灰色卡片「程式碼太長了（上限 64 KiB），刪掉一些再試」

**network_error（斷線、其他 4xx／5xx）**
- 不算失敗嘗試
- 主控台：灰色卡片「連不上伺服器，檢查網路後再試」，不倒數
- 4xx 屬前端程式錯誤，不另設玩家可見的畫面；開發模式下 `console.error` 印出 `error.code` 與 `error.message`

### 3.4 rustc 錯誤「翻譯成人話」

每張錯誤卡片分三層：人話、原文、延伸閱讀。

```
┌─────────────────────────────────────────────────────┐
│ ✗ 第 10 行第 17 欄  E0382                           │
│                                                     │
│ `bag` 已經交給別人了，這裡不能再用它。              │
│ 想一想：要先用完再交出去，還是交一份複製品？        │
│                                                     │
│ ▸ 顯示 rustc 原文                                   │
│ 📖 E0382 官方說明 ↗                                 │
└─────────────────────────────────────────────────────┘
```

- **第一行**：圖示（✗ error／⚠ warning）＋位置＋錯誤碼
  - `line` 有值：「第 L 行第 C 欄」
  - `line = null` 且 `rendered = null`：「隱藏測試」（只有 Submit 錯誤落在隱藏區段時兩者同時為 null，api.md §0.3）
  - `line = null` 且 `rendered` 有值：「無位置」（例：連結器錯誤）
- **人話**：前端內建錯誤碼字典 `errorHints`，key 為錯誤碼，value 為模板字串
  - 模板中的 `{name}` 代入 rustc `message` 第一組反引號內的文字（例：`` borrow of moved value: `bag` `` → `bag`）；`message` 沒有反引號時，`{name}` 代入「這個值」，以純文字呈現（不包 `<code>`；含 `{name}` 的反引號片段此時整段以純文字呈現，例 E0384 的「let mut 這個值」）
  - 模板中的反引號片段渲染為行內程式碼（`<code>`），畫面不顯示反引號字元
  - 字典內沒有的錯誤碼（含 `code = null` 與 lint 名）：人話層直接顯示 `message` 原文，不硬翻
  - 新增關卡時，該關設計的錯誤碼一併補進字典
- **rustc 原文**：預設收合，展開後用等寬字型顯示 `rendered`（含程式碼片段與 `help:` 建議）；`rendered = null` 時改顯示 `message`
- **官方說明**：連到 `explanation_url`，新分頁開啟；`explanation_url` 為空時不顯示這行
- 錯誤碼本身也是連結，與「官方說明」指向同一網址
- 卡片排序：error 在前、warning 在後；同級依行號排序，`line = null` 排在同級最後
- `note`、`help` 等級的診斷併入卡片原文，不另開卡片
- 超過 5 筆時只展開前 5 筆，其餘收進「還有 n 筆」

**`errorHints` 首批模板**（MVP 關卡設計的 4 個錯誤碼＋4 個常見錯誤碼）

- `E0384`：「`{name}` 宣告時沒有加 `mut`，之後不能再改它。想改的話，宣告時寫 `let mut {name}`。」
- `E0596`：「`{name}` 不是 `mut`，不能借出可修改的參考（例如呼叫 `push`）。宣告時改成 `let mut {name}`。」
- `E0382`：「`{name}` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？」
- `E0502`：「`{name}` 正被借去讀，同時又要被修改。先讓讀的借用結束（或複製一份出來），再修改。」
- `E0499`：「`{name}` 同時被借出兩個可修改的參考，一次只能有一個 `&mut`。」
- `E0308`：「型別對不上：這裡要的型別和你給的不一樣。看原文裡的 expected 與 found。」
- `E0425`：「找不到 `{name}`：可能拼錯字、還沒宣告，或被刪掉了。」
- `E0599`：「`{name}` 這個方法在這個型別上不存在。檢查拼字，或確認型別是不是你以為的那個。」

---

## 4. GridWorld 元件規格

GridWorld 把玩家程式的 stdout 轉成指令序列，驅動 Ferris 在格子地圖上移動。

### 4.1 元件介面

```ts
// GridWorld.vue
props: {
  grid: GridDef        // 來自 level.toml 的 [grid]
  stdout: string | null // null 表示尚未執行，顯示初始地圖
  status: RunStatus | null // Run 回傳的 status；null 表示尚未執行
  truncated: boolean    // execution.stdout_truncated
  pending: boolean      // Run／Submit 請求中：回第 0 步並半透明（§3.2）
}
// 不對外 emit 事件：播放進度、訊息列都在元件內部處理

type RunStatus = 'ok' | 'compile_error' | 'runtime_panic' | 'timeout' | 'memory_limit' | 'output_limit'
type GridDef = {        // 與 api.md §6 相同
  width: number; height: number
  start: [number, number]; goal: [number, number]
  walls: [number, number][]; gems: [number, number][]; doors: [number, number][]
}
```

- GridWorld **只負責視覺回放**；通關與否以後端 Submit 結果為準
- 前端解析規則必須與判題用的共用模擬器 `content/shared/grid_sim.rs`（curriculum.md §1.5）一致，規則文字以本節為準
- GridWorld 只吃 Run 的結果；Submit 不驅動 GridWorld
- 停止原因（§4.5）：`wall`、`door`、`out_of_bounds`、`invalid`、`pick_empty`、`open_nothing`、`step_limit`；由前端純函式模擬器（指令解析＋模擬）算出，只供元件內部顯示訊息用

### 4.2 地圖資料格式（level.toml）

座標採 `[x, y]`，`x` 為欄（左到右，從 0 起算），`y` 為列（上到下，從 0 起算）。

```toml
visual = "grid"

[grid]
width  = 6
height = 5
start  = [0, 0]
goal   = [5, 4]
walls  = [[1, 0], [1, 1], [1, 2], [3, 2], [3, 3], [4, 3]]
gems   = [[2, 0], [4, 4]]
doors  = [[3, 4]]
```

上面設定渲染結果：

```
    x→ 0   1   2   3   4   5
y↓  ┌───┬───┬───┬───┬───┬───┐
 0  │ 🦀│ ▓ │ 💎│   │   │   │
    ├───┼───┼───┼───┼───┼───┤
 1  │   │ ▓ │   │   │   │   │
    ├───┼───┼───┼───┼───┼───┤
 2  │   │ ▓ │   │ ▓ │   │   │
    ├───┼───┼───┼───┼───┼───┤
 3  │   │   │   │ ▓ │ ▓ │   │
    ├───┼───┼───┼───┼───┼───┤
 4  │   │   │   │ 🚪│ 💎│ 🏁│
    └───┴───┴───┴───┴───┴───┘
```

**欄位說明**
- `width`、`height`：整數，範圍 2–12
- `start`：Ferris 起點，必填
- `goal`：終點，必填
- `walls`：牆的座標陣列，可省略（視為空陣列）
- `gems`：寶石座標陣列，可省略
- `doors`：門的座標陣列，可省略；門初始為關閉，關閉時不能通過，`OPEN` 後永久開啟並可通過

**載入時驗證**（關卡作者寫錯時，開發模式下 console.error 並在可視化區顯示紅框錯誤，正式環境顯示「此關卡地圖設定有誤」）
- 所有座標必須在 `0 ≤ x < width`、`0 ≤ y < height` 內
- `start`、`goal`、牆、寶石、門彼此不可重疊（`start` 與 `goal` 也不可相同）
- 同一陣列內不可有重複座標

**通關條件（視覺上的判定）**
- 指令播放完畢時，Ferris 站在 `goal` 上，且 `gems` 全部被撿起
- 所有關卡一律「全撿＋到終點」，不提供「只要到終點」的設定

### 4.3 指令語法

玩家程式用 `println!` 一行印一個指令。

```
MOVE UP | MOVE DOWN | MOVE LEFT | MOVE RIGHT
PICK
OPEN UP | OPEN DOWN | OPEN LEFT | OPEN RIGHT
```

**解析規則**（逐行處理 stdout）
```
① 去除行首尾空白
  ↓
② 判斷是否為「指令嘗試」
  ↓
③ 驗證指令
  ↓
④ 加入指令序列或標為無效
```

① 去除行首尾空白，空行忽略
② 整行只由大寫英文字母、底線、空白組成（正規式 `^[A-Z_]+( +[A-Z_]+)*$`）就視為指令嘗試；否則視為一般輸出，只顯示在主控台，不影響 Ferris
  - 目的：玩家用 `println!("debug: {}", x)` 印除錯訊息不會被當成錯誤指令
③ 驗證
  - 動詞必須是 `MOVE`、`PICK`、`OPEN` 之一
  - `MOVE`、`OPEN` 必須接一個方向：`UP`、`DOWN`、`LEFT`、`RIGHT`
  - `PICK` 不接參數
  - 多個空白視為一個；**大小寫敏感**（`move up` 不會被當成指令嘗試，視為一般輸出）
④ 合法指令加入序列；不合法的指令嘗試也加入序列，類型標為 `invalid`，播放到它時停下並提示

- **步數上限 1,000 步**：指令序列（含 `invalid`）超過 1,000 步時，只保留前 1,000 步，播完第 1,000 步後停下，停止原因為 `step_limit`
  - 理由：玩家寫錯迴圈可能在 timeout 前印出上萬行指令，產生上萬個動畫步驟會讓分頁卡頓
  - 判題的共用模擬器同樣以 1,000 步為上限，超過視為失敗
- **輸出被截斷**（`truncated = true`）：最後一行可能不完整，解析前先丟棄最後一行

**語意**
- `MOVE <DIR>`：往該方向走一格；`UP` 為 y−1、`DOWN` 為 y+1、`LEFT` 為 x−1、`RIGHT` 為 x+1
- `PICK`：撿起 Ferris 腳下的寶石
- `OPEN <DIR>`：打開該方向相鄰格的門
- 遇到任何錯誤（見 §4.5）**立即停止播放**，後續指令不執行
  - 理由：讓玩家只需專注第一個錯誤；與 rustc「先修第一個錯」的習慣一致

### 4.4 播放控制

GridWorld 與 MemoryView 共用同一個 `<PlaybackBar>` 元件。

```
┌─────────────────────────────────────────────┐
│ [⏮ 從頭] [▶ 播放／⏸ 暫停] [⏭ 單步]  速度 [1x ▾] │
│ ●────────○──────────────  第 3 / 12 步       │
└─────────────────────────────────────────────┘
```

- **從頭**：回到第一個畫面並暫停；GridWorld 是第 0 步（初始地圖），MemoryView 是第 1 步（`memory_snapshots[0]`，沒有第 0 步）
- **播放／暫停**：同一顆按鈕切換；播到最後一步自動變回「播放」，再按則從頭播
- **單步**：前進一步並暫停；播放中按下也會先暫停
- **速度**：`0.5x`、`1x`、`2x`、`4x`
  - `1x` 時每步 500ms（含 300ms 動畫＋200ms 停頓）
  - 速度只存在記憶體（Pinia），切換關卡沿用，重新整理回到 `1x`；不寫 localStorage
- **進度條**：顯示「第 n / N 步」（目前步數／總步數，GridWorld 與 MemoryView 同一格式）；可點擊跳到任一步（跳步時直接計算該步狀態，不播放中間動畫）
- **自動播放時機**
  - GridWorld：Run 回傳 `ok`、`runtime_panic`、`timeout`、`memory_limit`、`output_limit`（且 `execution` 不為 null）時自動從第 1 步播放
  - MemoryView：不自動播放，見 §5.5
- `prefers-reduced-motion: reduce` 時：不做位移補間，直接跳格；速度選單仍可用

### 4.5 錯誤與回饋視覺

每種錯誤都要有「動畫＋圖示＋文字」三種線索，不能只靠顏色。

- **一般移動**：Ferris 平移到目標格（300ms ease-out），面向依方向左右翻轉
- **撞牆**（停止原因 `wall`）
  - Ferris 往牆的方向移動 30% 後彈回，左右抖動 2 次
  - 牆格閃紅框 2 次，Ferris 頭上出現「💥」圖示
  - 可視化區底部訊息列：「第 7 步：往右撞到牆了（4, 2）」
- **撞到關閉的門**（`door`）：同撞牆動畫，門格上顯示「🔒」，訊息「門還關著，先 OPEN <方向>」，`<方向>` 為撞門那一步 `MOVE` 的方向（門就在該方向相鄰格；例：往右撞門顯示「門還關著，先 OPEN RIGHT」）
- **走出地圖**（`out_of_bounds`）：同撞牆動畫，地圖外框該側閃紅，訊息「再走就掉出地圖了」
- **無效指令**（`invalid`）
  - Ferris 不移動，頭上出現「❓」泡泡
  - 訊息列顯示原始那一行並說明原因，依情況選一句（`<行>` 為去除頭尾空白後的原始行）：
    - 動詞不是 `MOVE`／`PICK`／`OPEN`（例 `JUMP`）：「第 n 步：看不懂 `<行>`，指令只有 MOVE、PICK、OPEN」
    - `MOVE`／`OPEN` 沒接方向或方向不對（例 `MOVE FORWARD`）：「第 n 步：看不懂 `<行>`，方向只能是 UP/DOWN/LEFT/RIGHT」
    - `MOVE`／`OPEN` 接了兩個以上的字（例 `MOVE UP UP`）：同上一句
    - `PICK` 後面多接東西（例 `PICK GEM`）：「第 n 步：看不懂 `<行>`，PICK 後面不用接東西」
  - 主控台該行加紅色左邊框與 ✗ 圖示
- **腳下沒寶石卻 PICK**（`pick_empty`）：Ferris 低頭動畫，頭上「❓」，訊息「這格沒有寶石」
- **該方向沒有門卻 OPEN**（`open_nothing`）：同上，訊息「那邊沒有門」
- **撿到寶石**：寶石放大後消失（200ms），右上角計數「💎 1 / 2」跳動一次
- **開門**：門圖示換成開啟狀態，格子變成可通行底色
- **抵達終點且全撿**：Ferris 跳一下，終點旗幟揮動；這是視覺回饋，不等於通關（Run 時仍顯示「按送出驗收」）
- **指令播完但沒到終點**：訊息列「指令跑完了，但 Ferris 還沒到終點」，Ferris 頭上「…」
- **到終點但寶石沒撿完**：訊息列「到終點了，但還有 n 顆寶石沒撿」，Ferris 頭上「…」
- **沒到終點也沒撿完**：用「指令跑完了，但 Ferris 還沒到終點」，寶石數看右上角計數
- **超過步數上限**（`step_limit`）：Ferris 頭上「…」，訊息列「指令超過 1,000 步，後面的不播放了」
- **程式被終止**（`timeout`／`memory_limit`／`output_limit`）：播到最後一個收到的指令後，訊息列顯示「時間到」「記憶體不足」或「輸出過長已截斷」

### 4.6 畫面元素

- 格子大小：依容器寬度自動縮放，單格 32–64px，正方形
- 圖層由下而上：地板 → 牆／門／終點 → 寶石 → Ferris → 錯誤標記
- 每種物件有獨立圖形，不只靠顏色區分；畫法見 §8.5（岩石牆、木門與鎖頭、切面寶石、旗子終點、螃蟹 Ferris）
- 地圖右上角固定顯示「💎 已撿 / 總數」
- 用 SVG 繪製（方便縮放與深淺色切換），不用 canvas

---

## 5. MemoryView 元件規格

MemoryView 按步驟播放 stack／heap 狀態，展示 ownership、move、借用。

### 5.1 資料格式：memory_snapshots

```ts
type MemorySnapshot = {
  line: number          // 對應編輯器行號（1 起算）
  note: string          // 這一步的說明文字，支援行內 markdown
  frames: StackFrame[]  // 由外到內，最後一個是目前執行中的函式
  heap: HeapBlock[]
  pointers: Pointer[]
  error: SnapshotError | null  // 只有錯誤發生的那一步有值
}

type StackFrame = {
  id: string            // 例："main"、"take#1"
  fn: string            // 顯示名稱，例："main"
  vars: StackVar[]
}

type StackVar = {
  id: string            // 全域唯一，pointers 用它當端點，例："main.gem"
  name: string          // 例："gem"
  type: string          // 例："String"、"&String"
  value: string | null  // 直接顯示的值，例："5"；指標型別可為 null
  state: 'live' | 'moved' | 'dropped'
}

type HeapBlock = {
  id: string            // 例："h1"
  type: string          // 例："String"
  value: string         // 例："\"ruby\""
  state: 'live' | 'freed'
}

type Pointer = {
  from: string          // StackVar.id
  to: string            // StackVar.id 或 HeapBlock.id
  kind: 'own' | 'shared' | 'mut'
}

type SnapshotError = {
  code: string | null   // rustc 錯誤碼，例："E0382"；執行期錯誤為 null
  target: string        // 出事的 StackVar.id 或 HeapBlock.id，須存在於同一步
}
```

- `memory_snapshots: MemorySnapshot[]`，陣列索引即步數
- **資料來源**：關卡作者手寫在 `level.toml` 的 `[[memory_snapshots]]`（寫法見 curriculum.md §5），經 `GET /api/levels/:id` 原樣傳給前端；MemoryView 只吃這個形狀
- 錯誤說明文字（編譯錯誤訊息、panic 訊息）寫在 `note`；`error` 只標出出事的位置與錯誤碼，供圖上畫錯誤標記（§5.3）
- `kind` 語意
  - `own`：擁有關係，例：`String` 的 stack 部分指向 heap 內容
  - `shared`：`&T` 共享借用
  - `mut`：`&mut T` 獨佔借用
- **v1.4 確認：第二季的集合與 closure 畫面不擴充型別**，以現有欄位依下列慣例表達
  - HashSet／HashMap 的桶（50 關）：每個**有內容的桶**是一個 `HeapBlock`，`type` 寫「桶 n」、`value` 寫元素（例 `Isle("Coral")`）；集合變數以多條 `own` 指標指向各桶；空桶不畫（MemoryView 的 heap 是縱向清單，桶多時會擠）；`error.target` 指向重複的那個桶
  - closure 的捕捉（47 關）：closure 本身是 frame 裡的一個 `StackVar`（`type` 寫 `impl FnMut()` 之類），借用捕捉畫 `shared`／`mut` 指標指向被捕捉的變數，`move` 捕捉畫 `own` 指標指向被移入的值或其 heap 區塊，原變數標 `moved`
  - 限制：指標的 `from` 只能是變數，不能從 heap 區塊出發；「桶陣列 → 節點」這種 heap 內部的指向無法表達，所以用「一桶一區塊」攤平；若之後關卡真的需要 heap → heap 指標（例：鏈結串列節點），最小擴充是放寬 `from` 可為 heap `id`，屆時再評估
- **借用狀態不另開欄位**，由 `pointers` 推導：
  - 有任何 `mut` 指標指向某變數 → 該變數顯示「被獨佔借用中，暫時不能用」
  - 有 `shared` 指標指向某變數 → 該變數顯示「被共享借用中，只能讀」

### 5.2 版面

```
┌ MemoryView ─────────────────────────────────────────────┐
│  Stack                          Heap                    │
│  ┌ main ─────────────────┐                              │
│  │ gem : String     ●────┼──own──────▶ ┌──────────────┐ │
│  │                       │             │ "ruby"       │ │
│  │ r   : &String    ●- - ┼ - & - ┐     └──────────────┘ │
│  └───────────────────────┘       │                      │
│  ┌ take ─────────────────┐       ▼                      │
│  │ g   : String  [已移動]│   （指向 gem）                │
│  │     ░░░░░░░░░░░░░░░░  │                              │
│  └───────────────────────┘                              │
├─────────────────────────────────────────────────────────┤
│ 第 3 步 · 第 4 行                                        │
│ gem 被 move 給 take 的參數 g，main 裡的 gem 不能再用了。  │
├─────────────────────────────────────────────────────────┤
│ [⏮] [▶] [⏭]  速度 [1x ▾]   ●──────○────  第 3 / 8 步    │
└─────────────────────────────────────────────────────────┘
```

- 左區 Stack：frames 由上往下排，**目前執行中的 frame（最後一個）加粗外框**
- 右區 Heap：blocks 由上往下排
- 中間用 SVG 畫指標箭頭
- 下方說明列：「第 n 步 · 第 L 行」＋ `note`
- 最下方是共用的 `<PlaybackBar>`（§4.4）
- 寬度不足 360px（可視化區容器寬度） 時改上下排列：Stack 在上、Heap 在下，箭頭改為垂直走向

### 5.3 視覺編碼

每種狀態用「顏色＋線型＋圖示＋文字標籤」四重編碼，任一單獨存在都能辨識。

- **`own` 指標**
  - 顏色 `--mem-own`（中性灰藍）
  - 實線、實心三角箭頭
  - 線上標籤：`own`
- **`shared` 指標（&）**
  - 顏色 `--borrow-shared`（藍）
  - **細實線**、空心箭頭
  - 線上標籤：`&`，旁邊眼睛圖示 👁（唯讀）
  - 被指向的變數框：藍色細外框＋右上角 `&` 徽章
- **`mut` 指標（&mut）**
  - 顏色 `--borrow-mut`（橘）
  - **粗虛線**（線寬 3px，dash 6 4）、實心箭頭
  - 線上標籤：`&mut`，旁邊鉛筆圖示 ✎（可寫）
  - 被指向的變數框：橘色粗外框＋鎖頭圖示 🔒＋文字「借出中」
- **`moved` 變數**
  - 整格灰階、斜線網底（hatch pattern）、名稱加刪除線
  - 右側徽章「已移動」
- **`dropped` 變數／`freed` heap block**
  - 虛線外框、透明度 40%、徽章「已釋放」
  - 下一步起從畫面移除
- **錯誤標記**（`error` 有值的那一步）
  - `target` 指向的變數框或 heap 區塊：`--danger` 粗實線外框（3px）＋✗ 圖示＋徽章（`code` 有值顯示錯誤碼，例「E0382」；`null` 顯示「執行期錯誤」）
  - `target` 是變數且這一步有從它出發的指標：該箭頭改用 `--danger` 顏色，線型與 `own`／`&`／`&mut` 標籤不變，標籤後加 ✗
  - 圖層在最上層，不被其他箭頭遮住
  - 與 `moved`、`freed` 等狀態疊加顯示，不取代原本的網底與徽章
- **步驟變化提示**：與上一步相比新增或狀態改變的項目，外框閃一次（300ms）；reduced-motion 時改為常駐小圓點「新」

### 5.4 與編輯器行號同步

播放到哪一步，編輯器就高亮哪一行。

```
① 步數改變
  ↓
② 讀取 snapshot.line
  ↓
③ 編輯器高亮該行
  ↓
④ 捲動到可見範圍
```

① 玩家操作 `<PlaybackBar>` 或自動播放前進，Pinia store 更新 `currentStep`（0 起算的陣列索引）
② 從 `memory_snapshots[currentStep].line` 取得行號
③ 用 Monaco `deltaDecorations` 加整行背景色 `--line-active`，並在行號左側加 ▶ 圖示（gutter glyph）；上一行的裝飾同時移除
④ 該行不在可見範圍時呼叫 `revealLineInCenterIfOutsideViewport`

- **反向同步**：在編輯器點某行行號左側的 ▶ 區域時，跳到第一個 `line` 等於該行的步驟；沒有對應步驟則不動作
- **程式碼已被修改**：玩家改過程式碼後，行號可能對不上
  - 說明列上方顯示黃色提示「程式碼已修改，行號高亮僅供參考」
  - 程式碼與 `starter` 相同時（例如按「重設」後）提示消失
- MemoryView 掛載時即同步第 1 步的行號高亮
- 播放到最後一步結束時保留最後一步的行高亮；離開關卡時移除所有行高亮裝飾
- GridWorld 不做行號同步（stdout 不帶行號資訊）

### 5.5 播放時機：快照是教材，不依賴 Run

`memory_snapshots` 是關卡作者手寫的教學腳本，與玩家程式碼和判題結果無關。

- 進入關卡、`GET /api/levels/:id` 回來後，MemoryView 立即顯示第 1 步（`memory_snapshots[0]`）並暫停，玩家可隨時播放、單步、跳步
- 不自動播放：避免玩家還沒讀故事就被動畫帶走
- Run／Submit 的任何結果（含 `compile_error`、請求中、請求失敗）都**不**清空、不重置、不半透明 MemoryView
- fix-compile 的 memory 關卡（09、11）starter 一定編譯失敗，玩家照樣能看完整段快照，包含錯誤那一步的錯誤標記（§5.3）

---

## 6. 星等、進度與解鎖規則

每關最多 3 顆星，三顆各自獨立判定。

### 6.1 星等條件

- **★1 通過**：任一次 Submit 全部測試通過
- **★2 沒看提示**：通過時，本次遊玩沒有揭露任何提示（`current.hintsRevealed = 0`）
- **★3 一次就過**：通過時，本次遊玩沒有失敗的 Submit（`current.failedSubmits = 0`）
  - Run 不計入次數，玩家可以盡情 Run
  - 計入失敗的 Submit：`compile_error`、`test_failed`、`timeout`、`memory_limit`、`output_limit`
  - 不計入：`server_busy`、`judge_unavailable`、`payload_too_large`、`network_error`
- 結算時三顆星**分別記錄、取歷史聯集**
  - 例：第一次玩拿到 ★1★3，重玩拿到 ★1★2 → 本關記為三顆星
  - 理由：每顆星代表一項成就，不因重玩而消失

**本次遊玩的起訖與計數歸零**

```
① 第一次進關
  ↓
② 累積計數
  ↓
③ Submit 通過
  ↓
④ 結算並歸零
```

① `current` 不存在時視為 `{ hintsRevealed: 0, failedSubmits: 0 }`
② 揭露提示、失敗的 Submit 都累加到 `current`，立即寫入 localStorage
③ 唯一的結束點；回地圖、重新整理、直接輸入網址、關分頁再回來、按「重設」都**不**結束本次遊玩
④ 先用 `current` 算出本次的三顆星並與歷史聯集，再把 `current` 歸零；之後的 Submit（含「再玩一次」、從地圖再進入）屬於新的一次遊玩

- 結果：未通關前看過的提示與失敗次數不會因任何操作消失，無法靠回地圖或重新整理繞過 ★2、★3
- 已知限制（接受）：草稿通關後保留（§6.6），重玩時直接再送一次舊程式碼即可補拿 ★2、★3；進度存在玩家自己的瀏覽器，本來就能手動清除，星等只作為自我挑戰，不做防弊

### 6.2 提示逐步揭露

- 提示區預設只顯示一顆按鈕「需要提示嗎？（k / n）」，`k` 為 `current.hintsRevealed`
- 確認一律用自訂 `ConfirmDialog`（海洋繪本風格 modal，§8.4），不用 `window.confirm`：開啟時焦點移入對話框、`Esc` 等同取消、關閉後焦點回到觸發按鈕（§7.3）
- `current.hintsRevealed = 0` 時點擊先彈出確認：「看提示會拿不到『沒看提示』這顆星，確定要看嗎？」；`k ≥ 1` 後不再確認
- 每點一次揭露下一則，已揭露的保留顯示；全部揭露後按鈕隱藏
- 進入關卡時依 `current.hintsRevealed` 還原已揭露的提示；通關後歸零，提示區回到收起狀態
- `hints` 為空陣列的關卡不顯示提示區，★2 照常判定

### 6.3 解鎖規則

```
① 第一個章節第 1 關
  ↓
② 同章節下一關
  ↓
③ 下一個章節第 1 關
```

① 預設解鎖
② 前一關至少 ★1 即解鎖
③ 前一個章節**所有關卡**至少 ★1 即解鎖

- 「章節」只算 `GET /api/levels` 的 `chapters` 內出現的章節（後端只回傳至少有一關已載入的章節）；例：若第 4 章沒有已載入的關卡，第 5 章的「前一個章節」是第 3 章
- 「第 1 關」「下一關」依 `(chapter, order)` 排序，只看已載入的關卡
- 星等不影響解鎖，只影響收集成就
- 章節地圖每章標題旁顯示累計星數「★ 已得 / 該章已載入關卡數 × 3」（例「★ 4 / 6」）

### 6.4 章節地圖節點狀態

- 章節區塊標題：「第 N 章 章節名稱」（`chapters[].title`）
- **鎖定**：灰色節點＋鎖頭圖示，路徑為虛線
- **可挑戰**：主色節點、緩慢脈動外圈（reduced-motion 時改為靜態粗外框）
- **已通過**：實心節點＋下方三格星（亮星／暗星），路徑為實線
- **目前所在**：可挑戰節點中排序最前的一個，Ferris 圖示站在這裡；全部通過時 Ferris 站在最後一關
- 每個節點下方顯示關卡編號與名稱，並以小圖示標出 kind（🔨 build／🩹 fix-compile／🔍 debug）

**關卡編號格式：`章-章內序`**

- `章` 為 `chapter`；`章內序` 為該關在同章「已載入關卡」中依 `order` 排序的位置，從 1 起算
- 例（全部關卡載入時）：01 → `1-1`、02 → `1-2`、05 → `2-1`、09 → `3-1`、11 → `3-3`、19 → `5-2`、28 → `7-3`、31 → `8-2`
- 編號只用於顯示，之後補關會改變；進度、草稿、路由一律用 `id`，不受影響
- 頂部列、節點、通關結算都用同一格式

### 6.5 localStorage 結構

```ts
// key: "rustquest.progress.v1"
type Progress = {
  version: 1
  levels: Record<string /* levelId */, LevelProgress>
}

type LevelProgress = {
  stars: { pass: boolean; noHint: boolean; firstTry: boolean } // 歷史聯集；pass 即「已通關」
  current: {                 // 本次遊玩，Submit 通過結算後歸零（§6.1）
    hintsRevealed: number
    failedSubmits: number
  }
}

// key: "rustquest.prefs.v1"
type Prefs = {
  theme: 'system' | 'light' | 'dark'     // 深淺色（§8.3）
}
```

- 「已通關」一律讀 `stars.pass`，不另存布林
- 讀取失敗（隱私模式存取被拒、JSON 壞掉）時：以空進度啟動，toast「進度無法讀取，已從頭開始」，不讓頁面壞掉
- 第一次進站或資料已被清除（localStorage 沒有該 key）：以空進度啟動，**不** toast；只有既有資料損毀或讀取失敗才 toast
- 寫入失敗時：toast「進度無法儲存」，遊戲照常進行
- `version` 不符時：以空進度啟動（目前只有 v1，不寫遷移程式）
- `Prefs` 缺欄位或值不合法時用預設值 `theme = 'system'`
- 草稿另存 key：`rustquest.draft.<levelId>`，見 §6.6

### 6.6 草稿自動儲存

玩家寫到一半的程式碼不會因重新整理或誤關分頁而消失。

```
① 玩家編輯
  ↓
② debounce 500 ms
  ↓
③ 寫入草稿
  ↓
④ 下次進關載入
```

① 編輯器內容改變
② 停止輸入 500 ms 後才寫，避免每次按鍵都寫 localStorage
③ 寫入 `rustquest.draft.<levelId>`，值為程式碼字串；每關一個 key，不併入 `Progress`，避免每次按鍵都重寫整份進度
④ 有草稿就載入草稿，沒有就用 API 的 `starter`

- **重設**：先彈出確認（`ConfirmDialog`，§6.2）「重設會把程式碼改回起始版本，目前的修改會消失，確定嗎？」，確定後程式碼改回 `starter` 並刪除該關草稿；程式碼已與 `starter` 相同時不彈確認
  - 重設不影響本次遊玩計數（§6.1）
- **通關後**：草稿保留，重玩時看到上次的程式碼
- **寫入失敗**：不提示（進度寫入失敗已有 toast），編輯照常

---

## 7. 可及性

目標符合 WCAG 2.1 AA；重點在「不只靠顏色」與「全程可用鍵盤」。

### 7.1 不只靠顏色

- 借用種類：顏色＋線型＋箭頭樣式＋文字標籤＋圖示（§5.3）
- moved／dropped：灰階之外再加斜線網底、刪除線、文字徽章
- 錯誤等級：紅／黃之外再加 ✗／⚠ 圖示與文字「錯誤」「警告」
- 測試結果：✓／✗／— 圖示＋「通過」「失敗」「未執行」文字
- 星等：亮星為實心、暗星為空心外框，不只靠金色與灰色
- 時間軸：持有／等待／訊息以實心條、空心虛線條、箭頭區分，再加 ⏳ ✉ ■ ✗ 圖示與訊息列文字；死鎖另有 ✗ 與「死鎖」字樣（§11.9）
- 驗證方式：用 Chrome DevTools 的 Deuteranopia、Protanopia、Tritanopia、Achromatopsia 模擬各檢查一次

### 7.2 鍵盤快捷鍵

macOS 用 `Cmd`，其他系統用 `Ctrl`，下文以 `Ctrl` 表示。

- `Ctrl+Enter`：執行（Run）
- `Ctrl+Shift+Enter`：送出（Submit）
- `Ctrl+Shift+R`：不綁定（瀏覽器保留）；重設只能用滑鼠或 Tab 到按鈕，避免誤觸清掉程式碼
- 可視化區取得焦點時：
  - `Space`：播放／暫停
  - `→`：單步前進
  - `Home`：從頭
- `Esc`：關閉 Modal、抽屜、跳過通關動畫；在確認對話框中等同「取消」
- 快捷鍵說明面板：由頂部列「⌨ 快捷鍵」按鈕開啟（不另設快捷鍵），列出本節全部按鍵與下方 `Ctrl+M` 說明

**實作注意**
- Monaco 預設 `Ctrl+Enter` 是「在下方插入新行」，必須用 `editor.addCommand` 覆寫，否則焦點在編輯器時快捷鍵會失效
- 焦點在編輯器外時，用全域 `keydown` 監聽；兩邊都要接
- Monaco 內 `Tab` 會插入縮排，鍵盤使用者需用 `Ctrl+M` 切換「Tab 移動焦點」模式；在快捷鍵說明面板中寫出這條

### 7.3 螢幕閱讀器與焦點

- 主控台結果區設 `aria-live="polite"`，結果回來時朗讀一句摘要（例：「編譯失敗，2 個錯誤」），不朗讀全部內容
- 可視化區提供文字替代：每步的說明列（GridWorld 訊息列、MemoryView `note`、TimelineView 訊息列）同時放進 `aria-live` 區
- 所有圖示按鈕都有 `aria-label`（例：「單步前進」）
- 焦點順序：頂部列 → 左欄 → 編輯器 → 操作列 → 可視化 → 主控台
- 焦點框：2px `--focus-ring` 外框，深淺色都要對比 ≥ 3:1
- Modal 開啟時焦點鎖在 Modal 內，關閉後回到觸發按鈕

### 7.4 動態效果

- 遵守 `prefers-reduced-motion: reduce`：通關動畫略過、格子移動不補間、脈動改靜態
- 所有自動播放都可暫停

---

## 8. 視覺風格

正式前端採「海洋繪本」風格（使用者已從 POC 三種風格中選定），只做這一種，分淺色與深色兩套。

### 8.1 風格定調

- 大圓角、柔和陰影、圓體字，移動帶輕微回彈，整體像手繪繪本
- 不做風格切換：`<html>` 只有 `data-theme="light|dark"`，沒有 `data-style`
- 數值來源：POC `poc/web/src/style.css` 的 B 風格與 `Ferris.vue`、`GridWorld.vue` 的 B 分支；POC 是拋棄式，本節為正式依據
- 字型由 Google Fonts 載入（Huninn、Fredoka、JetBrains Mono、Noto Sans TC）；載入失敗時退回後備字型

### 8.2 Token

所有顏色、字型、圓角、陰影、動畫曲線只透過 CSS 自訂屬性取用，元件內不寫死。

- 介面：`--bg`、`--bg-deco`、`--surface`、`--surface-muted`、`--border`、`--border-strong`、`--text`、`--text-muted`、`--heading`、`--accent`
- 品牌：`--brand`、`--brand-hover`、`--brand-contrast`、`--brand-soft`
- 狀態：`--success`、`--danger`、`--warning`、`--info`、`--neutral-busy`（伺服器忙碌，刻意不用紅色）、`--focus-ring`
- 地圖：`--grid-edge`、`--grid-floor`、`--grid-floor-2`、`--grid-floor-deco`、`--grid-wall`、`--grid-wall-2`、`--grid-gem`、`--grid-goal`、`--grid-door`
- Ferris：`--ferris-body`、`--ferris-shade`、`--ferris-ink`、`--ferris-eye`、`--ferris-blush`
- 星等：`--star-on`、`--star-off`、`--star-glow`
- 記憶體可視化（Okabe-Ito 色盲友善，非 POC 項目，沿用 v1.1）：`--mem-own`、`--borrow-shared`、`--borrow-mut`、`--mem-moved`、`--line-active`
- 時間軸（Okabe-Ito 加深，非 POC 項目，v1.4）：`--tl-lock-1`～`--tl-lock-4`，依鎖第一次出現的順序輪流使用（§11.9）；訊息箭頭沿用 `--info`、死鎖沿用 `--danger`
- 形狀與字型（淺深色共用）：見下方 `:root`
- POC 名稱對照：`--muted` → `--text-muted`、`--surface-2` → `--surface-muted`、`--focus` → `--focus-ring`

```css
:root {
  --font-ui: 'Huninn', 'Noto Sans TC', system-ui, sans-serif;
  --font-title: 'Fredoka', 'Huninn', sans-serif;
  --font-heading: 'Fredoka', 'Huninn', sans-serif;
  --font-code: 'JetBrains Mono', ui-monospace, monospace;
  --title-size-sm: 17px;
  --title-size-xs: 13px;
  --console-size: 13px;
  --radius: 18px;          /* 卡片、面板 */
  --radius-sm: 12px;       /* 輸入框、小元件 */
  --radius-lg: 28px;       /* Modal、頂部列 */
  --radius-pill: 999px;    /* 按鈕、切換鈕 */
  --radius-avatar: 50%;
  --border-w: 2px;
  --border-w-strong: 3px;
  --move-dur: 380ms;
  --move-ease: cubic-bezier(.34, 1.56, .64, 1);
  --confetti-radius: 50%;
}

[data-theme='light'] {
  --bg: #fff3e0; --bg-deco: #d8f1f0; --surface: #fffdf8; --surface-muted: #fff1dc;
  --border: #f0dcc0; --border-strong: #e3c39b; --text: #4a3426; --text-muted: #846a58;
  --heading: #2c769a; --accent: #4fb3bf;
  --brand: #bf4b1f; --brand-hover: #a8401a; --brand-contrast: #ffffff; --brand-soft: #ffe3d6;
  --success: #227458; --danger: #d32d24; --warning: #945f17; --info: #2c769a;
  --neutral-busy: #6d7680; --focus-ring: #0072B2;
  --shadow: 0 10px 24px rgba(166, 110, 60, .18);
  --shadow-sm: 0 4px 12px rgba(166, 110, 60, .14);
  --shadow-lg: 0 24px 60px rgba(120, 70, 30, .3);
  --grid-edge: #8fd3d8; --grid-floor: #fbe3b6; --grid-floor-2: #f8d9a0; --grid-floor-deco: #e5bf83;
  --grid-wall: #b08968; --grid-wall-2: #d4b08c; --grid-gem: #3cc0a0; --grid-goal: #ff7b9c;
  --grid-door: #8B5A2B;
  --ferris-body: #ff7b54; --ferris-shade: #e85d33; --ferris-ink: #5a3220; --ferris-eye: #5a3220;
  --ferris-blush: #ffb3b3;
  --star-on: #ffc94a; --star-off: #ecd9bf; --star-glow: 0 2px 0 #e0a21e;
  --mem-own: #56677A; --borrow-shared: #0072B2; --borrow-mut: #E69F00; --mem-moved: #9AA0A6;
  --line-active: rgba(0, 114, 178, 0.12);
  --tl-lock-1: #0072B2; --tl-lock-2: #B36B00; --tl-lock-3: #007A5A; --tl-lock-4: #B3478A;
}

[data-theme='dark'] {
  --bg: #0e2a3a; --bg-deco: #123a4d; --surface: #16384b; --surface-muted: #1d465c;
  --border: #255a72; --border-strong: #3a7a94; --text: #fdebd3; --text-muted: #a9c2cc;
  --heading: #8fe3cf; --accent: #6fd6df;
  --brand: #ff9b73; --brand-hover: #ffb090; --brand-contrast: #2a1408; --brand-soft: #3f3a44;
  --success: #6fe0b4; --danger: #ff8a80; --warning: #ffc66b; --info: #8ccbff;
  --neutral-busy: #969ea7; --focus-ring: #56B4E9;
  --shadow: 0 10px 24px rgba(0, 0, 0, .3);
  --shadow-sm: 0 4px 12px rgba(0, 0, 0, .25);
  --shadow-lg: 0 24px 60px rgba(0, 0, 0, .5);
  --grid-edge: #0a5a6e; --grid-floor: #e8cf9e; --grid-floor-2: #dcc08a; --grid-floor-deco: #bf9f68;
  --grid-wall: #7b5e4a; --grid-wall-2: #a1806a; --grid-gem: #4fe0bd; --grid-goal: #ff8fab;
  --grid-door: #B07A45;
  --ferris-body: #ff8a5c; --ferris-shade: #e8653a; --ferris-ink: #4a2414; --ferris-eye: #4a2414;
  --ferris-blush: #ffb3b3;
  --star-on: #ffd166; --star-off: #2f5c70; --star-glow: 0 0 12px rgba(255, 209, 102, .5);
  --mem-own: #9AA8B8; --borrow-shared: #56B4E9; --borrow-mut: #F0B840; --mem-moved: #5C6166;
  --line-active: rgba(86, 180, 233, 0.16);
  --tl-lock-1: #56B4E9; --tl-lock-2: #F0B840; --tl-lock-3: #4FD1A5; --tl-lock-4: #E59CC4;
}
```

- 對比要求：文字對 `--bg`／`--surface` ≥ 4.5:1、`--text-muted` ≥ 4.5:1、`--focus-ring` ≥ 3:1；POC 數值未逐一量測，實作時用 DevTools 檢查，不達標就微調並回寫本節
- 借用色在色盲模擬下仍需可分辨；但規格要求不能只靠顏色，線型與標籤才是主要區分（§5.3）
- `--tl-lock-*` 是圖形色，對 `--surface` 需 ≥ 3:1（WCAG 1.4.11）；估算值已達標，實作時量測，不達標就微調並回寫本節
- `--move-ease` 的回彈在 reduced-motion 時不使用：一律不做位移補間（§7.4）

### 8.3 深淺色模式

- `Prefs.theme` 預設 `system`，跟隨 `prefers-color-scheme`
- 頂部列 ☀/☾ 按鈕在「系統 → 淺色 → 深色」間循環，選擇存 `rustquest.prefs.v1`
- 切換方式：在 `<html>` 設 `data-theme="light|dark"`；`system` 時依媒體查詢決定實際值
- Monaco 主題同步切換：淺色 `ocean-light`（基於 `vs`）、深色 `ocean-dark`（基於 `vs-dark`），只覆寫背景色、行高亮色與字型以對齊 token
- 在 `<head>` 內聯一小段 script，於首次繪製前設定 `data-theme`，避免閃白

### 8.4 版面元素

- **頁面底色**：`--bg` 上疊兩層放射漸層：底部 `--bg-deco` 大橢圓（像海平面）、左上角 `--brand` 8% 淡光
- **頂部列**：浮起的圓角卡片，四周留 10–12px，`--border-w` `--border` 外框、`--radius-lg`、`--shadow-sm`
- **面板**（左欄、編輯器、可視化、主控台）：`--surface` 底、`--border` 外框、`--radius`、`--shadow`
- **按鈕**：一律膠囊形 `--radius-pill`
  - 主要（送出）：`--brand` 由上往下淡白漸層、底部 4px `--ferris-shade` 實色陰影＋柔光；hover 上浮 2px、按下下沉 3px 且陰影縮成 1px
  - 次要（執行）：`--surface` 底、`--brand` 外框與文字；hover 上浮 2px、底色 `--brand-soft`
  - 危險（重設）：透明文字按鈕，`--text-muted`，hover 時 `--danger`
  - disabled：透明度 .5
- **焦點框**：2px `--focus-ring` outline、offset 2px
- **編輯器檔名列**：`--surface-muted` 底，檔名前加 🦀
- **字型**：介面 `--font-ui`；關卡名、章節標題 `--font-heading`；程式碼與主控台 `--font-code`

### 8.5 Ferris 與地圖畫法

- **Ferris**（SVG，viewBox 64×48，線條 `--ferris-ink` 2.2px 圓角端點）
  - 身體：頂部波浪狀的橢圓殼，填 `--ferris-body`；兩隻鉗子向外上方張開；四隻腳；兩根眼柄
  - 眼睛：白色圓眼＋`--ferris-ink` 眼珠；臉頰 `--ferris-blush`
  - 表情：一般（圓眼）、開心（白色眼白上畫上彎弧線）、暈眩（眼睛畫 ×）、疑惑（一眼放大）；所有表情都保留眼白
  - 用途：一般＝對話泡泡頭像（48px）與地圖「目前所在」；開心＝抵達終點且全撿、通關動畫；暈眩＝撞牆、撞門、走出地圖；疑惑＝無效指令、`PICK` 空格、`OPEN` 沒有門
- **地圖外框**：整張地圖外圍 6px、圓角 26 的 `--grid-edge` 底板
- **地板**：每格內縮 3px、圓角 14 的方塊，棋盤交錯 `--grid-floor`／`--grid-floor-2`，格內兩顆 `--grid-floor-deco` 小圓點（沙粒）
- **牆**：不規則圓潤的岩石塊，填 `--grid-wall`、`--ferris-ink` 2px 描邊，加一道 `--grid-wall-2` 高光弧線與一顆圓點
- **終點**：`--ferris-ink` 旗桿＋波浪形 `--grid-goal` 旗子，底部 `--grid-floor-deco` 橢圓陰影；通關時旗子揮動
- **寶石**：切面菱形，填 `--grid-gem`、`--ferris-ink` 描邊與半透明切面線，右上角一顆黃色四角星閃光
- **門**：POC 未畫，比照牆的圓潤線條畫木門板，填 `--grid-door`；關閉時門上掛鎖，開啟後改為半透明門框
- **對話泡泡**：`--surface` 底、`--border` 框、`--radius`，左下角小三角指向 Ferris 頭像
- **通關動畫**：Ferris 左右揮鉗跳兩下＋圓點彩帶從上方落下（`--brand`、`--star-on`、`--grid-gem`，圓角 `--confetti-radius`）
- **章節地圖**：沿用頁面底色，不分章節換底色

---

## 9. 已定事項

原本規格撰寫時的未決項目都已定案，理由見 `docs/specs/decisions.md`。

- **Grid 通關條件**：一律「撿完全部寶石＋到終點」，`[grid]` 不加設定欄位
- **`OPEN` 的方向**：`OPEN <DIR>` 明確指定方向，Ferris 沒有「面向」狀態
- **錯誤即停**：撞牆、撞門、走出地圖、無效指令、`PICK` 空格、`OPEN` 沒有門都立即停止
- **後端資料**（已寫進 api.md）
  - Diagnostic 提供 `end_line`、`end_column`、`rendered`
  - 測試結果為 `{name, status, message}`，`status` 為 `passed`、`failed`、`not_run`；預期與實際從 `message` 拆解（§3.3）
  - JudgeStatus 七種，timeout、memory_limit、output_limit 都回傳已產生的 stdout
  - `memory_snapshots` 由關卡作者手寫在 `level.toml`；錯誤步驟以 `error` 標出出事位置（§5.1、§5.3）
  - 章節名稱來自 `GET /api/levels` 的 `chapters`（§0、§6.4）
- **v1.2 定案**（decisions.md §14）
  - Ferris 泡泡內容即 `story`（§1.4）；MemoryView 進關即可播放，不依賴 Run（§5.5）
  - 本次遊玩只在 Submit 通過時結束（§6.1）；關卡編號 `章-章內序`（§6.4）
  - 錯誤卡片「隱藏測試」與「無位置」以 `rendered` 是否為 null 區分（§3.4）
  - 主控台預設分頁只看結果狀態（§1.3）；413 獨立為 `payload_too_large`（§3.1）
  - 視覺風格定為「海洋繪本」，只做這一種，不做風格切換（§8）
- **v1.3 定案**（decisions.md §15）
  - 播放控制列步數一律「第 n / N 步」（§4.4、§5.2）；撞門訊息方向依實際撞門方向（§4.5）
  - 零寬度 marker 擴到 token 或至少 1 字元；panic marker 標整行（§3.3）
  - 前端請求逾時 60 秒（§3.2）；平板抽屜與手機首次進關只記在記憶體（§2.2、§2.3）
  - GridWorld `pending` prop，Submit 後還原到上次 Run 終點（§3.2、§4.1）；MemoryView 播完保留最後一步行高亮（§5.4）
  - 確認對話框用自訂 `ConfirmDialog`（§6.2、§6.6）；跨章鎖定 tooltip 指前一章第一個未通過關卡（§1.1）
  - 413 只看狀態碼；`Retry-After` 只接受整數秒；`{name}` 抓不到時「這個值」為純文字；Submit 超時預設「輸出」分頁（§3.3、§3.4）
- **v1.4 定案**（decisions.md §15 ⑬）
  - 新增 `visual = "timeline"` 與 TimelineView（§11）；事件來源為 Run stdout 的 `@tl|` 行，由 starter 檔尾的小幫手印出
  - 一關只有一種 visual；時間軸只陳述事實，不給修法

## 10. 需求外項目處理結果

- **已納入正文**
  - 自動儲存程式碼草稿：§6.6
  - 指令步數上限 1,000 步：§4.3
- **不做**
  - 播放「上一步」按鈕：進度條已可跳步
  - v1.4 時間軸：毫秒時間軸、訊息內容標籤、一關多種 visual（分頁切換）、Submit 驅動時間軸、編輯器同時高亮多行、泳道摺疊、事件原文切換鈕（理由見 decisions.md §15 ⑬）
  - 欄寬可拖曳調整：之後加不影響資料結構
  - ASCII 地圖寫法：`[grid]` 只用座標陣列
  - 章節以星數解鎖隱藏關卡：星等只有收集價值
  - v1.2 刪除（無資料來源或與既有呈現重複，decisions.md §14）：開場對話、「繼續上次進度」按鈕、章節完成過場與徽章、Ferris 反應台詞池、`ferris` fenced block、`LevelProgress.cleared`、`Prefs.playbackSpeed`、`Ctrl+.` 與 `?` 快捷鍵、GridWorld `step-change`／`finished` 事件與 `GridResult`、主控台 stdout 與播放步同步、Run 耗時顯示
- **排入後續版本**
  - 錯誤圖鑑：MVP 只涵蓋 4 種錯誤碼，收藏感不足；後續關卡補齊後再評估（decisions.md §12）

---

## 11. TimelineView 元件規格

TimelineView 把玩家程式 Run 時印出的 `@tl|` 事件畫成泳道圖：誰拿到鎖、誰在等、訊息何時送達、死鎖時誰卡在哪。

### 11.1 定位與資料來源

- 用在 `visual = "timeline"` 的關卡；一關只有一種 visual，不與 GridWorld 並存
- 資料：Run 回傳的 `execution.stdout`，只看 `@tl|` 開頭的行（格式見 architecture.md §3.11）
  - 事件由 starter 檔尾的時間軸小幫手（`mod timeline`）印出；玩家呼叫 `timeline::lock`、`timeline::send`、`timeline::recv`、`timeline::lane`、`timeline::note`
- TimelineView **只負責視覺回放**；通關只看 Submit，Submit 不驅動時間軸（與 GridWorld 相同）
- 時間軸是「這一次 Run 的真實順序」：執行緒交錯每次可能不同，畫面不假裝可重現
  - 圖例下方常駐一行小字：「每次執行的先後順序可能不同」
- **只陳述事實，不給修法**：文案寫「誰等誰、誰佔著什麼」，不寫「請依編號上鎖」「記得 drop(tx)」；修法屬於提示，揭露才計入 ★2（§6.1）

### 11.2 元件介面

```ts
// TimelineView.vue
props: {
  stdout: string | null    // null：尚未執行，或 execution = null
  status: RunStatus | null // Run 回傳的 status；null 表示尚未執行
  truncated: boolean       // execution.stdout_truncated
  pending: boolean         // Run／Submit 請求中（§11.7）
}
emits: {
  'line-change': [line: number | null]  // 目前事件的行號，交給 LevelView 做編輯器高亮（§11.8）
}
```

- 與 GridWorld 相同的四個 props，LevelView 依 `visual` 擇一渲染
- `line-change` 與 MemoryView 同名同語意，LevelView 沿用同一條高亮管線

### 11.3 版面

可視化區桌機寬約 390px，版面以窄寬度為基準設計，事件多時在元件內橫向捲動。

```
┌ 時間軸 ───────────────────────────────────────────┐
│ 🔒① 窄橋 0  🔒② 窄橋 1  ✉ 回報                     │
│ ▬ 持有  ┄ 等待  ➝ 訊息    每次執行的先後順序可能不同 │
│               1    5    10   15   20          步   │
│ ✉ main       ┄┄┄┄┄┄┄✉┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ …       │
│                     ▲（分身 2 的訊息送達）         │
│ ✗ 分身 0  死鎖  ●─▬▬①▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬ …           │
│                       ┄┄②⏳┄┄┄┄┄┄┄┄┄┄ ✗            │
│ ✗ 分身 1  死鎖   ●──▬▬②▬▬▬▬▬▬▬▬▬▬▬▬▬ …            │
│                        ┄┄①⏳┄┄┄┄┄┄┄┄┄ ✗           │
│ ■ 分身 2          ●─✉─■                           │
├────────────────────────────────────────────────────┤
│ 第 21 步 · 第 45 行：分身 1 想拿 窄橋 0（被 分身 0 佔著）│
│ ✗ 死鎖：分身 0 等 窄橋 1（被 分身 1 佔著）；         │
│         分身 1 等 窄橋 0（被 分身 0 佔著）           │
├────────────────────────────────────────────────────┤
│ [⏮] [▶] [⏭]  速度 [1x ▾]  ●──────────● 第 21 / 21 步 │
└────────────────────────────────────────────────────┘
```

- **圖例列**：每把鎖一個代號 ①②③…（依第一次出現的順序）＋鎖名、每個 channel ✉＋名稱；第二行是線型說明與「先後可能不同」小字
- **時間軸刻度**：x 軸是事件序號（第 n 步），每 5 步一個刻度；不是毫秒
- **泳道**：一條執行緒一列
  - 左側標籤欄固定（橫向捲動時不動）：狀態圖示＋泳道名稱；狀態圖示見 §11.9
  - 排序：`main` 置頂，其餘依第一個事件出現的先後
  - 泳道名稱即識別：`thread` 相同的事件畫在同一條泳道
  - **具名泳道**：有 `start` 事件的泳道（該執行緒呼叫過 `timeline::lane`）；沒有 `start` 的泳道（例如 `main`）只畫它的事件與區段，沒有開始、結束標記
  - 泳道在解析完全部事件時就全部配置好（版面不跳動）；還沒輪到第一個事件的泳道標籤以 40% 透明度顯示
  - 同一泳道同時有多個區段（例：握著窄橋 0 又在等窄橋 1）時往下疊一層，泳道高度隨最大重疊數增加
- **欄寬**：每步一欄，欄寬 = 可用寬度 ÷ 事件數，夾在 14–32px；超出時元件內橫向捲動，播放時自動把目前步捲進可見範圍
- **訊息列**：兩行
  - 第一行：目前這一步的事件句（§11.6 句型）
  - 第二行：播到最後一步時的結論（§11.6），或解析警示（格式不對的行數、超過上限）
- **播放控制列**：共用 `<PlaybackBar>`（§4.4）
- 泳道多時（上限受判題 `pids-limit 64` 約束）可視化區內縱向捲動，圖例與刻度列固定在頂部

**窄寬度（可視化區容器 < 360px，例：手機「結果」分頁）**

- 標籤欄縮為 64px，泳道名稱超出時省略（完整名稱放 `title` 與 `aria-label`）
- 圖例改為可換行；「先後可能不同」小字移到訊息列下方
- 其餘不變；永遠不讓頁面本身出現橫向捲軸，橫向捲動只發生在時間軸區內

### 11.4 事件解析

```ts
type TimelineEventKind =
  | 'start' | 'end'
  | 'lock_wait' | 'lock_got' | 'lock_free'
  | 'send' | 'recv_wait' | 'recv' | 'recv_closed'
  | 'note'

interface TimelineEvent {
  kind: TimelineEventKind
  line: number | null  // 編輯器行號；lock_free、end 為 null
  thread: string       // 泳道名稱
  target: string       // 鎖名、channel 名、標註文字；start、end 為空字串
}

interface ParsedTimeline {
  events: TimelineEvent[]  // 最多 1,000 個
  malformed: number        // 以 @tl| 開頭但格式不對的行數
  capped: boolean          // 是否超過 1,000 個被截掉
}

// domain/timeline/：純函式
function parseTimelineEvents(stdout: string, truncated: boolean): ParsedTimeline
```

**解析規則**（逐行處理 stdout）

```
① 切行
  ↓
② 截斷處理
  ↓
③ 判斷是否為事件
  ↓
④ 拆欄位並驗證
  ↓
⑤ 加入事件或計入格式錯誤
```

① 以 `\n` 切行，去掉行尾 `\r`；不去除行首空白
② `truncated = true` 時丟棄最後一行（可能不完整，與 grid 相同）
③ 行首（第 1 欄）就是 `@tl|` 才算事件行；其餘是一般輸出，只給主控台
④ 去掉 `@tl|` 後以 `|` 切成最多 4 段：`kind`、`line`、`thread`、`target`（第 4 段取剩下全部）
  - `kind` 必須是十種之一
  - `line` 為空字串（→ `null`）或 `^[1-9][0-9]*$`
  - `thread` 去除頭尾空白後不可為空；`TimelineEvent.thread` 存修剪後的值（泳道比對用修剪後的名稱）
  - `target`：`lock_*`、`send`、`recv*`、`note` 不可為空；`start`、`end` 的 `target` 一律當空字串
  - 段數不足 4 段即格式錯誤
⑤ 合法事件依行序加入；格式錯誤只累加 `malformed`，不加入、不中斷解析
  - 合法事件第 1,001 個起不再加入，`capped = true`

- 事件之間的一致性（例如沒有對應 `lock_got` 的 `lock_free`）不在解析時檢查；畫不出來的就略過，不報錯（§11.5）
- 泳道名稱的約束由關卡程式負責、前端不檢查（architecture.md §3.11）：同一關內各執行緒的 `lane` 名稱須唯一；`lane` 須在該執行緒第一個事件之前呼叫，否則前面的事件會落在 std 執行緒名的另一條泳道
- 事件行永遠不是 grid 指令嘗試（`@` 不在 `[A-Z_]`），兩個解析器互不干擾

### 11.5 狀態推導：泳道、區段、箭頭、死鎖

`deriveTimeline(events, step, status)` 算出「播到第 `step` 步」時的畫面（只看前 `step` 個事件）；以下欄位為建議形狀，名稱以前端單元測試釘死為準。

```ts
type LaneState = 'not_started' | 'running' | 'waiting_lock' | 'waiting_msg' | 'ended' | 'deadlocked'

interface TimelineState {
  lanes: { name: string; state: LaneState; line: number | null }[]   // line：該泳道最近一個有行號的事件
  segments: {
    lane: string
    kind: 'hold' | 'wait_lock' | 'wait_msg'
    target: string           // 鎖名或 channel 名
    from: number; to: number | null   // 步數；null 表示到目前仍未結束
    deadlocked: boolean
  }[]
  messages: { channel: string; from: { lane: string; step: number }; to: { lane: string; step: number } | null }[]
  marks: { lane: string; step: number; kind: 'start' | 'end' | 'recv_closed' | 'note'; text: string }[]
  deadlock: { lane: string; waits: string; heldBy: string }[] | null  // 依循環順序
}
```

**區段**

- `hold`：某泳道 `lock_got(L)` 開始，到同一泳道 `lock_free(L)` 結束；沒有對應 `lock_got` 的 `lock_free` 略過
- `wait_lock`：`lock_wait(L)` 開始，到**同一泳道的下一個事件**結束（正常是 `lock_got`）；理由：卡住的執行緒不會再印任何事件
- `wait_msg`：`recv_wait(ch)` 開始，到同一泳道的下一個事件結束（`recv` 或 `recv_closed`）

**訊息箭頭**

- 每個 channel 名稱各自配對：第 k 個 `send` 對第 k 個 `recv`（小幫手保證先進先出，architecture.md §3.11）
- 箭頭從送出那一步、送方泳道，指到收到那一步、收方泳道
- 還沒被收到的 `send`：畫一小段虛線殘箭，尾端 ✉，表示「訊息在 channel 裡排隊」
- 配對結果 `recv` 排在 `send` 之前（只可能是玩家印了假事件）：不畫箭頭
- `recv_closed`：收方泳道該步畫 ⊘，代表 channel 已關閉

**泳道狀態**（第 `step` 步）

- `not_started`：還沒有任何事件
- `ended`：已出現 `end`
- `deadlocked`：屬於下方死鎖循環
- `waiting_lock`／`waiting_msg`：最後一個事件是 `lock_wait`／`recv_wait`
- `running`：其他情況

**死鎖判定（循環等待）**

```
① 找出每把鎖的持有者
  ↓
② 建等待關係
  ↓
③ 找循環
  ↓
④ 標記
```

① 鎖 L 的持有者＝有未結束 `hold(L)` 區段的泳道（多個時取最近開始的）
② 泳道 T 最後一個事件是 `lock_wait(L)` 且 L 有持有者 H → 一條等待關係 T → H
  - H 就是 T 自己（同一執行緒重複鎖同一把 `Mutex`）也算，循環長度為 1
③ 每條泳道最多一條往外的等待關係，沿著走回到起點即為循環
  - **循環的第一條泳道**＝循環中依泳道排序（§11.3：`main` 置頂、其餘依首次出現）最前者；`deadlock` 陣列從它開始，沿等待關係排列
④ 循環上的泳道狀態改為 `deadlocked`；它們的 `wait_lock` 區段，以及被等待那把鎖的 `hold` 區段，`deadlocked = true`
- 只有鎖會形成等待關係；`recv_wait` 不算（看不到誰還握著 Sender），顯示為 `waiting_msg`
- 依事件順序保證，前綴出現循環就是真的死鎖，之後不會解開；因此每一步都判定，循環一出現就標紅並維持到最後
- 判定不看 `status`：就算 `status = ok`（`main` 沒 join 就結束，卡住的執行緒被一起收掉）也照實標出

### 11.6 結果狀態下的顯示

每個狀態都定義三件事：時間軸、訊息列結論、是否自動播放。

**事件句型**（訊息列第一行與 `aria-live`；前綴「第 n 步 · 第 L 行：」，`line = null` 時省略「· 第 L 行」）

- `start`：「T 開始」
- `end`：「T 結束」
- `lock_wait`：「T 想拿 L」；L 此刻有持有者 H 時補「（被 H 佔著）」，H 是 T 自己時補「（被自己佔著）」
- `lock_got`：「T 拿到 L」
- `lock_free`：「T 放開 L」
- `send`：「T 往 ch 送出一則訊息」
- `recv_wait`：「T 等 ch 的訊息」
- `recv`：「T 從 ch 收到一則訊息（S 送的）」，S 為配對的送方
- `recv_closed`：「ch 已關閉，T 不會再收到訊息」
- `note`：「T：文字」

**尚未執行（`stdout = null` 且 `status = null`）**

- 只顯示圖例區與空白軸，中央一行：「按『執行』，看看每條執行緒怎麼跑」
- 播放控制列 disabled

**結論只看具名泳道（§11.3）**：沒有 `start` 的泳道（例如 `main`）不參與「全部執行緒都結束了」「還有 n 條沒做完」「其他執行緒都已結束」的判斷，也不畫未結束尾端「…」；它本來就不會有 `end`

**ok（Run 成功）**

- 自動從第 1 步播放
- 結論：全部具名泳道都 `ended` →「全部執行緒都結束了」（沒有任何具名泳道時同此句）
- 有具名泳道沒有 `end`（`main` 先結束）→「程式結束時，還有 n 條執行緒沒做完（名稱、名稱）」，這些泳道尾端畫「…」
- 有死鎖循環 → 改用死鎖結論（見 timeout）

**compile_error**

- 不畫泳道；中央一行「程式還沒編譯成功」；播放控制列 disabled

**runtime_panic**

- 自動播放到最後一個事件；結論「程式在這裡結束（panic）」
- 沒有 `end` 的具名泳道尾端畫「…」

**timeout**（死鎖的主要情境）

- 自動播放到最後一個事件
- 結論依序擇一
  - 有死鎖循環：「✗ 死鎖：T1 等 L1（被 T2 佔著）；T2 等 L2（被 T1 佔著）」，從循環的第一條泳道（§11.5 ③）開始，依循環順序列出每一條
  - 沒有循環但有泳道在等：「時間到：還有 n 條執行緒在等」；各泳道的等待原因看標籤與區段
    - `waiting_msg` 的泳道，若其他**具名**泳道此時都已 `ended`（至少一條），標籤旁補「其他執行緒都已結束」（陳述事實，不說原因）
  - 都沒有：「時間到」
- 死鎖時另見 §11.8 編輯器高亮

**memory_limit**

- `execution` 有值：自動播放到最後一個事件，結論「記憶體不足」
- `execution = null`：同 compile_error，中央一行「程式還沒編譯成功」換成「編譯時用太多記憶體」

**output_limit**

- 先丟棄最後一行再解析（§11.4 ②），自動播放，結論「輸出過長已截斷」

**test_failed、Submit 的任何結果**

- Submit 不驅動時間軸：維持最後一次 Run 的畫面（§11.7）

**請求失敗（server_busy、judge_unavailable、payload_too_large、network_error）**

- 時間軸還原成請求前的畫面（上次 Run 的最後一步，或尚未執行的空白狀態），不清空、不重播
- 訊息由主控台卡片負責（§3.3），時間軸不另顯示

**零事件與解析警示**（與上面各狀態疊加）

- 有 `execution` 但 0 個合法事件：中央一行「這次執行沒有時間軸事件：要透過 `timeline::` 小幫手上鎖、送訊息，時間軸才看得到（按『重設』可取回小幫手）」
  - 主要情境：玩家刪掉了小幫手呼叫，或沿用第一季留下的草稿
- `malformed > 0`：結論後補「有 n 行時間軸事件格式不對，已略過」
- `capped = true`：結論後補「事件超過 1,000 個，後面的不顯示了」

### 11.7 播放控制與時機

- 共用 `<PlaybackBar>`（§4.4），速度與步長相同（1x 每步 500ms）
- **第 0 步**：泳道已配置、沒有任何區段；「從頭」回到第 0 步（同 GridWorld）
- 「第 n / N 步」的 N 為合法事件數；N = 0 時播放控制列 disabled
- **自動播放時機**：與 GridWorld 相同，Run 回傳 `ok`、`runtime_panic`、`timeout`、`memory_limit`、`output_limit` 且 `execution` 不為 null 時，從第 1 步播放
- **`pending`**：回第 0 步並半透明
  - Run 回來：依新結果自動播放
  - Submit 回來、請求失敗：還原到上次 Run 播放結束的那一步，不重播；從未 Run 過則回到空白狀態
- 點時間軸上的任一事件標記：跳到該步並暫停（不播放中間動畫）
- 鍵盤：可視化區取得焦點時沿用 §7.2（`Space`、`→`、`Home`）

### 11.8 與主控台、編輯器的互動

**主控台**

- timeline 關卡的「輸出」分頁**隱藏** `@tl|` 開頭的行，只顯示一般 `println!` 輸出
  - 有隱藏時，輸出頂部一行淡色小字「已隱藏 n 行時間軸事件」
  - 理由：33 的除錯心法是「卡住時看主控台最後幾行」，事件行會把玩家自己的輸出淹沒
- 隱藏只發生在畫面；`execution.stdout` 原文不變
- 其他分頁（編譯錯誤、測試結果）不受影響

**編輯器**

- 播放到哪一步，就 `line-change` 該事件的 `line`；`line = null`（`lock_free`、`end`）時沿用前一個有行號事件的行，不清除高亮
- 死鎖時（結論為死鎖的最後一步）：高亮循環的第一條泳道（循環中依泳道排序最前者，§11.5 ③）等待的那一行
- 點泳道標籤：`line-change` 該泳道在目前這一步之前最近一個有行號的事件；不改變目前步數
  - 用途：死鎖時逐一點各隻分身，看牠們各自停在哪一行
- 程式碼在這次 Run 之後被修改：沿用主控台「程式已修改」狀態（§3.1 ④），訊息列上方顯示黃色提示「程式碼已修改，行號高亮僅供參考」
- 不做反向同步（點行號跳到步驟）：同一行常對應多個事件，例如迴圈裡的 `lock`

### 11.9 視覺編碼與可及性

每種狀態用「填色或線型＋圖示＋文字」至少三重編碼，任一單獨存在都能辨識。

- **持有鎖（`hold`）**
  - 實心條，填該鎖的 `--tl-lock-n`，2px `--ferris-ink` 描邊
  - 條頭標鎖代號（①②…），條長不足 2 欄時代號移到 tooltip
- **等鎖（`wait_lock`）**
  - 空心條，2px **虛線**外框（dash 4 3），顏色為該鎖的 `--tl-lock-n`
  - 條頭 ⏳＋鎖代號
- **等訊息（`wait_msg`）**
  - 空心條，2px **點線**外框，`--info`
  - 條頭 ✉⏳
- **訊息箭頭**：`--info` 2px 實線、實心箭頭；未送達的殘箭為虛線，尾端 ✉
- **channel 關閉**：⊘ 圖示
- **開始／結束**：● 實心圓／■ 方塊
- **標註**：🚩 小旗，文字不畫在圖上（太窄），進訊息列與 tooltip
- **泳道狀態圖示**（標籤欄）：`running` ▶、`waiting_lock` ⏳、`waiting_msg` ✉、`ended` ■（標籤改 `--text-muted`）、`deadlocked` ✗＋「死鎖」字樣、`not_started` 40% 透明
- **死鎖**
  - 循環上的泳道：標籤 `--danger` 文字＋✗＋「死鎖」
  - `deadlocked = true` 的區段：外加 3px `--danger` 實線外框，條尾 ✗；原本的填色與虛線保留（疊加，不取代）
  - 從等待區段尾端畫一條 `--danger` 虛線箭頭指向持有者那條 `hold` 區段，箭頭標 ✗
  - 圖層在最上層
- **鎖的顏色**：依第一次出現的順序用 `--tl-lock-1`～`--tl-lock-4`，第 5 把起循環使用；顏色重複時靠代號與圖例區分
- **背景**：泳道交錯底色 `--surface`／`--surface-muted`，分隔線 `--border`
- 用 SVG 繪製（同 GridWorld）

**可及性**

- 每一步的事件句（§11.6）放進 `aria-live="polite"`；播到最後一步時再朗讀結論
- SVG 設 `role="img"`，`aria-label` 為摘要：「n 條執行緒、m 把鎖、k 個 channel，第 s / N 步」
- 泳道標籤是按鈕（§11.8），`aria-label` 含名稱與狀態，例：「分身 0，死鎖，等窄橋 1」
- 事件標記可用 Tab 走訪（只走目前已播放的事件），`Enter` 跳到該步
- `prefers-reduced-motion: reduce`：箭頭與區段直接出現、不做畫線動畫；自動捲動直接跳位
- 色盲模擬（§7.1 四種）下仍能分辨持有／等待／訊息／死鎖：主要靠實心、虛線、點線、✗ 與文字

### 11.10 測試方向（交給 BDD 與前端第一棒）

**前端單元測試（Vitest，純函式）**

- `parseTimelineEvents`
  - 十種 kind 各一筆合法事件；`line` 空字串為 `null`
  - 格式錯誤：未知 kind、段數不足、`line` 為 `0` 或非數字、`thread` 空白、`lock_*` 缺 `target`；都只累加 `malformed`
  - 行首有空白的 `@tl|` 不算事件；`\r\n` 行尾；`truncated` 時丟最後一行；第 1,001 個起 `capped`
  - 一般輸出行不進事件；同一份 stdout 給 `parseGridCommands` 不產生任何指令嘗試
- `deriveTimeline`
  - 泳道排序（`main` 置頂、其餘依首次出現）；`not_started` 到 `ended` 的狀態變化；`thread` 前後空白修剪後歸同一泳道
  - `hold` 開關、同泳道重疊時疊層；`wait_lock`、`wait_msg` 由同泳道下一個事件結束
  - 訊息依 channel 先進先出配對；未送達殘箭；`recv_closed`
  - 死鎖：兩條循環、三條循環、自己等自己；等待鏈不成環（A 等 B、B 在等訊息）不算死鎖；循環出現的那一步起維持標紅；`deadlock` 陣列從循環中泳道排序最前者開始（循環後出現的泳道先進入等待時亦同）
  - 結論選擇：ok 全結束、ok 有沒做完、timeout 死鎖、timeout 等訊息且其他都結束、timeout 無等待、runtime_panic、memory_limit、output_limit、零事件、malformed、capped
  - 具名泳道：`main` 沒有 `start` 且沒有 `end` 時，ok 仍為「全部執行緒都結束了」、runtime_panic 不在 `main` 畫「…」；`main` 在等訊息、其他具名泳道都 `ended` 時補「其他執行緒都已結束」

**元件測試**

- 七種狀態與尚未執行的畫面；`pending` 半透明與三種還原（Run、Submit、請求失敗）
- 死鎖時同時出現 ✗、「死鎖」文字、`--danger` 類別（不只靠顏色）
- `line-change` 的時機（播放、`line = null` 沿用、點泳道標籤、死鎖最後一步高亮循環第一條泳道的等待行）
- 窄寬度（< 360px）標籤欄縮寬、頁面無橫向捲軸
- 主控台在 timeline 關卡隱藏 `@tl|` 行並顯示隱藏行數

**e2e（playwright-bdd，`features/ui_timeline.feature`）**

- mock stdout 一律取自 runner 實際執行（33 starter 死鎖、33 solution、31 starter 讀到 0、30 solution）
- 33 starter Run → 主控台 timeout 卡片＋時間軸死鎖結論＋兩條泳道 ✗＋編輯器高亮等待那一行
- 33 solution Run → 「全部執行緒都結束了」、訊息箭頭數等於回報數
- 31 starter Run → 三條螃蟹泳道的標註都是「讀到 0」，且標註出現在「放開 計數器」之後
- 30 starter Run → 「程式還沒編譯成功」
- 舊草稿（沒有小幫手呼叫）Run → 零事件提示
- Submit 後時間軸還原、不重播
- `@docker` 一個情境：真後端跑 33 solution，時間軸出現 3 條分身泳道與 `main`
