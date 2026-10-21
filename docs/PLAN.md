# Rust Quest 開發計畫 Checklist

> 斷線續做：新 session 先讀本檔，再讀 `docs/specs/*.md`，從第一個未勾選項目接著做
> 狀態標記：`[x]` 完成、`[~]` 進行中、`[ ]` 未開始
> 角色分工與派工單範本：`docs/roles/README.md`

## 開發方式

```
① SDD 規格
  ↓
② BDD 情境
  ↓
③ TDD 實作
  ↓
④ 內容關卡
  ↓
⑤ 獨立驗收
  ↓
⑥ 提交
```

- ① `docs/specs/` 三份規格互相對齊後定稿
- ② `features/*.feature` 由規格推導，後端用 cucumber-rs、前端用 playwright-bdd 執行
- ③ 每個模組拆兩棒：寫測試跑紅燈 → 實作到綠燈
- ④ 每關附 solution，內容測試驗證 solution 通過、starter 失敗
- ⑤ 另派 agent 驗收，回報異動清單外的新增項目
- ⑥ 切分清單給使用者確認後才 commit

## 已定決策

- 技術：Vue 3 + Vite + TS + Pinia + Monaco；Axum 0.8；Rust 1.99.1
- 關卡一律 `rustc --edition 2024`
- 判題：Docker 沙箱 `rust-quest-runner:dev`，殼容器 + 兩次 `docker exec`（編譯、執行分開計時），超時 `docker kill`
- docker.sock 不掛進任何容器；backend 跑在 host
- 併發：Semaphore 限制排隊與執行數，滿載回 `503 JUDGE_BUSY` + `Retry-After`
- 進度存前端 localStorage，後端無狀態、無 DB、無登入
- Grid 地圖寫在 `level.toml` 的 `[grid]`，不由 stdout 產生
- Grid 通關：撿完全部寶石＋到終點；`OPEN <方向>`；撞牆立即停止
- memory_snapshots 由關卡作者手寫，型別以 ux.md 的 `MemorySnapshot` 為準
- 星等：三星各自判定（通過、沒看提示、第一次 Submit 就過），前端計算
- 隱藏測試模組命名 `__quest_hidden_tests`，避免與玩家 `mod tests` 衝突
- Submit 時測試原始碼可被讀到：學習用途接受
- 需求外已勾選加入：自動存草稿、grid 指令步數上限、Diagnostic `end_line`/`end_column`、`rendered` 原文、stdout 上限＋`truncated` 旗標
- `[grid]` 欄位採 ux.md 格式（width、height、start、goal、walls、gems、doors）；curriculum.md 提案的 `map` 字串與 `par` 不採用
- 關卡沿用正式編號，解鎖依 `order`
- grid 關卡共用的 `simulate()` 抽到 `content/shared/`，判題時併入隱藏測試模組
- 星等不加「編譯次數」「修改行數」條件
- 判題回傳保留 rustc warning（31 關以警告為線索）；死鎖：Run 靠執行 timeout 終止，Submit 由測試內看門狗（recv_timeout）判定失敗並給提示
- JudgeStatus：ok、compile_error、runtime_panic、test_failed、timeout、memory_limit、output_limit；保留 `compile.timed_out`
- TestResult 用 `status`（passed、failed、not_run）取代單一測試的 `passed` 布林
- 額外 docker 參數（nosuid、--init、--log-driver none、ulimit nofile/core）實作時用整合測試驗證，失敗就拿掉
- 交叉核對時的調整全數接受：02 關改 6×2 走廊、19 關加出口、05 關 delta 回傳 (x, y)、移除 SAY 指令與錯誤圖鑑（錯誤圖鑑後經決策辯證改排入後續版本）、Submit 不驅動 GridWorld、編譯失敗時 tests 列全部為 not_run、路由 `/` 與 `/level/:levelId`
- 待決事項由決策 agent 正反辯證後拍板（使用者授權），紀錄見 `docs/specs/decisions.md` §12
  - 做：判題器依 `[grid]` 產生 `level_grid()`，`tests.rs` 不手寫地圖
  - 做（MVP）：MemorySnapshot 加 `error: { code, target } | null`，MemoryView 畫錯誤標記
  - 不做：`assert_cleared` 加 `#[track_caller]`
  - 排入後續版本：錯誤圖鑑，只收錄通關關卡的設計錯誤碼
