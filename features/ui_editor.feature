# 執行：playwright-bdd（frontend e2e）
# 規格：ux.md §1.2、§1.3、§1.4、§2、§3、§6.6、§7.2、§7.3、§8.3；api.md §0.2、§0.3；architecture.md §3.6、§7.2、§7.4；decisions.md §14 ③⑥、§15
# 預設以 page.route mock API；「mock 回應範例」指 api.md v1.3 中同名的 JSON 範例，step 直接載入為 fixture：
#   Run「成功」「編譯錯誤」「執行超時」「panic」：api.md §3
#   Submit「全部通過」「部分失敗，含 warning」「編譯錯誤（錯誤落在隱藏測試區段）」「執行超時」：api.md §4
# 「Ctrl」在 macOS 代表 Cmd（ux.md §7.2）

Feature: 關卡頁編輯器、Run／Submit 與結果顯示
  玩家在編輯器寫程式，按 Run 試跑、按 Submit 驗收
  每種判題結果與請求失敗都有對應的主控台、編輯器與可視化畫面

  Background:
    Given mock API 回傳測試用 8 關 fixture 的關卡列表（含 chapters）與詳細資料
    And mock API 的 GET "/api/health" 回 status "ok"
    And 玩家進度中測試用 8 關 fixture 都已拿到 ★1

  # ---------- 關卡頁進入 ----------

  # ux.md §1.4（泡泡內容即 story，判題結果不改泡泡內容）
  Scenario: Ferris 對話泡泡顯示關卡 story，判題後不變
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    Then 左欄頂部的 Ferris 對話泡泡顯示關卡 "09-move-the-gem" 的 story 全文
    And 泡泡沒有「下一句」按鈕
    When 玩家點「執行」
    Then Ferris 對話泡泡內容與執行前相同

  # ux.md §1.3 fix-compile「關卡開啟時不自動編譯」
  Scenario: fix-compile 關卡開啟時不自動送出 Run
    When 玩家開啟關卡 "09-move-the-gem"
    Then 任務橫幅顯示「這段程式編不過，先按執行看錯誤訊息，再修好它」
    And mock API 共收到 0 次 Run 請求

  # ux.md §1.2「GET /api/levels/:id 回 404：顯示『找不到這關』頁，附回地圖連結」
  Scenario: 關卡不存在時顯示找不到這關
    Given mock API 的 GET "/api/levels/99-unknown" 回 404，error.code 為 "LEVEL_NOT_FOUND"
    When 玩家直接開啟網址 "/level/99-unknown"
    Then 畫面顯示「找不到這關」
    And 畫面有回地圖的連結

  # ---------- Run／Submit 請求中 ----------

  # ux.md §3.2 Loading
  Scenario: Run 請求中的按鈕、編輯器與主控台狀態
    Given mock API 的 Run 延遲 2 秒後回「成功」範例
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 「執行」按鈕顯示 spinner 與文字「判題中…」
    And 「送出」與「重設」按鈕為 disabled
    And 編輯器仍可輸入文字
    And 主控台顯示骨架列

  # ux.md §3.2 Loading（Submit 請求中：被按下的「送出」顯示 spinner，另一顆與重設 disabled）
  Scenario: Submit 請求中的按鈕、編輯器與主控台狀態
    Given mock API 的 Submit 延遲 2 秒後回「全部通過」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then 「送出」按鈕顯示 spinner 與文字「判題中…」
    And 「執行」與「重設」按鈕為 disabled
    And 編輯器仍可輸入文字
    And 主控台顯示骨架列

  # ux.md §3.2「超過 3 秒仍未回應」
  Scenario: 請求超過 3 秒時提示第一次編譯比較久
    Given mock API 的 Run 延遲 5 秒後回「成功」範例
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」並等待 3 秒
    Then 主控台顯示「第一次編譯比較久，請稍候」

  # ux.md §3.2「同一時間只允許一個請求；loading 中再按快捷鍵不重送」
  Scenario: 請求中再按快捷鍵不會重送
    Given mock API 的 Run 延遲 2 秒後回「成功」範例
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    And 玩家在請求中按 "Control+Enter" 與 "Control+Shift+Enter"
    And 等待請求完成
    Then mock API 共收到 1 次 Run 請求、0 次 Submit 請求

  # ux.md §3.2「送出期間程式碼被改，結果回來後主控台頂部顯示提示」
  Scenario: 請求中修改程式碼，結果標示為修改前的結果
    Given mock API 的 Run 延遲 2 秒後回「成功」範例
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    And 玩家在請求中於編輯器輸入 "// 改一下"
    And 等待請求完成
    Then 主控台頂部顯示「這是修改前那份程式碼的結果」

  # ---------- 快捷鍵 ----------

  # ux.md §7.2 快捷鍵與「實作注意」（編輯器內外都要接、macOS 用 Cmd）
  Scenario Outline: 用快捷鍵觸發 Run 與 Submit
    Given 瀏覽器平台為 "<platform>"
    And mock API 的 Run 回「成功」範例
    And mock API 的 Submit 回「全部通過」範例
    And 玩家開啟關卡 "01-hello-ferris"
    And 焦點在<focus>
    When 玩家按 "<keys>"
    Then mock API 收到 1 次 <request> 請求
    And 編輯器內容與按鍵前相同

    Examples:
      | platform | focus    | keys                | request |
      | Windows  | 編輯器內 | Control+Enter       | Run     |
      | Windows  | 編輯器外 | Control+Enter       | Run     |
      | Windows  | 編輯器內 | Control+Shift+Enter | Submit  |
      | Windows  | 編輯器外 | Control+Shift+Enter | Submit  |
      | macOS    | 編輯器內 | Meta+Enter          | Run     |
      | macOS    | 編輯器內 | Meta+Shift+Enter    | Submit  |

  # ---------- ok ----------

  # ux.md §3.3 ok（Run 成功）
  Scenario: Run 成功顯示輸出並提醒按送出驗收
    Given mock API 的 Run 回「成功」範例
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台切到「輸出」分頁
    And 「輸出」分頁頂部顯示綠色列「✓ 執行完成」
    And 「輸出」分頁顯示 7 行 stdout，第一行為 "MOVE DOWN"
    And 頂部列下方顯示「看起來可以了，按『送出』驗收」
    And 不出現通關結算 Modal

  # ux.md §3.3 ok（Run 成功）「有 warning 時照 compile_error 的規則畫黃色波浪線」「warning 卡片放在輸出分頁頂部，可收合」
  Scenario: Run 成功但有 warning 時畫黃色波浪線
    Given mock API 的 Run 回應：
      """
      {
        "status": "ok",
        "compile": { "success": true, "timed_out": false, "diagnostics": [
          { "code": "unused_variables", "message": "unused variable: `x`", "line": 2, "column": 9,
            "end_line": 2, "end_column": 10, "level": "warning", "explanation_url": null,
            "rendered": "warning: unused variable: `x`\n" }
        ] },
        "execution": { "exit_code": 0, "stdout": "MOVE DOWN\n", "stderr": "", "timed_out": false,
          "stdout_truncated": false, "stderr_truncated": false }
      }
      """
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 編輯器第 2 行第 9 欄到第 10 欄有 warning 等級的黃色波浪線 marker
    And 編輯器沒有 error 等級的 marker
    And 「輸出」分頁頂部有可收合的 warning 卡片，第一行含「⚠」與「第 2 行第 9 欄」
    And 「輸出」分頁顯示 stdout "MOVE DOWN"

  # ux.md §3.3 ok（Submit 通關）、§1.1 ⑥ ⑦
  Scenario: Submit 通過顯示全部打勾並進入通關動畫與結算
    Given mock API 的 Submit 回「全部通過」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then 主控台切到「測試結果」分頁
    And 測試結果依序為：
      | name                  | icon | text |
      | deposit_new_item      | ✓    | 通過 |
      | duplicate_is_skipped  | ✓    | 通過 |
      | gull_shares_the_chest | ✓    | 通過 |
    And 播放通關動畫
    And 通關動畫結束後顯示通關結算 Modal，按鈕有「下一關」「再玩一次」「回地圖」

  # ux.md §1.1 ⑥「點任意處或按 Esc 可跳過」、§7.2
  Scenario: 按 Esc 跳過通關動畫
    Given mock API 的 Submit 回「全部通過」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    And 通關動畫播放中玩家按 "Escape"
    Then 立即顯示通關結算 Modal

  # ux.md §1.1 ⑥「prefers-reduced-motion: reduce 時直接略過」
  Scenario: 減少動態效果時直接進入通關結算
    Given 瀏覽器設定 prefers-reduced-motion 為 "reduce"
    And mock API 的 Submit 回「全部通過」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then 不播放通關動畫
    And 顯示通關結算 Modal

  # ---------- compile_error：波浪線與錯誤卡片 ----------

  # ux.md §3.3 compile_error、§3.4、§7.3；architecture.md §7.4
  Scenario: 編譯錯誤畫紅色波浪線並顯示錯誤卡片
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    Then 主控台切到「編譯錯誤 (1)」分頁
    And 編輯器第 10 行第 17 欄到第 20 欄有 error 等級的紅色波浪線 marker
    And 第 1 張錯誤卡片第一行含「✗」「第 10 行第 17 欄」「E0382」
    And 第 1 張錯誤卡片的人話層文字為「bag 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（clone）？」
    And 該人話層中 "bag" 與 "clone" 以行內程式碼 <code> 呈現，畫面不顯示反引號字元
    And 第 1 張錯誤卡片的「rustc 原文」為收合狀態
    And 第 1 張錯誤卡片有「E0382 官方說明」連結，指向 "https://doc.rust-lang.org/error_codes/E0382.html" 並在新分頁開啟
    And 第 1 張錯誤卡片的錯誤碼「E0382」本身是指向同一網址的連結
    And 主控台的 aria-live 區朗讀「編譯失敗，1 個錯誤」

  # ux.md §1.3「主控台預設分頁只看結果狀態，不看 kind」、§3.3 compile_error（Submit 的 tests 全為未執行但不自動切過去）
  Scenario Outline: 編譯失敗一律切到編譯錯誤分頁
    Given mock API 的 <mode> 回 compile_error，diagnostics 只有一筆 E0384 error 位於第 3 行
    And 玩家開啟關卡 "<level>"
    When 玩家點「<button>」
    Then 主控台切到「編譯錯誤 (1)」分頁

    Examples:
      | mode   | level           | button |
      | Run    | 01-hello-ferris | 執行   |
      | Submit | 01-hello-ferris | 送出   |
      | Submit | 02-mut-matters  | 送出   |

  # ux.md §3.3 compile_error「Submit 的 tests 全部為 not_run，測試結果分頁照常列出」
  Scenario: Submit 編譯失敗時測試結果分頁列出未執行
    Given mock API 的 Submit 回「編譯錯誤（錯誤落在隱藏測試區段）」範例
    And 玩家開啟關卡 "05-compass-enum"
    When 玩家點「送出」
    And 玩家切到「測試結果」分頁
    Then 測試結果依序為：
      | name                    | icon | text   |
      | command_text            | —    | 未執行 |
      | delta_matches_direction | —    | 未執行 |
      | route_reaches_temple    | —    | 未執行 |

  # ux.md §3.4 第一行（line = null 且 rendered = null →「隱藏測試」）、§3.3 compile_error（line = null 不畫 marker、點了不動作）
  Scenario: 錯誤落在隱藏測試區段時卡片標示隱藏測試
    Given mock API 的 Submit 回「編譯錯誤（錯誤落在隱藏測試區段）」範例
    And 玩家開啟關卡 "05-compass-enum"
    When 玩家點「送出」
    Then 主控台切到「編譯錯誤 (4)」分頁
    And 4 張錯誤卡片的位置欄都顯示「隱藏測試」
    And 編輯器沒有任何 marker
    And 第 1 張錯誤卡片的人話層文字為「找不到 delta：可能拼錯字、還沒宣告，或被刪掉了。」
    And 該人話層中 "delta" 以行內程式碼 <code> 呈現，畫面不顯示反引號字元
    When 玩家點第 1 張錯誤卡片
    Then 編輯器游標位置與點擊前相同

  # ux.md §3.4 第一行（line = null 且 rendered 有值 →「無位置」）
  Scenario: 沒有位置但有原文的診斷標示無位置
    Given mock API 的 Run 回 compile_error，diagnostics 只有一筆：
      """
      { "code": null, "message": "linking with `cc` failed", "line": null, "column": null,
        "end_line": null, "end_column": null, "level": "error", "explanation_url": null,
        "rendered": "linking with `cc` failed" }
      """
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 第 1 張錯誤卡片的位置欄顯示「無位置」
    And 編輯器沒有任何 marker

  # ux.md §3.4「rustc 原文：預設收合，展開後用等寬字型顯示 rendered」
  Scenario: 展開錯誤卡片的 rustc 原文
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    And 玩家點第 1 張錯誤卡片的「顯示 rustc 原文」
    Then 卡片以等寬字型顯示該筆 Diagnostic 的 rendered 全文
    And 原文含 "value borrowed here after move"

  # ux.md §3.3 compile_error「滑鼠移到波浪線上：hover 顯示人話＋錯誤碼連結」、§3.4 errorHints
  Scenario: 滑鼠移到波浪線上顯示人話與錯誤碼連結
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    And 玩家把滑鼠移到第 10 行第 17 欄的波浪線上
    Then hover 內容的文字含「bag 已經交給別人了，這裡不能再用它。」
    And hover 內容中 "bag" 以行內程式碼 <code> 呈現，不顯示反引號字元
    And hover 內容含指向 "https://doc.rust-lang.org/error_codes/E0382.html" 的錯誤碼連結

  # ux.md §3.3 compile_error「點錯誤卡片：編輯器捲到該行、游標移到該欄、該行背景閃一下」
  Scenario: 點錯誤卡片跳到錯誤位置
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    And 玩家點第 1 張錯誤卡片
    Then 編輯器第 10 行在可見範圍內
    And 編輯器游標位於第 10 行第 17 欄
    And 編輯器第 10 行背景閃一下

  # ux.md §3.3 compile_error「end_column = null 時標到 end_line 行尾」；architecture.md §7.4
  Scenario: end_column 為 null 時波浪線標到行尾
    Given mock API 的 Run 回 compile_error，diagnostics 只有一筆 error：line 3、column 5、end_line 4、end_column null
    And 玩家開啟關卡 "01-hello-ferris"，編輯器第 4 行長度為 41 個字元（01 starter 第 4 行）
    When 玩家點「執行」
    Then 編輯器第 3 行第 5 欄到第 4 行第 42 欄有 error 等級的紅色波浪線 marker

  # ux.md §3.3 compile_error「零寬度範圍」；architecture.md §7.4（擴到 token 結尾、至少 1 字元、行尾改標前一字元）
  # 01 starter：第 2 行 "fn route() -> Vec<String> {"、第 3 行 "    let mut cmds = Vec::new();"（30 字元）
  Scenario Outline: 零寬度 span 的波浪線至少涵蓋一個字元
    Given mock API 的 Run 回 compile_error，diagnostics 只有一筆 error：line <line>、column <col>、end_line <line>、end_column <col>
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 編輯器第 <line> 行第 <start> 欄到第 <end> 欄有 error 等級的紅色波浪線 marker

    Examples:
      | case             | line | col | start | end |
      | 位置在 token 開頭 | 2    | 4   | 4     | 9   |
      | 位置在空白上      | 3    | 1   | 1     | 2   |
      | 位置在行尾        | 3    | 31  | 30    | 31  |

  # ux.md §3.4 卡片排序（error 在前、同級依行號、line = null 排同級最後）
  Scenario: 錯誤卡片依等級與行號排序
    Given mock API 的 Run 回 compile_error，diagnostics 依序為：
      | code  | level   | line | column | end_line | end_column | rendered |
      | E0425 | error   | null | null   | null     | null       | null     |
      | null  | warning | 2    | 9      | 2        | 10         | 原文     |
      | E0384 | error   | 7    | 5      | 7        | 10         | 原文     |
      | E0382 | error   | 3    | 1      | 3        | 4          | 原文     |
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 錯誤卡片依序為 "E0382, E0384, E0425, warning"
    And 編輯器共有 3 個 marker

  # ux.md §3.4「超過 5 筆時只展開前 5 筆，其餘收進『還有 n 筆』」
  Scenario: 超過 5 筆錯誤時其餘收合
    Given mock API 的 Run 回 compile_error，diagnostics 有 7 筆 error
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台切到「編譯錯誤 (7)」分頁
    And 展開顯示 5 張錯誤卡片
    And 顯示「還有 2 筆」

  # ux.md §3.4「note、help 等級的診斷併入卡片原文，不另開卡片」
  Scenario: note 與 help 等級不另開卡片
    Given mock API 的 Run 回 compile_error，diagnostics 依序為：
      | code  | level | line | column | end_line | end_column | rendered |
      | E0382 | error | 10   | 17     | 10       | 20         | 原文     |
      | null  | note  | 9    | 25     | 9        | 28         | 原文     |
      | null  | help  | 9    | 25     | 9        | 28         | 原文     |
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    Then 只顯示 1 張錯誤卡片

  # ux.md §3.4「字典內沒有的錯誤碼（含 code = null 與 lint 名）：人話層直接顯示 message 原文」「explanation_url 為空時不顯示這行」
  Scenario: 字典沒有的錯誤碼直接顯示 message 原文
    Given mock API 的 Run 回 compile_error，diagnostics 只有一筆：
      """
      { "code": null, "message": "expected `;`, found `println`", "line": 2, "column": 26,
        "end_line": 2, "end_column": 27, "level": "error", "explanation_url": null,
        "rendered": "error: expected `;`, found `println`\n" }
      """
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 第 1 張錯誤卡片的人話為 "expected `;`, found `println`"
    And 第 1 張錯誤卡片沒有「官方說明」連結

  # ux.md §3.4「message 沒有反引號時，{name} 代入『這個值』，以純文字呈現」
  Scenario: 抓不到名稱時人話層以純文字「這個值」代入
    Given mock API 的 Run 回 compile_error，diagnostics 只有一筆：
      """
      { "code": "E0382", "message": "use of moved value", "line": 2, "column": 5,
        "end_line": 2, "end_column": 9, "level": "error",
        "explanation_url": "https://doc.rust-lang.org/error_codes/E0382.html",
        "rendered": "error[E0382]: use of moved value\n" }
      """
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 第 1 張錯誤卡片的人話層文字為「這個值已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（clone）？」
    And 該人話層中 "這個值" 不在 <code> 內，"clone" 以行內程式碼 <code> 呈現

  # ux.md §3.4「rendered = null 時改顯示 message」
  Scenario: 沒有 rustc 原文時原文區顯示 message
    Given mock API 的 Submit 回「編譯錯誤（錯誤落在隱藏測試區段）」範例
    And 玩家開啟關卡 "05-compass-enum"
    When 玩家點「送出」
    And 玩家點第 1 張錯誤卡片的「顯示 rustc 原文」
    Then 第 1 張錯誤卡片的原文區顯示 "cannot find function `delta` in this scope"

  # ux.md §3.1 ④「修改後只清除編輯器 marker，主控台內容保留並淡化顯示『程式已修改』」；architecture.md §7.4
  Scenario: 修改程式碼後清除波浪線、主控台淡化
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    And 玩家在編輯器輸入一個空白
    Then 編輯器沒有任何 marker
    And 主控台仍顯示 1 張錯誤卡片，並淡化顯示「程式已修改」

  # ---------- runtime_panic ----------

  # ux.md §3.3 runtime_panic（panic marker 標整行）
  Scenario: Run panic 顯示紅色卡片並標出 panic 行
    Given mock API 的 Run 回「panic」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「執行」
    Then 主控台切到「輸出」分頁
    And 「輸出」分頁最後一張卡片標題為「程式在執行途中當掉了（panic）」
    And 該卡片顯示 stderr 原文，含 "RefCell already borrowed"
    And 編輯器第 15 行整行有 error 等級的紅色波浪線 marker

  # ---------- test_failed ----------

  # ux.md §3.3 test_failed、§7.1（✓／✗／— 圖示＋文字）
  Scenario: Submit 部分失敗時逐列顯示測試結果
    Given mock API 的 Submit 回「部分失敗，含 warning」範例
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「送出」
    Then 主控台切到「測試結果」分頁
    And 「測試結果」分頁標題顯示「通過 1 / 共 5」
    And 測試結果依序為：
      | name                            | icon | text |
      | one_crab_one_haul               | ✗    | 失敗 |
      | hundred_crabs_share_one_counter | ✗    | 失敗 |
      | three_crabs                     | ✗    | 失敗 |
      | hundred_crabs                   | ✗    | 失敗 |
      | no_crab                         | ✓    | 通過 |
    And 只有 "one_crab_one_haul" 那一列預設展開

  # ux.md §3.3 test_failed「預期 vs 實際」、§1.3 debug
  Scenario: 失敗訊息拆成實際與預期並排
    Given mock API 的 Submit 回「部分失敗，含 warning」範例
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「送出」
    And 玩家展開 "hundred_crabs" 那一列
    Then 該列標題顯示「有寶石在途中消失了」
    And 該列並排顯示「實際（left）」為 "0"、「預期（right）」為 "5050"
    And 差異處反白

  # ux.md §3.3 test_failed「拆不出來時直接顯示 message 原文」
  Scenario: 無法拆解的失敗訊息顯示原文
    Given mock API 的 Submit 回 test_failed，"tests" 為：
      | name      | status | message                  |
      | some_test | failed | RefCell already borrowed |
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then "some_test" 那一列顯示 "RefCell already borrowed"
    And 該列沒有「實際（left）」「預期（right）」欄

  # ux.md §3.3 test_failed「not_run：— 圖示＋『未執行』，灰色；不可展開」
  Scenario: 未執行的測試不可展開
    Given mock API 的 Submit 回「執行超時」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    And 玩家切到「測試結果」分頁
    Then "gull_shares_the_chest" 那一列顯示「—」圖示與文字「未執行」，為灰色
    And 點擊 "gull_shares_the_chest" 那一列不會展開

  # ux.md §3.3 test_failed「測試程式中途當掉」；architecture.md §3.6 程式 abort；api.md §0.3（exit 134）；decisions.md §15 ⑬ 第二季課程追加裁定、⑭
  # 回應取自 rustc 1.99.1 實際執行 49-echo-rune starter 的 Submit（--test --test-threads=1，exit 134）；thread id 依執行而異
  # 49 不在測試用 8 關 fixture，本畫面只依 Submit 回應，借 28-refcell-panic 開啟
  Scenario: 測試程式中途 stack overflow 時顯示整個當掉的卡片與 stderr
    Given mock API 的 Submit 回應：
      """
      { "status": "test_failed", "passed": false,
        "compile": { "success": true, "timed_out": false, "diagnostics": [] },
        "execution": { "exit_code": 134,
          "stdout": "\nrunning 4 tests\ntest __quest_hidden_tests::count_gems_in_spell ... ok\ntest __quest_hidden_tests::zz_route_reads_the_spell ... ",
          "stderr": "\nthread '__quest_hidden_tests::zz_route_reads_the_spell' (30) has overflowed its stack\nfatal runtime error: stack overflow, aborting\n",
          "timed_out": false, "stdout_truncated": false, "stderr_truncated": false },
        "tests": [
          { "name": "count_gems_in_spell", "status": "passed", "message": null },
          { "name": "zz_route_reads_the_spell", "status": "not_run", "message": null },
          { "name": "zz_rune_display_each", "status": "not_run", "message": null },
          { "name": "zz_rune_display_inside_sentence", "status": "not_run", "message": null } ] }
      """
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then 主控台切到「測試結果」分頁
    And 「測試結果」分頁標題顯示「通過 1 / 共 4」
    And 「測試結果」分頁頂部顯示紅色卡片「測試程式在執行途中整個當掉了，後面的測試沒有機會跑」
    And 該卡片以等寬字型顯示 stderr 原文，含 "has overflowed its stack" 與 "fatal runtime error: stack overflow, aborting"
    And 測試結果依序為：
      | name                            | icon | text   |
      | count_gems_in_spell             | ✓    | 通過   |
      | zz_route_reads_the_spell        | —    | 未執行 |
      | zz_rune_display_each            | —    | 未執行 |
      | zz_rune_display_inside_sentence | —    | 未執行 |
    And 沒有任何測試列展開

  # ux.md §3.3 test_failed「測試程式中途當掉」的條件（tests 沒有任何 failed）
  Scenario: 有失敗測試時不顯示整個當掉的卡片
    Given mock API 的 Submit 回「部分失敗，含 warning」範例
    And 玩家開啟關卡 "31-shared-counter"
    When 玩家點「送出」
    Then 「測試結果」分頁沒有「測試程式在執行途中整個當掉了，後面的測試沒有機會跑」卡片

  # ux.md §3.3 runtime_panic「若 panic 訊息含 main.rs:行:欄，編輯器在該行畫紅色波浪線」；architecture.md §3.6 程式 abort（Run 為 runtime_panic、沒有 main.rs:行:欄，不畫 panic marker）
  # stderr 取自 rustc 1.99.1 實際執行 49-echo-rune starter 的 Run（exit 134）
  Scenario: Run 時 stack overflow 顯示 panic 卡片但不畫 panic marker
    Given mock API 的 Run 回應：
      """
      { "status": "runtime_panic",
        "compile": { "success": true, "timed_out": false, "diagnostics": [] },
        "execution": { "exit_code": 134, "stdout": "",
          "stderr": "\nthread 'main' (11179858) has overflowed its stack\nfatal runtime error: stack overflow, aborting\n",
          "timed_out": false, "stdout_truncated": false, "stderr_truncated": false } }
      """
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「執行」
    Then 主控台切到「輸出」分頁
    And 「輸出」分頁最後一張卡片標題為「程式在執行途中當掉了（panic）」
    And 該卡片顯示 stderr 原文，含 "has overflowed its stack"
    And 編輯器沒有任何 error 等級的 marker

  # ---------- timeout／memory_limit／output_limit ----------

  # ux.md §3.3 timeout（execution.timed_out）
  Scenario: 執行超時顯示橘色卡片與常見原因
    Given mock API 的 Run 回「執行超時」範例
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示卡片「程式跑太久被停下來了（超過 5 秒）」
    And 卡片含「可能有無窮迴圈或死鎖，檢查 `loop`、`while` 的結束條件與鎖的順序」
    And 卡片上方顯示 stdout 3 行 "MOVE DOWN"

  # ux.md §3.3 timeout（compile.timed_out）
  Scenario: 編譯超時顯示卡片
    Given mock API 的 Run 回應：
      """
      { "status": "timeout",
        "compile": { "success": false, "timed_out": true, "diagnostics": [] },
        "execution": null }
      """
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示卡片「編譯太久被停下來了（超過 15 秒）」

  # ux.md §3.3 timeout「Submit 時已有結果的測試照常顯示，其餘為未執行」「預設分頁為輸出」
  Scenario: Submit 超時時已有結果的測試照常顯示
    Given mock API 的 Submit 回「執行超時」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then 主控台切到「輸出」分頁
    And 主控台顯示卡片「程式跑太久被停下來了（超過 5 秒）」
    When 玩家切到「測試結果」分頁
    Then 測試結果依序為：
      | name                  | icon | text   |
      | deposit_new_item      | ✓    | 通過   |
      | duplicate_is_skipped  | ✓    | 通過   |
      | gull_shares_the_chest | —    | 未執行 |

  # ux.md §3.3 memory_limit（執行階段）
  Scenario: 執行時記憶體超過上限顯示卡片與常見原因
    Given mock API 的 Run 回 status "memory_limit"，stdout 為 "MOVE DOWN\n"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示卡片「程式用太多記憶體被停下來了（超過 256 MB）」
    And 卡片含「可能有無限遞迴，或一次配置了超大的 `Vec`」
    And 卡片上方顯示 stdout "MOVE DOWN"

  # ux.md §3.3 memory_limit「編譯階段 OOM（execution = null）時卡片改為…，不顯示常見原因」
  Scenario: 編譯時記憶體超過上限顯示編譯階段卡片
    Given mock API 的 Run 回應：
      """
      { "status": "memory_limit",
        "compile": { "success": false, "timed_out": false, "diagnostics": [] },
        "execution": null }
      """
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示卡片「編譯時用太多記憶體被停下來了（超過 256 MB）」
    And 卡片不含「可能有無限遞迴，或一次配置了超大的 `Vec`」

  # ux.md §3.3 output_limit
  Scenario: 輸出超過上限顯示卡片與截斷標記
    Given mock API 的 Run 回 status "output_limit"，stdout 為 "MOVE DOWN\nMOVE DOWN\nMOVE DO"，stdout_truncated 為 true
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示卡片「輸出太多被停下來了（超過 64 KiB）」
    And 卡片含「可能在無窮迴圈裡 `println!`」
    And 主控台 stdout 結尾標「（已截斷）」

  # ---------- 請求失敗：server_busy／judge_unavailable／payload_too_large／network_error ----------

  # ux.md §3.3 server_busy（執行與送出一起 disabled 並各自倒數、重設不受影響、不自動重送；Retry-After 只接受整數秒）
  Scenario Outline: 判題忙碌時兩顆按鈕一起倒數且不自動重送
    Given mock API 的 Run 回 503，error.code 為 "JUDGE_BUSY"，<header>
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示灰色卡片「伺服器現在有點忙，等一下再試」
    And 「執行」按鈕顯示「執行（<secs> 秒後可再試）」並為 disabled
    And 「送出」按鈕顯示「送出（<secs> 秒後可再試）」並為 disabled
    And 「重設」按鈕為可點擊
    When 經過 <secs> 秒
    Then 「執行」與「送出」按鈕恢復為可點擊
    And mock API 共收到 1 次 Run 請求

    Examples:
      | header                       | secs |
      | 標頭 "Retry-After" 為 "2"    | 2    |
      | 沒有 "Retry-After" 標頭      | 10   |
      | 標頭 "Retry-After" 為 "abc"  | 10   |
      | 標頭 "Retry-After" 為 "1.5"  | 10   |

  # ux.md §3.3 judge_unavailable（不倒數、不 disabled）
  Scenario: 判題服務未就緒時顯示說明但不停用按鈕
    Given mock API 的 Run 回 503，error.code 為 "JUDGE_UNAVAILABLE"，沒有 "Retry-After" 標頭
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示灰色卡片「判題服務還沒準備好，關卡內容仍可瀏覽」
    And 「執行」與「送出」按鈕為可點擊，且不顯示倒數

  # ux.md §3.3 judge_unavailable「進入關卡頁時 /api/health 回 degraded」；api.md §5
  Scenario: 健康檢查降級時直接停用 Run 與 Submit
    Given mock API 的 GET "/api/health" 回 status "degraded"
    When 玩家開啟關卡 "01-hello-ferris"
    Then 「執行」與「送出」按鈕為 disabled
    And 按鈕旁顯示「判題服務還沒準備好，關卡內容仍可瀏覽」
    And 左欄故事與提示區仍可瀏覽

  # ux.md §3.3 judge_unavailable「重新整理頁面才會重新檢查」
  Scenario: 降級狀態在重新整理後重新檢查
    Given mock API 的 GET "/api/health" 回 status "degraded"
    And 玩家開啟關卡 "01-hello-ferris"
    When mock API 的 GET "/api/health" 改為回 status "ok"
    And 玩家重新整理頁面
    Then 「執行」與「送出」按鈕為可點擊

  # ux.md §3.1、§3.3 payload_too_large（只看 HTTP 413，不讀 body）；api.md §0.2
  Scenario Outline: 程式碼過長時顯示專屬卡片
    Given mock API 的 Run <response>
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示灰色卡片「程式碼太長了（上限 64 KiB），刪掉一些再試」
    And 「執行」按鈕為可點擊

    Examples:
      | response                                 |
      | 回 413，error.code 為 "PAYLOAD_TOO_LARGE" |
      | 回 413，body 為純文字 "Payload Too Large" |

  # ux.md §3.1、§3.3 network_error（斷線、逾時，以及 400、404、500 等其他 4xx／5xx）；§3.2 前端請求逾時 60 秒
  Scenario Outline: 連線失敗與其他錯誤碼顯示同一張灰色卡片且不倒數
    Given mock API 的 Run <failure>
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「執行」
    Then 主控台顯示灰色卡片「連不上伺服器，檢查網路後再試」
    And 「執行」按鈕為可點擊

    Examples:
      | failure                                |
      | 連線中斷                               |
      | 超過 60 秒沒有回應                     |
      | 回 400，error.code 為 "INVALID_REQUEST" |
      | 回 404，error.code 為 "LEVEL_NOT_FOUND" |
      | 回 500，error.code 為 "INTERNAL_ERROR"  |

  # ---------- 草稿自動儲存與重設 ----------

  # ux.md §6.6 ①②③；architecture.md §7.2 useDraftStore
  Scenario: 停止輸入 500 ms 後寫入草稿
    Given 玩家開啟關卡 "01-hello-ferris"
    When 玩家在編輯器輸入 "// 草稿"
    Then 輸入後 300 ms 時 localStorage 沒有 key "rustquest.draft.01-hello-ferris"
    And 停止輸入 500 ms 後 localStorage 的 "rustquest.draft.01-hello-ferris" 等於編輯器目前內容

  # ux.md §6.6 ④
  Scenario: 重新整理後載入草稿
    Given localStorage 的 "rustquest.draft.01-hello-ferris" 為 "fn main() { println!(\"MOVE DOWN\"); }"
    When 玩家開啟關卡 "01-hello-ferris"
    Then 編輯器內容為 "fn main() { println!(\"MOVE DOWN\"); }"

  # ux.md §6.6 ④「沒有就用 API 的 starter」
  Scenario: 沒有草稿時載入 starter
    Given localStorage 沒有 key "rustquest.draft.01-hello-ferris"
    When 玩家開啟關卡 "01-hello-ferris"
    Then 編輯器內容等於 mock API 關卡 "01-hello-ferris" 的 starter

  # ux.md §6.6「重設：先彈出確認，確定後程式碼改回 starter 並刪除該關草稿」
  Scenario: 確認後重設改回 starter 並刪除草稿
    Given localStorage 的 "rustquest.draft.01-hello-ferris" 為 "fn main() {}"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「重設」
    Then 出現確認「重設會把程式碼改回起始版本，目前的修改會消失，確定嗎？」
    When 玩家確認
    Then 編輯器內容等於 mock API 關卡 "01-hello-ferris" 的 starter
    And localStorage 沒有 key "rustquest.draft.01-hello-ferris"

  # ux.md §6.6 重設確認（取消時不改動）
  Scenario: 取消重設時程式碼與草稿不變
    Given localStorage 的 "rustquest.draft.01-hello-ferris" 為 "fn main() {}"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「重設」
    And 玩家取消
    Then 編輯器內容為 "fn main() {}"
    And localStorage 的 "rustquest.draft.01-hello-ferris" 為 "fn main() {}"

  # ux.md §6.2、§6.6 ConfirmDialog（頁面內 modal、焦點移入、Esc 等同取消）；§7.2 Esc
  Scenario: 重設確認是頁面內對話框，按 Esc 取消
    Given localStorage 的 "rustquest.draft.01-hello-ferris" 為 "fn main() {}"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「重設」
    Then 出現頁面內的確認對話框，沒有觸發瀏覽器原生 confirm
    And 焦點在確認對話框內
    When 玩家按 "Escape"
    Then 確認對話框關閉，焦點回到「重設」按鈕
    And 編輯器內容為 "fn main() {}"

  # ux.md §6.6「程式碼已與 starter 相同時不彈確認」
  Scenario: 程式碼等於 starter 時重設不彈確認
    Given localStorage 沒有 key "rustquest.draft.01-hello-ferris"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點「重設」
    Then 沒有出現確認對話框
    And 編輯器內容等於 mock API 關卡 "01-hello-ferris" 的 starter

  # ux.md §6.6「通關後：草稿保留」
  Scenario: 通關後草稿保留
    Given localStorage 的 "rustquest.draft.28-refcell-panic" 為 "// 我的解答"
    And mock API 的 Submit 回「全部通過」範例
    And 玩家開啟關卡 "28-refcell-panic"
    When 玩家點「送出」
    Then localStorage 的 "rustquest.draft.28-refcell-panic" 為 "// 我的解答"

  # ux.md §6.6「寫入失敗：不提示，編輯照常」
  Scenario: 草稿寫入失敗時不提示
    Given localStorage 寫入會失敗
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家在編輯器輸入 "// 草稿" 並停止輸入 500 ms
    Then 畫面沒有出現關於草稿的 toast
    And 編輯器內容含 "// 草稿"

  # ---------- 快捷鍵說明、深淺色、平板與手機版面 ----------

  # ux.md §2.1 頂部列、§7.2「快捷鍵說明面板：由頂部列按鈕開啟，列出全部按鍵與 Ctrl+M 說明」
  Scenario: 開啟快捷鍵說明面板
    Given 瀏覽器平台為 "Windows"
    And 玩家開啟關卡 "01-hello-ferris"
    When 玩家點頂部列「⌨ 快捷鍵」
    Then 快捷鍵說明面板列出「Ctrl+Enter」「Ctrl+Shift+Enter」「Space」「→」「Home」「Esc」
    And 快捷鍵說明面板含「Ctrl+M」切換 Tab 移動焦點的說明
    When 玩家按 "Escape"
    Then 快捷鍵說明面板關閉

  # ux.md §8.3（system → light → dark 循環、存 rustquest.prefs.v1、html data-theme、Monaco 主題同步）；§6.5 Prefs.theme
  Scenario: 深淺色切換依序循環並寫入偏好
    Given localStorage 沒有 key "rustquest.prefs.v1"
    And 瀏覽器設定 prefers-color-scheme 為 "dark"
    When 玩家開啟關卡 "01-hello-ferris"
    Then HTML 根元素的 data-theme 為 "dark"
    And Monaco 主題為 "ocean-dark"
    When 玩家點頂部列深淺色切換按鈕
    Then HTML 根元素的 data-theme 為 "light"
    And Monaco 主題為 "ocean-light"
    And localStorage 的 "rustquest.prefs.v1" 的 theme 為 "light"
    When 玩家點頂部列深淺色切換按鈕
    Then HTML 根元素的 data-theme 為 "dark"
    And localStorage 的 "rustquest.prefs.v1" 的 theme 為 "dark"
    When 玩家點頂部列深淺色切換按鈕
    Then localStorage 的 "rustquest.prefs.v1" 的 theme 為 "system"
    And HTML 根元素的 data-theme 為 "dark"

  # ux.md §8.3「選擇存 rustquest.prefs.v1」、§6.5「Prefs 缺欄位或值不合法時用預設值 system」
  Scenario Outline: 重新整理後套用已存的深淺色偏好
    Given 瀏覽器設定 prefers-color-scheme 為 "light"
    And localStorage 的 "rustquest.prefs.v1" 為 `<prefs>`
    When 玩家開啟關卡 "01-hello-ferris"
    Then HTML 根元素的 data-theme 為 "<theme>"

    Examples:
      | prefs                | theme |
      | {"theme": "dark"}    | dark  |
      | {"theme": "light"}   | light |
      | {"theme": "purple"}  | light |
      | {not json            | light |

  # ux.md §2.1 桌機三欄
  Scenario: 桌機寬度為三欄版面
    Given 瀏覽器視窗寬度為 1280px
    When 玩家開啟關卡 "01-hello-ferris"
    Then 關卡頁同時顯示左欄（故事）、中欄（編輯器）、右欄（可視化與主控台）

  # ux.md §2.2 平板兩欄、抽屜第一次進關自動展開、關閉後同關不再自動展開（只記在記憶體）
  Scenario: 平板寬度左欄收成抽屜，同一關只自動展開一次
    Given 瀏覽器視窗寬度為 1000px
    When 玩家開啟關卡 "01-hello-ferris"
    Then 編輯器與可視化並排顯示
    And 「☰ 故事」抽屜為展開，內容含關卡 "01-hello-ferris" 的 story
    When 玩家按 "Escape"
    Then 「☰ 故事」抽屜為關閉
    When 玩家回章節地圖後再點節點進入關卡 "01-hello-ferris"
    Then 「☰ 故事」抽屜為關閉
    When 玩家重新整理頁面
    Then 「☰ 故事」抽屜為展開
    And localStorage 除了 "rustquest.progress.v1"、"rustquest.prefs.v1" 與 "rustquest.draft." 開頭的 key 以外沒有其他 key

  # ux.md §2.3 手機分頁（首次進關預設故事、之後程式；Run 後自動切結果）
  Scenario: 手機寬度改為三個分頁
    Given 瀏覽器視窗寬度為 390px
    And mock API 的 Run 回「成功」範例
    When 玩家開啟關卡 "01-hello-ferris"
    Then 畫面有「故事」「程式」「結果」三個分頁，目前為「故事」
    And 底部操作列有「重設」「執行」「送出」，每顆按鈕觸控區至少 44×44px
    When 玩家回章節地圖後再點節點進入關卡 "01-hello-ferris"
    Then 目前分頁為「程式」
    When 玩家點「執行」
    Then 目前分頁為「結果」
