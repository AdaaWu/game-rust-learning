# 執行：playwright-bdd（frontend e2e）
# 規格：ux.md §3.3、§7.1–§7.4、§11；architecture.md §2.2、§3.11、§6.2 ⑦、§7.5；api.md §0.3、§2、§3；decisions.md §15 ⑬⑭⑮
# 預設以 page.route mock API；@docker 情境打真實後端（需 Docker 與 runner 映像）
# mock 關卡：測試用 8 關 fixture 套用第二季 timeline 調整
#   31-shared-counter 的 visual 改為 "timeline"，另加 30-send-the-crabs、33-boss-great-migration（visual 皆為 "timeline"，grid 與 memory_snapshots 為 null）
#   三關的 starter 為 loader 組好的版本：starter.rs ＋ 一個空行 ＋ content/shared/timeline.rs（architecture.md §2.2）
# stdout 範例（「」內的名稱）全文見檔尾附錄，一律取自實際執行，不手寫：
#   rustc 1.99.1 `--edition 2024 -A dead_code -F unsafe_code` 編譯「玩家程式碼 ＋ architecture.md §3.11 小幫手」
#   逾時範例在 5 秒時 kill，保留 kill 前已輸出的內容（architecture.md §3.11「timeout、截斷、panic 時已輸出的事件」）
#   行號取自 content/ 落檔版 30、31、33（starter.rs／solution.rs ＋ content/shared/timeline.rs），對 127.0.0.1:3000 實際 Run 驗證
# 「播放結束」指自動播放停在最後一步；「泳道狀態」的圖示見 ux.md §11.9

