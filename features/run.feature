# 執行：cucumber-rs（backend/tests/cucumber.rs）
# 規格：api.md §0.2、§0.3、§3；architecture.md §1.4、§3.1、§3.5、§3.6、§3.7、§6.2 ②③
# @docker：需要 Docker daemon 與 rust-quest-runner:dev 映像；其餘情境不需要

Feature: 執行玩家程式（POST /api/levels/:id/run）
  玩家按 Run 時，後端只編譯並執行玩家的 main，回傳編譯診斷與輸出
  編譯失敗、panic、超時都是正常的判題結果，回 200

  Background:
    Given 後端以 content/ 全部 67 關的關卡內容啟動

  # ---------- 成功 ----------

  # api.md §3 範例：成功；§0.3 CompileResult、ExecutionResult
  @docker
  Scenario: 印出指令的程式執行成功
    When 對關卡 "01-hello-ferris" 送出 Run，code 為該關 solution.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "ok"
    And 回應 JSON 的 "compile.success" 為 true
    And 回應 JSON 的 "compile.timed_out" 為 false
    And 回應 JSON 的 "compile.diagnostics" 為空陣列
    And 回應 JSON 的 "execution.exit_code" 為 0
    And 回應 JSON 的 "execution.stdout" 為 "MOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE UP\n"
    And 回應 JSON 的 "execution.stderr" 為 ""
    And 回應 JSON 的 "execution.timed_out" 為 false
    And 回應 JSON 的 "execution.stdout_truncated" 為 false
    And 回應 JSON 的 "execution.stderr_truncated" 為 false
    And 回應 JSON 的 "compile" 的 key 恰好為 "success, timed_out, diagnostics"
    And 回應 JSON 的 "execution" 的 key 恰好為 "exit_code, stdout, stderr, timed_out, stdout_truncated, stderr_truncated"
    And 回應 JSON 不含 key "passed"、"tests"

  # api.md §0.3 Diagnostic（warning 一律保留）、ux.md §3.3（只有 warning 視為編譯成功）
  @docker
  Scenario: 只有 warning 時照常執行並保留 warning
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          let x = 1;
          println!("MOVE DOWN");
      }
      """
    Then 回應 JSON 的 "status" 為 "ok"
    And 回應 JSON 的 "compile.success" 為 true
    And 回應 JSON 的 "execution.stdout" 為 "MOVE DOWN\n"
    And "compile.diagnostics" 含一筆：
      | code             | level   | message                | line | column | end_line | end_column | explanation_url |
      | unused_variables | warning | unused variable: `x`   | 2    | 9      | 2        | 10         | null            |
    And 該筆診斷的 "rendered" 以 "warning: unused variable: `x`" 開頭

  # ---------- 編譯錯誤 ----------

  # api.md §3 範例：編譯錯誤；§0.3 Diagnostic（end 位置、rendered）
  @docker
  Scenario: 編譯錯誤回傳診斷的起訖位置與 rustc 原文
    When 對關卡 "09-move-the-gem" 送出 Run，code 為該關 starter.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "compile_error"
    And 回應 JSON 的 "compile.success" 為 false
    And 回應 JSON 的 "compile.timed_out" 為 false
    And 回應 JSON 的 "execution" 為 null
    And "compile.diagnostics" 含一筆：
      | code  | level | message                          | line | column | end_line | end_column | explanation_url                                    |
      | E0382 | error | borrow of moved value: `bag`     | 10   | 17     | 10       | 20         | https://doc.rust-lang.org/error_codes/E0382.html |
    And 該筆診斷的 "rendered" 以 "error[E0382]: borrow of moved value: `bag`" 開頭
    And 該筆診斷的 "rendered" 包含 "/tmp/main.rs:10:17"

  # architecture.md §3.7 過濾規則
  @docker
  Scenario: 編譯錯誤不回傳摘要訊息與 failure-note
    When 對關卡 "09-move-the-gem" 送出 Run，code 為該關 starter.rs
    Then "compile.diagnostics" 沒有任何一筆的 "message" 以 "aborting due to" 開頭
    And "compile.diagnostics" 沒有任何一筆的 "message" 包含 "warning emitted" 或 "warnings emitted"
    And "compile.diagnostics" 沒有任何一筆的 "message" 以 "For more information about this error" 開頭
    And "compile.diagnostics" 每一筆的 "level" 都屬於 "error, warning, note, help"

  # api.md §0.3：explanation_url 僅 E\d{4} 有值；architecture.md §3.7：無 code 時為 null
  @docker
  Scenario: 語法錯誤沒有錯誤碼時 code 與 explanation_url 為 null
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          println!("MOVE DOWN")
          println!("MOVE UP");
      }
      """
    Then 回應 JSON 的 "status" 為 "compile_error"
    And "compile.diagnostics" 第一筆的 "code" 為 null
    And "compile.diagnostics" 第一筆的 "explanation_url" 為 null
    And "compile.diagnostics" 第一筆的 "level" 為 "error"
    And "compile.diagnostics" 第一筆的 "line" 為 2

  # api.md §3「沒有 main 的程式碼會得到 rustc E0601 的 compile_error」
  @docker
  Scenario: 沒有 main 時回 E0601 編譯錯誤
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn route() -> Vec<String> {
          Vec::new()
      }
      """
    Then 回應 JSON 的 "status" 為 "compile_error"
    And "compile.diagnostics" 含一筆 "code" 為 "E0601"、"level" 為 "error" 的診斷
    And 回應 JSON 的 "execution" 為 null

  # ---------- 執行期 panic ----------

  # api.md §3 範例：panic；§0.3 JudgeStatus runtime_panic
  @docker
  Scenario: 程式 panic 回 runtime_panic 與 exit 101
    When 對關卡 "28-refcell-panic" 送出 Run，code 為該關 starter.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "runtime_panic"
    And 回應 JSON 的 "compile.success" 為 true
    And 回應 JSON 的 "execution.exit_code" 為 101
    And 回應 JSON 的 "execution.timed_out" 為 false
    And 回應 JSON 的 "execution.stderr" 包含 "panicked at /tmp/main.rs:15:11:"
    And 回應 JSON 的 "execution.stderr" 包含 "RefCell already borrowed"

  # api.md §0.3：std::process::exit(n) 非 0 也歸 runtime_panic
  @docker
  Scenario: 以非 0 結束碼離開也歸為 runtime_panic
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          println!("MOVE DOWN");
          std::process::exit(3);
      }
      """
    Then 回應 JSON 的 "status" 為 "runtime_panic"
    And 回應 JSON 的 "execution.exit_code" 為 3
    And 回應 JSON 的 "execution.stdout" 為 "MOVE DOWN\n"

  # ---------- 超時 ----------

  # api.md §3 範例：執行超時；architecture.md §3.5（保留已讀到的 stdout）、§6.2 ③（容器確實被移除）
  @docker
  Scenario: 執行超過 5 秒回 timeout 並保留已輸出的 stdout
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          println!("MOVE DOWN");
          println!("MOVE DOWN");
          println!("MOVE DOWN");
          loop {}
      }
      """
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "timeout"
    And 回應 JSON 的 "compile.timed_out" 為 false
    And 回應 JSON 的 "execution.timed_out" 為 true
    And 回應 JSON 的 "execution.exit_code" 為 null
    And 回應 JSON 的 "execution.stdout" 為 "MOVE DOWN\nMOVE DOWN\nMOVE DOWN\n"
    And Docker 中沒有帶 label "rust-quest.judge=1" 的容器

  # api.md §3「編譯超時範例」、§0.3 CompileResult.timed_out
  # 編譯 15 秒上限不易用真實程式穩定觸發；step 以縮短 compile_timeout 的 JudgeConfig 建構（architecture.md §3.9）
  @docker
  Scenario: 編譯超時回 timeout 且 execution 為 null
    Given 判題設定的編譯 timeout 為 1 毫秒
    When 對關卡 "01-hello-ferris" 送出 Run，code 為該關 solution.rs
    Then 回應 JSON 的 "status" 為 "timeout"
    And 回應 JSON 的 "compile.timed_out" 為 true
    And 回應 JSON 的 "compile.success" 為 false
    And 回應 JSON 的 "execution" 為 null

  # ---------- 記憶體上限 ----------

  # api.md §0.3 memory_limit、ExecutionResult.exit_code（OOM 也為 null）；architecture.md §3.5、§3.6 ⑤
  @docker
  Scenario: 執行階段超過 256 MB 被終止回 memory_limit 並保留已輸出的 stdout
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          println!("MOVE DOWN");
          let big = vec![1u8; 512 * 1024 * 1024];
          println!("{}", big.len());
      }
      """
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "memory_limit"
    And 回應 JSON 的 "compile.success" 為 true
    And 回應 JSON 的 "execution.exit_code" 為 null
    And 回應 JSON 的 "execution.timed_out" 為 false
    And 回應 JSON 的 "execution.stdout" 為 "MOVE DOWN\n"

  # api.md §0.3 CompileResult（編譯階段 OOM）；architecture.md §3.5、§3.6 ②
  # 1 GiB 的 static 陣列會讓 rustc 在常數求值時耗盡 256 MB（已在 rust-quest-runner:dev 實測 rustc 回 137）
  @docker
  Scenario: 編譯階段超過 256 MB 被終止回 memory_limit 且 execution 為 null
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      static BIG: [u8; 1 << 30] = [1; 1 << 30];
      fn main() {
          println!("{}", BIG[0]);
      }
      """
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "memory_limit"
    And 回應 JSON 的 "compile.success" 為 false
    And 回應 JSON 的 "compile.timed_out" 為 false
    And 回應 JSON 的 "execution" 為 null

  # ---------- 輸出上限與截斷 ----------

  # api.md §0.3 ExecutionResult（64 KiB、*_truncated）；architecture.md §3.5（讀到上限就殺）
  @docker
  Scenario: stdout 超過 64 KiB 回 output_limit 並截斷
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          loop {
              println!("MOVE DOWN");
          }
      }
      """
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "output_limit"
    And 回應 JSON 的 "execution.stdout_truncated" 為 true
    And 回應 JSON 的 "execution.timed_out" 為 false
    And 回應 JSON 的 "execution.exit_code" 為 null
    And 回應 JSON 的 "execution.stdout" 的 UTF-8 位元組長度為 65536
    And 回應 JSON 的 "execution.stdout" 以 "MOVE DOWN\n" 開頭

  # api.md §0.3「切在 UTF-8 字元中間時往前退」；architecture.md §3.5
  # 「寶」為 3 位元組：65536 = 3 × 21845 + 1，往前退後保留 65535 位元組
  @docker
  Scenario: 截斷位置落在多位元組字元中間時往前退到合法邊界
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          loop {
              print!("寶");
          }
      }
      """
    Then 回應 JSON 的 "status" 為 "output_limit"
    And 回應 JSON 的 "execution.stdout_truncated" 為 true
    And 回應 JSON 的 "execution.stdout" 的 UTF-8 位元組長度為 65535
    And 回應 JSON 的 "execution.stdout" 由 21845 個 "寶" 組成
    And 回應 JSON 的 "execution.stdout" 不含 "�"

  # api.md §0.3：stdout、stderr 各自上限；architecture.md §3.5
  @docker
  Scenario: stderr 超過 64 KiB 也回 output_limit
    When 對關卡 "01-hello-ferris" 送出 Run，code 為：
      """
      fn main() {
          loop {
              eprintln!("oops");
          }
      }
      """
    Then 回應 JSON 的 "status" 為 "output_limit"
    And 回應 JSON 的 "execution.stderr_truncated" 為 true
    And 回應 JSON 的 "execution.stdout_truncated" 為 false
    And 回應 JSON 的 "execution.stderr" 的 UTF-8 位元組長度為 65536

  # ---------- 請求驗證（不需 Docker）----------

  # api.md §0.1、§0.2 INVALID_REQUEST（含 Content-Type 不是 application/json）；architecture.md §6.2 ②
  Scenario Outline: 請求不合法回 400 INVALID_REQUEST
    When 對關卡 "01-hello-ferris" 的 run 端點送出 <content_type>、body 為 <body>
    Then 回應狀態碼為 400
    And 回應 JSON 的 "error.code" 為 "INVALID_REQUEST"
    And 回應 JSON 的 "error" 的 key 恰好為 "code, message"

    Examples:
      | case                     | content_type                      | body                       |
      | 不是合法 JSON            | Content-Type "application/json"   | `{"code": `                |
      | 缺 code                  | Content-Type "application/json"   | `{}`                       |
      | code 非字串              | Content-Type "application/json"   | `{"code": 123}`            |
      | code 為空字串            | Content-Type "application/json"   | `{"code": ""}`             |
      | Content-Type 不是 JSON   | Content-Type "text/plain"         | `{"code": "fn main() {}"}` |
      | 缺 Content-Type          | 不帶 Content-Type                 | `{"code": "fn main() {}"}` |

  # api.md §0.2 LEVEL_NOT_FOUND
  Scenario: 對不存在的關卡 Run 回 404
    When 對關卡 "99-unknown" 送出 Run，code 為 "fn main() {}"
    Then 回應狀態碼為 404
    And 回應 JSON 為：
      """
      { "error": { "code": "LEVEL_NOT_FOUND", "message": "找不到關卡：99-unknown" } }
      """

  # api.md §0.2 檢查順序；architecture.md §1.4 ③（body 驗證先於關卡查找）
  Scenario: 關卡不存在且 body 無效時回 400
    When 對關卡 "99-unknown" 的 run 端點送出 Content-Type "application/json"、body 為 `{}`
    Then 回應狀態碼為 400
    And 回應 JSON 的 "error.code" 為 "INVALID_REQUEST"

  # api.md §0「code 64 KiB（UTF-8 位元組）、body 128 KiB」、§0.2 PAYLOAD_TOO_LARGE
  Scenario: code 超過 64 KiB 回 413
    When 對關卡 "01-hello-ferris" 送出 Run，code 為 65537 位元組的字串
    Then 回應狀態碼為 413
    And 回應 JSON 的 "error.code" 為 "PAYLOAD_TOO_LARGE"

  # api.md §0「body 128 KiB」
  Scenario: request body 超過 128 KiB 回 413
    When 對關卡 "01-hello-ferris" 送出 Run，body 為 code "fn main() {}" 且以 JSON 空白補到 131073 位元組
    Then 回應狀態碼為 413
    And 回應 JSON 的 "error.code" 為 "PAYLOAD_TOO_LARGE"

  # api.md §3「上限 64 KiB」邊界：剛好 64 KiB 不被擋（內容為無 main 的註解，預期 E0601）
  @docker
  Scenario: code 剛好 64 KiB 可正常判題
    When 對關卡 "01-hello-ferris" 送出 Run，code 為 "//" 後接 65534 個 "a"
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "compile_error"
    And "compile.diagnostics" 含一筆 "code" 為 "E0601"、"level" 為 "error" 的診斷
