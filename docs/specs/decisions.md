# Rust Quest 規格決策紀錄

> 狀態：v1.3（定稿；§13 補保留清單、新增 §15 v1.3 收尾定案）
> 來源：`docs/PLAN.md`「已定決策」與規格交叉核對結果
> 格式：每條一行結論＋一行理由；細節見括號內的規格章節

---

## 1. 技術與環境

- **前端 Vue 3 + Vite + TS + Pinia + Monaco；後端 Axum 0.8；Rust 1.99.1**
  - 理由：團隊熟悉的組合；Monaco 內建 Rust 高亮與 marker API
- **關卡一律 `rustc --edition 2024`**（architecture.md §3.2）
  - 理由：與 runner 映像版本一致；28、31 關刻意教 2024 的暫存值規則
- **進度存前端 localStorage；後端無狀態、無 DB、無登入**（api.md §0）
  - 理由：MVP 單人學習用，不需要帳號；後端可水平擴充也好測
- **關卡啟動時一次載入，不做 hot reload**（architecture.md §5.1）
  - 理由：列表與詳細頁零檔案 I/O；改關卡重啟即可，需求未要求即時更新
- **JudgeConfig 用程式內常數，不開環境變數覆寫**（architecture.md §3.9）
  - 理由：整合測試可直接建構不同值；目前沒有部署端調參需求

## 2. 判題沙箱

- **Docker 沙箱 `rust-quest-runner:dev`，殼容器 + 兩次 `docker exec`**（architecture.md §3.1）
  - 理由：編譯與執行分開計時、診斷與 stdout 走不同 pipe，不必設計可被偽造的分隔符號
- **超時一律 `docker kill` 整個容器**（architecture.md §3.1）
  - 理由：殺 `docker exec` 的 CLI 不會停掉容器內程序
- **docker.sock 不掛進任何容器；backend 跑在 host**（architecture.md §3.3）
  - 理由：掛 sock 等同交出 host root
- **額外加固參數（nosuid、`--init`、`--log-driver none`、ulimit nofile／core）實作時以整合測試驗證，失敗就拿掉**（architecture.md §3.3）
  - 理由：尚未實測，不能讓未驗證的參數擋住主流程
- **死鎖與無窮迴圈靠執行 timeout（5 秒）終止**（architecture.md §3.5）
  - 理由：33 關刻意製造死鎖，沙箱本來就必須有時間上限

## 3. 併發

- **雙 Semaphore 限制排隊與執行數，滿載回 `503 JUDGE_BUSY` + `Retry-After`**（architecture.md §3.4）
  - 理由：滿載是全站資源不足，不是單一 client 送太多；429 保留給之後的 per-IP 限流
- **Run 與 Submit 共用同一組 permit**（architecture.md §3.4）
  - 理由：兩者成本相近，分開會讓總容器數失控

## 4. 判題結果格式

- **JudgeStatus 七種：`ok`、`compile_error`、`runtime_panic`、`test_failed`、`timeout`、`memory_limit`、`output_limit`**（api.md §0.3）
  - 理由：前端只需一套分支；記憶體與輸出超限的提示文案和常見原因不同，不併入 panic
- **保留 `compile.timed_out` 與 `execution.timed_out`**（api.md §0.3）
  - 理由：同為 `timeout`，前端要區分「編譯太久」與「程式跑太久」
- **stdout／stderr 上限 64 KiB，超過回 `*_truncated = true` 並終止**（api.md §0.3）
  - 理由：無限 `println!` 只丟棄不殺會空轉到 timeout
- **timeout、memory_limit、output_limit 都回傳已截取的 stdout**（api.md §3）
  - 理由：GridWorld 要播放到中斷點
- **後端不回傳任何星等欄位（移除 `star_inputs`）**（api.md §4）
  - 理由：星等條件不含 warning 數，前端已有計算所需的全部資料

## 5. 診斷（Diagnostic）

- **Diagnostic 含 `end_line`、`end_column`**（api.md §0.3）
  - 理由：前端直接對應 Monaco 範圍，不用「單字結尾」推估跨 token 的錯誤
- **Diagnostic 含 `rendered` 原文，primary span 在隱藏區段時為 `null`**（api.md §0.3）
  - 理由：原文有程式碼片段與 `help:` 建議，是最好的教材；隱藏區段的原文會洩漏測試行號
- **判題回傳保留 rustc warning**（architecture.md §3.7）
  - 理由：31 關以 `unused_assignments` 警告為線索
- **過濾 `aborting due to`、`warning(s) emitted` 與 `failure-note`**（architecture.md §3.7）
  - 理由：摘要訊息對玩家沒有資訊量

## 6. 測試結果（TestResult）

- **TestResult 為 `{name, status, message}`，`status` 為 `passed`、`failed`、`not_run`；單一測試不再有 `passed` 布林，頂層 `passed` 保留**（api.md §0.3）
  - 理由：「沒跑到」與「失敗」對玩家意義不同，布林表達不了
- **`tests` 永遠列出全部隱藏測試，依 `tests.rs` 宣告順序；編譯失敗時全部 `not_run`**（api.md §4）
  - 理由：前端從 `tests` 就能算出總數與各狀態數量，畫面穩定；v1.2 起不再回傳 `summary`（§14）
- **`message` 去掉 `thread ... panicked at` 行與 backtrace 提示行**（architecture.md §3.6）
  - 理由：該行含隱藏區段行號與暫存路徑；rustc 1.99 起還多了 thread id
- **預期與實際由前端從 `message` 的 `left:`／`right:` 拆解，不另開欄位**（ux.md §3.3）
  - 理由：libtest 文字輸出沒有結構化的預期值；拆不出來就顯示原文

## 7. 隱藏測試與合併流程

- **隱藏測試模組命名 `__quest_hidden_tests`，由判題器包外層，`tests.rs` 只寫模組內容**（architecture.md §3.7、curriculum.md §1.4）
  - 理由：避免與玩家自己的 `mod tests` 衝突；外層結構集中在一處