- 視覺風格：B「海洋繪本」，token 以 `poc/web/src/style.css` 的 B 風格為準，保留深淺色
- `poc/` 永久保留，作為獨立 Demo
- v1.2 定案摘要（詳見 decisions.md §14）：章節名稱來自 `content/chapters.toml`、`GET /api/levels` 回 `chapters`；Ferris 泡泡內容即 `story`；memory 快照進關即可播放；★2/★3 以「本次遊玩」計，僅 Submit 通過時結束；關卡編號「章-章內序」；請求失敗四類 server_busy／judge_unavailable／payload_too_large／network_error
- JudgeConfig 加 `docker_bin` 供測試模擬 docker 不可用；Run 也加 `-A dead_code`
- rustc 原文 message 的反引號照字面保留；只有 errorHints 模板渲染 `<code>`
- MVP 之外的 24 關完整內容以 `content/` 為唯一來源，curriculum.md 保留 §3 摘要（避免兩份漂移）
- CI 只在 PR 合入 main（每季一包）與手動觸發時跑；平常開發以本機兩層測試為主（快速層：cargo test、vitest、vue-tsc；完整層：--ignored、e2e，於驗收時跑）
- commit message 採 `type(scope) 中文摘要`（不加冒號）（2026-10-21 起；main 既有 6 個 `[type]` commit 不改寫）
- timeline 摘要：第 8 章 30、31、33 以 stdout `@tl|` 事件呈現執行緒泳道、鎖持有／等待、訊息箭頭與死鎖（decisions ⑬）；小幫手 content/shared/timeline.rs 由 loader 接在 starter／solution 尾端；一關一種 visual
- 需求外不做：風格切換器、JudgeConfig 環境變數覆寫、關卡 hot reload、上一步按鈕、欄寬拖曳、ASCII 地圖、隱藏關

## 階段 0：環境

- [x] 安裝 Rust 1.99.1（rustfmt、clippy），`source $HOME/.cargo/env`
- [x] Docker 29.7.2 可用，建好 `runner/Dockerfile` → `rust-quest-runner:dev`，實測冷啟動 0.6s
- [x] `git init -b main`、`.gitignore`
- [x] `backend/` Axum 骨架、`frontend/` Vite vue-ts 骨架，build 皆通過
- [x] 前端補裝 `@types/markdown-it`、`dompurify` 改直接依賴

## 階段 1：SDD 規格

- [x] `docs/specs/curriculum.md`：8 章 32 關、MVP 8 關（01、02、05、09、11、19、28、31）完整內容（程式碼未經 rustc 實測）
- [x] `docs/specs/architecture.md`、`docs/specs/api.md`：v0.3
- [x] `docs/specs/ux.md`
- [x] 交叉核對四份規格，統一欄位（grid、memory_snapshots、Diagnostic、結果狀態），套用上方決策，全部 v1.0
- [x] 建 `docs/specs/decisions.md` 記錄決策與理由
- [x] MVP 8 關程式碼以 rustc 1.99.1 edition 2024 實測：solution 通過、starter 依預期失敗（尚未在 runner 映像內跑）
- [x] 決策辯證四項待決事項，結論寫入 decisions.md §12 並同步四份規格（v1.1）
- [x] 驗收 agent 第一輪：不通過，4 項阻擋缺口（章節名稱、Ferris 對話來源、memory 快照顯示時機、grid 內容測試）＋文字落差
- [x] 架構師修正缺口並升 v1.2（decisions.md §13 實作必要參數、§14 v1.2 定案）
- [x] 驗收 agent 第二輪：通過，13 項次要問題
- [x] 架構師收尾 13 項次要問題與三批追加裁定，規格升 v1.3（decisions §15）；features 210 情境兩個 parser 通過
- [x] 內容測試以五行 header 與 tests.rs 補換行規則在 runner 重跑（後端 B4a 綠燈後由 content_levels 測試覆蓋）

## 插隊：POC Demo（拋棄式，不走 TDD）

- [x] `poc/`（`poc/run.sh` → http://127.0.0.1:5180，截圖在 `poc/screenshots/`）：一關可玩（02 mut-matters，fix-compile + grid），三種視覺風格可切換，真實 Docker 判題
- [x] 使用者選定風格 B「海洋繪本」（含深淺色）；正式前端只做 B，不做風格切換器
- [x] POC 永久保留於 `poc/`（使用者要求），正式版不引用其程式碼

> 工作模式：使用者要求全部完成才通知；待決事項派決策 agent 辯證拍板

## 階段 2：BDD 情境

