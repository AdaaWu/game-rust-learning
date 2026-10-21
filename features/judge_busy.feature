# 執行：cucumber-rs（backend/tests/cucumber.rs）
# 規格：api.md §0.1、§0.2、§3 錯誤、§5；architecture.md §1.2 ③、§3.2、§3.4、§3.5、§3.9、§3.10、§6.2 ②③；decisions.md §14 ④、§15
# @docker：需要 Docker daemon 與 rust-quest-runner:dev 映像；其餘情境不需要
# 「Docker daemon 無法連線」以 JudgeConfig.docker_bin 設為不存在的路徑模擬（architecture.md §3.9），不需 Docker；「映像不存在」需要真的 daemon，標 @docker

Feature: 判題服務滿載與不可用
  判題資源不足時，run／submit 回 503 JUDGE_BUSY 並附 Retry-After
  Docker 不可用時回 503 JUDGE_UNAVAILABLE；關卡瀏覽不受影響，健康檢查回報降級

  Background:
    Given 後端以 content/ 全部 67 關的關卡內容啟動

  # ---------- 滿載 503 JUDGE_BUSY ----------

  # architecture.md §3.4 ①、§6.2 ②（max_concurrent = 0、max_queue = 0 不需 docker 即可測）；api.md §0.1、§0.2、§3 錯誤
  Scenario Outline: 排隊名額為 0 時立即回 503 JUDGE_BUSY
    Given 判題設定 max_concurrent 為 0、max_queue 為 0
    When 對關卡 "01-hello-ferris" 送出 <mode>，code 為 "fn main() {}"
    Then 回應狀態碼為 503
    And 回應標頭 "Retry-After" 為 "2"
    And 回應 JSON 為：
      """
      {
        "error": {
          "code": "JUDGE_BUSY",
          "message": "目前判題人數過多，請稍後再試"
        }
      }
      """

    Examples:
      | mode   |
      | Run    |
      | Submit |

  # architecture.md §3.4 ③（先檢查名額再檢查 Docker）；api.md §0.2 檢查順序
  Scenario: 滿載時即使 Docker 無法連線也回 503 JUDGE_BUSY
    Given 判題設定 max_concurrent 為 0、max_queue 為 0
    And 判題設定的 docker_bin 為不存在的路徑 "/nonexistent/docker"
    When 對關卡 "01-hello-ferris" 送出 Run，code 為 "fn main() {}"
    Then 回應狀態碼為 503
    And 回應標頭 "Retry-After" 為 "2"
    And 回應 JSON 的 "error.code" 為 "JUDGE_BUSY"

  # architecture.md §3.4（Run 與 Submit 共用同一組 permit）、§3.4 ①
  @docker
  Scenario: 執行名額與排隊名額都被佔滿時，下一個請求回 503 JUDGE_BUSY
    Given 判題設定 max_concurrent 為 1、max_queue 為 0
    And 已有一個 Run 請求正在執行無窮迴圈程式
    When 對關卡 "28-refcell-panic" 送出 Submit，code 為該關 solution.rs
    Then 回應狀態碼為 503
    And 回應標頭 "Retry-After" 為 "2"
    And 回應 JSON 的 "error.code" 為 "JUDGE_BUSY"

  # architecture.md §3.4 ②（排隊等候超時同樣回 503 JUDGE_BUSY）、§3.9（測試可建構不同的 queue_wait_timeout）
  @docker
  Scenario: 排隊等候超時回 503 JUDGE_BUSY
    Given 判題設定 max_concurrent 為 1、max_queue 為 1、排隊等候 timeout 為 1 秒
    And 已有一個 Run 請求正在執行無窮迴圈程式
    When 對關卡 "01-hello-ferris" 送出 Run，code 為 "fn main() {}"
    Then 回應狀態碼為 503
    And 回應標頭 "Retry-After" 為 "2"
    And 回應 JSON 的 "error.code" 為 "JUDGE_BUSY"

  # architecture.md §3.4 ④（permit 為 RAII，判題結束後歸還）
  @docker
  Scenario: 前一個判題結束後名額歸還，可再次判題
    Given 判題設定 max_concurrent 為 1、max_queue 為 0
    And 對關卡 "01-hello-ferris" 送出 Run 並收到 "timeout" 結果
    When 對關卡 "01-hello-ferris" 送出 Run，code 為該關 solution.rs
    Then 回應狀態碼為 200
    And 回應 JSON 的 "status" 為 "ok"

  # ---------- Docker 不可用 503 JUDGE_UNAVAILABLE ----------

  # api.md §0.2（不附 Retry-After）、§3 錯誤範例；architecture.md §3.2（不自動 pull）、§6.2 ③
  @docker
  Scenario Outline: runner 映像不存在時 run／submit 回 503 JUDGE_UNAVAILABLE
    Given 判題設定的 runner 映像為不存在的 tag "rust-quest-runner:does-not-exist"
    When 對關卡 "01-hello-ferris" 送出 <mode>，code 為 "fn main() {}"
    Then 回應狀態碼為 503
    And 回應沒有 "Retry-After" 標頭
    And 回應 JSON 為：
      """
      {
        "error": {
          "code": "JUDGE_UNAVAILABLE",
          "message": "判題服務未就緒：找不到 runner 映像"
        }
      }
      """
    And Docker 本機映像清單中沒有 "rust-quest-runner:does-not-exist"

    Examples:
      | mode   |
      | Run    |
      | Submit |

  # api.md §0.2、§3 錯誤（四種原因共用同一個 code，回應不帶原因代碼）；architecture.md §1.2 ③、§3.9 docker_bin、§3.10
  Scenario Outline: Docker daemon 無法連線時 run／submit 回 503 JUDGE_UNAVAILABLE
    Given 判題設定的 docker_bin 為不存在的路徑 "/nonexistent/docker"
    When 對關卡 "01-hello-ferris" 送出 <mode>，code 為 "fn main() {}"
    Then 回應狀態碼為 503
    And 回應沒有 "Retry-After" 標頭
    And 回應 JSON 的 "error.code" 為 "JUDGE_UNAVAILABLE"
    And 回應 JSON 的 "error" 的 key 恰好為 "code, message"

    Examples:
      | mode   |
      | Run    |
      | Submit |

  # api.md §0.2「關卡仍可瀏覽」、§5「degraded：關卡可瀏覽」；architecture.md §1.2 ③（失敗不中止啟動）
  Scenario: Docker 不可用時關卡列表與詳細資料仍可瀏覽
    Given 判題設定的 docker_bin 為不存在的路徑 "/nonexistent/docker"
    When 送出 GET "/api/levels"
    Then 回應狀態碼為 200
    And 回應 JSON 的 "levels" 長度為 67
    When 送出 GET "/api/levels/01-hello-ferris"
    Then 回應狀態碼為 200

  # ---------- 健康檢查 ----------

  # api.md §5 Response 200（正常）
  @docker
  Scenario: 判題服務可用時健康檢查回 ok
    When 送出 GET "/api/health"
    Then 回應狀態碼為 200
    And 回應 JSON 為：
      """
      { "status": "ok", "judge": { "available": true, "reason": null } }
      """

  # api.md §5 Response 200（降級）；architecture.md §6.2 ③
  @docker
  Scenario: runner 映像不存在時健康檢查回 degraded 且仍為 200
    Given 判題設定的 runner 映像為不存在的 tag "rust-quest-runner:does-not-exist"
    When 送出 GET "/api/health"
    Then 回應狀態碼為 200
    And 回應 JSON 為：
      """
      { "status": "degraded", "judge": { "available": false, "reason": "runner_image_missing" } }
      """

  # api.md §5（judge.reason = docker_unreachable）；architecture.md §3.10（docker_bin 無法執行視為 docker_unreachable）、§6.2 ②
  Scenario: Docker daemon 無法連線時健康檢查回 degraded
    Given 判題設定的 docker_bin 為不存在的路徑 "/nonexistent/docker"
    When 送出 GET "/api/health"
    Then 回應狀態碼為 200
    And 回應 JSON 為：
      """
      { "status": "degraded", "judge": { "available": false, "reason": "docker_unreachable" } }
      """