- **合併順序：使用者程式碼 → 換行 → 模組開頭 → 共用模擬器（僅 grid）→ `tests.rs` → `}`**（architecture.md §3.7）
  - 理由：使用者程式碼在最前面，行號與編輯器一致，隱藏區段判斷只需比較行數
- **Submit 以 `--exact` 指定隱藏測試全名執行**（architecture.md §3.3）
  - 理由：改名後玩家的 `#[test]` 不再衝突，必須排除才不會影響判定
- **Submit 時測試原始碼可被 `include_str!` 讀到：學習用途接受**（architecture.md §4.3）
  - 理由：防範成本高，對學習情境沒有實質損害；對外開放前重新評估

## 8. Grid 關卡

- **地圖寫在 `level.toml` 的 `[grid]`，不由 stdout 產生**（ux.md §4.2）
  - 理由：地圖是關卡設定，不該讓玩家程式能改寫
- **`[grid]` 採 ux.md 格式（width、height、start、goal、walls、gems、doors）；不採字元 `map` 與 `par`**（ux.md §4.2）
  - 理由：座標陣列可直接驗證；`par` 對應的星等條件已取消
- **座標一律 `[x, y]`，x 為欄、y 為列，從 0 起算**（ux.md §4.2）
  - 理由：與 SVG 繪圖座標一致；05 關的 `delta` 也改用 `(x, y)` 避免混淆
- **指令集為 `MOVE <方向>`、`PICK`、`OPEN <方向>`；不保留 `PICK GEM`、`SAY`**（ux.md §4.3）
  - 理由：以 ux.md 的解析規則為準；非指令行本來就只顯示在主控台，不需要 `SAY`
- **通關＝撿完全部寶石＋停在終點；撞牆等錯誤立即停止**（ux.md §4.2、§4.3）
  - 理由：規則單一，玩家只需專注第一個錯誤
- **指令步數上限 1,000 步，前端與判題一致**（ux.md §4.3、curriculum.md §1.5）
  - 理由：避免上萬個動畫步驟凍結分頁；判題同規則才不會「畫面失敗、判題通過」
- **共用 `simulate()` 抽到 `content/shared/grid_sim.rs`，判題時併入隱藏測試模組**（curriculum.md §1.5）
  - 理由：每關複製一份會漂移；前端解析器只需對齊一份
- **grid 關卡的 `level_grid()` 由判題器依 `[grid]` 產生，`tests.rs` 不重寫關卡地圖**（curriculum.md §1.4、§1.6，architecture.md §2.2、§3.7）
  - 關卡地圖以外的測試專用地圖可自寫（19 關 `grid_from`），不違反單一來源（v1.2 釐清）
  - 理由：地圖只有一個來源；原本的內容測試只證明 solution 在兩張圖都能通關，抓不到解答路線以外的差異（見 §12 ①）
- **Submit 不驅動 GridWorld；可視化只吃 Run 的 stdout**（ux.md §4.1）
  - 理由：Submit 的 stdout 是 libtest 輸出，TestResult 也不帶 stdout

## 9. Memory 關卡

- **memory_snapshots 由關卡作者手寫，型別以 ux.md 的 `MemorySnapshot` 為準**（ux.md §5.1、curriculum.md §5）
  - 理由：fix-compile 關卡的 starter 無法執行，只能手寫
- **錯誤訊息、Rc 計數、RefCell 借用狀態寫進文字欄位（`note`、heap `value`）**（curriculum.md §5.3）
  - 理由：這些只需要讀，不需要在圖上畫
- **錯誤步驟以 `error: { code, target }` 標出出事位置，MemoryView 畫錯誤標記**（ux.md §5.1、§5.3，curriculum.md §5.2）
  - 理由：錯誤那一步是整關的重點，只寫在 `note` 時圖上看不出來（見 §12 ②）
- **陣列索引即步數，不帶 `step`；不區分 starter／solution 來源**（curriculum.md §5.2）
  - 理由：與 ux.md 型別一致；修正後的步驟以 `note` 開頭「（修正後）」標示

## 10. 進度、星等與解鎖

- **星等三星各自判定：通過、沒看提示、第一次 Submit 就過；前端計算**（ux.md §6.1）
  - 理由：每顆星代表一項成就，取歷史聯集不會因重玩消失
- **星等不加「指令數」「編譯次數」「修改行數」條件**（ux.md §6.1、curriculum.md §6.3）
  - 理由：後端無狀態記錄不到，且會鼓勵玩家為分數而非理解而改
- **計入失敗的 Submit：`compile_error`、`test_failed`、`timeout`、`memory_limit`、`output_limit`；503、413 與網路錯誤不計**（ux.md §6.1）
  - 理由：系統問題不是玩家的錯
- **關卡沿用正式編號，依 `(chapter, order)` 排序；解鎖照 ux.md §6.3**（api.md §1）
  - 理由：MVP 只做部分關卡，正式編號讓之後補關不必重排
- **草稿自動儲存，每關一個 key `rustquest.draft.<levelId>`，debounce 500 ms**（ux.md §6.6）
  - 理由：重新整理不丟程式碼；獨立 key 避免每次按鍵重寫整份進度

## 11. 需求外不做

- **JudgeConfig 環境變數覆寫、關卡 hot reload、上一步按鈕、欄寬拖曳、ASCII 地圖、隱藏關**
  - 理由：需求未要求，之後加不影響既有資料結構
- **共用模擬器 `assert_cleared` 不加 `#[track_caller]`**（見 §12 ③）
  - 理由：panic 位置行回傳前就被去掉，玩家看不到
- **錯誤圖鑑排入後續版本，MVP 不做**（見 §12 ④）
  - 理由：MVP 只涵蓋 4 種錯誤碼，收藏感撐不起來；之後加只多一個 localStorage key

## 12. 決策辯證

