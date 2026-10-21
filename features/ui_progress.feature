# 執行：playwright-bdd（frontend e2e）
# 規格：ux.md §1.1、§1.2、§2.1、§3.3、§6.1–§6.5；api.md §1；architecture.md §7.1、§7.2、§7.6；decisions.md §10、§14 ①②③、§15
# 預設以 page.route mock API；@docker 情境打真實後端（需 Docker 與 runner 映像）
# 「進度」指 localStorage key "rustquest.progress.v1"（結構見 ux.md §6.5）
# mock 關卡列表為測試用 8 關 fixture（取 MVP 8 關，非真後端的 33 關），chapters 為 1 沙灘甦醒、2 潮汐羅盤、3 所有權之海、5 漂流瓶、7 智慧指標洞窟、8 群蟹大遷徙（api.md §1、curriculum.md §1.7）

Feature: 星等、解鎖與提示計數
  三顆星各自判定並取歷史聯集，由前端計算後存進 localStorage
  本次遊玩只在 Submit 通過時結束；關卡依已載入的章節順序解鎖

  Background:
    Given mock API 回傳測試用 8 關 fixture 的關卡列表（含 chapters）與詳細資料
    And mock API 的 GET "/api/health" 回 status "ok"

  # ---------- 星等 ----------

  # ux.md §6.1（三顆星依本次遊玩的 current 判定）；architecture.md §7.6
  Scenario Outline: 通關時依本次遊玩計算三顆星
    Given 進度為空
    And mock API 的 Submit 依序回：<responses>
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家揭露 <hints> 則提示
    And 玩家依序點「送出」直到通關
    Then 通關結算顯示 ★通過 <pass>、★沒看提示 <noHint>、★一次就過 <firstTry>
    And 進度中 "01-hello-ferris" 的 stars 為 pass <pass>、noHint <noHint>、firstTry <firstTry>

    Examples:
      | hints | responses                             | pass | noHint | firstTry |
      | 0     | status "ok"                           | true | true   | true     |
      | 1     | status "ok"                           | true | false  | true     |
      | 0     | status "test_failed"、status "ok"     | true | true   | false    |
      | 2     | status "compile_error"、status "ok"   | true | false  | false    |

  # ux.md §6.5「已通關一律讀 stars.pass，不另存布林」
  Scenario: 進度只以 stars.pass 記錄通關
    Given 進度為空
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「送出」
    Then 進度中 "01-hello-ferris" 的 key 恰好為 "stars, current"

  # ux.md §6.1「計入失敗的 Submit」
  Scenario Outline: 判題失敗的 Submit 計為一次失敗嘗試
    Given 進度為空
    And mock API 的 Submit 回 status "<status>"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「送出」
    Then 進度中 "01-hello-ferris" 的 current.failedSubmits 為 1

    Examples:
      | status        |
      | compile_error |
      | test_failed   |
      | timeout       |
      | memory_limit  |
      | output_limit  |

  # ux.md §6.1「不計入：server_busy、judge_unavailable、payload_too_large、network_error」；architecture.md §7.6
  Scenario Outline: 系統問題與程式碼過長造成的 Submit 失敗不計入
    Given 進度為空
    And mock API 的 Submit <failure>
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「送出」
    Then 進度中 "01-hello-ferris" 的 current.failedSubmits 為 0

    Examples:
      | failure                                                    |
      | 回 503，error.code 為 "JUDGE_BUSY"，標頭 "Retry-After" 為 "2" |
      | 回 503，error.code 為 "JUDGE_UNAVAILABLE"，沒有 "Retry-After" 標頭 |
      | 回 413，error.code 為 "PAYLOAD_TOO_LARGE"                   |
      | 連線中斷                                                   |
      | 回 500，error.code 為 "INTERNAL_ERROR"                     |

  # ux.md §1.1 ④「Run 可無限次，不影響星等」、§6.1「Run 不計入次數」
  Scenario: Run 失敗不影響一次就過
    Given 進度為空
    And mock API 的 Run 回 status "compile_error"
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」3 次
    And 玩家點「送出」
    Then 通關結算顯示 ★一次就過 true

  # ux.md §6.1「三顆星分別記錄、取歷史聯集」範例
  Scenario: 重玩取得的星與歷史星取聯集
    Given 進度中 "01-hello-ferris" 的 stars 為 pass true、noHint false、firstTry true，current 為 hintsRevealed 0、failedSubmits 0
    And mock API 的 Submit 依序回：status "test_failed"、status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家依序點「送出」直到通關
    Then 進度中 "01-hello-ferris" 的 stars 為 pass true、noHint true、firstTry true

  # ---------- 本次遊玩的起訖 ----------

  # ux.md §6.1「唯一的結束點是 Submit 通過；回地圖、重新整理、直接輸入網址、重設都不結束本次遊玩」；§1.2
  Scenario Outline: 未通關前任何操作都不歸零本次遊玩計數
    Given 進度中 "01-hello-ferris" 的 stars 全為 false，current 為 hintsRevealed 2、failedSubmits 1
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家<action>
    Then 進度中 "01-hello-ferris" 的 current 為 hintsRevealed 2、failedSubmits 1
    And 提示區顯示第 1、2 則提示

    Examples:
      | action                                   |
      | 回章節地圖後再點節點進入同一關           |
      | 重新整理頁面                             |
      | 直接開啟網址 "/level/01-hello-ferris"    |
      | 在編輯器輸入 "// 改" 後點「重設」並確認  |

  # ux.md §6.1 ④「先用 current 算出本次的三顆星並與歷史聯集，再把 current 歸零」
  Scenario: Submit 通過後本次遊玩計數歸零
    Given 進度中 "01-hello-ferris" 的 stars 全為 false，current 為 hintsRevealed 1、failedSubmits 2
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「送出」
    Then 通關結算顯示 ★沒看提示 false、★一次就過 false
    And 進度中 "01-hello-ferris" 的 current 為 hintsRevealed 0、failedSubmits 0

  # ux.md §6.1 ① ②（current 不存在時視為 0；揭露提示、失敗的 Submit 立即寫入）
  Scenario: 第一次進關時計數從 0 開始並立即寫入
    Given 進度為空
    And mock API 的 Submit 回 status "test_failed"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家揭露 1 則提示
    Then 進度中 "01-hello-ferris" 的 current.hintsRevealed 為 1
    When 玩家點「送出」
    Then 進度中 "01-hello-ferris" 的 current.failedSubmits 為 1

  # ux.md §1.1 ⑦ 通關結算、§7.1（亮星實心、暗星空心外框）
  Scenario: 通關結算說明每顆星的條件與未達成的拿法
    Given 進度為空
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家揭露 1 則提示
    And 玩家點「送出」
    Then 通關結算中每顆星旁顯示達成條件與是否達成
    And 「沒看提示」星旁顯示一句怎麼拿到，例如「不看提示再挑戰一次就能拿到」
    And 亮星為實心、暗星為空心外框
    And 通關結算有按鈕「下一關」「再玩一次」「回地圖」，「下一關」為主要按鈕

  # ux.md §1.1 ⑧（下一關已解鎖：直接進入；仍鎖定或沒有下一關：回章節地圖）
  Scenario Outline: 通關結算的下一關按鈕
    Given 進度中已拿到 ★1 的關卡為 "<cleared>"
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "<level>"
    When 玩家點「送出」
    And 玩家點「下一關」
    Then 網址為 "<url>"

    Examples:
      | cleared                                                                                 | level             | url                    |
      |                                                                                         | 01-hello-ferris   | /level/02-mut-matters  |
      | 01-hello-ferris, 02-mut-matters, 05-compass-enum, 09-move-the-gem, 11-one-writer-rule, 19-question-mark, 28-refcell-panic | 31-shared-counter | / |

  # ---------- 提示揭露 ----------

  # ux.md §6.2（k = 0 時確認、k ≥ 1 後不再確認）
  Scenario: 第一次看提示需要確認，之後不再確認
    Given 進度為空
    And 玩家開啟關卡 "01-hello-ferris"
    Then 提示區顯示按鈕「需要提示嗎？（0 / 3）」
    When 玩家點提示按鈕
    Then 出現確認「看提示會拿不到『沒看提示』這顆星，確定要看嗎？」
    When 玩家確認
    Then 顯示第 1 則提示
    And 提示區顯示按鈕「需要提示嗎？（1 / 3）」
    When 玩家再點提示按鈕
    Then 沒有出現確認
    And 顯示第 1、2 則提示

  # ux.md §6.2「全部揭露後按鈕隱藏」
  Scenario: 全部提示揭露後按鈕隱藏
    Given 進度為空
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家揭露 3 則提示
    Then 顯示第 1、2、3 則提示
    And 提示區沒有提示按鈕

  # ux.md §6.2「進入關卡時依 current.hintsRevealed 還原已揭露的提示」
  Scenario: 重新整理後還原已揭露的提示且不再確認
    Given 進度為空
    And 玩家開啟關卡 "01-hello-ferris"
    And 玩家揭露 2 則提示
    When 玩家重新整理頁面
    Then 顯示第 1、2 則提示
    And 提示區顯示按鈕「需要提示嗎？（2 / 3）」
    When 玩家點提示按鈕
    Then 沒有出現確認
    And 顯示第 1、2、3 則提示

  # ux.md §6.2「通關後歸零，提示區回到收起狀態」
  Scenario: 通關後重玩時提示收起
    Given 進度為空
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    And 玩家揭露 2 則提示
    When 玩家點「送出」
    And 玩家點「再玩一次」
    Then 提示區顯示按鈕「需要提示嗎？（0 / 3）」
    And 沒有顯示任何提示內容

  # ux.md §6.2「hints 為空陣列的關卡不顯示提示區，★2 照常判定」
  Scenario: 沒有提示的關卡不顯示提示區
    Given mock API 關卡 "01-hello-ferris" 的 hints 為空陣列
    And 進度為空
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    Then 左欄沒有提示區
    When 玩家點「送出」
    Then 通關結算顯示 ★沒看提示 true

  # ---------- 解鎖 ----------

  # ux.md §6.3（只算 chapters 內出現的章節、依 (chapter, order) 排序）；architecture.md §7.2
  Scenario Outline: 依章節與前一關進度解鎖
    Given 進度中已拿到 ★1 的關卡為 "<cleared>"
    When 玩家開啟章節地圖
    Then 關卡 "<level>" 的節點為 <state>

    Examples:
      | cleared                                                            | level              | state  |
      |                                                                    | 01-hello-ferris    | 可挑戰 |
      |                                                                    | 02-mut-matters     | 鎖定   |
      | 01-hello-ferris                                                    | 02-mut-matters     | 可挑戰 |
      | 01-hello-ferris                                                    | 05-compass-enum    | 鎖定   |
      | 01-hello-ferris, 02-mut-matters                                    | 05-compass-enum    | 可挑戰 |
      | 01-hello-ferris, 02-mut-matters, 05-compass-enum                   | 09-move-the-gem    | 可挑戰 |
      | 01-hello-ferris, 02-mut-matters, 05-compass-enum                   | 11-one-writer-rule | 鎖定   |

  # ux.md §6.3「MVP 沒有第 4 章關卡，第 5 章的前一個章節是第 3 章」；測試用 8 關 fixture 沒有第 4、6 章；decisions.md §14 ③
  Scenario Outline: 跨過沒有關卡的章節解鎖
    Given 進度中已拿到 ★1 的關卡為 "<cleared>"
    When 玩家開啟章節地圖
    Then 關卡 "<level>" 的節點為 <state>
    And 章節地圖沒有第 4 章與第 6 章的區塊

    Examples:
      | cleared                                                                                   | level            | state  |
      | 01-hello-ferris, 02-mut-matters, 05-compass-enum, 09-move-the-gem                         | 19-question-mark | 鎖定   |
      | 01-hello-ferris, 02-mut-matters, 05-compass-enum, 09-move-the-gem, 11-one-writer-rule     | 19-question-mark | 可挑戰 |
      | 01-hello-ferris, 02-mut-matters, 05-compass-enum, 09-move-the-gem, 11-one-writer-rule, 19-question-mark | 28-refcell-panic | 可挑戰 |

  # ux.md §6.3「星等不影響解鎖」
  Scenario: 只拿到 ★1 也能解鎖下一關
    Given 進度中 "01-hello-ferris" 的 stars 為 pass true、noHint false、firstTry false，current 為 hintsRevealed 0、failedSubmits 0
    When 玩家開啟章節地圖
    Then 關卡 "02-mut-matters" 的節點為 可挑戰

  # ---------- 章節地圖與頂部列 ----------

  # ux.md §6.4 章節區塊標題、節點狀態；§6.3 累計星數「★ 已得 / 該章已載入關卡數 × 3」
  Scenario: 章節地圖顯示章節名稱、節點狀態與累計星數
    Given 進度中 "01-hello-ferris" 的 stars 為 pass true、noHint true、firstTry false，current 為 hintsRevealed 0、failedSubmits 0
    When 玩家開啟章節地圖
    Then 章節區塊標題依序為「第 1 章 沙灘甦醒」「第 2 章 潮汐羅盤」「第 3 章 所有權之海」「第 5 章 漂流瓶」「第 7 章 智慧指標洞窟」「第 8 章 群蟹大遷徙」
    And 第 1 章顯示「★ 2 / 6」
    And 第 2 章顯示「★ 0 / 3」
    And 第 3 章顯示「★ 0 / 6」
    And 關卡 "01-hello-ferris" 的節點為實心，下方 2 顆亮星、1 顆暗星
    And 關卡 "02-mut-matters" 的節點為可挑戰，且 Ferris 圖示站在這裡
    And 關卡 "05-compass-enum" 的節點有鎖頭圖示，路徑為虛線

  # ux.md §6.4「全部通過時 Ferris 站在最後一關」
  Scenario: 全部通過時 Ferris 站在最後一關
    Given 進度中測試用 8 關 fixture 都已拿到 ★1
    When 玩家開啟章節地圖
    Then Ferris 圖示站在關卡 "31-shared-counter" 的節點

  # ux.md §6.4 關卡編號格式「章-章內序」
  Scenario Outline: 節點以「章-章內序」顯示關卡編號、名稱與 kind 圖示
    Given 進度中測試用 8 關 fixture 都已拿到 ★1
    When 玩家開啟章節地圖
    Then 關卡 "<id>" 的節點下方顯示「<label>」與 kind 圖示「<icon>」

    Examples:
      | id                 | label             | icon |
      | 01-hello-ferris    | 1-1 第一步        | 🔨   |
      | 02-mut-matters     | 1-2 黏住的腳      | 🩹   |
      | 05-compass-enum    | 2-1 神奇羅盤      | 🔨   |
      | 09-move-the-gem    | 3-1 交出去的背包  | 🩹   |
      | 11-one-writer-rule | 3-2 一寫多讀      | 🩹   |
      | 19-question-mark   | 5-1 沒有寶石的島  | 🔍   |
      | 28-refcell-panic   | 7-1 寶物箱卡住了  | 🔍   |
      | 31-shared-counter  | 8-1 消失的寶石    | 🔍   |

  # ux.md §2.1 頂部列「第 N 章 章節名稱 ▸ 關卡編號 關卡名」、§1.2（關卡頁一律先取得 GET /api/levels）
  Scenario: 直接開啟關卡網址時頂部列顯示章節名稱與編號
    Given 進度中測試用 8 關 fixture 都已拿到 ★1
    When 玩家直接開啟網址 "/level/11-one-writer-rule"
    Then mock API 收到 1 次 GET "/api/levels" 請求
    And 頂部列顯示「第 3 章 所有權之海 ▸ 3-2 一寫多讀」

  # ux.md §1.1 ②「點鎖定節點：不跳頁，節點旁顯示 tooltip」（同章：前一關）
  Scenario: 點鎖定節點顯示需先完成的關卡
    Given 進度為空
    When 玩家開啟章節地圖
    And 玩家點關卡 "02-mut-matters" 的節點
    Then 網址仍為 "/"
    And 節點旁顯示 tooltip「先完成〈第一步〉」

  # ux.md §1.1 ②（跨章鎖定：前一章第一個未通過的關卡）
  Scenario: 點跨章鎖定的節點顯示前一章第一個未通過的關卡
    Given 進度中已拿到 ★1 的關卡為 "01-hello-ferris, 02-mut-matters, 05-compass-enum, 09-move-the-gem"
    When 玩家開啟章節地圖
    And 玩家點關卡 "19-question-mark" 的節點
    Then 網址仍為 "/"
    And 節點旁顯示 tooltip「先完成〈一寫多讀〉」

  # ux.md §1.2「直接輸入鎖定關卡的 URL」；architecture.md §7.1 beforeEnter
  Scenario: 直接輸入鎖定關卡的網址會導回地圖
    Given 進度為空
    When 玩家直接開啟網址 "/level/02-mut-matters"
    Then 網址為 "/"
    And 顯示 toast「這關還沒解鎖」

  # ---------- localStorage 異常 ----------

  # ux.md §6.5「讀取失敗（JSON 壞掉）時：以空進度啟動，toast」
  Scenario: 進度資料損壞時以空進度啟動
    Given localStorage 的 "rustquest.progress.v1" 為 "{not json"
    When 玩家開啟章節地圖
    Then 顯示 toast「進度無法讀取，已從頭開始」
    And 關卡 "01-hello-ferris" 的節點為 可挑戰
    And 關卡 "02-mut-matters" 的節點為 鎖定

  # ux.md §6.5「第一次進站（沒有資料）不 toast」
  Scenario: 第一次進站沒有進度資料時不提示
    Given localStorage 沒有 key "rustquest.progress.v1"
    When 玩家開啟章節地圖
    Then 沒有顯示 toast「進度無法讀取，已從頭開始」
    And 關卡 "01-hello-ferris" 的節點為 可挑戰

  # ux.md §6.5「version 不符時：以空進度啟動」
  Scenario: 進度版本不符時以空進度啟動
    Given localStorage 的 "rustquest.progress.v1" 為 `{"version": 2, "levels": {"01-hello-ferris": {"stars": {"pass": true, "noHint": true, "firstTry": true}}}}`
    When 玩家開啟章節地圖
    Then 關卡 "01-hello-ferris" 的節點為 可挑戰
    And 關卡 "02-mut-matters" 的節點為 鎖定

  # ux.md §6.5「寫入失敗時：toast，遊戲照常進行」
  Scenario: 進度寫入失敗時提示但照常通關
    Given 進度為空
    And localStorage 寫入會失敗
    And mock API 的 Submit 回 status "ok"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「送出」
    Then 顯示 toast「進度無法儲存」
    And 顯示通關結算 Modal

  # ---------- 真實後端 ----------

  # architecture.md §6.2 ⑦ E2E 主流程（打真後端）
  @docker
  Scenario: 第一關從 Run 到通關、解鎖下一關並在重新整理後保留進度
    Given 前端連到真實後端，判題服務可用
    And 進度為空
    And 玩家從章節地圖進入關卡 "01-hello-ferris"
    When 玩家把編輯器內容換成該關 solution.rs
    And 玩家點「執行」
    Then 主控台「輸出」分頁第一行為 "MOVE DOWN"
    When 玩家點「送出」
    Then 顯示通關結算 Modal
    When 玩家點「回地圖」
    Then 關卡 "02-mut-matters" 的節點為 可挑戰
    When 玩家重新整理頁面
    Then 關卡 "01-hello-ferris" 的節點為 已通過
    And 關卡 "02-mut-matters" 的節點為 可挑戰