Feature: TimelineView 並行時間軸
  玩家程式透過檔尾的時間軸小幫手把上鎖、送訊息等事件印到 stdout
  前端依 Run 的 stdout 畫出每條執行緒的泳道，只陳述誰等誰、誰佔著什麼；通關以 Submit 結果為準

  Background:
    Given mock API 回傳測試用 8 關 fixture 的關卡列表與詳細資料，並套用第二季 timeline 調整
    And mock API 的 GET "/api/health" 回 status "ok"
    And 玩家進度中測試用 8 關 fixture 與 30-send-the-crabs、33-boss-great-migration 都已拿到 ★1

  # ---------- 尚未執行與零事件 ----------

  # ux.md §11.6 尚未執行、§11.7「N = 0 時播放控制列 disabled」、§11.1「圖例下方常駐一行小字」
  Scenario: 尚未執行時只顯示空白軸與提示
    When 玩家開啟關卡 "33-boss-great-migration"
    Then 時間軸中央顯示「按『執行』，看看每條執行緒怎麼跑」
    And 時間軸沒有任何泳道
    And 時間軸圖例下方顯示「每次執行的先後順序可能不同」
    And 播放控制列為 disabled

  # ux.md §11.6 零事件提示；decisions.md §15 ⑬ 不做「第一季草稿遷移」
  Scenario: 沿用第一季草稿（沒有小幫手呼叫）時顯示零事件提示
    Given localStorage 的 "rustquest.draft.31-shared-counter" 為第一季 31-shared-counter 的 starter.rs 全文
    And mock API 的 Run 回 status "ok"，stdout 為「31 第一季草稿」
    When 玩家開啟關卡 "31-shared-counter"
    And 玩家點「執行」
    Then 時間軸中央顯示「這次執行沒有時間軸事件：要透過 `timeline::` 小幫手上鎖、送訊息，時間軸才看得到（按『重設』可取回小幫手）」
    And 時間軸沒有任何泳道
    And 播放控制列為 disabled
    And 主控台「輸出」分頁內容為 "總共 0 顆寶石"
    And 主控台「輸出」分頁沒有「已隱藏 n 行時間軸事件」提示

  # ---------- 33 starter：死鎖 ----------

  # ux.md §11.10 e2e「33 starter Run → 主控台 timeout 卡片＋時間軸死鎖結論＋兩條泳道 ✗＋編輯器高亮等待那一行」
  # ux.md §11.3 泳道排序（main 置頂、其餘依首次出現）、§11.5 ③「循環的第一條泳道＝循環中依泳道排序最前者」、§11.6 timeout、§11.8 死鎖高亮
  # 本範例中分身 1 比分身 0 先出現，循環從分身 1 開始
  Scenario: 33 starter 逾時時標出死鎖循環並高亮循環第一條泳道的等待行
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 主控台顯示卡片「程式跑太久被停下來了（超過 5 秒）」
    And 播放控制列顯示「第 14 / 14 步」
    And 泳道由上而下依序為 "main"、"分身 2"、"分身 1"、"分身 0"
    And 時間軸訊息列第一行為「第 14 步 · 第 39 行：分身 0 想拿 窄橋 1（被 分身 1 佔著）」
    And 時間軸訊息列第二行為「✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）」
    And 泳道標籤依序為：
      | lane   | icon | text |
      | main   | ✉    |      |
      | 分身 2 | ■    |      |
      | 分身 1 | ✗    | 死鎖 |
      | 分身 0 | ✗    | 死鎖 |
    And 編輯器第 39 行有整行背景高亮與行號左側「▶」圖示
    And 編輯器只有 1 行有行高亮

  # ux.md §11.5「每一步都判定，循環一出現就標紅」、§11.3「握著一把又在等另一把時往下疊一層」、§11.6 lock_wait 句型
  Scenario: 循環在第二條等待出現的那一步才標紅
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點進度條跳到第 13 步
    Then 時間軸訊息列第一行為「第 13 步 · 第 39 行：分身 1 想拿 窄橋 0（被 分身 0 佔著）」
    And 時間軸沒有任何 ✗ 標記
    And 泳道 "分身 1" 的標籤圖示為「⏳」
    And 泳道 "分身 1" 疊成 2 層：持有 ② 的區段與等待 ① 的區段
    When 玩家點進度條跳到第 14 步
    Then 泳道 "分身 0" 與 "分身 1" 的標籤顯示「✗」與「死鎖」

  # ux.md §11.5 ④（循環上的 wait_lock 區段與被等待那把鎖的 hold 區段 deadlocked）、§11.9 死鎖編碼（疊加不取代）；decisions.md §15 ⑯（只標未結束的等待區段）
  Scenario: 死鎖區段疊加紅框與條尾 ✗，並畫出指向持有者的紅色虛線箭頭
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 時間軸區段依泳道與起點排列為：
      | lane   | kind      | target | from | to   |
      | main   | wait_msg  | 回報   | 1    | 11   |
      | main   | wait_msg  | 回報   | 12   | null |
      | 分身 1 | wait_lock | 窄橋 1 | 9    | 10   |
      | 分身 1 | hold      | 窄橋 1 | 10   | null |
      | 分身 1 | wait_lock | 窄橋 0 | 13   | null |
      | 分身 0 | wait_lock | 窄橋 0 | 7    | 8    |
      | 分身 0 | hold      | 窄橋 0 | 8    | null |
      | 分身 0 | wait_lock | 窄橋 1 | 14   | null |
    And deadlocked 的區段恰好為：
      | lane   | kind      | target | from |
      | 分身 1 | hold      | 窄橋 1 | 10   |
      | 分身 1 | wait_lock | 窄橋 0 | 13   |
      | 分身 0 | hold      | 窄橋 0 | 8    |
      | 分身 0 | wait_lock | 窄橋 1 | 14   |
    And 已結束的等待區段 "分身 1" wait_lock 窄橋 1（第 9–10 步）與 "分身 0" wait_lock 窄橋 0（第 7–8 步）不是 deadlocked
    And 泳道 "main" 的區段都不是 deadlocked
    And 每個 deadlocked 區段都有 3px "--danger" 實線外框與條尾 ✗，且保留原本的填色或虛線
    And 時間軸有 2 條 "--danger" 虛線箭頭，各標 ✗：分身 1 的等待區段指向分身 0 的 hold 區段、分身 0 的等待區段指向分身 1 的 hold 區段

  # ux.md §11.5 訊息箭頭（未收到的 send 畫殘箭、第 k 個 send 對第 k 個 recv）、§11.6 recv 句型
  Scenario: 訊息送出後先畫殘箭，被收到時改畫箭頭
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點進度條跳到第 10 步
    Then 時間軸有 0 條訊息箭頭、1 條尾端為 ✉ 的虛線殘箭，殘箭從泳道 "分身 2" 第 3 步出發
    When 玩家點進度條跳到第 11 步
    Then 時間軸有 1 條訊息箭頭：從泳道 "分身 2" 第 3 步指到泳道 "main" 第 11 步
    And 時間軸沒有殘箭
    And 時間軸訊息列第一行為「第 11 步 · 第 58 行：main 從 回報 收到一則訊息（分身 2 送的）」

  # ux.md §11.3 圖例列（鎖代號依第一次出現的順序）、時間軸刻度（每 5 步一個刻度）
  Scenario: 圖例依鎖第一次出現的順序編號並列出 channel
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 時間軸圖例依序為：鎖代號「①」對 "窄橋 0"、鎖代號「②」對 "窄橋 1"、channel「✉」對 "回報"
    And 時間軸圖例第二行含「▬ 持有」「┄ 等待」「➝ 訊息」與「每次執行的先後順序可能不同」
    And 時間軸刻度標示 "1"、"5"、"10"

  # ux.md §11.3「泳道在解析完全部事件時就全部配置好；還沒輪到第一個事件的泳道標籤以 40% 透明度顯示」、§11.7 第 0 步
  Scenario: 泳道一開始就全部配置，還沒輪到的泳道半透明
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點「從頭」
    Then 播放控制列顯示「第 0 / 14 步」
    And 播放已暫停
    And 泳道由上而下依序為 "main"、"分身 2"、"分身 1"、"分身 0"
    And 時間軸沒有任何區段
    And 4 條泳道的標籤透明度都是 40%
    When 玩家點「單步」
    Then 泳道 "main" 的標籤透明度為 100%
    And 泳道 "分身 2"、"分身 1"、"分身 0" 的標籤透明度為 40%

  # ---------- 33 solution 與其他結論 ----------

  # ux.md §11.10 e2e「33 solution Run → 全部執行緒都結束了、訊息箭頭數等於回報數」；§11.6 結論只看具名泳道；§11.5 recv_closed
  Scenario: 33 solution 全部執行緒都結束了
    Given mock API 的 Run 回 status "ok"，stdout 為「33 solution」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 播放控制列顯示「第 29 / 29 步」
    And 泳道由上而下依序為 "main"、"分身 2"、"分身 0"、"分身 1"
    And 時間軸訊息列第二行為「全部執行緒都結束了」
    And 時間軸有 3 條訊息箭頭，都指到泳道 "main"
    And 時間軸沒有殘箭
    And 泳道 "main" 第 29 步有 ⊘ 標記
    And 時間軸訊息列第一行為「第 29 步 · 第 63 行：回報 已關閉，main 不會再收到訊息」
    And 泳道 "分身 2"、"分身 0"、"分身 1" 各有 1 個 ● 與 1 個 ■
    And 泳道 "main" 沒有 ●、■ 與「…」

  # ux.md §11.7「Run 回傳 ok … 且 execution 不為 null 時，從第 1 步播放」「1x 每步 500ms」
  Scenario: Run 成功後自動從第 1 步開始播放
    Given mock API 的 Run 回 status "ok"，stdout 為「33 solution」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」
    Then 播放控制列顯示「第 1 / 29 步」
    And 播放按鈕顯示為「暫停」
    And 約 500 ms 後播放控制列顯示「第 2 / 29 步」

  # ux.md §11.6 timeout「沒有循環但有泳道在等」「waiting_msg 的泳道，若其他具名泳道此時都已 ended，標籤旁補『其他執行緒都已結束』」；decisions.md §15 ⑬ 33 套用效果
  # decisions.md §15 ⑯「還有 n 條執行緒在等」只計具名泳道，main 等訊息另以「main 等訊息」描述；本例沒有具名泳道在等
  Scenario: 修好鎖順序但指揮官仍在等訊息時只陳述事實
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 修好鎖順序未 drop(tx)」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 播放控制列顯示「第 28 / 28 步」
    And 時間軸訊息列第二行為「時間到：main 等訊息」
    And 泳道 "main" 的標籤圖示為「✉」，標籤旁顯示「其他執行緒都已結束」
    And 時間軸沒有任何 ✗ 標記
    And 時間軸有 3 條訊息箭頭

  # ux.md §11.5「只有鎖會形成等待關係；recv_wait 不算」、§11.10「等待鏈不成環（A 等 B、B 在等訊息）不算死鎖」；decisions.md §15 ⑯（n 只計具名泳道，本例兩條皆具名、沒有 main）
  Scenario: 等待鏈沒有成環不算死鎖
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「等待鏈」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 時間軸訊息列第一行為「第 6 步 · 第 19 行：分身 B 想拿 窄橋 0（被 分身 A 佔著）」
    And 時間軸訊息列第二行為「時間到：還有 2 條執行緒在等」
    And 泳道標籤依序為：
      | lane   | icon | text |
      | 分身 A | ✉    |      |
      | 分身 B | ⏳   |      |
    And 泳道 "分身 A" 標籤旁沒有「其他執行緒都已結束」
    And 時間軸沒有任何 ✗ 標記

  # ux.md §11.5 ②「H 就是 T 自己也算，循環長度為 1」、§11.6 lock_wait「H 是 T 自己時補『（被自己佔著）』」、timeout 死鎖結論、§11.8 死鎖高亮；decisions.md §15 ⑯（結論句也寫「被自己佔著」）
  Scenario: 同一條執行緒重複鎖同一把 Mutex 也是死鎖
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「自己等自己」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 時間軸訊息列第一行為「第 4 步 · 第 10 行：分身 0 想拿 窄橋 0（被自己佔著）」
    And 時間軸訊息列第二行為「✗ 死鎖：分身 0 等 窄橋 0（被自己佔著）」
    And 泳道 "分身 0" 的標籤顯示「✗」與「死鎖」
    And 編輯器第 10 行有整行背景高亮與行號左側「▶」圖示

  # ux.md §11.6 timeout「都沒有：『時間到』」
  Scenario: 逾時但沒有任何泳道在等時只顯示時間到
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「31 結尾無窮迴圈」
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」並等待播放結束
    Then 播放控制列顯示「第 21 / 21 步」
    And 時間軸訊息列第二行為「時間到」

  # ux.md §11.10 e2e「31 starter Run → 三條螃蟹泳道的標註都是『讀到 0』，且標註出現在『放開 計數器』之後」；§11.6 note、lock_free 句型；§11.6 具名泳道（main 沒有 start）
  Scenario: 31 starter 三隻螃蟹都在放開計數器之後才插旗讀到 0
    Given mock API 的 Run 回 status "ok"，stdout 為「31 starter」
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」並等待播放結束
    Then 泳道由上而下依序為 "main"、"螃蟹 0"、"螃蟹 1"、"螃蟹 2"
    And 時間軸的 🚩 標記依序為：
      | lane   | step | text   | hold 區段 |
      | 螃蟹 0 | 5    | 讀到 0 | 3–4       |
      | 螃蟹 1 | 11   | 讀到 0 | 9–10      |
      | 螃蟹 2 | 17   | 讀到 0 | 15–16     |
    And 時間軸訊息列第二行為「全部執行緒都結束了」
    And 泳道 "main" 沒有 ●、■ 與「…」
    When 玩家點進度條跳到第 4 步
    Then 時間軸訊息列第一行為「第 4 步：螃蟹 0 放開 計數器」
    When 玩家點進度條跳到第 5 步
    Then 時間軸訊息列第一行為「第 5 步 · 第 7 行：螃蟹 0：讀到 0」

  # ux.md §11.6 ok「有具名泳道沒有 end（main 先結束）」，這些泳道尾端畫「…」
  Scenario: main 先結束時列出還沒做完的具名泳道
    Given mock API 的 Run 回 status "ok"，stdout 為「30 未 join 分身」
    And 玩家開啟關卡 "30-send-the-crabs"
    When 玩家點「執行」並等待播放結束
    Then 泳道由上而下依序為 "點名 0"、"點名 2"、"點名 1"、"分身 1"、"分身 0"、"分身 2"
    And 時間軸訊息列第二行為「程式結束時，還有 3 條執行緒沒做完（分身 1、分身 0、分身 2）」
    And 泳道 "分身 1"、"分身 0"、"分身 2" 尾端畫「…」
    And 泳道 "點名 0"、"點名 2"、"點名 1" 尾端沒有「…」，標籤圖示為「■」

  # ux.md §11.6 runtime_panic（自動播放到最後一個事件、沒有 end 的具名泳道才畫「…」）；architecture.md §3.11「panic：lane 與 guard 的 Drop 照常執行」
  Scenario: Run panic 時播到最後一個事件並標示程式在這裡結束
    Given mock API 的 Run 回 status "runtime_panic"，stdout 為「31 螃蟹 panic」，stderr 為「31 螃蟹 panic」的 stderr
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」並等待播放結束
    Then 播放控制列顯示「第 17 / 17 步」
    And 時間軸訊息列第二行為「程式在這裡結束（panic）」
    And 泳道由上而下依序為 "螃蟹 0"、"螃蟹 1"、"螃蟹 2"
    And 時間軸沒有任何泳道尾端畫「…」

  # ux.md §11.10 e2e「30 starter Run → 程式還沒編譯成功」；§11.6 compile_error、memory_limit（execution = null）
  Scenario Outline: 編譯失敗或編譯階段 OOM 時不畫泳道
    Given mock API 的 Run 回 status "<status>"，execution 為 null
    And 玩家開啟關卡 "30-send-the-crabs"
    When 玩家點「執行」
    Then 時間軸中央顯示「<text>」
    And 時間軸沒有任何泳道
    And 播放控制列為 disabled

    # compile_error 對應 30 starter 實際編譯結果：E0373、E0277 兩個錯誤（decisions.md §15 ⑬ 30 套用）
    Examples:
      | status        | text                 |
      | compile_error | 程式還沒編譯成功     |
      | memory_limit  | 編譯時用太多記憶體   |

  # ux.md §11.6 memory_limit「execution 有值：自動播放到最後一個事件，結論『記憶體不足』」
  # stdout 取自「31 solution」實際輸出；OOM 無法在本機重現，status 由 mock 指定
  Scenario: 執行階段記憶體不足時播到最後一個事件
    Given mock API 的 Run 回 status "memory_limit"，stdout 為「31 solution」
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」並等待播放結束
    Then 播放控制列顯示「第 21 / 21 步」
    And 時間軸訊息列第二行為「記憶體不足」

  # ux.md §11.6 output_limit（先丟棄最後一行再解析）、零事件與解析警示（capped）；§11.4 ② ⑤
  Scenario: 輸出超過上限時結論為截斷並提示事件超過上限
    Given mock API 的 Run 回 status "output_limit"，stdout 為「31 結尾無限上鎖（截斷）」，stdout_truncated 為 true
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」
    And 玩家點進度條跳到第 1000 步
    Then 播放控制列顯示「第 1000 / 1000 步」
    And 時間軸訊息列第二行依序含「輸出過長已截斷」與「事件超過 1,000 個，後面的不顯示了」

  # ux.md §11.4 ③④⑤（未知 kind、line 為 0、段數不足、thread 空白都只累加 malformed；行首有空白不算事件）、§11.6 malformed 警示
  Scenario: 格式不對的事件行略過並在結論後提示
    Given mock API 的 Run 回 status "ok"，stdout 為「31 格式不對的事件」
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」並等待播放結束
    Then 播放控制列顯示「第 21 / 21 步」
    And 時間軸訊息列第二行依序含「全部執行緒都結束了」與「有 4 行時間軸事件格式不對，已略過」

  # ux.md §11.4 ⑤（第 1,001 個起不加入）、§11.7「N 為合法事件數」、§11.3 欄寬夾在 14–32px、超出時元件內橫向捲動
  Scenario: 事件超過 1,000 個時只畫前 1,000 個且在元件內橫向捲動
    Given 瀏覽器視窗寬度為 1280px
    And mock API 的 Run 回 status "ok"，stdout 為「31 三百隻螃蟹」
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」
    Then 播放控制列顯示「第 1 / 1000 步」
    And 時間軸每步欄寬為 14px
    And 時間軸區可橫向捲動
    And 頁面本身沒有橫向捲軸
    When 玩家點進度條跳到第 1000 步
    Then 時間軸訊息列第二行含「事件超過 1,000 個，後面的不顯示了」
    And 第 1000 步的欄位在時間軸區的可見範圍內

  # ux.md §11.3 欄寬上限 32px
  Scenario: 事件很少時欄寬不超過 32px
    Given 瀏覽器視窗寬度為 1280px
    And mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「自己等自己」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 時間軸每步欄寬為 32px
    And 時間軸區不能橫向捲動

  # ---------- 播放控制 ----------

  # ux.md §11.7「點時間軸上的任一事件標記：跳到該步並暫停（不播放中間動畫）」
  Scenario: 點事件標記跳到該步並暫停
    Given mock API 的 Run 回 status "ok"，stdout 為「33 solution」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」
    And 播放中玩家點泳道 "分身 0" 第 13 步的事件標記
    Then 播放控制列顯示「第 13 / 29 步」
    And 播放已暫停
    And 第 1 步到第 13 步之間沒有播放中間動畫

  # ux.md §11.7「鍵盤：可視化區取得焦點時沿用 §7.2」
  Scenario Outline: 時間軸取得焦點時的鍵盤操作
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點進度條跳到第 5 步
    And 焦點在可視化區
    And 玩家按 "<key>"
    Then <result>

    Examples:
      | key        | result                         |
      | Space      | 播放按鈕顯示為「暫停」         |
      | ArrowRight | 播放控制列顯示「第 6 / 14 步」 |
      | Home       | 播放控制列顯示「第 0 / 14 步」 |

  # ux.md §11.9「事件標記可用 Tab 走訪（只走目前已播放的事件），Enter 跳到該步」
  Scenario: 用 Tab 走訪已播放的事件標記並以 Enter 跳步
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點進度條跳到第 5 步
    Then 時間軸可用 Tab 走訪的事件標記恰好 5 個，依第 1 到第 5 步排列
    When 玩家用 Tab 移到第 3 步的事件標記並按 "Enter"
    Then 播放控制列顯示「第 3 / 14 步」
    And 播放已暫停

  # ---------- 請求中與還原 ----------

  # ux.md §11.7 pending「回第 0 步並半透明」「Run 回來：依新結果自動播放」
  Scenario: 再次 Run 時回到第 0 步並半透明，回來後依新結果播放
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And mock API 的 Run 改為延遲 2 秒後回 status "ok"，stdout 為「33 solution」
    And 玩家點「執行」
    Then 播放控制列顯示「第 0 / 14 步」
    And 可視化區為半透明
    When 等待請求完成
    Then 可視化區不是半透明
    And 播放控制列顯示「第 1 / 29 步」

  # ux.md §11.7「Submit 回來：還原到上次 Run 播放結束的那一步，不重播」、§11.6 test_failed 與 Submit 的任何結果
  Scenario: Submit 回來後還原到上次 Run 的最後一步，不重播
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And mock API 的 Submit 延遲 2 秒後回 status "test_failed"
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點「送出」
    Then 播放控制列顯示「第 0 / 14 步」
    And 可視化區為半透明
    When 等待請求完成
    Then 可視化區不是半透明
    And 播放控制列顯示「第 14 / 14 步」
    And 播放已暫停
    And 時間軸訊息列第二行為「✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）」

  # ux.md §11.6 請求失敗「時間軸還原成請求前的畫面，不清空、不重播；訊息由主控台卡片負責，時間軸不另顯示」、§11.7
  Scenario Outline: 請求失敗時時間軸還原成上次 Run 的畫面
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And mock API 的 Run 改為<failure>
    And 玩家點「執行」
    Then 主控台顯示灰色卡片「<card>」
    And 播放控制列顯示「第 14 / 14 步」
    And 播放已暫停
    And 時間軸訊息列第二行為「✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）」
    And 時間軸沒有顯示「<card>」

    Examples:
      | failure                                                     | card                                           |
      | 回 503，error.code 為 "JUDGE_BUSY"，標頭 "Retry-After" 為 "2" | 伺服器現在有點忙，等一下再試                   |
      | 回 503，error.code 為 "JUDGE_UNAVAILABLE"，沒有 "Retry-After" 標頭 | 判題服務還沒準備好，關卡內容仍可瀏覽     |
      | 回 413，error.code 為 "PAYLOAD_TOO_LARGE"                    | 程式碼太長了（上限 64 KiB），刪掉一些再試      |
      | 連線中斷                                                    | 連不上伺服器，檢查網路後再試                   |

  # ux.md §11.7「從未 Run 過則回到空白狀態」
  Scenario: 從未 Run 過就 Submit 時時間軸維持空白狀態
    Given mock API 的 Submit 回 status "test_failed"
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「送出」
    Then 時間軸中央顯示「按『執行』，看看每條執行緒怎麼跑」
    And 時間軸沒有任何泳道
    And 播放控制列為 disabled

  # ---------- 與編輯器的互動 ----------

  # ux.md §11.8「line = null（lock_free、end）時沿用前一個有行號事件的行，不清除高亮」
  Scenario: 沒有行號的事件沿用前一個有行號事件的行
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點進度條跳到第 5 步
    Then 時間軸訊息列第一行為「第 5 步：分身 2 結束」
    And 編輯器第 36 行有整行背景高亮與行號左側「▶」圖示
    And 編輯器只有 1 行有行高亮

  # ux.md §11.8「點泳道標籤：line-change 該泳道在目前這一步之前最近一個有行號的事件；不改變目前步數」、§11.9 泳道標籤是按鈕
  Scenario: 點泳道標籤高亮該泳道最近一個有行號的事件
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家點泳道標籤 "main"
    Then 編輯器第 58 行有整行背景高亮與行號左側「▶」圖示
    And 播放控制列顯示「第 14 / 14 步」
    When 玩家點泳道標籤 "分身 2"
    Then 編輯器第 44 行有整行背景高亮與行號左側「▶」圖示
    And 播放控制列顯示「第 14 / 14 步」

  # ux.md §11.8「程式碼在這次 Run 之後被修改：沿用主控台『程式已修改』狀態，訊息列上方顯示黃色提示」
  Scenario: Run 之後修改程式碼時提示行號僅供參考
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    And 玩家在編輯器輸入一個空白
    Then 時間軸訊息列上方顯示黃色提示「程式碼已修改，行號高亮僅供參考」

  # ---------- 主控台 ----------

  # ux.md §11.8 主控台（timeline 關卡的「輸出」分頁隱藏 @tl| 開頭的行，頂部顯示隱藏行數）
  Scenario: 主控台隱藏時間軸事件行並顯示隱藏行數
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」
    Then 主控台「輸出」分頁頂部顯示淡色小字「已隱藏 14 行時間軸事件」
    And 主控台「輸出」分頁的 stdout 依序為：
      | line                  |
      | 分身 2 回報完畢，讓出 0 座窄橋 |
      | 分身 0 站上窄橋 0     |
      | 分身 1 站上窄橋 1     |
    And 主控台「輸出」分頁沒有任何以 "@tl|" 開頭的行

  # ux.md §11.8（隱藏以 @tl| 開頭的行，含格式不對的事件行）、§11.4 ③（行首不是 @tl| 的行是一般輸出）
  Scenario: 行首有空白的事件行照常顯示在主控台
    Given mock API 的 Run 回 status "ok"，stdout 為「31 格式不對的事件」
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「執行」
    Then 主控台「輸出」分頁頂部顯示淡色小字「已隱藏 25 行時間軸事件」
    And 主控台「輸出」分頁的 stdout 共 2 行，第 1 行為 "總共 15 顆寶石"
    And 主控台「輸出」分頁第 2 行為 " @tl|note|5|main|行首有空白"（行首一個空白）

  # ---------- 版面與可及性 ----------

  # ux.md §11.3 窄寬度（可視化區容器 < 360px：標籤欄 64px、名稱省略但完整名稱在 title 與 aria-label、小字移到訊息列下方、頁面不出現橫向捲軸）
  Scenario: 手機寬度下時間軸縮窄標籤欄且頁面不橫向捲動
    Given 瀏覽器視窗寬度為 375px
    And mock API 的 Run 回 status "ok"，stdout 為「30 未 join 分身」
    And 玩家開啟關卡 "30-send-the-crabs"
    When 玩家點「執行」
    Then 目前分頁為「結果」
    And 時間軸容器寬度小於 360px
    And 時間軸標籤欄寬度為 64px
    And 泳道 "分身 1" 標籤的 title 為 "分身 1"，aria-label 含 "分身 1"
    And 「每次執行的先後順序可能不同」顯示在時間軸訊息列下方
    And 頁面本身沒有橫向捲軸

  # ux.md §11.9 可及性（aria-live 朗讀事件句與結論、SVG role="img" 摘要、泳道標籤 aria-label 含名稱與狀態）
  Scenario: 事件句與結論放進 aria-live，圖與泳道標籤有文字摘要
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then aria-live 區含「第 14 步 · 第 39 行：分身 0 想拿 窄橋 1（被 分身 1 佔著）」
    And aria-live 區含「✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）」
    And 時間軸 SVG 的 role 為 "img"，aria-label 為「4 條執行緒、2 把鎖、1 個 channel，第 14 / 14 步」
    And 泳道標籤按鈕 "分身 0" 的 aria-label 為「分身 0，死鎖，等窄橋 1」
    And 泳道標籤按鈕 "分身 1" 的 aria-label 為「分身 1，死鎖，等窄橋 0」

  # ux.md §11.9「每種狀態用填色或線型＋圖示＋文字至少三重編碼」、§7.1 不只靠顏色
  Scenario: 持有、等鎖、等訊息與死鎖不只靠顏色區分
    Given mock API 的 Run 回 status "timeout"（execution.timed_out 為 true），stdout 為「33 starter 死鎖」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 泳道 "分身 0" 的 hold 區段為實心條，條頭標「①」
    And 泳道 "分身 0" 第 14 步起的 wait_lock 區段為虛線外框，條頭標「⏳②」
    And 泳道 "main" 第 12 步起的 wait_msg 區段為點線外框，條頭標「✉⏳」
    And 泳道 "分身 0" 的標籤同時有「✗」圖示、「死鎖」文字與 "--danger" 文字色

  # ux.md §11.9「prefers-reduced-motion: reduce：箭頭與區段直接出現、不做畫線動畫；自動捲動直接跳位」、§7.4
  Scenario: 減少動態效果時箭頭與區段直接出現
    Given 瀏覽器設定 prefers-reduced-motion 為 "reduce"
    And mock API 的 Run 回 status "ok"，stdout 為「33 solution」
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家點「執行」並等待播放結束
    Then 時間軸的訊息箭頭與區段都沒有畫線動畫

  # ---------- 真實後端 ----------

  # ux.md §11.10 e2e「@docker 一個情境：真後端跑 33 solution，時間軸出現 3 條分身泳道與 main」；architecture.md §2.2（starter 已含小幫手）
  # 交錯順序每次不同，只檢查不受交錯影響的值
  @docker
  Scenario: 真實後端執行 33 solution 時畫出分身泳道與 main
    Given 前端連到真實後端，判題服務可用
    And 玩家進度中 "33-boss-great-migration" 已解鎖
    And 玩家開啟關卡 "33-boss-great-migration"
    When 玩家把編輯器內容換成該關 solution.rs 接上 content/shared/timeline.rs（architecture.md §2.2 組法）
    And 玩家點「執行」並等待播放結束
    Then 泳道第一條為 "main"，其餘恰好為 "分身 0"、"分身 1"、"分身 2"（順序不限）
    And 時間軸訊息列第二行為「全部執行緒都結束了」
    And 時間軸有 3 條訊息箭頭，都指到泳道 "main"
    And 時間軸沒有任何 ✗ 標記