使用者授權由決策 agent 正反辯證後拍板。每題格式：正方、反方、自我反駁、拍板。

### ① 判題器由 `[grid]` 自動產生 `level_grid()`

- **正方**
  - 單一來源：地圖只寫在 `level.toml`，整類「畫面與判題地圖不同」的 bug 直接消失
  - 原本的防漂移檢查太弱：「solution 在兩張圖都通關」抓不到解答路線外的差異（例如 `tests.rs` 多一道牆、少一扇門），玩家換條路線就會「畫面通關、判題失敗」
  - 要做「強」檢查，本來就得從 `[grid]` 產生 Rust 程式碼來比對；產生器既然要寫，直接拿來用最省
  - 現在加 vs 之後加：後端與 `content/` 都還沒寫，現在只改規格；之後加要改每個 grid 關卡的 `tests.rs`（MVP 3 關、全課程十多關）
- **反方**
  - 合併流程多一段程式碼產生，違反原決策「保持合併流程單純」
  - 關卡作者在 `tests.rs` 看不到地圖，要另開 `level.toml`
  - 產生器與 `grid_sim.rs` 的 `Grid` 欄位綁在一起，改結構要兩邊一起改
- **自我反駁**
  - 產生器是純函式，約 20 行，啟動時產生一次存進 `Level`，Submit 只做字串串接，熱路徑零成本
  - 產生的程式碼只放在隱藏區段，不影響使用者行號對應
  - 欄位綁定由內容測試兜底：`Grid` 改了而產生器沒改，所有 grid 關卡 solution 會編譯失敗
  - 座標經 loader 驗證為 0–11 的整數，產生程式碼沒有注入風險
  - 以 MVP 4 個 grid 關卡（01、02、05、19）實測：產生的 `level_grid()` 併入後 solution 全數通過、無警告；19 關用多張地圖沒用到它，`#[allow(dead_code)]` 已涵蓋
- **拍板：做**。單一來源比「兩份 + 弱檢查」可靠，現在改成本最低
- **影響**
  - curriculum.md §1.3、§1.4（不手寫 `level_grid`、保留名稱加入 `level_grid`）、§1.6、§4.1–4.3 的 `tests.rs`、§4.6 註記、§7
  - architecture.md §2.2（loader 啟動時產生）、§3.7 合併檔組成 ④、§6.2 單元測試與內容測試
  - PLAN.md：階段 3a 新增 B1a、B4a

### ② MemorySnapshot 加結構化 `error` 欄位

- **正方**
  - MVP 三個 memory 關卡（09、11、28）的高潮都是「錯誤那一步」，現在那一步的圖和前一步幾乎一樣，只有 `note` 文字不同
  - curriculum.md §6.5「錯誤要有畫面」是既定原則，現有型別做不到：用 `pointers` 硬畫會被誤認為合法借用
  - 從 `note` 解析錯誤碼與目標太脆弱，文字改寫就壞
  - MemoryView（F5）還沒做，現在把錯誤標記納入第一版，比做完再回頭改元件與 e2e 便宜
- **反方**
  - 原決策「ux.md 型別已定，額外欄位沒有元件會讀」
  - 四份規格、loader 驗證、MemoryView、TS 型別都要動，MVP 範圍變大
  - 欄位是選填、向下相容，之後再加成本差不多
- **自我反駁**
  - 「沒有元件會讀」的前提已改變：這次同時規定 MemoryView 要畫，欄位有讀者
  - 向下相容只代表資料不用遷移；MVP 正是驗證「有不有趣、學不學得會」的版本，錯誤那一步沒畫面就等於驗證打折
  - 範圍收到最小：只有 `code` 與 `target` 兩個欄位，訊息仍留在 `note`；箭頭標紅由「`target` 變數出發的指標」推導，不另開指標欄位
  - 三關的 `target` 都能明確指出：09 被 move 後又用的 `report.bag`、11 懸空的 `first_then_add.first`（它的箭頭轉紅）、28 還沒歸還的 `deposit.view`（它的箭頭轉紅）
- **拍板：做（MVP）**，形狀 `error: { code: string | null, target: string } | null`
- **影響**
  - ux.md §5.1 型別、§5.3 錯誤標記視覺編碼、§9
  - api.md §0.3 `MemorySnapshot` 說明與範例、§2 範例、§6 TS 型別
  - curriculum.md §5.2、§5.3、§5.4、§4.4／§4.5／§4.7 錯誤步驟補 `error`、§7
  - architecture.md §2.2 loader 驗證、§6.2 loader 單元測試
  - PLAN.md：階段 2 新增 `ui_memory.feature` 錯誤標記情境；階段 3a B1a；階段 3b F5a

### ③ 共用模擬器 `assert_cleared` 加 `#[track_caller]`

- **正方**
  - 一行屬性，成本幾乎為零
  - 斷言失敗時 panic 位置指向呼叫端，而不是 `grid_sim.rs` 內部
- **反方**
  - 位置資訊沒有讀者：architecture.md §3.6 回傳前就去掉 `panicked at <路徑>:<行>:<欄>` 這行，玩家看不到
  - 實測（rustc 1.99.1）只對 `assert_eq!` 有效；最常見的失敗是 `simulate()` 回錯誤，由 `unwrap_or_else` 的 closure 內 `panic!`，位置仍指向 closure（closure 的 `track_caller` 尚未穩定）
  - 要真正生效得改寫 `assert_cleared` 的控制流，換來的仍是會被去掉的資訊
- **自我反駁**
  - 關卡作者本機除錯可能想看位置；但測試名稱已唯一對應到失敗的測試，MVP 每個測試只呼叫一次 `assert_cleared`
- **拍板：不做**。資訊在回傳前就被去掉，且對主要失敗路徑無效
- **重新評估時機**：判題改為回傳 panic 位置，或出現單一測試多次呼叫 `assert_cleared` 的關卡

### ④ 錯誤圖鑑（收集遇過的 rustc 錯誤碼）

