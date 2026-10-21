# 執行：cucumber-rs（backend/tests/cucumber.rs）
# 規格：api.md §0、§1、§2、§6；architecture.md §1.3、§2.2、§6.2 ②；curriculum.md §1.7
# 本檔情境不需要 Docker

Feature: 關卡列表與關卡詳細資料
  前端需要取得關卡資訊來畫章節地圖與關卡頁
  後端只回傳公開欄位，絕不洩漏解答與隱藏測試

  Background:
    Given 後端以 content/ 全部 33 關的關卡內容啟動

  # api.md §1、§0（格式）
  Scenario: 關卡列表依章節再依關卡順序排序
    When 送出 GET "/api/levels"
    Then 回應狀態碼為 200
    And 回應標頭 "Content-Type" 為 "application/json; charset=utf-8"
    And 回應 JSON 的 "levels" 依序為：
      | id                      | chapter | order | title            | kind        | visual |
      | 01-hello-ferris         | 1       | 1     | 第一步           | build       | grid   |
      | 02-mut-matters          | 1       | 2     | 黏住的腳         | fix-compile | grid   |
      | 03-loop-the-shore       | 1       | 3     | 海岸巡邏         | build       | grid   |
      | 04-type-tides           | 1       | 4     | 潮汐表           | fix-compile | grid   |
      | 05-compass-enum         | 2       | 5     | 神奇羅盤         | build       | grid   |
      | 06-ferris-struct        | 2       | 6     | Ferris 的狀態欄  | build       | grid   |
      | 07-exhaustive-match     | 2       | 7     | 羅盤多了斜角     | fix-compile | grid   |
      | 08-pattern-treasure     | 2       | 8     | 寶石鑑定         | build       | grid   |
      | 09-move-the-gem         | 3       | 9     | 交出去的背包     | fix-compile | memory |
      | 10-borrow-dont-take     | 3       | 10    | 借看不給拿       | build       | memory |
      | 11-one-writer-rule      | 3       | 11    | 一寫多讀         | fix-compile | memory |
      | 12-two-claws            | 3       | 12    | 兩隻鉗子         | fix-compile | memory |
      | 13-slice-the-reef       | 3       | 13    | 切開珊瑚礁       | build       | memory |
      | 14-missing-lifetime     | 4       | 14    | 古碑上的兩個名字 | fix-compile | memory |
      | 15-dangling-relic       | 4       | 15    | 帶不出的寶物     | fix-compile | memory |
      | 16-relic-struct         | 4       | 16    | 導遊卷軸         | build       | memory |
      | 17-boss-ancient-tablet  | 4       | 17    | BOSS：古碑解讀   | debug       | memory |
      | 18-maybe-gem            | 5       | 18    | 也許有寶石       | build       | grid   |
      | 19-question-mark        | 5       | 19    | 沒有寶石的島     | debug       | grid   |
      | 20-bottle-message       | 5       | 20    | 漂流瓶密語       | build       | none   |
      | 21-overflow-current     | 5       | 21    | 逆流             | debug       | grid   |
      | 22-trait-crafts         | 6       | 22    | 會走路的都是夥伴 | build       | grid   |
      | 23-generic-bag          | 6       | 23    | 萬用寶物袋       | fix-compile | none   |
      | 24-iterator-tide        | 6       | 24    | 一行退潮         | build       | grid   |
      | 25-lazy-current         | 6       | 25    | 不動的計數器     | debug       | none   |
      | 26-box-the-cave         | 7       | 26    | 無限深的洞窟     | fix-compile | memory |
      | 27-shared-shell         | 7       | 27    | 共用貝殼屋       | build       | memory |
      | 28-refcell-panic        | 7       | 28    | 寶物箱卡住了     | debug       | memory |
      | 29-leaky-cycle          | 7       | 29    | 永不消失的漩渦   | debug       | memory |
      | 30-send-the-crabs       | 8       | 30    | 派出分身         | fix-compile | none   |
      | 31-shared-counter       | 8       | 31    | 消失的寶石       | debug       | none   |
      | 32-message-relay        | 8       | 32    | 螃蟹傳令兵       | build       | grid   |
      | 33-boss-great-migration | 8       | 33    | BOSS：大遷徙     | debug       | grid   |

  # api.md §1 chapters；curriculum.md §1.7（只列有關卡已載入的章節，依 chapter 升冪；章節缺漏時的過濾由 routes.rs 單元測試覆蓋）
  Scenario: 關卡列表附上有關卡的章節名稱
    When 送出 GET "/api/levels"
    Then 回應 JSON 的 "chapters" 依序為：
      | chapter | title        |
      | 1       | 沙灘甦醒     |
      | 2       | 潮汐羅盤     |
      | 3       | 所有權之海   |
      | 4       | 生命週期遺跡 |
      | 5       | 漂流瓶       |
      | 6       | 工匠之島     |
      | 7       | 智慧指標洞窟 |
      | 8       | 群蟹大遷徙   |

  # api.md §1：列表不含程式碼、測試、解答
  Scenario: 關卡列表的每一筆只有摘要欄位
    When 送出 GET "/api/levels"
    Then 回應 JSON 的 key 恰好為 "chapters, levels"
    And 回應 JSON 的 "levels" 每一筆的 key 恰好為 "id, chapter, order, title, kind, visual"
    And 回應 JSON 的 "chapters" 每一筆的 key 恰好為 "chapter, title"

  # api.md §0「不回傳的欄位」、architecture.md §6.2 ②
  Scenario: 關卡列表的 JSON 任何層級都沒有 solution 與 tests
    When 送出 GET "/api/levels"
    Then 回應 JSON 任何層級都不含 key "solution"
    And 回應 JSON 任何層級都不含 key "tests"

  # api.md §2（visual = grid 範例）、§0.3 GridDef
  Scenario: 取得 grid 關卡的詳細資料
    When 送出 GET "/api/levels/01-hello-ferris"
    Then 回應狀態碼為 200
    And 回應 JSON 的 "visual" 為 "grid"
    And 回應 JSON 的 "grid" 為：
      """
      {
        "width": 5,
        "height": 3,
        "start": [0, 0],
        "goal": [4, 1],
        "walls": [[2, 0], [2, 1]],
        "gems": [],
        "doors": []
      }
      """
    And 回應 JSON 的 "memory_snapshots" 為 null
    And 回應 JSON 的 "hints" 是長度 3 的字串陣列
    And 回應 JSON 的 "starter" 等於關卡 "01-hello-ferris" 的 starter.rs 全文
    And 回應 JSON 含字串欄位 "story" 與 "concept"

  # api.md §2（visual = memory 範例）、§0.3 MemorySnapshot（省略的 value、error 回 null，不省略 key）
  Scenario: 取得 memory 關卡的詳細資料
    When 送出 GET "/api/levels/09-move-the-gem"
    Then 回應狀態碼為 200
    And 回應 JSON 的 "visual" 為 "memory"
    And 回應 JSON 的 "grid" 為 null
    And 回應 JSON 的 "memory_snapshots" 長度為 4
    And 回應 JSON 的 "memory_snapshots[0].line" 為 8
    And 回應 JSON 的 "memory_snapshots[0].note" 為 "bag 擁有 heap 上的三個字串"
    And 回應 JSON 的 "memory_snapshots[1].frames[0].vars[0]" 為：
      """
      { "id": "report.bag", "name": "bag", "type": "Vec<String>", "value": null, "state": "moved" }
      """
    And 回應 JSON 的 "memory_snapshots[1].pointers" 為：
      """
      [ { "from": "count_items.bag", "to": "h1", "kind": "own" } ]
      """
    And 回應 JSON 的 "memory_snapshots" 每一筆都沒有 key "step"
    And 回應 JSON 的 "memory_snapshots[0].error" 為 null
    And 回應 JSON 的 "memory_snapshots[3].error" 為：
      """
      { "code": "E0382", "target": "report.bag" }
      """

  # api.md §2：visual = none 時 grid、memory_snapshots 皆為 null
  Scenario: 取得無可視化關卡的詳細資料
    When 送出 GET "/api/levels/31-shared-counter"
    Then 回應狀態碼為 200
    And 回應 JSON 的 "visual" 為 "none"
    And 回應 JSON 的 "grid" 為 null
    And 回應 JSON 的 "memory_snapshots" 為 null

  # api.md §2「絕不包含 solution、tests」、architecture.md §1.3 ④
  Scenario Outline: 關卡詳細資料任何層級都不洩漏解答與隱藏測試
    When 送出 GET "/api/levels/<id>"
    Then 回應狀態碼為 200
    And 回應 JSON 任何層級都不含 key "solution"
    And 回應 JSON 任何層級都不含 key "tests"
    And 回應內容不含關卡 "<id>" 的 tests.rs 中任一個測試函式名稱

    Examples:
      | id                      |
      | 01-hello-ferris         |
      | 02-mut-matters          |
      | 03-loop-the-shore       |
      | 04-type-tides           |
      | 05-compass-enum         |
      | 06-ferris-struct        |
      | 07-exhaustive-match     |
      | 08-pattern-treasure     |
      | 09-move-the-gem         |
      | 10-borrow-dont-take     |
      | 11-one-writer-rule      |
      | 12-two-claws            |
      | 13-slice-the-reef       |
      | 14-missing-lifetime     |
      | 15-dangling-relic       |
      | 16-relic-struct         |
      | 17-boss-ancient-tablet  |
      | 18-maybe-gem            |
      | 19-question-mark        |
      | 20-bottle-message       |
      | 21-overflow-current     |
      | 22-trait-crafts         |
      | 23-generic-bag          |
      | 24-iterator-tide        |
      | 25-lazy-current         |
      | 26-box-the-cave         |
      | 27-shared-shell         |
      | 28-refcell-panic        |
      | 29-leaky-cycle          |
      | 30-send-the-crabs       |
      | 31-shared-counter       |
      | 32-message-relay        |
      | 33-boss-great-migration |

  # api.md §2 錯誤、§0.1（不提供 details）
  Scenario: 關卡不存在回 404 LEVEL_NOT_FOUND
    When 送出 GET "/api/levels/99-unknown"
    Then 回應狀態碼為 404
    And 回應 JSON 為：
      """
      {
        "error": {
          "code": "LEVEL_NOT_FOUND",
          "message": "找不到關卡：99-unknown"
        }
      }
      """
