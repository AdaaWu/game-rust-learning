# 執行：playwright-bdd（frontend e2e）
# 規格：ux.md §3.2、§3.3、§4.4、§5、§7.1、§7.3、§7.4；api.md §0.3 MemorySnapshot；architecture.md §7.5；decisions.md §9、§12 ②、§14 ①③、§15
# 預設以 page.route mock API；關卡詳細資料的 memory_snapshots 取自 curriculum.md v1.3 對應關卡的 level.toml：
#   09-move-the-gem：4 步，line 依序 8、3、9、10；第 4 步 error = { code "E0382", target "report.bag" }
#   11-one-writer-rule：3 步，line 依序 3、4、4；第 2 步 error = { code "E0502", target "first_then_add.first" }
#   28-refcell-panic：4 步，line 依序 21、11、15、13；第 3 步 error = { code null, target "deposit.view" }

Feature: MemoryView 記憶體可視化
  memory 關卡依關卡作者手寫的快照，逐步播放 stack／heap 與指標
  借用與錯誤以顏色、線型、圖示、文字多重標示，並與編輯器行號同步

  Background:
    Given mock API 回傳測試用 8 關 fixture 的關卡列表與詳細資料
    And mock API 的 GET "/api/health" 回 status "ok"
    And 玩家進度中測試用 8 關 fixture 都已拿到 ★1

  # ---------- 步驟播放 ----------

  # ux.md §5.5（進關立即顯示第 1 步並暫停、不自動播放）、§5.2 說明列；api.md §0.3（第 0 筆是第 1 步）
  Scenario: 進入 memory 關卡即顯示第 1 步快照並暫停
    When 玩家開啟關卡 "09-move-the-gem"
    Then 播放控制列顯示「第 1 / 4 步」
    And 播放按鈕顯示為「播放」
    And 說明列顯示「第 1 步 · 第 8 行」與 "bag 擁有 heap 上的三個字串"
    And Stack 區有 1 個 frame "report"，含變數 "bag : Vec<String>"，值 "len=3 cap=3"
    And Heap 區有 1 個區塊，型別 "[String; 3]"，值 "珍珠 | 貝殼 | 海星"
    And mock API 共收到 0 次 Run 請求

  # ux.md §4.4 單步、§5.2「目前執行中的 frame（最後一個）加粗外框」
  Scenario: 單步前進到下一個快照
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「單步」
    Then 播放控制列顯示「第 2 / 4 步」
    And 說明列顯示「第 2 步 · 第 3 行」與 "擁有權移動到 count_items 的參數，report 的 bag 失效"
    And Stack 區由上而下為 frame "report"、"count_items"
    And frame "count_items" 為加粗外框，frame "report" 不是

  # ux.md §5.5「Run／Submit 的任何結果都不清空、不重置、不半透明 MemoryView」、§3.3「可視化一律只指 GridWorld」
  Scenario: fix-compile 關卡 Run 編譯失敗後 MemoryView 不變
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    And 玩家點進度條跳到第 3 步
    When 玩家點「執行」
    Then 主控台切到「編譯錯誤 (1)」分頁
    And 播放控制列顯示「第 3 / 4 步」
    And 說明列顯示「第 3 步 · 第 9 行」
    And MemoryView 不是半透明，也沒有顯示「程式還沒編譯成功」
    When 玩家點「單步」
    Then 播放控制列顯示「第 4 / 4 步」

  # ux.md §3.2「MemoryView：不受影響，照常可播放」
  Scenario: 請求中 MemoryView 照常可播放
    Given mock API 的 Run 延遲 2 秒後回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「執行」
    And 請求中玩家點「單步」
    Then 播放控制列顯示「第 2 / 4 步」
    And MemoryView 不是半透明

  # ux.md §4.4「從頭：MemoryView 是第 1 步（memory_snapshots[0]，沒有第 0 步）」
  Scenario: 從頭回到第 1 步並暫停
    Given 玩家開啟關卡 "09-move-the-gem"
    And 玩家點進度條跳到第 3 步
    When 玩家點「從頭」
    Then 播放控制列顯示「第 1 / 4 步」
    And 說明列顯示「第 1 步 · 第 8 行」
    And 播放按鈕顯示為「播放」

  # ux.md §4.4 播放（1x 每步 500ms）、播到最後一步自動變回「播放」
  Scenario: 播放到最後一步後按鈕變回播放
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「播放」並等待播放結束
    Then 播放控制列顯示「第 4 / 4 步」
    And 播放按鈕顯示為「播放」

  # ux.md §4.4 進度條跳步
  Scenario: 點進度條跳到指定步
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 3 步
    Then 說明列顯示「第 3 步 · 第 9 行」與 "count_items 結束，參數被 drop，heap 資料釋放"

  # ux.md §5.1 note「支援行內 markdown」
  Scenario: 說明列渲染行內 markdown
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 4 步
    Then 說明列中 "E0382" 以粗體顯示

  # ---------- 變數與 heap 狀態 ----------

  # ux.md §5.3 moved 變數、§7.1
  Scenario: 已移動的變數以灰階、網底、刪除線與徽章標示
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 2 步
    Then 變數 "report.bag" 整格灰階並有斜線網底
    And 變數 "report.bag" 的名稱有刪除線
    And 變數 "report.bag" 右側有徽章「已移動」

  # ux.md §5.3 freed heap block「下一步起從畫面移除」
  Scenario: 已釋放的 heap 區塊標示後於下一步移除
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 3 步
    Then heap 區塊 "h1" 為虛線外框、透明度 40%，並有徽章「已釋放」
    When 玩家點「單步」
    Then Heap 區沒有區塊 "h1"

  # ---------- 借用標示（不只靠顏色）----------

  # ux.md §5.3 own 指標
  Scenario: own 指標為實線實心箭頭並標示 own
    Given 玩家開啟關卡 "09-move-the-gem"
    Then 從 "report.bag" 到 "h1" 有一條實線、實心三角箭頭的指標
    And 該指標的線上標籤為 "own"

  # ux.md §5.1 借用狀態由 pointers 推導、§5.3 mut 指標、§7.1
  Scenario: &mut 借用以粗虛線、標籤、圖示與文字標示
    Given 玩家開啟關卡 "11-one-writer-rule"
    Then 從 "first_then_add.bag" 到 "main.bag" 有一條線寬 3px、dash "6 4" 的虛線實心箭頭
    And 該指標的線上標籤為 "&mut"，旁邊有「✎」圖示
    And 變數 "main.bag" 為橘色粗外框，並有「🔒」圖示與文字「借出中」
    And 變數 "main.bag" 顯示「被獨佔借用中，暫時不能用」

  # ux.md §5.1、§5.3 shared 指標、§7.1
  Scenario: & 共享借用以細實線、空心箭頭、標籤、圖示與文字標示
    Given 玩家開啟關卡 "28-refcell-panic"
    When 玩家點進度條跳到第 2 步
    Then 從 "deposit.chest" 到 "main.chest" 有一條細實線、空心箭頭的指標
    And 該指標的線上標籤為 "&"，旁邊有「👁」圖示
    And 變數 "main.chest" 為藍色細外框，右上角有「&」徽章
    And 變數 "main.chest" 顯示「被共享借用中，只能讀」

  # ux.md §5.3「四重編碼，任一單獨存在都能辨識」、§7.1 驗證方式
  Scenario: 灰階模擬下仍能從文字分辨指標種類
    Given 瀏覽器以 Achromatopsia 模擬色彩
    And 玩家開啟關卡 "11-one-writer-rule"
    Then 每一條指標都有文字標籤，且標籤屬於 "own, &, &mut"

  # ---------- 錯誤標記（decisions.md §12 ②）----------

  # ux.md §5.3 錯誤標記；api.md §0.3 MemorySnapshot.error
  Scenario Outline: 錯誤步驟以紅色粗外框、✗ 與徽章標出出事的位置
    Given 玩家開啟關卡 "<level>"
    When 玩家點進度條跳到第 <step> 步
    Then "<target>" 的外框為 --danger 色、3px 粗實線
    And "<target>" 有「✗」圖示與徽章「<badge>」
    And 錯誤標記圖層位於所有指標箭頭之上

    Examples:
      | level              | step | target               | badge      |
      | 09-move-the-gem    | 4    | report.bag           | E0382      |
      | 11-one-writer-rule | 2    | first_then_add.first | E0502      |
      | 28-refcell-panic   | 3    | deposit.view         | 執行期錯誤 |

  # ux.md §5.3「target 是變數且這一步有從它出發的指標：該箭頭改用 --danger 顏色，線型與標籤不變，標籤後加 ✗」
  Scenario Outline: 從出事變數出發的箭頭轉紅，線型與標籤不變
    Given 玩家開啟關卡 "<level>"
    When 玩家點進度條跳到第 <step> 步
    Then 從 "<target>" 到 "<to>" 的指標為 --danger 色
    And 該指標仍為細實線、空心箭頭
    And 該指標的線上標籤為 "& ✗"
    And 其他指標都不是 --danger 色

    Examples:
      | level              | step | target               | to |
      | 11-one-writer-rule | 2    | first_then_add.first | h1 |
      | 28-refcell-panic   | 3    | deposit.view         | h1 |

  # ux.md §5.3「與 moved、freed 等狀態疊加顯示，不取代原本的網底與徽章」
  Scenario: 錯誤標記與已移動狀態疊加顯示
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 4 步
    Then 變數 "report.bag" 同時有徽章「已移動」與「E0382」
    And 變數 "report.bag" 仍有斜線網底與刪除線

  # ux.md §5.1「error 只有錯誤發生的那一步有值」
  Scenario: 非錯誤步驟沒有錯誤標記
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 2 步
    Then MemoryView 中沒有 --danger 色外框、「✗」圖示或錯誤碼徽章

  # ---------- 步驟變化提示 ----------

  # ux.md §5.3 步驟變化提示
  Scenario: 新增或狀態改變的項目外框閃一次
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「單步」
    Then frame "count_items" 與變數 "report.bag" 的外框閃一次（300ms）

  # ux.md §5.3「reduced-motion 時改為常駐小圓點『新』」、§7.4
  Scenario: 減少動態效果時以小圓點標示變化
    Given 瀏覽器設定 prefers-reduced-motion 為 "reduce"
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「單步」
    Then 變數 "report.bag" 旁有常駐小圓點「新」

  # ---------- 行號同步 ----------

  # ux.md §5.4 ①②③
  Scenario: 步數改變時編輯器高亮對應行
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點進度條跳到第 2 步
    Then 編輯器第 3 行有整行背景高亮與行號左側「▶」圖示
    When 玩家點「單步」
    Then 編輯器第 9 行有整行背景高亮與「▶」圖示
    And 編輯器第 3 行沒有高亮與「▶」圖示

  # ux.md §5.4 ④
  Scenario: 高亮行不在可見範圍時捲到畫面中央
    Given 玩家開啟關卡 "09-move-the-gem"
    And 編輯器高度只能顯示 3 行，且已捲到第 1 行
    When 玩家點進度條跳到第 4 步
    Then 編輯器第 10 行在可見範圍內

  # ux.md §5.4 反向同步
  Scenario Outline: 點行號左側的 ▶ 區域跳到對應步驟
    Given 玩家開啟關卡 "09-move-the-gem"
    And 玩家點進度條跳到第 1 步
    When 玩家點編輯器第 <line> 行行號左側的 ▶ 區域
    Then 播放控制列顯示「<progress>」

    Examples:
      | line | progress    |
      | 3    | 第 2 / 4 步 |
      | 10   | 第 4 / 4 步 |
      | 5    | 第 1 / 4 步 |

  # ux.md §5.4「程式碼已被修改」「程式碼與 starter 相同時（例如按『重設』後）提示消失」；§6.6 重設確認
  Scenario: 修改程式碼後提示行號僅供參考，重設後消失
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家在編輯器輸入一個空白
    Then 說明列上方顯示黃色提示「程式碼已修改，行號高亮僅供參考」
    When 玩家點「重設」並確認
    Then 不顯示「程式碼已修改，行號高亮僅供參考」

  # ux.md §5.4「程式碼與 starter 相同時提示消失」
  Scenario: 手動改回與 starter 相同時提示消失
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家在編輯器輸入一個空白
    And 玩家刪掉剛輸入的空白
    Then 不顯示「程式碼已修改，行號高亮僅供參考」

  # ux.md §5.4（提示只看程式碼是否等於 starter，Run 不會讓它消失）
  Scenario: 修改後 Run 不會讓提示消失
    Given mock API 的 Run 回「編譯錯誤」範例
    And 玩家開啟關卡 "09-move-the-gem"
    When 玩家在編輯器輸入一個空白
    And 玩家點「執行」
    Then 說明列上方顯示黃色提示「程式碼已修改，行號高亮僅供參考」

  # ux.md §5.4「掛載時即同步第 1 步的行號高亮」
  Scenario: 進關即高亮第 1 步的行
    When 玩家開啟關卡 "09-move-the-gem"
    Then 編輯器第 8 行有整行背景高亮與行號左側「▶」圖示

  # ux.md §5.4「播放到最後一步結束時保留最後一步的行高亮；離開關卡時移除所有行高亮裝飾」
  Scenario: 播放結束時保留最後一步的行高亮，離開關卡後移除
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「播放」並等待播放結束
    Then 編輯器第 10 行有整行背景高亮與「▶」圖示
    And 編輯器只有 1 行有行高亮
    When 玩家回章節地圖後再點節點進入關卡 "01-hello-ferris"
    Then 編輯器沒有任何行高亮與「▶」圖示

  # ---------- 版面與可及性 ----------

  # ux.md §5.2 上下排列門檻（主控裁定改為 360px，讓桌機 1280 寬維持左右並排）
  Scenario: 可視化區寬度不足 360px 時 Stack 在上、Heap 在下
    Given 可視化區寬度為 340px
    And 玩家開啟關卡 "09-move-the-gem"
    Then Stack 區位於 Heap 區上方

  # ux.md §5.2 上下排列門檻（主控裁定改為 360px）
  Scenario: 桌機 1280 寬時 Stack 與 Heap 左右並排
    Given 瀏覽器視窗寬度為 1280px
    And 玩家開啟關卡 "09-move-the-gem"
    Then Stack 區與 Heap 區左右並排

  # ux.md §7.3「MemoryView note 同時放進 aria-live 區」、「所有圖示按鈕都有 aria-label」
  Scenario: 說明列放進 aria-live 區且播放按鈕有 aria-label
    Given 玩家開啟關卡 "09-move-the-gem"
    When 玩家點「單步」
    Then aria-live 區含 "擁有權移動到 count_items 的參數，report 的 bag 失效"
    And 「單步」按鈕的 aria-label 為「單步前進」