- **正方**
  - 把「被編譯器擋下」從挫折轉成收藏，貼合「編譯器是老師」的 Rust 精神
  - 資料已在手：`Diagnostic.code`，前端記錄即可，後端不用改
- **反方**
  - MVP 8 關的設計錯誤只有 E0384、E0596、E0382、E0502 四種，收藏冊幾乎是空的，反而顯得單薄
  - 「收集任何遇過的錯誤」會鼓勵故意寫錯刷圖鑑，與星等「不為分數而改」的原則衝突（§10）
  - 要新頁面、路由、localStorage 結構、每個錯誤碼的繁中說明文案；單題學習價值已由診斷卡片的錯誤碼、`explanation_url` 與「翻譯成人話」（ux.md §3.4）提供
- **自我反駁**
  - 收藏的價值會隨關卡數成長：全課程設計錯誤約 14 種（E0384、E0596、E0308、E0004、E0382、E0502、E0499、E0106、E0515、E0597、E0369、E0277、E0072、E0373），那時才撐得起收藏
  - 之後加的成本：新增獨立 localStorage key，不必遷移既有進度；唯一損失是 MVP 期間的遭遇紀錄，MVP 沒有正式玩家，可忽略
- **拍板：排入後續版本**。MVP 收藏內容太少且有刷錯誘因，延後加不增加成本
- **後續設計方向**：只收錄「通關關卡時該關設計的錯誤碼」，不收任意遭遇的錯誤，避免刷圖鑑
- **觸發條件**：第 3 章以後的關卡補齊、可收錄的錯誤碼達 10 種以上

## 13. 實作必要參數

驗收員列出規格中「不在已定決策」的設定值；以下逐項判定為實作必要而保留（安全上限、錯誤處理、規格要求的功能必需），刪除的項目見 §14 ④。

- **request body 上限 128 KiB**（architecture.md §2.2）
  - 理由：在反序列化前擋掉超大 body；`code` 64 KiB 加 JSON 跳脫後仍放得下
- **編譯 stderr 上限 1 MiB**（architecture.md §3.5）
  - 理由：rustc 錯誤輸出可被巨集刻意灌大，沒上限會吃光 backend 記憶體
- **診斷最多回傳 50 筆**（architecture.md §3.7）
  - 理由：限制回應大小與前端卡片數；玩家通常只需修前幾個錯
- **health Docker 檢查快取 10 秒**（architecture.md §3.10）
  - 理由：每次 health 都跑兩次 docker CLI，未快取時可被輕易打成 DoS
- **排隊等待上限 10 秒**（architecture.md §3.4）
  - 理由：已定決策「排隊等候超時回 503」需要一個數值；10 秒內前端仍顯示判題中
- **容器啟動 timeout 10 秒**（architecture.md §3.5）
  - 理由：daemon 卡住時 `docker run` 可能無限等待，必須有上限才能回 `JUDGE_UNAVAILABLE`
- **容器最長存活 60 秒（`sleep 60`）**（architecture.md §3.1）
  - 理由：backend 當掉時的最後保險，60 秒涵蓋編譯 15 秒＋執行 5 秒＋啟動 10 秒仍有餘裕
- **`max_concurrent = max(1, CPU 核數 / 2)`、`max_queue = max_concurrent × 4`**（architecture.md §3.9）
  - 理由：已定決策「雙 Semaphore」需要預設值；每個判題吃滿 1 核，留一半給 host 與 Docker daemon
- **容器 label `rust-quest.judge=1`**（architecture.md §3.1）
  - 理由：啟動時清理殘留容器只能靠 label 篩選，不會誤刪其他容器
- **Run 與 Submit 編譯都加 `-A dead_code`**（architecture.md §3.3；v1.3 起 Run 也加）
  - 理由：`--test` 模式下 `main` 與只被它呼叫的函式都算未使用；Run 時關卡骨架常有尚未用到的函式（05 關正確解答），兩種模式警告一致
- **`Retry-After: 2`（僅 `JUDGE_BUSY`）**（api.md §0.2）
  - 理由：已定決策「滿載回 503 + Retry-After」需要數值；2 秒約為一次判題時間
- **`errorHints` 錯誤碼字典，首批 MVP 4 碼＋E0499、E0308、E0425、E0599**（ux.md §3.4）
  - 理由：§12 ④ 延後錯誤圖鑑的理由之一就是卡片已有「翻譯成人話」；常見碼取自玩家最容易打錯的情況
- **`Prefs.theme`（深淺色偏好）**（ux.md §6.5、§8.3）
  - 理由：深淺色切換要能跨重新整理保留；風格已定為海洋繪本，不另存風格欄位（§14 ⑤）
- **loader 拒絕未知 TOML 欄位、`id` 須等於目錄名**（architecture.md §2.2）
  - 理由：拼錯欄位名會被默默忽略而產生錯誤關卡；`id` 同時是 URL 與草稿 key，必須與目錄一致才不會對錯檔

### v1.3 補列（第二輪驗收列出的需求外新增項目，判定保留）

- **`content/chapters.toml` 與 `ChapterSummary`（`GET /api/levels` 的 `chapters`）**（api.md §1、architecture.md §2.2）
  - 理由：頂部列與章節地圖要顯示章節名稱（第一輪阻擋缺口），沒有來源就只能寫死在前端；已於 §14 ① 定案
- **loader 其餘驗證規則**：`id` 格式 `^[0-9]{2}-[a-z0-9-]+$`、`order` 全域唯一、`title`／`story`／`concept` 非空、`hints` 每則非空、`chapter` 須在 `chapters.toml`、`error.code` 格式、`line ≥ 1`（architecture.md §2.2）
  - 理由：每條都對應一種「啟動成功但畫面或判題錯亂」的內容錯誤；fail fast 比玩家遇到才發現便宜，且全是啟動時一次性成本