- [x] `features/levels.feature`：列表、詳細、不洩漏 solution/tests、404
- [x] `features/run.feature`：成功輸出、編譯錯誤診斷、panic、timeout、stdout 截斷（`@docker`）
- [x] `features/submit.feature`：全通過、部分失敗、not_run、診斷不洩漏測試行號（`@docker`）
- [x] `features/judge_busy.feature`：滿載 503、Docker 不可用 503、health degraded
- [x] `features/ui_editor.feature`：Run/Submit、快捷鍵、錯誤波浪線、草稿自動存
- [x] `features/ui_grid.feature`：指令動畫、撞牆停止、步數上限、通關條件
- [x] `features/ui_memory.feature`：步驟播放、借用標示、行號同步
- [x] `features/ui_memory.feature` 補錯誤標記情境：`error.target` 紅框與徽章、出事箭頭轉紅（decisions.md §12 ②）
- [x] `features/ui_progress.feature`：星等、解鎖、提示計數不歸零

- [x] features/ 依規格 v1.2 同步（8 檔、約 193 情境）
- [x] step 句型共用詞彙表：不另建檔，由 B7、F8 各自在 step definitions 內整理
- [x] gherkin 0.16 crate 與 @cucumber/gherkin 解析 8 檔通過

## 階段 3a：後端 TDD（每項先紅燈再綠燈）

> 第一棒（B1–B6 測試＋紅燈）完成約 214 測試、紅燈確認；驗收員審測試：有條件通過，P1/P2 已補、fixture 以 v1.3 重產（3 綠 200 紅 34 ignored）；第二棒完成：cargo test 203 綠、--ignored 35 綠（含 8 關內容測試、grid 通關測試）；使用者授權不中途確認，改由驗收員審測試後再進第二棒

- [x] B1 關卡 loader：啟動一次載入、`[grid]` 驗證、測試名稱抽取
- [x] B1a loader：grid 關卡啟動時依 `[grid]` 產生 `level_grid()` 原始碼；驗證 `memory_snapshots` 的 `error.target` 存在、`error.code` 格式
- [x] B2 rustc JSON 診斷解析（含 end 位置、隱藏測試行回 null）
- [x] B3 測試輸出解析（passed/failed/not_run、panic message）
- [x] B4 docker 指令組裝（Vec 參數、安全回歸測試）
- [x] B4a 合併檔組成：grid 關卡依序併入共用模擬器、產生的 `level_grid()`、`tests.rs`；`user_line_count` 計算
- [x] B5 judge 執行器：暫存目錄 755/644、timeout kill、清理殘留容器（`#[ignore]` 需 docker）
- [x] B6 路由：levels、run、submit、health、錯誤格式、503
- [x] B7a health docker CLI 10 秒逾時（TDD）；順帶修正 JSON Content-Type 補 charset=utf-8
- [x] B7b 其餘 docker 指令（啟動清理 ps/rm、kill）加逾時；image inspect 逾時改判 docker_unreachable（TDD）
- [x] B7 cucumber：4 檔 78 情境全通過（`cargo test` 跑非 docker、`-- --ignored` 跑 @docker）

## 階段 3b：前端 TDD（每項先紅燈再綠燈）

> 第一棒（F1–F7 測試＋紅燈）完成 558 案例、490 紅；驗收員審測試：不通過 → 已補齊（644 案例、75 綠 569 紅；新增測試的複查併入階段 5 驗收）；第二棒完成 634/644，10 紅為測試缺陷（已核准修正）；真機煙霧測試淺深色 20 項通過；收尾完成：644/644 綠、Monaco 瘦身 dist 14M→4.7M、Ferris favicon、對比度 AA 全達標（token 已回寫 ux §8.2）

- [x] F1 API client 與型別（照 api.md §6）
- [x] F2 progress store：星等、解鎖、提示計數、草稿
- [x] F3 grid 指令解析與模擬器（純函式）
- [x] F4 GridWorld 元件（播放/暫停/單步、步數上限）
- [x] F5 MemoryView 元件
- [x] F5a MemoryView 錯誤標記：`error.target` 紅框、✗、錯誤碼徽章，從它出發的指標轉紅
- [x] F6 Editor 元件：Monaco、診斷 marker、快捷鍵
- [x] F7 關卡頁、章節地圖、通關結算
- [x] F8 playwright-bdd（4 檔 223 情境全通過、連跑兩次；修 7 個實作 bug 皆先補 Vitest 紅燈） e2e 接 `features/ui_*.feature`

## 階段 4：關卡內容

