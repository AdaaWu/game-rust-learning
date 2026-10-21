# 執行：cucumber-rs（backend/tests/cucumber.rs）
# 規格：api.md §0.2、§0.3、§4；decisions.md §14 ①②；architecture.md §3.3、§3.6、§3.7、§3.8、§6.2 ②③；decisions.md §15
# @docker：需要 Docker daemon 與 rust-quest-runner:dev 映像；其餘情境不需要

Feature: 送出驗收（POST /api/levels/:id/submit）
  玩家按 Submit 時，後端把玩家程式碼與隱藏測試合併後編譯執行
  回傳每個隱藏測試的結果，且不洩漏隱藏測試的原始碼與位置

  Background:
    Given 後端以 content/ 全部 33 關的關卡內容啟動

  # ---------- 全通過 ----------

  # api.md §4 範例：全部通過
  @docker
  Scenario: 解答通過全部隱藏測試
    When 對關卡 "28-refcell-panic" 送出 Submit，code 為該關 solution.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "ok"
    And 回應 JSON 的 "passed" 為 true
    And 回應 JSON 的 "compile.success" 為 true
    And 回應 JSON 的 "execution.exit_code" 為 0
    And 回應 JSON 的 "tests" 依序為：
      | name                  | status | message |
      | deposit_new_item      | passed | null    |
      | duplicate_is_skipped  | passed | null    |
      | gull_shares_the_chest | passed | null    |
    And 回應 JSON 不含 key "summary"

  # architecture.md §3.7 合併檔 ④（grid 關卡併入共用模擬器與依 [grid] 產生的 level_grid()）；decisions.md §12 ①
  @docker
  Scenario Outline: grid 關卡的解答以產生的地圖通過隱藏測試
    When 對關卡 "<id>" 送出 Submit，code 為該關 solution.rs
    Then 回應 JSON 的 "status" 為 "ok"
    And 回應 JSON 的 "passed" 為 true
    And 回應 JSON 的 "tests" 每一筆的 "status" 都是 "passed"

    Examples:
      | id              |
      | 01-hello-ferris |
      | 02-mut-matters  |
      | 05-compass-enum |
      | 19-question-mark |

  # architecture.md §3.7「模組內的模擬器與測試輔助項目會遮蔽 use super::* 引入的同名項目」；curriculum.md §1.5
  @docker
  Scenario: 玩家自己定義 Grid 與 level_grid 不會與判題器衝突
    When 對關卡 "01-hello-ferris" 送出 Submit，code 為該關 solution.rs 後面接上：
      """
      struct Grid;
      fn level_grid() -> Grid {
          Grid
      }
      """
    Then 回應 JSON 的 "status" 為 "ok"

  # ---------- 部分失敗 ----------

  # api.md §4 範例：部分失敗，含 warning；§0.3 TestResult（message 去除 panicked at 與 backtrace 提示行）
  # tests 依 tests.rs 宣告順序（one_crab_one_haul、hundred_crabs_share_one_counter、three_crabs、hundred_crabs、no_crab），不是 libtest 輸出的字母順序
  @docker
  Scenario: 部分測試失敗回 test_failed 並附 panic 訊息
    When 對關卡 "31-shared-counter" 送出 Submit，code 為該關 starter.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "test_failed"
    And 回應 JSON 的 "passed" 為 false
    And 回應 JSON 的 "compile.success" 為 true
    And 回應 JSON 的 "execution.exit_code" 為 101
    And 回應 JSON 的 "tests" 依序為：
      | name                            | status | message                                                                                      |
      | one_crab_one_haul               | failed | assertion `left == right` failed: 螃蟹交了 5 顆寶石，計數器卻沒有變\n  left: 10\n right: 15       |
      | hundred_crabs_share_one_counter | failed | assertion `left == right` failed: 一百隻螃蟹同時交寶石，有寶石在途中消失了\n  left: 0\n right: 5050 |
      | three_crabs                     | failed | assertion `left == right` failed: 三隻螃蟹的寶石沒有全部算進去\n  left: 0\n right: 15           |
      | hundred_crabs                   | failed | assertion `left == right` failed: 有寶石在途中消失了\n  left: 0\n right: 5050                   |
      | no_crab                         | passed | null                                                                                         |
    And "tests" 每一筆的 "message" 都不含 "panicked at"
    And "tests" 每一筆的 "message" 都不含 "RUST_BACKTRACE"
    And "tests" 每一筆的 "name" 都不含 "__quest_hidden_tests::"

  # api.md §0.3「warning 一律保留回傳」、§4 範例（31 關以警告為線索）；decisions.md §14 ①（2 筆警告）
  @docker
  Scenario: Submit 保留使用者程式碼中的 warning 與位置
    When 對關卡 "31-shared-counter" 送出 Submit，code 為該關 starter.rs
    Then "compile.diagnostics" 依序為：
      | code               | level   | line | column | end_line | end_column | explanation_url |
      | unused_variables   | warning | 6    | 9      | 6        | 16         | null            |
      | unused_assignments | warning | 7    | 5      | 7        | 16         | null            |
    And "compile.diagnostics" 第 2 筆的 "rendered" 以 "warning: value assigned to `sum` is never read" 開頭

  # api.md §4「不會出現 runtime_panic：測試中的 panic 歸類為 test_failed」
  @docker
  Scenario: 測試中的 RefCell panic 歸為 test_failed
    When 對關卡 "28-refcell-panic" 送出 Submit，code 為該關 starter.rs
    Then 回應 JSON 的 "status" 為 "test_failed"
    And 回應 JSON 的 "tests" 依序為：
      | name                  | status |
      | deposit_new_item      | failed |
      | duplicate_is_skipped  | passed |
      | gull_shares_the_chest | failed |
    And "tests" 中 "deposit_new_item" 的 "message" 包含 "RefCell already borrowed"

  # ---------- not_run ----------

  # api.md §4 範例：執行超時；§4「執行中途被終止：已有結果的照實回傳，其餘（含終止當下正在跑的那個）為 not_run」
  @docker
  Scenario: 執行中途超時時，已跑完的照實回傳、其餘為 not_run
    When 對關卡 "28-refcell-panic" 送出 Submit，code 為該關 solution.rs 並在 deposit 開頭加入「箱子為空時進入無窮迴圈」：
      """
      if chest.borrow().items.is_empty() {
          loop {}
      }
      """
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "timeout"
    And 回應 JSON 的 "passed" 為 false
    And 回應 JSON 的 "execution.timed_out" 為 true
    And 回應 JSON 的 "execution.exit_code" 為 null
    And 回應 JSON 的 "tests" 依序為：
      | name                  | status  | message |
      | deposit_new_item      | passed  | null    |
      | duplicate_is_skipped  | passed  | null    |
      | gull_shares_the_chest | not_run | null    |

  # ---------- 編譯失敗 ----------

  # api.md §4「編譯失敗、編譯超時：全部為 not_run」；decisions.md §6
  @docker
  Scenario: 使用者程式碼編譯失敗時全部測試為 not_run 且行號對應編輯器
    When 對關卡 "09-move-the-gem" 送出 Submit，code 為該關 starter.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "compile_error"
    And 回應 JSON 的 "passed" 為 false
    And 回應 JSON 的 "execution" 為 null
    And "compile.diagnostics" 含一筆：
      | code  | level | line | column | end_line | end_column |
      | E0382 | error | 10   | 17     | 10       | 20         |
    And 回應 JSON 的 "tests" 每一筆的 "status" 都是 "not_run"、"message" 都是 null
    And 回應 JSON 的 "tests" 長度等於關卡 "09-move-the-gem" 的 tests.rs 測試數

  # api.md §4 範例：編譯錯誤（錯誤落在隱藏測試區段，05 關刪除 delta）；§0.3 Diagnostic；architecture.md §3.7 行號對應
  @docker
  Scenario: 錯誤落在隱藏測試區段時不洩漏位置與原文
    When 對關卡 "05-compass-enum" 送出 Submit，code 為該關 solution.rs 刪除整個 "delta" 函式（含上方註解）
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "compile_error"
    And 回應 JSON 的 "passed" 為 false
    And 回應 JSON 的 "execution" 為 null
    And "compile.diagnostics" 共 4 筆，每一筆都是：
      | code  | level | message                                     | line | column | end_line | end_column | rendered | explanation_url                                    |
      | E0425 | error | cannot find function `delta` in this scope | null | null   | null     | null       | null     | https://doc.rust-lang.org/error_codes/E0425.html |
    And 回應 JSON 的 "tests" 依序為：
      | name                    | status  | message |
      | command_text            | not_run | null    |
      | delta_matches_direction | not_run | null    |
      | route_reaches_temple    | not_run | null    |

  # api.md §4「測試原始碼不回傳」；architecture.md §3.7「不洩漏隱藏測試位置」
  @docker
  Scenario: 隱藏區段的診斷不帶出隱藏測試的行號或原始碼
    When 對關卡 "05-compass-enum" 送出 Submit，code 為該關 solution.rs 刪除整個 "delta" 函式（含上方註解）
    Then 回應內容不含 "__quest_hidden_tests"
    And 回應內容不含 "assert_eq!(delta("
    And 回應內容不含 "fn level_grid"
    And 回應內容不含 "/tmp/main.rs"

  # architecture.md §3.7：span 從使用者程式碼跨進隱藏區段時 end_line 截在使用者最後一行、end_column 為 null
  # 需穩定觸發此情形的輸入目前規格未給，改由 diagnostics.rs 單元測試以 fixture 覆蓋（architecture.md §6.2 ①）

  # api.md §0.3 CompileResult（編譯階段 OOM：Submit 的 tests 全部 not_run）；architecture.md §3.5
  @docker
  Scenario: 編譯階段 OOM 時全部測試為 not_run
    When 對關卡 "28-refcell-panic" 送出 Submit，code 為該關 solution.rs 後面接上：
      """
      static BIG: [u8; 1 << 30] = [1; 1 << 30];
      """
    Then 回應 JSON 的 "status" 為 "memory_limit"
    And 回應 JSON 的 "passed" 為 false
    And 回應 JSON 的 "compile.success" 為 false
    And 回應 JSON 的 "execution" 為 null
    And 回應 JSON 的 "tests" 每一筆的 "status" 都是 "not_run"、"message" 都是 null

  # ---------- 玩家自寫測試 ----------

  # architecture.md §3.3（--exact 只跑隱藏測試）、§3.8；api.md §4「玩家自己寫的 #[test] 不執行、不列入」
  @docker
  Scenario: 玩家自己寫的 mod tests 不衝突也不被執行
    When 對關卡 "28-refcell-panic" 送出 Submit，code 為該關 solution.rs 後面接上：
      """
      #[cfg(test)]
      mod tests {
          #[test]
          fn my_own_test() {
              panic!("這個測試不該被執行");
          }
      }
      """
    Then 回應 JSON 的 "status" 為 "ok"
    And 回應 JSON 的 "tests" 的 name 依序為 "deposit_new_item, duplicate_is_skipped, gull_shares_the_chest"
    And 回應 JSON 的 "execution.stdout" 不含 "my_own_test"

  # ---------- 請求驗證（不需 Docker）----------

  # api.md §4「錯誤：同 /run」；§0.2（檢查順序：body 驗證先於關卡查找，關卡不存在且 body 無效回 400）
  Scenario Outline: Submit 請求不合法時回對應錯誤碼
    When 對關卡 "<level>" 的 submit 端點送出 Content-Type "<content_type>"、body 為 <body>
    Then 回應狀態碼為 <status>
    And 回應 JSON 的 "error.code" 為 "<code>"

    Examples:
      | level           | content_type     | body                       | status | code            |
      | 01-hello-ferris | application/json | `{"code": `                | 400    | INVALID_REQUEST |
      | 01-hello-ferris | application/json | `{}`                       | 400    | INVALID_REQUEST |
      | 01-hello-ferris | application/json | `{"code": ""}`             | 400    | INVALID_REQUEST |
      | 01-hello-ferris | text/plain       | `{"code": "fn main() {}"}` | 400    | INVALID_REQUEST |
      | 99-unknown      | application/json | `{"code": "fn main() {}"}` | 404    | LEVEL_NOT_FOUND |
      | 99-unknown      | application/json | `{}`                       | 400    | INVALID_REQUEST |

  # api.md §0「code 64 KiB」
  Scenario: Submit 的 code 超過 64 KiB 回 413
    When 對關卡 "01-hello-ferris" 送出 Submit，code 為 65537 位元組的字串
    Then 回應狀態碼為 413
    And 回應 JSON 的 "error.code" 為 "PAYLOAD_TOO_LARGE"