- **ux 設定值**：`Retry-After` 缺漏或非整數時 10 秒、請求超過 3 秒的慢提示、錯誤卡片先展開 5 筆、播放速度 0.5x–4x 與 1x＝500ms、前端請求逾時 60 秒（ux.md §3.2、§3.3、§3.4、§4.4）
  - 理由：BDD 需要可斷言的確切數值；60 秒見 §15 ②
- **`JudgeConfig.docker_bin`**（architecture.md §3.9）
  - 理由：不需 Docker 就能測「daemon 無法連線」，原本這組情境無法在 CI 穩定重現（§15 ①）
- **刪除：`@e2e-real` 標籤**
  - 理由：打真後端的前端情境本來就標 `@docker`，兩個標籤篩的是同一組情境，只留 `@docker`

## 14. v1.2 驗收缺口定案

使用者授權架構師自行正反比較後選優點多者；每條一行結論＋一行理由。

### ① 驗收員阻擋級與文字落差

- **章節名稱放 `content/chapters.toml`，由 loader 載入，`GET /api/levels` 回傳 `chapters`（只列有關卡的章節）**（api.md §1、architecture.md §2.2、curriculum.md §1.7）
  - 理由：比前端寫死多一個檔案，但名稱與關卡同在 `content/`，補章節不用改前端；不另開端點，地圖本來就要打這支
- **Ferris 對話泡泡內容＝`story`，整段一個泡泡；MVP 8 關 `story` 改寫為 Ferris 第一人稱；刪除 `ferris` fenced block**（ux.md §1.4、curriculum.md §1.8）
  - 理由：不新增欄位與解析規則；`story` 本來就是 3–4 句短文，拆泡泡與「下一句」換不到教學價值
- **MemoryView 是教材：進關即顯示第 1 步可播放，不受任何 Run／Submit 結果影響；可視化空白規則只套用 GridWorld**（ux.md §3.3、§5.5）
  - 理由：09、11 的 starter 必定編譯失敗，若依賴 Run 玩家永遠看不到快照；快照本來就與玩家程式碼無關
- **確認 architecture.md §6.2 內容測試全部可行；「Run stdout 套 `[grid]` 須通關」改用 `include!` 引入 `grid_sim.rs` 實作**（architecture.md §6.2 ④）
  - 理由：同一份模擬器、同一份 `[grid]`，不用寫第二個解析器
- **本次遊玩只在 Submit 通過時結束；回地圖、重新整理、直接輸入網址、重設都不歸零**（ux.md §6.1）
  - 正方：未通關前無法以任何操作繞過 ★2／★3，規則只有一個結束點，store 不必追蹤「從哪裡進來」
  - 反方：玩家卡關很久想「重新開始」也拿不回 ★2
  - 拍板：採用；★2／★3 本來就是「這次有沒有看提示」，卡關看過提示就不該拿，通關後重玩即可補星
  - 已知限制：通關後重玩可直接送出保留的草稿補星；進度在玩家本機，不做防弊
- **31 關 starter 警告為 2 筆（`unused_variables`、`unused_assignments`），curriculum 文字依 rustc 1.99.1 實測修正**（curriculum.md §4.8）
  - 理由：api.md §4 範例本來就是 2 筆且正確，以實測為準
- **api.md「錯誤落在隱藏測試區段」範例改為 05 關刪除 `delta`：4 筆 E0425 全在隱藏區段**（api.md §4）
  - 理由：玩家 `main`／`route` 不呼叫 `delta`，錯誤只會出現在隱藏測試；已在本機 rustc 與 `rust-quest-runner:dev` 實測
- **`phase` 統一為 `'idle' | 'running' | 'submitting'`，ux 狀態機同名；映像名 `rust:1.99.1-slim`；前端指令用 npm**（architecture.md §1.1、§6.3、§7.2，ux.md §3.1）
  - 理由：對齊實際檔案（`runner/Dockerfile`、`frontend/package-lock.json`）
- **Runner Dockerfile 以實物為準改規格（useradd uid 1000、`WORKDIR /tmp`，不設 `HOME=/tmp`）**（architecture.md §3.2）
  - 理由：實物已實測可用；v1.2 再於 `--read-only` 下確認 rustc 不寫 `HOME`，改規格比改映像少一次重建與重測

### ② 次要模糊

- **錯誤卡片位置欄：`line = null` 且 `rendered = null` 顯示「隱藏測試」，`rendered` 有值顯示「無位置」**（ux.md §3.4、api.md §0.3）
  - 理由：沿用既有欄位語意即可區分，不新增旗標欄位
- **關卡編號顯示為 `章-章內序`，章內序＝同章已載入關卡依 `order` 的位置**（ux.md §6.4）
  - 正方：不加欄位；反方：之後補關會讓編號改變
  - 拍板：採用；編號只供顯示，進度與路由都用 `id`
- **`docker exec` 回 137 且非 host kill 即判 `memory_limit`，編譯與執行兩段都適用；判定順序改以階段描述**（architecture.md §3.5、§3.6）
  - 理由：rustc 被 OOM 殺掉時沒有正常錯誤輸出，判成 `compile_error` 會讓玩家找不到錯
- **被終止的程式 `exit_code` 一律 `null`（含 OOM），不原樣回 137**（api.md §0.3）
  - 理由：單一規則「被 kill 就是 null」；是哪種終止由 `status` 表達
- **編譯 OOM：`status = memory_limit`、`compile.success = false`、`execution = null`、Submit 的 `tests` 全 `not_run`**（api.md §0.3）
  - 理由：與編譯超時的回應形狀一致，前端用 `execution === null` 判斷是編譯階段
- **`output_limit` Run 與 Submit 共用判定，規格註明 Submit 實務上極少觸發，BDD 只寫 Run**（api.md §0.3、architecture.md §3.6）
  - 理由：判定程式只有一套；硬性排除 Submit 反而要多寫分支