- [x] MVP 8 關寫入 `content/levels/NN-slug/`、`content/chapters.toml`、`content/shared/grid_sim.rs`（runner 內實測 8 關 solution 通過、starter 依預期失敗、loader 規則 0 錯誤）
- [x] 內容測試：每關 solution 通過、starter 失敗、grid solution 通關（`cargo test -- --ignored`）
- [x] 量測 rustc 峰值記憶體：熱快取峰值約 79MB，256m 足夠（併發與冷啟動仍待量）
- [x] 其餘 24 關依 curriculum.md §3 分 4 批平行落檔並各自在 runner 實測（共 32 關）
- [x] 關卡擴充後同步資料相依的 features 斷言（後端完成：32 關全綠、cucumber 102 情境、內容測試 38.9s；ui_* 完成：e2e 224/224 連跑兩次、Vitest 662/662）（levels.feature 關卡數與章節、ui_progress 真後端情境）與 backend cucumber、e2e 重跑

## 階段 5：驗收

- [x] 驗收 agent 第一輪：工程全綠（後端 212＋35、cucumber 102、Vitest 662、e2e 224）、API／UX／安全符合；不通過原因：30、31、32、14 可不用該關觀念取巧通過、缺 README
- [x] 驗收 agent 第二輪：不通過——e2e ui_editor 部分失敗情境未隨 api.md 範例同步（回歸）、33 關仍可不固定鎖順序通過；其餘全通過
- [x] E1 ui_editor.feature 部分失敗情境同步為 31 關 5 個測試並重跑 e2e
- [x] E2 判題編譯加 `-F unsafe_code`（TDD）＋新取巧反例 fixture（33 release_each／first_only、32 tryiter、14/30 unsafe）
- [x] E3 33 關補「同時持有所有橋」測試、32 關補慢螃蟹測試、31 hints 提醒原子性
- [x] 驗收 agent 第三輪：通過（後端 212＋36 ignored 全綠、cucumber 103、Vitest 662、e2e 224 首次全綠；取巧僅剩已知殘留）、規格對照、列出異動清單外新增項目
- [x] 前後端走完前 3 關（驗收員以 Playwright 真機走查，含 01、02、05、09、17、21、22、28、32、33）

## 階段 5a：驗收修正

- [x] C1 內容反作弊測試：取巧解答放 `backend/tests/fixtures/cheats/<level-id>/*.rs`，`--ignored` 內容測試斷言每個取巧檔 Submit 不通過（先紅）
- [x] C2 第 8 章：補強 30、31、32 tests.rs（測試自開執行緒呼叫玩家函式、assert Send/Sync）；新增 channel build 關為 32，原 Boss 改為 33
- [x] C3 其他關：14 加指標相等檢查；22 擴充 `Box<dyn Trait>`；hints 首條改提問式（03、06、10、13、16、20）；13 concept 修正 Copy 與存放位置；07 concept 註明 `_` 的限制；17 hints 提及 `char_indices`
- [x] C4 同步（另含：22 concept「海鷗飛越岩石」改為與模擬器一致、curriculum §3 的 22 補 dyn、反例須可編譯且以測試失敗擋下）：levels.feature 與後端 step 關卡數 32→33、curriculum §3 摘要更新、後端與 e2e 重跑
- [x] D1 根目錄 README（runner 映像、後端、前端、測試指令、埠號）、`engines` 宣告 Node 版本、移除未用的 tower-http／tower、清掉 `let _ = level;`

## 階段 6：提交

- [x] 產出 commit 切分清單給使用者確認（排除 `frontend/.vscode/`）
- [x] 使用者同意後提交（6 個 commit：8bc40ac…d570590）

## 第二季：擴充關卡＋並行可視化（2026-10-21 使用者核准）

> 範圍：新增第 9–12 章（每章 6–7 關、debug 比例逐章提高）與第 13 章「除錯試煉場」（6–8 關全 debug）；新增並行時間軸可視化並套用到第 8 章
> 使用者追加：關卡可以多，重點是後段要有 debug 關卡，難度漸進式上升（每關標難度、第 13 章逐關遞增無回落）
> 流程同第一季：SDD → BDD → TDD（兩棒）→ 內容（含反取巧反例）→ 三輪驗收 → 提交清單給使用者確認

### S2-1 SDD

- [x] 課程：curriculum v1.4 新增第 9–13 章 34 關（34–67，build 7、fix-compile 5、debug 22），全遊戲 13 章 67 關；每關標難度與 debug 階梯 D1–D5；章名寫入 §1.7
- [x] 架構＋UX：timeline 可視化規格（v1.4，decisions ⑬：stdout `@tl|` 事件、shared/timeline.rs 小幫手、TimelineView、一關一種 visual）（事件來源與格式、loader 驗證、TimelineView 元件、互動狀態、與 grid 的取捨）；第 8 章哪些關改用 timeline
- [x] 驗收員檢查規格 v1.4：不通過（main 泳道無 end、curriculum 未同步 ⑬⑭、67 缺複習內容）
- [x] 規格 v1.5 同步修正（含 curriculum 同步 timeline，decisions ⑮）