# ---------- 附錄：stdout 範例全文 ----------
# 每行以「# > 」開頭，其後即 stdout 原文（含行首空白）；每個範例最後一行之後有一個換行
#
# 「33 starter 死鎖」：33 starter.rs 依 decisions.md ⑬ 套用小幫手；Run 逾時 kill
# > @tl|recv_wait|58|main|回報
# > @tl|start|36|分身 2|
# > @tl|send|44|分身 2|回報
# > @tl|start|36|分身 1|
# > 分身 2 回報完畢，讓出 0 座窄橋
# > @tl|end||分身 2|
# > @tl|start|36|分身 0|
# > @tl|lock_wait|39|分身 0|窄橋 0
# > @tl|lock_got|39|分身 0|窄橋 0
# > 分身 0 站上窄橋 0
# > @tl|lock_wait|39|分身 1|窄橋 1
# > @tl|lock_got|39|分身 1|窄橋 1
# > 分身 1 站上窄橋 1
# > @tl|recv|58|main|回報
# > @tl|recv_wait|58|main|回報
# > @tl|lock_wait|39|分身 1|窄橋 0
# > @tl|lock_wait|39|分身 0|窄橋 1
#
# 「33 solution」：33 solution.rs 套用小幫手，收訊改為 while let Ok(r) = timeline::recv(&rx, "回報")；exit 0
# > @tl|start|36|分身 2|
# > @tl|recv_wait|63|main|回報
# > @tl|send|47|分身 2|回報
# > @tl|recv|63|main|回報
# > @tl|recv_wait|63|main|回報
# > 分身 2 回報完畢，讓出 0 座窄橋
# > @tl|end||分身 2|
# > @tl|start|36|分身 0|
# > @tl|lock_wait|42|分身 0|窄橋 0
# > @tl|lock_got|42|分身 0|窄橋 0
# > 分身 0 站上窄橋 0
# > @tl|start|36|分身 1|
# > @tl|lock_wait|42|分身 1|窄橋 0
# > @tl|lock_wait|42|分身 0|窄橋 1
# > @tl|lock_got|42|分身 0|窄橋 1
# > 分身 0 站上窄橋 1
# > @tl|send|47|分身 0|回報
# > @tl|recv|63|main|回報
# > @tl|recv_wait|63|main|回報
# > 分身 0 回報完畢，讓出 2 座窄橋
# > @tl|lock_free||分身 0|窄橋 0
# > @tl|lock_free||分身 0|窄橋 1
# > @tl|end||分身 0|
# > @tl|lock_got|42|分身 1|窄橋 0
# > 分身 1 站上窄橋 0
# > @tl|lock_wait|42|分身 1|窄橋 1
# > @tl|lock_got|42|分身 1|窄橋 1
# > 分身 1 站上窄橋 1
# > @tl|send|47|分身 1|回報
# > @tl|recv|63|main|回報
# > @tl|recv_wait|63|main|回報
# > 分身 1 回報完畢，讓出 2 座窄橋
# > @tl|lock_free||分身 1|窄橋 0
# > @tl|lock_free||分身 1|窄橋 1
# > @tl|end||分身 1|
# > @tl|recv_closed|63|main|回報
# > 指揮官收到 17 道指令，大遷徙出發！
# > MOVE DOWN
# > MOVE DOWN
# > MOVE RIGHT
# > MOVE RIGHT
# > PICK
# > MOVE RIGHT
# > MOVE UP
# > MOVE UP
# > MOVE RIGHT
# > PICK
# > MOVE RIGHT
# > OPEN DOWN
# > MOVE DOWN
# > MOVE DOWN
# > MOVE RIGHT
# > PICK
# > MOVE DOWN
#
# 「33 修好鎖順序未 drop(tx)」：33 starter 只把上鎖順序改成排序後的 order，未 drop(tx)；Run 逾時 kill
# > @tl|recv_wait|60|main|回報
# > @tl|start|36|分身 0|
# > @tl|lock_wait|41|分身 0|窄橋 0
# > @tl|lock_got|41|分身 0|窄橋 0
# > 分身 0 站上窄橋 0
# > @tl|start|36|分身 1|
# > @tl|lock_wait|41|分身 1|窄橋 0
# > @tl|start|36|分身 2|
# > @tl|send|46|分身 2|回報
# > 分身 2 回報完畢，讓出 0 座窄橋
# > @tl|end||分身 2|
# > @tl|recv|60|main|回報
# > @tl|recv_wait|60|main|回報
# > @tl|lock_wait|41|分身 0|窄橋 1
# > @tl|lock_got|41|分身 0|窄橋 1
# > 分身 0 站上窄橋 1
# > @tl|send|46|分身 0|回報
# > 分身 0 回報完畢，讓出 2 座窄橋
# > @tl|lock_free||分身 0|窄橋 0
# > @tl|lock_free||分身 0|窄橋 1
# > @tl|end||分身 0|
# > @tl|lock_got|41|分身 1|窄橋 0
# > 分身 1 站上窄橋 0
# > @tl|recv|60|main|回報
# > @tl|recv_wait|60|main|回報
# > @tl|lock_wait|41|分身 1|窄橋 1
# > @tl|lock_got|41|分身 1|窄橋 1
# > 分身 1 站上窄橋 1
# > @tl|send|46|分身 1|回報
# > 分身 1 回報完畢，讓出 2 座窄橋
# > @tl|lock_free||分身 1|窄橋 0
# > @tl|lock_free||分身 1|窄橋 1
# > @tl|end||分身 1|
# > @tl|recv|60|main|回報
# > @tl|recv_wait|60|main|回報
#
# 「等待鏈」：分身 A 握窄橋 0 後等 回報（沒有人送），分身 B 等窄橋 0；main 只 join；Run 逾時 kill
# > @tl|start|11|分身 A|
# > @tl|lock_wait|12|分身 A|窄橋 0
# > @tl|lock_got|12|分身 A|窄橋 0
# > @tl|recv_wait|13|分身 A|回報
# > @tl|start|18|分身 B|
# > @tl|lock_wait|19|分身 B|窄橋 0
#
# 「自己等自己」：分身 0 對同一把 Mutex 連續 timeline::lock 兩次；Run 逾時 kill（Linux／macOS std Mutex 重複上鎖會卡住）
# > @tl|start|8|分身 0|
# > @tl|lock_wait|9|分身 0|窄橋 0
# > @tl|lock_got|9|分身 0|窄橋 0
# > @tl|lock_wait|10|分身 0|窄橋 0
#
# 「31 starter」：curriculum.md §4.8 starter.rs；exit 0，編譯有 1 筆 unused_assignments 警告
# > @tl|start|18|螃蟹 0|
# > @tl|lock_wait|6|螃蟹 0|計數器
# > @tl|lock_got|6|螃蟹 0|計數器
# > @tl|lock_free||螃蟹 0|計數器
# > @tl|note|7|螃蟹 0|讀到 0
# > @tl|end||螃蟹 0|
# > @tl|start|18|螃蟹 1|
# > @tl|lock_wait|6|螃蟹 1|計數器
# > @tl|lock_got|6|螃蟹 1|計數器
# > @tl|lock_free||螃蟹 1|計數器
# > @tl|note|7|螃蟹 1|讀到 0
# > @tl|end||螃蟹 1|
# > @tl|start|18|螃蟹 2|
# > @tl|lock_wait|6|螃蟹 2|計數器
# > @tl|lock_got|6|螃蟹 2|計數器
# > @tl|lock_free||螃蟹 2|計數器
# > @tl|note|7|螃蟹 2|讀到 0
# > @tl|end||螃蟹 2|
# > @tl|lock_wait|26|main|計數器
# > @tl|lock_got|26|main|計數器
# > @tl|lock_free||main|計數器
# > 總共 0 顆寶石
#
# 「31 solution」：curriculum.md §4.8 solution.rs；exit 0
# > @tl|start|18|螃蟹 0|
# > @tl|lock_wait|6|螃蟹 0|計數器
# > @tl|lock_got|6|螃蟹 0|計數器
# > @tl|note|7|螃蟹 0|讀到 0
# > @tl|lock_free||螃蟹 0|計數器
# > @tl|end||螃蟹 0|
# > @tl|start|18|螃蟹 2|
# > @tl|lock_wait|6|螃蟹 2|計數器
# > @tl|lock_got|6|螃蟹 2|計數器
# > @tl|note|7|螃蟹 2|讀到 3
# > @tl|lock_free||螃蟹 2|計數器
# > @tl|end||螃蟹 2|
# > @tl|start|18|螃蟹 1|
# > @tl|lock_wait|6|螃蟹 1|計數器
# > @tl|lock_got|6|螃蟹 1|計數器
# > @tl|note|7|螃蟹 1|讀到 10
# > @tl|lock_free||螃蟹 1|計數器
# > @tl|end||螃蟹 1|
# > @tl|lock_wait|26|main|計數器
# > @tl|lock_got|26|main|計數器
# > @tl|lock_free||main|計數器
# > 總共 15 顆寶石
#
# 「31 結尾無窮迴圈」：31 solution 的 main 結尾加 loop {}；Run 逾時 kill
# > @tl|start|18|螃蟹 0|
# > @tl|lock_wait|6|螃蟹 0|計數器
# > @tl|lock_got|6|螃蟹 0|計數器
# > @tl|note|7|螃蟹 0|讀到 0
# > @tl|lock_free||螃蟹 0|計數器
# > @tl|end||螃蟹 0|
# > @tl|start|18|螃蟹 1|
# > @tl|lock_wait|6|螃蟹 1|計數器
# > @tl|lock_got|6|螃蟹 1|計數器
# > @tl|note|7|螃蟹 1|讀到 3
# > @tl|lock_free||螃蟹 1|計數器
# > @tl|end||螃蟹 1|
# > @tl|start|18|螃蟹 2|
# > @tl|lock_wait|6|螃蟹 2|計數器
# > @tl|lock_got|6|螃蟹 2|計數器
# > @tl|note|7|螃蟹 2|讀到 8
# > @tl|lock_free||螃蟹 2|計數器
# > @tl|end||螃蟹 2|
# > @tl|lock_wait|26|main|計數器
# > @tl|lock_got|26|main|計數器
# > @tl|lock_free||main|計數器
# > 總共 15 顆寶石
#
# 「31 螃蟹 panic」：31 solution 的 add_haul 在 gems == 5 時於鎖內 panic!("螃蟹把寶石掉進海裡了")；檔名 main.rs，exit 101
# > @tl|start|21|螃蟹 0|
# > @tl|lock_wait|6|螃蟹 0|計數器
# > @tl|lock_got|6|螃蟹 0|計數器
# > @tl|note|7|螃蟹 0|讀到 0
# > @tl|lock_free||螃蟹 0|計數器
# > @tl|end||螃蟹 0|
# > @tl|start|21|螃蟹 1|
# > @tl|lock_wait|6|螃蟹 1|計數器
# > @tl|lock_got|6|螃蟹 1|計數器
# > @tl|note|7|螃蟹 1|讀到 3
# > @tl|lock_free||螃蟹 1|計數器
# > @tl|end||螃蟹 1|
# > @tl|start|21|螃蟹 2|
# > @tl|lock_wait|6|螃蟹 2|計數器
# > @tl|lock_got|6|螃蟹 2|計數器
# > @tl|lock_free||螃蟹 2|計數器
# > @tl|end||螃蟹 2|
#   stderr（thread id 依執行而異）：
# > 
# > thread '<unnamed>' (11192978) panicked at main.rs:9:9:
# > 螃蟹把寶石掉進海裡了
# > note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
# > 
# > thread '<unnamed>' (11192979) panicked at main.rs:6:51:
# > called `Result::unwrap()` on an `Err` value: PoisonError { .. }
# > 
# > thread 'main' (11192969) panicked at main.rs:26:18:
# > called `Result::unwrap()` on an `Err` value: Any { .. }
#
# 「31 格式不對的事件」：31 solution 的 main 結尾自行 println! 五行假事件（未知 kind、line 為 0、段數不足、thread 空白、行首空白）；exit 0
# > @tl|start|18|螃蟹 0|
# > @tl|lock_wait|6|螃蟹 0|計數器
# > @tl|lock_got|6|螃蟹 0|計數器
# > @tl|note|7|螃蟹 0|讀到 0
# > @tl|lock_free||螃蟹 0|計數器
# > @tl|end||螃蟹 0|
# > @tl|start|18|螃蟹 1|
# > @tl|lock_wait|6|螃蟹 1|計數器
# > @tl|lock_got|6|螃蟹 1|計數器
# > @tl|note|7|螃蟹 1|讀到 3
# > @tl|lock_free||螃蟹 1|計數器
# > @tl|end||螃蟹 1|
# > @tl|start|18|螃蟹 2|
# > @tl|lock_wait|6|螃蟹 2|計數器
# > @tl|lock_got|6|螃蟹 2|計數器
# > @tl|note|7|螃蟹 2|讀到 8
# > @tl|lock_free||螃蟹 2|計數器
# > @tl|end||螃蟹 2|
# > @tl|lock_wait|26|main|計數器
# > @tl|lock_got|26|main|計數器
# > @tl|lock_free||main|計數器
# > 總共 15 顆寶石
# > @tl|jump|5|main|海灘
# > @tl|lock_got|0|main|計數器
# > @tl|note|5|main
# > @tl|lock_wait|5|  |計數器
# >  @tl|note|5|main|行首有空白
#
# 「30 未 join 分身」：30 solution 套用小幫手，dispatch 的分身 lane 後 sleep 200ms，main 不 join、sleep 50ms 後結束；exit 0
# > @tl|start|10|點名 0|
# > @tl|end||點名 0|
# > @tl|start|10|點名 2|
# > @tl|end||點名 2|
# > 分身 0 到！
# > @tl|start|10|點名 1|
# > @tl|end||點名 1|
# > 分身 1 到！
# > 分身 2 到！
# > @tl|start|34|分身 1|
# > @tl|start|34|分身 0|
# > @tl|start|34|分身 2|
#
# 「31 第一季草稿」：第一季 content/levels/31-shared-counter/starter.rs（沒有小幫手）；exit 0
# > 總共 0 顆寶石
#
# 「31 三百隻螃蟹」：31 solution 的 main 改為 gather((1..=300).collect())；exit 0；1,803 個合法事件、56,256 bytes（未超過 64 KiB），最後一行為「總共 45150 顆寶石」
#   事件格式同「31 solution」，每隻螃蟹 6 個事件，main 結尾 3 個事件；泳道出現順序依執行而異
#
# 「31 結尾無限上鎖（截斷）」：31 solution 的 main 結尾以 loop 反覆 timeline::lock 一把新的 Mutex（名稱「計數器」）；取前 65,536 bytes 模擬判題截斷，stdout_truncated 為 true
#   截斷後以 @tl| 開頭的行共 2,115 行（最後一行不完整，丟棄後 2,114 個合法事件）；前 1,000 個事件的泳道為 main、螃蟹 1、螃蟹 2、螃蟹 0