- **loader 驗證範圍明列：`id` 等於目錄名、`hints` 可省略或為空、`frames`／`heap`／`pointers`／`vars` 省略為 `[]`、拒絕未知欄位**（architecture.md §2.2）
  - 理由：省略即空陣列讓 TOML 簡潔；boss 關設計上可不給提示

### ③ 前端行為（BDD 分析師提出）

- **請求失敗分四種：`server_busy`、`judge_unavailable`、`payload_too_large`、`network_error`（含其他 4xx／5xx）**（ux.md §3.1、§3.3，api.md §0.2）
  - 理由：413 是玩家可自行修正的情況，值得專屬文案；其他 4xx 是前端程式錯誤，玩家看到同一句即可
- **主控台預設分頁只看結果狀態：`compile_error` → 編譯錯誤、`test_failed` → 測試結果，不看 kind**（ux.md §1.3）
  - 理由：build 關卡編譯失敗時測試全是「未執行」，切到測試結果沒有資訊
- **`server_busy` 倒數時「執行」「送出」一起 disabled 並各自倒數；`judge_unavailable` 不倒數**（ux.md §3.3）
  - 理由：判題忙碌是全站狀態，兩顆按鈕打同一組 permit；unavailable 不附 `Retry-After`，玩家自行重試
- **步數上限文案統一為「指令超過 1,000 步，後面的不播放了」**（ux.md §4.5、architecture.md §7.5）
- **補齊 GridWorld 訊息：到終點但寶石沒撿完、動詞不明（`JUMP`）、`PICK` 多接參數（`PICK GEM`）**（ux.md §4.5）
  - 理由：BDD 需要可斷言的確切文字
- **PlaybackBar「從頭」：GridWorld 回第 0 步（初始地圖），MemoryView 回第 1 步**（ux.md §4.4）
  - 理由：MemoryView 沒有「執行前」的快照
- **章節累計星數分母＝該章已載入關卡數 × 3**（ux.md §6.3）
  - 理由：MVP 只載入部分關卡，用全課程關卡數會永遠拿不滿
- **「重設」先彈確認；程式碼已等於 `starter` 時不彈**（ux.md §6.6）
  - 理由：重設會刪草稿且無法復原，誤按代價高；確認對話框成本低
- **解鎖的「前一章」只算有已載入關卡的章節**（ux.md §6.3）
  - 理由：MVP 沒有第 4、6 章關卡，否則第 5、7 章永遠鎖住

### ④ 範圍紀律：刪除項目

判定原則：重複、可由其他欄位推算、或沒有具體使用情境者刪除。

- **api**
  - health 的 `version`、`levels_loaded`、`judge.docker_server_version`、`runner_image`、`max_concurrent`、`in_flight`、`queued`：前端只用 `status` 與 `judge.available`／`reason`
  - `compile.duration_ms`、`execution.duration_ms`：唯一讀者是「耗時」顯示，也一併刪除；且非決定性數值會讓 BDD 範例無法精確比對
  - `summary`：前端可由 `tests` 計數
  - `error.details`：前端只依 `code` 分支，等候秒數已在 `Retry-After` header
  - `ROUTE_NOT_FOUND`、`METHOD_NOT_ALLOWED`：前端不會觸發，沿用 axum 預設
  - `UNSUPPORTED_MEDIA_TYPE`：併入 `400 INVALID_REQUEST`，前端一律送 JSON，多一個碼沒有分支價值
  - `JUDGE_UNAVAILABLE` 的 `Retry-After: 10`：前端改為不倒數
- **architecture**
  - `-C opt-level=0 -C debuginfo=0`：即 rustc 預設值，寫出來沒有作用
- **ux**
  - `LevelProgress.cleared`：與 `stars.pass` 重複
  - `Prefs.playbackSpeed`：速度只在記憶體保留，重新整理回 1x 沒有實害
  - 開場對話、「繼續上次進度」按鈕、章節完成過場與徽章、Ferris 反應台詞池：沒有資料來源，或與地圖自動捲動、主控台卡片重複
  - `Ctrl+.`、`?` 快捷鍵：提示與說明面板都有按鈕；保留說明面板（寫 `Ctrl+M` 可及性說明）
  - GridWorld `step-change`／`finished` 事件與 `GridResult`：沒有外部讀者；連帶刪除「主控台 stdout 與播放步同步」

### ⑤ 視覺風格

- **正式前端只做「海洋繪本」風格（POC B），淺色深色兩套；不做風格切換器，`Prefs` 只保留 `theme`（深淺色）**（ux.md §8、§6.5）
  - 理由：使用者已在 POC 上選定 B，多風格切換只剩維護成本

### ⑥ BDD 同步後補充（主控）

- ux §1.1 ⑤ Submit 失敗的分頁改為依結果狀態，對齊 §3.3
  - 理由：§1.3、§3.3 已定 compile_error 一律切編譯錯誤分頁
- errorHints 模板的反引號片段渲染為行內程式碼
  - 理由：與錯誤卡片原文層的程式碼呈現一致，測試以 `<code>` 元素文字比對
- 「程式碼已修改」提示只在程式碼等於 starter 時消失，Run 不影響
  - 理由：快照對照的是 starter 行號，Run 不改變程式碼與快照的差異

## 15. v1.3 收尾定案

第二輪驗收 13 項次要問題與工程師第一棒疑義。使用者授權架構師自行正反比較後選優點多者；每條一行結論＋一行理由。

### ① 判題器（主控裁定）

- **`JudgeConfig` 加 `docker_bin`（預設 `"docker"`）；測試設為不存在路徑模擬 daemon 無法連線，相關情境不標 `@docker`**（architecture.md §3.9、§3.10）
  - 理由：不必真的停掉 daemon 就能測 `docker_unreachable`，CI 可穩定跑
- **「映像不存在」需要真的 daemon，歸 Docker 整合測試層並標 `@docker`**（architecture.md §6.2 ③）
  - 理由：沒有 daemon 時分不出「映像不存在」與「連不上」，以 feature 為準