### S2-2 BDD

- [x] `features/ui_timeline.feature`（40 情境）、ui_editor 中途當掉卡片、後端 feature 中 visual 相依處（關卡數最後統一同步）

### S2-3 TDD

- [x] 後端（loader_timeline 17、routes、timeline_events、content_levels 10/10 全綠）：loader 支援 timeline（visual 列舉、接上 shared/timeline.rs、驗證規則）與內容測試（solution 無循環等待、泳道收尾、33 starter 有循環）（第一棒紅燈 → 第二棒綠燈）
- [x] 前端（Vitest 835 全綠、build 通過）：parseTimelineEvents／deriveTimeline＋TimelineView、主控台隱藏 `@tl|` 行、中途當掉卡片（第一棒紅燈 → 第二棒綠燈）；e2e 接 ui_timeline.feature（e2e 273 全綠、Vitest 837）

### S2-4 內容

- [x] 第 8 章：30、31、33 改用 timeline（33 移除 [grid]），32 維持 grid；content/shared/timeline.rs 建立
- [x] 第 9–13 章平行落檔：34–67 共 34 關＋約 100 個取巧反例，各自在 runner 連跑 3 次通過（67 總關卡），每關附取巧反例 fixture
- [x] 落檔後檢查：49 zz_、54 matcher 改玩家完成、67 依 v1.5（check_to_cell 改 todo!、只由測試呼叫）
- [x] 落檔後用正式 content 重跑 timeline stdout，同步 ui_timeline 附錄與前端 fixture（行號）、e2e mock levels.json 加入 30／31／33 timeline 版
- [x] 31 關改 timeline 後 warning 變 1 筆：同步 submit.feature、api.md §4 範例
- [x] 後端內容測試、cheats 測試、cucumber（後端 250＋ignored 45 全綠、cucumber 102 情境、67 關內容約 100s；前端 Vitest 839、e2e 273）、前端 e2e 全綠

### S2-5 驗收

- [x] 驗收員第一輪：工程全綠；不通過——38、59 可取巧、54 提示太直白，另有提示／故事／規格文字落差
- [x] 修正：38／59／54 測試與提示、12 關改用欄位比對、提示與故事落差、39 註解、curriculum v1.6 同步（反例 113 個、content_levels 全綠）
- [x] 驗收員第二輪：有條件通過——全綠、上輪問題皆修；新發現改寫斷言巨集可讓隱藏測試失效（全關卡）
- [x] 斷言巨集加固：65 個 tests.rs＋grid_sim 共 676 處改完整路徑、level_grid 用 ::std::vec!、6 個新反例（共 119 個）、curriculum 54／63 落差
- [x] 驗收員第三輪：巨集加固有效、全綠（後端 250＋45、cucumber 102、Vitest 839、e2e 273）；新發現遮蔽 std 方法／型別類取巧——grid_sim 修正，其餘列已知殘留
- [x] grid_sim 22 處方法改完整語法＋01 trait_contains 反例（反例共 120 個、content_levels 全綠）

### S2-0 GitHub 與 CI

- [x] 個人帳號 AdaaWu 建 private repo game-rust-learning、push main（https://github.com/AdaaWu/game-rust-learning）
- [x] `.github/workflows/ci.yml`（backend、backend-docker、frontend）與精簡 README（PR #2 取代 #1：CI 三個 job 全綠，待使用者 merge）

### S2-6 提交

- [x] commit 切分清單給使用者確認
- [x] 使用者同意後提交並開 PR 到 main：5 個 commit（37427e2…c67939b），PR #3；合併 main 解衝突、CI 三個 job 全綠後合入 main

## 後續版本（MVP 之後）

- [ ] 公開部署評估（前端 GitHub Pages、後端另找主機、沙箱強化、限流、成本）：使用者 2026-10-21 決定先擱置

- [ ] CI 前端 e2e 約 15–18 分鐘，接近 30 分鐘上限：關卡增加時改用 shard 拆 job
- [ ] Docker 判題測試改用每個測試行程專屬 label，避免與執行中的後端或 e2e 互相干擾
- [ ] timeline 目前步數直線、手機版可視化區比例
- [ ] 補 20、23、25 無可視化關卡的畫面（第 8 章時間軸已排入第二季）
- [ ] 難度曲線微調：09、11、12 連續單行修正、03 跳太快

- [ ] ContainerGuard Drop 的 `docker rm -f` 加逾時（daemon 不回應時會殘留執行緒，不影響請求）

- [ ] 錯誤圖鑑：第 3 章以後關卡補齊、可收錄錯誤碼達 10 種以上時再評估（decisions.md §12 ④）
