# 執行：playwright-bdd（frontend e2e）
# 規格：ux.md §3.2、§3.3、§4、§7.2、§7.3、§7.4；architecture.md §7.5；decisions.md §8、§14 ③、§15
# 預設以 page.route mock API
# 測試地圖沿用 ux.md §4.2 範例（mock 關卡 id "fixture-grid"，第 1 章第 1 關）：
#   width 6、height 5、start [0, 0]、goal [5, 4]
#   walls [[1,0],[1,1],[1,2],[3,2],[3,3],[4,3]]、gems [[2,0],[4,4]]、doors [[3,4]]
# 「通關路線」指以下 20 行 stdout（撿完 2 顆寶石並停在終點）：
#   MOVE DOWN ×4、MOVE RIGHT ×2、MOVE UP ×4、PICK、MOVE DOWN ×4、OPEN RIGHT、MOVE RIGHT、MOVE RIGHT、PICK、MOVE RIGHT

Feature: GridWorld 指令動畫
  玩家程式用 println! 一行印一個指令，前端依 Run 的 stdout 驅動 Ferris 在地圖上移動
  GridWorld 只做視覺回放，通關以 Submit 結果為準

  Background:
    Given mock API 只有一章（第 1 章）一關 "fixture-grid"，visual 為 "grid"，地圖為 ux.md §4.2 範例
    And mock API 的 GET "/api/health" 回 status "ok"
    And 玩家開啟關卡 "fixture-grid"

  # ---------- 初始與自動播放 ----------

  # ux.md §4.1（stdout 為 null 時顯示初始地圖）、§4.6
  Scenario: 尚未執行時顯示初始地圖
    Then Ferris 位於 (0, 0)
    And 地圖有 6 個牆格、2 顆寶石、1 扇關閉的門、1 個終點旗子
    And 地圖右上角顯示「💎 0 / 2」

  # ux.md §4.4「自動播放時機」、§4.4 速度「1x 時每步 500ms」
  Scenario: Run 成功後自動從第 1 步開始播放
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    Then 播放控制列顯示「第 1 / 20 步」
    And 播放按鈕顯示為「暫停」
    And 約 500 ms 後播放控制列顯示「第 2 / 20 步」

  # ---------- 播放、暫停、單步、從頭、跳步 ----------

  # ux.md §4.4 播放／暫停、單步
  Scenario: 暫停後單步前進
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    And 玩家點「暫停」
    And 玩家點「單步」
    Then 播放已暫停
    And 步數比按「單步」前多 1

  # ux.md §4.4「單步：播放中按下也會先暫停」
  Scenario: 播放中按單步會暫停
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    And 播放中玩家點「單步」
    Then 播放已暫停
    And 播放按鈕顯示為「播放」

  # ux.md §4.4「從頭：回到第 0 步（初始地圖），暫停」
  Scenario: 從頭回到初始地圖並暫停
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    And 玩家點進度條跳到第 10 步
    And 玩家點「從頭」
    Then 播放控制列顯示「第 0 / 20 步」
    And Ferris 位於 (0, 0)
    And 播放已暫停

  # ux.md §4.4「播到最後一步自動變回播放，再按則從頭播」
  Scenario: 播完後再按播放從頭播
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」並等待播放結束
    Then 播放按鈕顯示為「播放」
    When 玩家點「播放」
    Then 播放控制列顯示「第 1 / 20 步」

  # ux.md §4.4 進度條「跳步時直接計算該步狀態，不播放中間動畫」
  Scenario: 點進度條跳到指定步
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    And 玩家點進度條跳到第 11 步
    Then Ferris 位於 (2, 0)
    And 地圖右上角顯示「💎 1 / 2」
    And (2, 0) 的寶石已消失

  # ux.md §4.4 速度「只存在記憶體，重新整理回到 1x；不寫 localStorage」；§6.5 Prefs 只有 theme
  Scenario: 播放速度只存在記憶體，重新整理後回到 1x
    When 玩家把速度改為 "2x"
    Then 速度選單顯示 "2x"
    And localStorage 的 "rustquest.prefs.v1" 不含 "playbackSpeed"
    When 玩家重新整理頁面
    Then 速度選單顯示 "1x"

  # ux.md §7.2「可視化區取得焦點時」
  Scenario Outline: 可視化區的鍵盤操作
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    And 玩家點進度條跳到第 5 步
    And 焦點在可視化區
    And 玩家按 "<key>"
    Then <result>

    Examples:
      | key        | result                                   |
      | Space      | 播放按鈕顯示為「暫停」                   |
      | ArrowRight | 播放控制列顯示「第 6 / 20 步」           |
      | Home       | 播放控制列顯示「第 0 / 20 步」           |

  # ux.md §3.2「可視化區：停止播放，回到第 0 步並半透明」、§4.1 pending prop
  Scenario: 再次送出請求時可視化回到第 0 步並半透明
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」並等待播放結束
    And mock API 的 Run 改為延遲 2 秒後回應
    And 玩家點「執行」
    Then 播放控制列顯示「第 0 / 20 步」
    And 可視化區為半透明

  # ---------- 通關條件（視覺上）----------

  # ux.md §4.2「通關條件（視覺上的判定）」、§4.5「抵達終點且全撿」、§3.3 ok（Run 成功不等於通關）
  Scenario: 撿完全部寶石並停在終點只是視覺回饋
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」並等待播放結束
    Then Ferris 位於 (5, 4)
    And 地圖右上角顯示「💎 2 / 2」
    And Ferris 跳一下、終點旗幟揮動
    And 頂部列下方顯示「看起來可以了，按『送出』驗收」
    And 不出現通關結算 Modal

  # ux.md §4.5「指令播完但沒到終點」
  Scenario: 指令播完但沒到終點
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      MOVE DOWN
      MOVE DOWN
      """
    When 玩家點「執行」並等待播放結束
    Then Ferris 位於 (0, 2)
    And 訊息列顯示「指令跑完了，但 Ferris 還沒到終點」
    And Ferris 頭上顯示「…」

  # ux.md §4.2 通關條件、§4.5「到終點但寶石沒撿完」
  Scenario: 到達終點但寶石沒撿完
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      MOVE DOWN
      MOVE DOWN
      MOVE DOWN
      MOVE DOWN
      MOVE RIGHT
      MOVE RIGHT
      OPEN RIGHT
      MOVE RIGHT
      MOVE RIGHT
      PICK
      MOVE RIGHT
      """
    When 玩家點「執行」並等待播放結束
    Then Ferris 位於 (5, 4)
    And 地圖右上角顯示「💎 1 / 2」
    And 訊息列顯示「到終點了，但還有 1 顆寶石沒撿」
    And Ferris 頭上顯示「…」
    And Ferris 沒有跳一下、終點旗幟沒有揮動

  # ux.md §4.5 撿到寶石、開門
  Scenario: 撿寶石與開門的畫面變化
    Given mock API 的 Run 回 status "ok"，stdout 為「通關路線」
    When 玩家點「執行」
    And 玩家點進度條跳到第 16 步
    Then (3, 4) 的門顯示為開啟狀態、格子為可通行底色
    And 地圖右上角顯示「💎 1 / 2」

  # ---------- 撞牆等錯誤立即停止 ----------

  # ux.md §4.3「遇到任何錯誤立即停止播放」、§4.5 各錯誤的訊息
  Scenario Outline: 遇到錯誤時停在該步、後續指令不執行
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      <first>
      MOVE DOWN
      """
    When 玩家點「執行」並等待播放結束
    Then 播放停在第 1 步，進度條總步數為 2
    And Ferris 位於 (0, 0)
    And Ferris 頭上顯示「<icon>」
    And 訊息列顯示「<message>」

    Examples:
      | first      | icon | message                    |
      | MOVE RIGHT | 💥   | 第 1 步：往右撞到牆了（1, 0） |
      | MOVE UP    | 💥   | 再走就掉出地圖了            |
      | PICK       | ❓   | 這格沒有寶石                |
      | OPEN DOWN  | ❓   | 那邊沒有門                  |

  # ux.md §4.5 撞牆「牆格閃紅框 2 次」、走出地圖「地圖外框該側閃紅」
  Scenario: 撞牆時牆格閃紅框
    Given mock API 的 Run 回 status "ok"，stdout 為 "MOVE RIGHT\n"
    When 玩家點「執行」並等待播放結束
    Then (1, 0) 的牆格閃紅框 2 次

  # ux.md §4.5 撞到關閉的門
  Scenario: 撞到關閉的門時停止並提示先開門
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      MOVE DOWN
      MOVE DOWN
      MOVE DOWN
      MOVE DOWN
      MOVE RIGHT
      MOVE RIGHT
      MOVE RIGHT
      MOVE RIGHT
      """
    When 玩家點「執行」並等待播放結束
    Then 播放停在第 7 步
    And Ferris 位於 (2, 4)
    And (3, 4) 的門格上顯示「🔒」
    And 訊息列含「門還關著，先 OPEN RIGHT」

  # ux.md §4.5 撞到關閉的門（<方向> 依撞門那一步 MOVE 的方向）
  Scenario: 從右側撞門時提示往左開門
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      MOVE DOWN
      MOVE DOWN
      MOVE DOWN
      MOVE RIGHT
      MOVE RIGHT
      MOVE UP
      MOVE UP
      MOVE RIGHT
      MOVE RIGHT
      MOVE RIGHT
      MOVE DOWN
      MOVE DOWN
      MOVE DOWN
      MOVE LEFT
      MOVE LEFT
      """
    When 玩家點「執行」並等待播放結束
    Then 播放停在第 15 步
    And Ferris 位於 (4, 4)
    And (3, 4) 的門格上顯示「🔒」
    And 訊息列含「門還關著，先 OPEN LEFT」

  # ---------- 無效指令與一般輸出 ----------

  # ux.md §4.3 ③④、§4.5 無效指令
  Scenario: 方向錯誤的指令為無效指令
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      MOVE DOWN
      MOVE FORWARD
      MOVE DOWN
      """
    When 玩家點「執行」並等待播放結束
    Then 播放停在第 2 步
    And Ferris 位於 (0, 1)
    And Ferris 頭上顯示「❓」
    And 訊息列顯示「第 2 步：看不懂 `MOVE FORWARD`，方向只能是 UP/DOWN/LEFT/RIGHT」
    And 主控台 "MOVE FORWARD" 那一行有紅色左邊框與「✗」圖示

  # ux.md §4.3 ③、§4.5 無效指令的三種說明；decisions.md §8（不保留 PICK GEM、SAY）、§14 ③
  Scenario Outline: 不合法的指令嘗試依原因顯示對應說明
    Given mock API 的 Run 回 status "ok"，stdout 為 "<line>\nMOVE DOWN\n"
    When 玩家點「執行」並等待播放結束
    Then 播放停在第 1 步
    And Ferris 頭上顯示「❓」
    And 訊息列顯示「<message>」

    Examples:
      | line       | message                                                    |
      | JUMP       | 第 1 步：看不懂 `JUMP`，指令只有 MOVE、PICK、OPEN            |
      | SAY HELLO  | 第 1 步：看不懂 `SAY HELLO`，指令只有 MOVE、PICK、OPEN       |
      | MOVE       | 第 1 步：看不懂 `MOVE`，方向只能是 UP/DOWN/LEFT/RIGHT        |
      | OPEN       | 第 1 步：看不懂 `OPEN`，方向只能是 UP/DOWN/LEFT/RIGHT        |
      | MOVE UP UP | 第 1 步：看不懂 `MOVE UP UP`，方向只能是 UP/DOWN/LEFT/RIGHT  |
      | PICK GEM   | 第 1 步：看不懂 `PICK GEM`，PICK 後面不用接東西              |

  # ux.md §4.3 ②（不是指令嘗試的行只顯示在主控台）、①（空行忽略、去除首尾空白）、③（多個空白視為一個、大小寫敏感）
  Scenario: 非指令行只顯示在主控台，不影響 Ferris
    Given mock API 的 Run 回 status "ok"，stdout 為：
      """
      debug: x = 1
      move up

        MOVE   DOWN
      箱內 2 樣
      MOVE DOWN
      """
    When 玩家點「執行」並等待播放結束
    Then 進度條總步數為 2
    And Ferris 位於 (0, 2)
    And 主控台「輸出」分頁顯示 "debug: x = 1"、"move up"、"箱內 2 樣"
    And 沒有無效指令提示

  # ---------- 步數上限 1,000 ----------

  # ux.md §4.3 步數上限、§4.5 超過步數上限；architecture.md §7.5；decisions.md §8
  Scenario Outline: 指令步數上限 1,000 步
    Given mock API 的 Run 回 status "ok"，stdout 為 <count> 行，依序交替 "MOVE DOWN"、"MOVE UP"
    When 玩家點「執行」並等待播放結束
    Then 進度條總步數為 <total>
    And 播放停在第 <total> 步
    And 訊息列顯示「<message>」

    Examples:
      | count | total | message                              |
      | 1000  | 1000  | 指令跑完了，但 Ferris 還沒到終點      |
      | 1001  | 1000  | 指令超過 1,000 步，後面的不播放了     |

  # ux.md §4.3「指令序列（含 invalid）超過 1,000 步時，只保留前 1,000 步」
  Scenario: 無效指令也計入步數上限
    Given mock API 的 Run 回 status "ok"，stdout 為 1000 行交替 "MOVE DOWN"、"MOVE UP"，再接 1 行 "JUMP"
    When 玩家點「執行」並等待播放結束
    Then 進度條總步數為 1000
    And 訊息列顯示「指令超過 1,000 步，後面的不播放了」

  # ---------- 程式被終止、panic、編譯失敗 ----------

  # ux.md §3.3 timeout／memory_limit、§4.5「程式被終止」；architecture.md §7.5
  Scenario Outline: 程式被終止時播放到最後一個收到的指令
    Given mock API 的 Run 回 status "<status>"，stdout 為 "MOVE DOWN\nMOVE DOWN\n"，stdout_truncated 為 false
    When 玩家點「執行」並等待播放結束
    Then 進度條總步數為 2
    And Ferris 位於 (0, 2)
    And 訊息列顯示「<message>」

    Examples:
      | status       | message    |
      | timeout      | 時間到     |
      | memory_limit | 記憶體不足 |

  # ux.md §3.3 output_limit、§4.3「輸出被截斷：解析前先丟棄最後一行」
  Scenario: 輸出被截斷時丟棄不完整的最後一行
    Given mock API 的 Run 回 status "output_limit"，stdout 為 "MOVE DOWN\nMOVE DOWN\nMOVE RI"，stdout_truncated 為 true
    When 玩家點「執行」並等待播放結束
    Then 進度條總步數為 2
    And 沒有無效指令提示
    And 訊息列顯示「輸出過長已截斷」

  # ux.md §3.3 runtime_panic「播放到 panic 前最後一個指令，停在該步並標示」
  Scenario: panic 時停在最後一個指令並標示程式在這裡結束
    Given mock API 的 Run 回 status "runtime_panic"，stdout 為 "MOVE DOWN\n"
    When 玩家點「執行」並等待播放結束
    Then 播放停在第 1 步
    And Ferris 位於 (0, 1)
    And 可視化區標示「程式在這裡結束」

  # ux.md §3.3 compile_error「GridWorld 維持初始地圖並顯示『程式還沒編譯成功』」、memory_limit「execution = null 時同 compile_error」、§4.4 自動播放（execution 不為 null）
  Scenario Outline: 編譯失敗或編譯階段 OOM 時維持初始地圖
    Given mock API 的 Run 回 status "<status>"，execution 為 null
    When 玩家點「執行」
    Then 可視化區顯示「程式還沒編譯成功」
    And Ferris 位於 (0, 0)

    Examples:
      | status        |
      | compile_error |
      | memory_limit  |

  # ux.md §3.3 test_failed、§4.1「Submit 不驅動 GridWorld」；decisions.md §8
  Scenario: Submit 不重播，維持最後一次 Run 的畫面
    Given mock API 的 Run 回 status "ok"，stdout 為 "MOVE DOWN\nMOVE DOWN\n"
    And mock API 的 Submit 回 status "test_failed"
    When 玩家點「執行」並等待播放結束
    And 玩家點「送出」
    Then Ferris 位於 (0, 2)
    And 播放控制列顯示「第 2 / 2 步」

  # ux.md §3.2（pending：請求中回第 0 步並半透明；Submit 回來後還原到上次 Run 的終點）、§4.1 pending prop
  Scenario: Submit 請求中回第 0 步，回來後還原到上次 Run 的終點
    Given mock API 的 Run 回 status "ok"，stdout 為 "MOVE DOWN\nMOVE DOWN\n"
    And mock API 的 Submit 延遲 2 秒後回 status "test_failed"
    When 玩家點「執行」並等待播放結束
    And 玩家點「送出」
    Then 播放控制列顯示「第 0 / 2 步」
    And 可視化區為半透明
    When 等待請求完成
    Then 可視化區不是半透明
    And Ferris 位於 (0, 2)
    And 播放控制列顯示「第 2 / 2 步」

  # ---------- 動態效果 ----------

  # ux.md §4.4、§7.4「prefers-reduced-motion: reduce 時不做位移補間，直接跳格」
  Scenario: 減少動態效果時 Ferris 直接跳格
    Given 瀏覽器設定 prefers-reduced-motion 為 "reduce"
    And mock API 的 Run 回 status "ok"，stdout 為 "MOVE DOWN\n"
    When 玩家點「執行」
    Then Ferris 從 (0, 0) 到 (0, 1) 沒有位移補間動畫

  # ux.md §7.3「GridWorld 訊息列同時放進 aria-live 區」
  Scenario: 訊息列內容放進 aria-live 區
    Given mock API 的 Run 回 status "ok"，stdout 為 "MOVE RIGHT\n"
    When 玩家點「執行」並等待播放結束
    Then aria-live 區含「第 1 步：往右撞到牆了（1, 0）」