- **Run 編譯也加 `-A dead_code`**（architecture.md §3.3）
  - 理由：05 關正確解答在 Run 時會冒出 dead_code 警告，與 Submit 不一致
- **檢查順序：body 驗證 → 關卡查找 → 排隊名額 → 執行 permit → Docker 可用性**（architecture.md §1.4、§3.4，api.md §0.2）
  - 理由：與 axum extractor 順序一致；滿載時不必再打 docker CLI，回 `JUDGE_BUSY` 的 `Retry-After` 對玩家也較有用
- **Submit 結果不完整一律 `test_failed`：exit 0 但缺結果、或 exit 非 0 但無 `failed` 者，缺結果的測試為 `not_run`；頂層 `passed` 只在全部 `passed` 時為真**（architecture.md §3.6、api.md §4）
  - 理由：不讓 harness 崩潰或輸出被截斷時誤判通過；timeout／OOM／output_limit 仍依原優先順序
- **隱藏模組開頭定為五行（加 `#[allow(unused_imports)]`）；Submit 時隱藏區段的 warning 過濾不回傳，error 照舊回傳且位置為 `null`**（architecture.md §3.7、curriculum.md §1.6）
  - 理由：玩家無法修正隱藏區段，顯示其警告只會造成困惑；隱藏區段行號因此不寫死（api.md §4）
- **`user_line_count = user_code.lines().count()`；程式碼不以換行結尾才補一個換行，結尾空行原樣保留**（architecture.md §3.7）
  - 理由：定義與 Rust 標準函式一致，單元測試可直接比對
- **`tests.rs` 不以換行結尾時，在結尾 `}` 前補一個換行**（architecture.md §3.7 ⑤）
  - 理由：最後一行若是註解會把 `}` 吃掉，造成整個合併檔編譯失敗
- **進度第一次進站（沒有資料）不 toast，只有既有資料損毀或讀取失敗才 toast**（ux.md §6.5）
  - 理由：新玩家不該一進站就看到錯誤提示
- **`level_grid()` 產生單行，欄位順序同 `grid_sim.rs` 的 `Grid`**（architecture.md §3.7 ④）
  - 理由：格式固定才能在 B1a 單元測試釘死字串
- **docker 參數每個旗標與值拆成獨立元素；`-v` 值為 `<dir>:/work:ro`；容器名為 `rq-judge-` 加唯一後綴**（architecture.md §3.1、§3.3）
  - 理由：安全回歸測試可逐元素斷言；不經 shell 拼接

### ② 前端行為

- **前端 Run／Submit 請求逾時 60 秒，逾時歸 `network_error`**（ux.md §3.2、architecture.md §7.2）
  - 正方：判題最壞約 40 秒（排隊 10＋啟動 10＋編譯 15＋執行 5），60 秒不會誤殺正常請求
  - 反方：30 秒較快給回饋；但會誤殺排隊加冷啟動的合法請求
  - 拍板：60 秒；誤殺會讓玩家重送、加重滿載，代價大於多等 20 秒
- **平板抽屜「已自動展開過」與手機「首次進關」只記在記憶體（Pinia），重新整理後重置**（ux.md §2.2、§2.3）
  - 正方：不新增 localStorage 欄位；重新整理後再看一次故事沒有實害
  - 反方：存進 localStorage 可跨重新整理記住，但多一個持久欄位與損壞處理
  - 拍板：只存記憶體
- **零寬度 span 的 marker：擴到該位置 token 結尾；沒有 token 至少 1 字元；非空行行尾改標前一字元**（architecture.md §7.4、ux.md §3.3）
  - 理由：Monaco 不畫零寬 marker，缺分號這類常見錯誤會變成看不到的波浪線
- **panic marker 標整行**（ux.md §3.3、architecture.md §7.4）
  - 理由：panic 位置的欄位指向巨集呼叫點，標整行最不易誤導
- **撞門訊息方向取撞門那一步 `MOVE` 的方向**（ux.md §4.5）
  - 理由：門必在該方向相鄰格，不需另算；原文寫死 `RIGHT`
- **播放控制列步數一律「第 n / N 步」**（ux.md §4.4、§5.2）
  - 理由：GridWorld 與 MemoryView 共用 `PlaybackBar`，格式不該分兩套
- **GridWorld 加 `pending` prop：請求中回第 0 步並半透明；Submit 回來還原到上次 Run 的終點**（ux.md §3.2、§4.1）
  - 理由：Submit 不驅動 GridWorld，不還原會讓玩家看不到上次 Run 的結果
- **MemoryView 掛載時同步第 1 步行號；播完保留最後一步行高亮，離開關卡才移除**（ux.md §5.4）
  - 理由：播完立即清掉讓玩家對不上最後一步說的是哪一行
- **看提示與重設確認用自訂 `ConfirmDialog`（modal、焦點移入、Esc 取消），不用 `window.confirm`**（ux.md §6.2、§6.6）
  - 理由：原生對話框無法套海洋繪本風格，e2e 也較難穩定操作
- **跨章鎖定 tooltip 的「前一關」＝前一章第一個未通過的關卡**（ux.md §1.1）
  - 理由：跨章解鎖條件是整章通過，指出第一個缺的關卡最可行動
- **413 只看狀態碼；`Retry-After` 只接受整數秒，其餘用 10 秒**（ux.md §3.3、api.md §0.2）
  - 理由：413 可能由 body limit 層回應而非 JSON；後端只送整數秒，不必解析 HTTP 日期
- **`{name}` 抓不到時「這個值」以純文字呈現，不包 `<code>`**（ux.md §3.4）
  - 理由：「這個值」不是程式碼，包成 `<code>` 會誤導
- **Submit 超時的主控台預設分頁為「輸出」**（ux.md §3.3）
  - 理由：超時卡片與部分輸出在「輸出」分頁；測試結果可自行切換

### ③ 文字落差

- api.md §2 範例 `story` 改第一人稱，與 `content/levels/01-*/level.toml` 一致
- curriculum.md §6.3 ★3 改為「本次遊玩沒有失敗的 Submit」，與 ux.md §6.1 一致
- api.md §4 隱藏區段範例刪除寫死的行號
- architecture.md §3.1 ⑤ Submit 執行指令補 `--exact …`，與 §3.3 一致
- `ui_progress.feature`「重設並確認」前先修改程式碼，否則不會出現確認
- `ui_editor.feature` 行尾情境改用 01 starter 第 4 行實際長度（41 字元）


### ⑦ v1.3 後補充（主控）

- 平板「未讀提示 badge」不做
  - 理由：語意未定義且無學習效益，抽屜按鈕本身已可見
- 手機「結果」分頁小圓點只在結果到達時使用者不在該分頁才出現
  - 理由：Run 預設自動切到結果，小圓點只用來提醒切走的使用者
- Submit 的 memory_limit 與 output_limit 預設分頁為「輸出」
  - 理由：與 timeout 一致，玩家需先看到被終止前的輸出

### ⑧ 前端第二棒補充（主控）

- 保留前端補上的狀態文案：主控台空狀態三句、地圖標題 Rust Quest、地圖載入失敗、載入中、一次就過提示
  - 理由：屬規格狀態的必要畫面文字，沒有就會出現空白區塊
- Monaco 改用 editor.api 只註冊 rust
  - 理由：整包引入使 dist 多出未使用的 worker（ts.worker 約 6.9MB）
- 地圖頁不加深淺色切換鈕
  - 理由：規格只放在關卡頁頂部列，地圖頁沿用已存偏好

### ⑨ 對比度微調（主控）

- 依 WCAG AA 量測調整 ux §8.2 token：淺色 text-muted、heading、info、brand、brand-hover、danger、warning、success、neutral-busy；深色 neutral-busy
  - 理由：淺色原值 14 組不達 4.5:1，送出按鈕白字僅 2.56；brand 改深鏽橘讓白字與橘字兩用都達標，Ferris 身體色不變

### ⑩ F8 後補充（主控）

- errorHints 抓不到名稱時，「這個值」與中文相鄰的空白去掉
  - 理由：中文句中不需空白分隔，feature 文案即為無空白
- MemoryView 改上下排列的門檻改為可視化區容器寬度 360px（原 480px）
  - 理由：桌機 1280 寬時可視化區約 394px，480px 門檻會讓桌機也上下排列、Heap 需捲動
- 前端開發模式掛 `window.__rqMonaco` 供 e2e 讀 Monaco 狀態，正式 build 移除
  - 理由：marker、裝飾、可見範圍無法從 DOM 穩定讀取

### ⑪ 最終驗收後補充（主控）

- 內容測試加入取巧反例：每個取巧解答 Submit 必須不通過
  - 理由：只驗「solution 過、starter 掛」無法保證玩家必須用到該關觀念
- 新增 channel build 關為 32，最終 Boss 改為 33
  - 理由：mpsc::channel 首次出現在最終 Boss，難度跳太大
- `dyn Trait` 併入 22 關、Send/Sync 斷言併入 30、31 關，不另開新關
  - 理由：避免大量重新編號，觀念放在最相關的關卡
- 第 8 章並行可視化、trait debug 關、HashMap／Display 排入後續版本
  - 理由：需要新的 visual 類型或新章節，超出本次修正範圍
- 33 關 Submit 改用測試內看門狗偵測死鎖，Run 仍以 timeout 收場
  - 理由：Submit 能在數秒內逐測試給出「互等」提示，Run 保留先看主控台的除錯教學
- 33 關「修好鎖順序但 plan 不開執行緒」列為已知殘留
  - 理由：channel 已在 32 關強制練習，再抓需靠計時、測試不穩
- 取巧反例必須能編譯，靠測試斷言擋下才算有效
  - 理由：因介面不符而編譯失敗的反例無法證明測試能抓到邏輯取巧
- 33 關「全域大鎖」視為取巧擋下
  - 理由：本關教的是逐資源上鎖並固定順序，不同窄橋不該互相等待
- 30 關以 `Box::leak` 回傳 `&'static Map` 列為已知殘留
  - 理由：要擋需追蹤記憶體洩漏，測試成本過高

### ⑫ 最終驗收第二輪後補充（主控）

- 判題編譯加 `-F unsafe_code`（Run 與 Submit）
  - 理由：所有關卡皆不需 unsafe，禁止後可擋 `unsafe impl Send`、transmute 類取巧
- 33 關補測試：分身必須同時持有路徑上的所有橋（外部佔住橋 1 時，分身不得先放開橋 0）
  - 理由：「每座橋鎖完就放」「只鎖第一座」繞過固定鎖順序觀念
- 32 關補「慢螃蟹」測試
  - 理由：sleep 後 try_iter 可同時繞過 recv_timeout 與 drop(tx)
- 已知殘留：31 分兩次上鎖不加 sleep（競態偵測不穩定，改在 hints 提醒原子性）、32 relay 不開執行緒、14 以泛型 `T: AsRef<str>` 回傳輸入
  - 理由：要穩定擋下需依賴計時或過度限制合法寫法
- 31 關「add_haul 已正確修好、gather 直接加總」不算取巧
  - 理由：本關觀念（guard 內原地修改、跨執行緒共享 Mutex）已由測試自開 100 條執行緒呼叫 add_haul 驗證
- 33 關 try_lock 逾時退讓重試（c33_trylock_backoff）列為已知殘留
  - 理由：這是另一種合法的死鎖避免手法，難度高於標準解，寫得出來代表已理解死鎖成因
- 以 `unsafe_code` 診斷碼編譯失敗的反例視為有效（反例第二行標記 `// expect: compile_error unsafe_code`）
  - 理由：`-F unsafe_code` 正是擋下這類取巧的機制，屬 ⑪「反例必須能編譯」的唯一例外
