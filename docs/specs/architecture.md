# Rust Quest 系統架構規格（SDD）

> 狀態：v1.3（定稿；收尾第二輪驗收次要問題與工程師疑義，決策見 decisions.md §13–§15）
> 範圍：系統元件、後端模組、Judge（判題器）、安全、效能、測試策略、前端架構
> 相關文件：`docs/specs/api.md`（API 契約）、`docs/specs/ux.md`（前端行為）、`docs/specs/curriculum.md`（關卡內容）、`docs/specs/decisions.md`（決策與理由）、`features/*.feature`（BDD 情境）

---

## 0. 名詞定義

- **Judge（判題器）**：接收使用者程式碼，在沙箱內編譯、執行，回傳結構化結果的後端模組
- **Run 模式**：只編譯並執行使用者的 `main`，回傳 stdout／stderr，不跑隱藏測試
- **Submit 模式**：使用者程式碼 + 隱藏測試模組 `__quest_hidden_tests`（grid 關卡含共用模擬器，見 §3.7）合併後以 `rustc --test` 編譯，執行測試 binary，回傳每個測試的結果
- **隱藏區段**：合併檔中使用者程式碼之後的所有行；診斷落在這裡時不回傳位置
- **Runner 映像（`rust-quest-runner`）**：判題用的 Docker 映像，內含固定版本的 Rust 工具鏈
- **判題容器**：每次判題建立一個、用完即丟的 Docker 容器
- **Diagnostic（診斷）**：rustc 輸出的錯誤或警告，解析成 `{code, message, line, column, end_line, end_column, level, explanation_url, rendered}`
- **Semaphore（號誌）**：限制同時進行的工作數量的計數器；拿到 permit（許可）才能做事，做完歸還
- **熱路徑（hot path）**：被呼叫最頻繁、對延遲最敏感的程式路徑

---

## 1. 系統元件與資料流

### 1.1 元件總覽

- **frontend/（Vue 3 + Vite + TypeScript）**
  - 關卡列表、關卡頁（說明 / Monaco 編輯器 / 可視化與輸出）
  - 進度（已解鎖、星等）、草稿存 `localStorage`
- **backend/（Rust Axum，跑在 host）**
  - 無狀態 HTTP API；不需 DB、不需登入
  - 啟動時一次載入 `content/levels/` 到記憶體
  - 透過 docker CLI（`tokio::process::Command`）建立判題容器
- **runner/（Docker 映像定義）**
  - `runner/Dockerfile`：基於官方 `rust:1.99.1-slim`（內容見 §3.2）
  - 事先 `docker build` 好，判題時不拉映像
- **content/chapters.toml**：章節編號與名稱（§2.2），經 `GET /api/levels` 的 `chapters` 給前端
- **content/levels/**：關卡資料（`level.toml`、`starter.rs`、`tests.rs`、`solution.rs`）
- **content/shared/grid_sim.rs**：grid 關卡共用模擬器，Submit 時併入隱藏測試模組（內容見 curriculum.md §1.5）
- **Docker daemon（host 上）**：實際建立與隔離容器

### 1.2 啟動流程

```
① 讀設定
  ↓
② 載入關卡
  ↓
③ 檢查 Docker
  ↓
④ 清理殘留容器
  ↓
⑤ 開始監聽
```

① 讀取判題參數（timeout、上限、併發數）；MVP 用程式內常數（見 §3.9）
② 讀入 `content/chapters.toml` 與 `content/shared/grid_sim.rs`，掃描 `content/levels/*/`，解析 `level.toml`、讀入三個 `.rs` 檔、依 §2.2 驗證；任一項失敗則**啟動失敗**（fail fast），錯誤訊息含檔案路徑與原因
③ 執行 `docker version` 與 `docker image inspect rust-quest-runner:<tag>`；失敗**不中止啟動**，標記 Judge 為 `unavailable`，`/api/health` 回 `degraded`
④ `docker ps -aq --filter label=rust-quest.judge=1` 找出上次異常結束留下的判題容器，`docker rm -f` 清掉
⑤ 預設綁定 `127.0.0.1:3000`；前端開發時由 Vite proxy 轉送 `/api`

### 1.3 讀取關卡資料流

```
① 前端進入頁面
  ↓
② GET /api/levels 或 /api/levels/:id
  ↓
③ 讀記憶體 LevelStore
  ↓
④ 回傳 JSON
```

① 路由切換到關卡列表或關卡頁
② 由 Pinia store 發出；已快取則不再請求
③ `Arc<LevelStore>` 查 `HashMap`，無檔案 I/O
④ 對外 DTO 不含 `solution`、`tests`（型別層級就不存在這兩個欄位）

### 1.4 Run／Submit 資料流

```
① 使用者按 Run／Submit
  ↓
② POST /run 或 /submit
  ↓
③ 驗證請求
  ↓
④ 取得 permit
  ↓
⑤ 準備暫存目錄
  ↓
⑥ 啟動判題容器
  ↓
⑦ 容器內編譯
  ↓
⑧ 容器內執行
  ↓
⑨ 刪除容器與暫存目錄
  ↓
⑩ 解析並回傳
  ↓
⑪ 前端更新畫面
```

① 按鈕進入 loading，請求中禁止重複送出
② body：`{ "code": "..." }`
③ 先驗 body（大小 → JSON → `code` 非空字串且不超過 64 KiB），再查關卡；失敗回 413／400／404（見 api.md §0.2）
  - 關卡不存在且 body 也無效時回 400：body 驗證先於關卡查找，與 axum extractor 執行順序一致
④ 排隊滿 → 立即回 `503 JUDGE_BUSY` + `Retry-After`；排隊等候超時同樣回 503；拿到 permit 後才檢查 Docker 可用性（§3.4）
⑤ host 建立 `tempfile::TempDir`，寫入 `main.rs`（Submit 時為 §3.7 的合併檔）
⑥ `docker run -d`（參數見 §3.3），以 `-v <暫存目錄>:/work:ro` 掛入
⑦ `docker exec` 複製到 `/tmp` 並 `rustc` 編譯；host 端計時，超時 `docker kill`
⑧ 編譯成功才執行；Run 執行 `main`、Submit 執行測試 binary；host 端另計時、截斷輸出
⑨ 無論成功、失敗、超時、client 斷線都執行（RAII guard）
⑩ 診斷解析器、測試輸出解析器轉成 JSON
⑪ Monaco 標記診斷；grid 關卡把 stdout 指令餵給 GridWorld；Submit 通過時前端計算星等並寫進度

---

## 2. 後端模組切分

### 2.1 目錄結構（建議）

```
backend/
├── Cargo.toml
├── src/
│   ├── main.rs            # 組裝 AppState、啟動 server
│   ├── lib.rs             # 匯出 app()，供整合測試 oneshot
│   ├── routes/
│   │   ├── mod.rs         # Router 組裝、body 大小上限
│   │   ├── levels.rs      # GET /api/levels、/api/levels/:id
│   │   ├── judge.rs       # POST run / submit
│   │   └── health.rs      # GET /api/health
│   ├── error.rs           # ApiError → 統一錯誤格式
│   ├── content/
│   │   ├── mod.rs         # LevelStore
│   │   ├── loader.rs      # 掃描、解析、驗證
│   │   └── model.rs       # Level（內部）、LevelSummary / LevelDetail（對外 DTO）
│   └── judge/
│       ├── mod.rs         # Judge：permit、流程編排
│       ├── docker.rs      # docker 指令組裝、容器生命週期、kill
│       ├── exec.rs        # 執行 docker CLI、timeout、輸出上限
│       ├── diagnostics.rs # rustc JSON 診斷解析器
│       └── test_output.rs # libtest 文字輸出解析器
├── tests/                 # tower::ServiceExt::oneshot 整合測試
│   └── cucumber.rs        # cucumber-rs runner（harness = false）
runner/
└── Dockerfile
```

### 2.2 模組責任

- **routes**
  - 負責：HTTP 解析、請求驗證、呼叫 content／judge、轉成回應 DTO
  - 不負責：任何判題邏輯
  - `DefaultBodyLimit::max(128 KiB)` 在 Router 層擋超大 body
  - 自訂 `Json` extractor rejection，轉成統一錯誤格式（axum 預設回純文字）：所有 rejection（含缺 `Content-Type` 的 415）一律回 `400 INVALID_REQUEST`，body 超過上限回 `413 PAYLOAD_TOO_LARGE`
  - 不存在的路徑、錯誤方法沿用 axum 預設 404／405，不包統一格式（api.md §0.2）
- **error**
  - `ApiError` enum → `(StatusCode, Json<ErrorBody>)`
  - 錯誤碼常數集中於此，與 api.md 一一對應
- **content（content loader）**
  - 啟動時載入並驗證全部章節與關卡，之後唯讀；任一項違反即啟動失敗
  - TOML 一律拒絕未知欄位（serde `deny_unknown_fields`），拼錯的 key 不會被默默忽略
  - **`content/chapters.toml`**
    - 格式：`[[chapters]]` 陣列，每筆 `chapter`（正整數）、`title`（非空字串）
    - `chapter` 不可重複；檔案不存在時啟動失敗
    - 關卡的 `chapter` 必須出現在 `chapters.toml`；`chapters.toml` 可以有尚無關卡的章節，API 只回傳有關卡的章節（api.md §1）
  - **`level.toml` 必填欄位**：`id`、`chapter`、`order`、`title`、`kind`、`visual`、`story`、`concept`
    - `id` 必須等於所在目錄名，且符合 `^[0-9]{2}-[a-z0-9-]+$`；不可重複
    - `order` 正整數，全域不可重複（課程正式編號）；`(chapter, order)` 因此也唯一
    - `title`、`story`、`concept` 去除空白後不可為空
    - `kind`、`visual` 為合法 enum
  - **選填欄位與預設值**
    - `hints`：省略視為 `[]`；每則不可為空字串
    - `[grid]` 的 `walls`、`gems`、`doors`：省略視為 `[]`
    - `memory_snapshots[]` 的 `frames`、`heap`、`pointers`、`frames[].vars`：省略視為 `[]`
    - `vars[].value`、`error`、`error.code`：省略視為 `null`
  - **檔案**：`starter.rs`、`solution.rs`、`tests.rs` 必須存在且非空；`tests.rs` 至少一個 `#[test]`
  - **grid**：`visual = "grid"` 時 `[grid]` 必填，其他 visual 不可有；依 ux.md §4.2 驗證（`width`／`height` 2–12、座標在範圍內、各物件不重疊、`start ≠ goal`、陣列內不重複）
  - `visual = "grid"` 時依 `[grid]` 產生 `fn level_grid() -> Grid` 原始碼字串，存進內部 `Level`；Submit 只做字串串接（§3.7），熱路徑無額外成本
    - 產生格式見 §3.7 ④；確切字串以 backend 單元測試（B1a）釘死為準
  - **memory**：`visual = "memory"` 時 `memory_snapshots` 必填且至少一步，其他 visual 不可有；反序列化成 ux.md §5.1 的型別
    - 同一步內 frame `id`、變數 `id`、heap `id` 不重複
    - `pointers` 的 `from`（變數 `id`）、`to`（變數或 heap `id`）與 `error.target` 必須指向同一步存在的 `id`
    - `error.code` 有值時須符合 `^E\d{4}$`
    - `line` ≥ 1；是否超過檔案行數由內容測試檢查（§6.2 ④），loader 不比對
  - `content/shared/grid_sim.rs` 不存在時啟動失敗
  - 啟動時從 `tests.rs` 依宣告順序抽出測試函式名稱清單（`#[test]` 下一個 `fn` 的名稱），供 `tests` 排序、`--exact` 過濾與 `not_run` 標示（見 §3.6）
  - 內部 `Level` 含 `solution`、`tests`；對外 `LevelSummary`、`LevelDetail` 是**不同型別**，不含這兩欄，避免序列化時誤洩
- **judge**
  - `mod.rs`：持有兩個 Semaphore、組合「準備 → 容器 → 編譯 → 執行 → 清理 → 解析」流程
  - `docker.rs`：組出 `docker run／exec／kill／rm` 參數（純函式，可單元測試）、`ContainerGuard`（Drop 時刪容器）
  - `exec.rs`：以 `tokio::process` 執行指令、計時、讀 pipe 至上限即停
  - `diagnostics.rs`：rustc `--error-format=json` → `Vec<Diagnostic>`（純函式）
  - `test_output.rs`：libtest 文字輸出 → `Vec<TestResult>`（純函式）

### 2.3 AppState

```rust
struct AppState {
    levels: Arc<LevelStore>,
    judge: Arc<Judge>,   // 內含 Semaphore、JudgeConfig、Docker 狀態
}
```

- 只有一個 Judge 實作，**不開 trait**（無第二實作前不抽象）
- 測試以「真 docker（標 `#[ignore]`）」與「純函式單元測試」兩層處理（見 §6）

---

## 3. Judge 設計

### 3.1 判題容器生命週期

```
① 取得 permit
  ↓
② 建立暫存目錄
  ↓
③ docker run -d
  ↓
④ exec 編譯
  ↓
⑤ exec 執行
  ↓
⑥ docker rm -f
  ↓
⑦ 刪暫存目錄
  ↓
⑧ 歸還 permit
```

① 見 §3.4
② `tempfile::Builder::new().prefix("rq-judge-").tempdir()`；目錄權限 `0755`、`main.rs` 權限 `0644`，讓容器內 uid 1000 可讀
③ 容器 PID 1 是 `sleep <容器最長存活秒數>`，容器只是「殼」；名稱 `rq-judge-` 前綴加唯一後綴（例 uuid），帶 label `rust-quest.judge=1`
④ `docker exec <name> sh -c 'cp /work/main.rs /tmp/ && rustc ...'`，host 端套編譯 timeout
⑤ 編譯成功才進行；`docker exec <name> /tmp/main`（Run）或 `/tmp/main --test-threads=1 --exact …`（Submit，完整參數見 §3.3），host 端套執行 timeout 與輸出上限
⑥ 由 `ContainerGuard` 保證執行；Drop 時 `tokio::spawn` 一個 `docker rm -f <name>`
⑦ `TempDir` Drop 自動刪除
⑧ permit 是 RAII，離開 scope 自動歸還

**為什麼選「`run -d` 殼容器 + 兩次 `exec`」，而不是單一 `docker run` 跑腳本**

- 編譯與執行要**分開計時**；用 exec 分兩段，host 能精確知道是哪一段超時
- 編譯診斷（stderr JSON）與程式輸出（stdout）各自走獨立 pipe，不必在單一輸出流裡設計分隔符號（使用者 stdout 可能偽造分隔符號）
- 代價：多 2 次 docker CLI 呼叫，約 +50–150 ms
- 不採用的替代方案：單一 `docker run` + 映像內的小型 entry 程式負責分段計時並輸出一份 JSON（多一個要維護的程式，MVP 不需要）

**超時一定要殺容器，不是殺 CLI**

- 殺掉 `docker exec` 的 CLI 行程**不會**停掉容器內的程序
- 超時流程：`docker kill <name>`（SIGKILL 整個容器，含所有子孫行程）→ 再由 guard `docker rm -f`
- 容器 PID 1 的 `sleep N` 是最後保險：即使 host 端 backend 當掉，容器最多活 N 秒後自行結束（`--rm` 自動移除）

**Client 中途斷線**

- axum 會 drop handler future → `ContainerGuard`、`TempDir`、permit 依序 Drop → 容器被刪、目錄被刪、permit 歸還
- `tokio::process::Command` 一律 `kill_on_drop(true)`，避免 CLI 行程殘留

### 3.2 Runner 映像（`runner/Dockerfile`）

- 基底：`rust:1.99.1-slim`（**已定案**）；正式使用時建議再加上 digest 固定（`rust:1.99.1-slim@sha256:...`），避免同 tag 被覆寫
  - slim 版已含 `gcc`、`libc6-dev`（rustc 連結需要）
- 映像 tag：`rust-quest-runner:dev`（**已定案**），backend 設定中寫死同一個 tag
  - 注意：`dev` 是可變 tag，重建映像後舊容器不受影響、新判題立即用新版；之後若要多版本並存再改成帶版號的 tag
- 編譯一律 `rustc --edition 2024`（**已定案**），關卡 `starter.rs`／`solution.rs`／`tests.rs` 都以 2024 edition 撰寫
- Dockerfile 以 `runner/Dockerfile` 實物為準（已實測：冷啟動 0.6 秒；v1.2 於 `--read-only` 下編譯 MVP 關卡確認可用）
  ```dockerfile
  FROM rust:1.99.1-slim
  RUN useradd --uid 1000 --create-home runner
  USER 1000:1000
  WORKDIR /tmp
  ```
  - 建立 uid 1000 的 `runner` 使用者：`HOME=/home/runner`，與 `--user 1000:1000` 對應
  - `RUSTUP_HOME`、`CARGO_HOME`、`PATH` 沿用基底映像設定，不另設
  - 不設 `ENV HOME=/tmp`：判題只呼叫 `rustc`（不用 cargo），不寫入 `HOME`；rootfs 唯讀時實測可正常編譯
  - 所有寫入都在 `/tmp`（tmpfs），由 `docker exec` 指令明確指定路徑
- 不安裝網路工具、不加 crate（判題只用 `std`）
- 建置：`docker build -t rust-quest-runner:dev runner/`（寫進 README 與 Makefile／justfile）
- 判題時**不拉映像**：映像不存在 → Judge 標 `unavailable`，回 503，不自動 pull

### 3.3 `docker run` 參數

**已實測可用的基準參數**（使用者於 `rust-quest-runner:dev` 驗證）

```
--network none
--memory 256m --memory-swap 256m
--cpus 1
--pids-limit 64
--read-only
--tmpfs /tmp:rw,exec,size=64m
--cap-drop ALL
--security-opt no-new-privileges
--user 1000:1000
-v <host暫存目錄>:/work:ro
```

**完整指令（基準參數 + 殼容器所需 + 建議加固）**

```
docker run -d --rm
  --name rq-judge-<uuid>
  --label rust-quest.judge=1
  --network none
  --memory 256m --memory-swap 256m
  --cpus 1
  --pids-limit 64
  --read-only
  --tmpfs /tmp:rw,exec,nosuid,size=64m
  --cap-drop ALL
  --security-opt no-new-privileges
  --user 1000:1000
  --init
  --log-driver none
  --ulimit nofile=256:256
  --ulimit core=0
  -v <host暫存目錄>:/work:ro
  rust-quest-runner:dev
  sleep <容器最長存活秒數>
```

- `-d --rm`、`--name`、`--label`、`sleep N`：殼容器生命週期必要（§3.1），非安全參數
- `nosuid`、`--init`、`--log-driver none`、`--ulimit nofile`、`--ulimit core`：**建議加固，尚未實測**；實作時先以 Docker 整合測試驗證，任一項造成問題就拿掉並記錄原因

逐項說明：

- `--network none`：完全無網路；rustc 只用 std，不需下載
- `--memory 256m --memory-swap 256m`：兩者相等＝**禁用 swap**；小型單檔 rustc + 連結多在 100–200 MB，256m 夠用但貼近，若出現 OOM（exit 137）回報為 `memory_limit`，並在內容測試中監控
  - 注意：**tmpfs 使用量算進容器記憶體**，`/tmp` 寫滿 64m 會壓縮可用記憶體
- `--cpus 1`：每容器最多 1 核；總 CPU 用量由 Semaphore 數量控制
- `--pids-limit 64`：擋 fork bomb；rustc 本身會開數個 thread（thread 也算 pid），64 足夠
  - **不使用 `--ulimit nproc`**：nproc 在 Linux 以「host 上同 uid」計數，會跟其他容器、host 程序互相干擾
- `--read-only` + `--tmpfs /tmp:rw,exec,size=64m`：rootfs 唯讀；`/tmp` 需 `exec` 才能執行編出來的 binary
  - 建議加 `nosuid`：無成本加固
- `--cap-drop ALL`、`--security-opt no-new-privileges`：移除所有 Linux capability、禁止 setuid 提權
- `--user 1000:1000`：非 root；對應映像內的 `runner` 使用者（§3.2）
- `--init`（建議）：以 tini 當 PID 1 的 reaper，回收 fork 出來的殭屍行程；否則 `sleep` 當 PID 1 不會回收
- `--log-driver none`（建議）：殼容器本身沒輸出，避免 daemon 累積日誌檔
- `--ulimit nofile=256:256`、`core=0`（建議）：限制開檔數、不產生 core dump
- `-v <暫存目錄>:/work:ro`：唯讀掛入原始碼
  - **權限（已實測確認）**：`tempfile` 預設建立 `0700` 目錄，容器內 uid 1000 讀不到；建立後必須明確 `chmod 0755` 目錄、`0644` 檔案（`std::fs::set_permissions`）
  - 這是必做項，寫進 `judge` 單元測試：建立暫存目錄後斷言權限位元
  - macOS Docker Desktop：`$TMPDIR`（`/var/folders/...`）預設在共享清單內；若改用其他路徑需確認 File Sharing 設定
- **不掛載 `docker.sock`** 到任何容器（等同 host root 權限）
- 保留 Docker 預設 seccomp profile，**禁止** `--privileged`、`seccomp=unconfined`

**`docker exec` 參數**

- 編譯（Run）：`docker exec <name> sh -c 'cp /work/main.rs /tmp/main.rs && rustc --edition 2024 --error-format=json -A dead_code -F unsafe_code -o /tmp/main /tmp/main.rs'`
  - 不加 `-C opt-level`、`-C debuginfo`：rustc 預設即為 0（不最佳化、無除錯資訊），整數溢位檢查保持開啟（21 關需要）
  - `-A dead_code`：關卡骨架常有尚未呼叫的函式或 enum 變體（例 05 關正確解答），Run 與 Submit 一致關掉，避免同一份程式碼兩種模式警告不同
  - `-F unsafe_code`：所有關卡皆不需 unsafe，Run 與 Submit 一致禁止，擋 `unsafe impl Send`、`transmute` 類取巧；`-F` 為 forbid，使用者寫 `#![allow(unsafe_code)]` 也無法解除（decisions.md ⑫）
- 編譯（Submit）：同上，再加 `--test`（`--test` 模式下 `main` 與只被 `main` 用到的函式都算未使用，`-A dead_code` 讓它們不冒出玩家看不懂的警告）
- **參數組裝**：docker 與 rustc 參數一律以 `Vec<String>` 傳給 `Command::args`，每個旗標與值拆成獨立元素（例 `["--network", "none"]`），不用 `--network=none` 或空白串接；`-v` 的值為單一元素 `<暫存目錄>:/work:ro`
- 執行（Run）：`docker exec <name> /tmp/main`
- 執行（Submit）：`docker exec <name> /tmp/main --test-threads=1 --exact __quest_hidden_tests::<測試名 1> __quest_hidden_tests::<測試名 2> ...`
  - 單執行緒：輸出順序穩定、解析可靠、不吃滿 pids
  - `--exact` 加完整名稱：只跑隱藏測試，玩家自己寫的 `#[test]` 不執行（測試名來自啟動時抽出的清單，不含使用者輸入）

### 3.4 併發控制：Semaphore 與 503 策略

採「雙 Semaphore」：一個限制**排隊總量**、一個限制**實際執行數**。

```
① 嘗試拿排隊名額
  ↓
② 等待執行 permit
  ↓
③ 檢查 Docker
  ↓
④ 執行判題
  ↓
⑤ 依序歸還
```

① `admission.try_acquire()`（容量 = `max_concurrent + max_queue`）；失敗立即回 `503 JUDGE_BUSY`、`Retry-After: 2`
② `tokio::time::timeout(queue_wait_timeout, exec_permits.acquire())`；超時回 `503 JUDGE_BUSY`
③ 讀 §3.10 的可用性快取；不可用回 `503 JUDGE_UNAVAILABLE`
  - 順序固定：滿載時即使 Docker 不可用（映像不存在、daemon 無法連線）也先回 `JUDGE_BUSY`
④ 一個 permit 涵蓋整個判題（容器啟動 + 編譯 + 執行 + 刪除），因為整段都佔用 CPU／記憶體
⑤ RAII 自動歸還，handler 被 drop 也會歸還

**為什麼是 503 而不是 429**

- `429 Too Many Requests` 語意是「**這個 client** 送太多」；MVP 沒有身分、沒有 per-client 計數
- 滿載是**全站資源**不足 → `503 Service Unavailable` + `Retry-After` 語意正確
- `429` 保留給之後的 per-IP rate limit

**Run 與 Submit 共用同一組 permit**：兩者成本相近，分開會讓總容器數失控。

### 3.5 Timeout 與輸出上限

| 項目 | 預設值 | 超過時 |
|---|---|---|
| 排隊等待 | 10 s | `503 JUDGE_BUSY` |
| 容器啟動（`docker run -d`） | 10 s | `docker rm -f` 該容器，回 `503 JUDGE_UNAVAILABLE` |
| 編譯（exec 計時） | 15 s | `docker kill`，`status = "timeout"`、`compile.timed_out = true`、`execution = null` |
| 執行（exec 計時） | 5 s | `docker kill`，`status = "timeout"`、`execution.timed_out = true`、`exit_code = null`，**保留已讀到的 stdout／stderr** |
| 容器最長存活（`sleep N`） | 60 s | 容器自行結束（保險） |
| stdout／stderr（各自） | 64 KiB | 保留前 64 KiB、`docker kill`，`*_truncated = true`，`exit_code = null`，`status = "output_limit"` |
| 編譯 stderr | 1 MiB | 超過部分丟棄、不終止編譯、不影響 `status`；診斷最多回傳 50 筆 |
| request `code` | 64 KiB | `413 PAYLOAD_TOO_LARGE` |

- 編譯 timeout 包含容器內 `cp`；容器冷啟動時間**不計入**編譯 timeout（另有啟動 timeout）
- 輸出上限採「讀到上限就殺」：無限 `println!` 迴圈若只丟棄不殺，仍會空轉到 timeout
- stdout 邊讀邊累積到 buffer；timeout 或超限時，buffer 內已讀到的內容照樣回傳（前端 GridWorld 靠這段播放到中斷點）
- 截斷位置若切在 UTF-8 多位元組字元中間，往前退到合法邊界
- `docker exec` 回 exit 137 且非 host 主動 kill → 判定為 OOM，`status = "memory_limit"`；編譯與執行兩段都適用
  - 執行階段 OOM：`execution.exit_code = null`（不原樣回 137）、保留已讀到的 stdout／stderr
  - 編譯階段 OOM：`compile.success = false`、`compile.timed_out = false`、`execution = null`、Submit 的 `tests` 全部 `not_run`

### 3.6 結果判定

Run 與 Submit 共用**同一組 `status` 列舉**，前端只需一套分支。

**`status` 判定（依階段，先符合者為準）**

```
① 編譯超時
  ↓
② 編譯 OOM
  ↓
③ 編譯失敗
  ↓
④ 執行超時
  ↓
⑤ 執行 OOM
  ↓
⑥ 輸出超限
  ↓
⑦ 執行失敗
  ↓
⑧ 成功
```

① host 編譯計時超過 15 秒並 kill → `timeout`，`compile.timed_out = true`
② 編譯 exec 回 137 且非 host kill → `memory_limit`（rustc 被 OOM 殺掉時沒有正常的錯誤輸出，不能判成 `compile_error`）
③ rustc 正常結束且 exit 非 0 → `compile_error`
④ host 執行計時超過 5 秒並 kill → `timeout`，`execution.timed_out = true`
⑤ 執行 exec 回 137 且非 host kill → `memory_limit`（輸出已截斷時屬 host kill，改依 ⑥ 判為 `output_limit`）
⑥ stdout 或 stderr 超過 64 KiB，host kill → `output_limit`
⑦ Run：exit 非 0（panic 為 exit 101；`std::process::exit(n)` 非 0 也歸此類）→ `runtime_panic`
　 Submit：至少一個測試 `failed` → `test_failed`（libtest 失敗時 exit 101）
⑧ Run：exit 0；Submit：所有測試 `passed` 且 exit 0 → `ok`

- **Submit 補充規則**（套用在 ⑦ ⑧；①–⑥ 仍依原優先順序）
  - 頂層 `passed = true` 只在宣告的每一個測試都是 `passed` 時成立
  - exit 0 但有宣告的測試沒有結果 → 缺結果者為 `not_run`，`status = test_failed`
  - exit 非 0 且沒有任何 `failed` 測試（例如 harness 崩潰）→ 沒有結果者為 `not_run`，`status = test_failed`
- ①–③ 發生時不進入執行階段，`execution = null`
- 列舉順序對照 api.md §0.3：`timeout` → `memory_limit` → `compile_error` → `output_limit` → `runtime_panic`／`test_failed` → `ok`
- `output_limit` Run 與 Submit 共用判定；Submit 時 libtest 捕捉測試輸出，實務上極少觸發，BDD 只寫 Run 情境
- 每種模式實際會出現的值：
  - Run：`ok`、`compile_error`、`runtime_panic`、`timeout`、`memory_limit`、`output_limit`
  - Submit：`ok`、`compile_error`、`test_failed`、`timeout`、`memory_limit`、`output_limit`
- `memory_limit`、`output_limit` 定案為獨立狀態，不併入 `runtime_panic`：前端提示文案與常見原因不同（ux.md §3.3）

**libtest 文字輸出解析（`test_output.rs`）**

- 不使用 `--format json`：在 stable 需 `-Z unstable-options`，不可靠
- 規則：
  - `^test (\S+) \.\.\. (ok|FAILED|ignored)$` → 每個測試結果
  - `^---- (\S+) stdout ----$` 到下一個 `----` 或 `^failures:$` 之間 → 該測試的失敗訊息（panic message）
  - panic message 去掉 `thread '<名稱>' (<thread id>) panicked at <路徑>:<行>:<欄>:` 這行（rustc 1.99 起帶 thread id，解析需容許有無）與 `note: run with \`RUST_BACKTRACE=1\`` 提示行，並去掉頭尾空行
- 輸出每個測試 `{name, status, message}`，順序依啟動時抽出的 `tests.rs` 宣告順序
  - `name`：去掉 `__quest_hidden_tests::` 前綴
  - `status`：`ok` → `passed`；`FAILED` → `failed`；`ignored` 或輸出中沒有結果 → `not_run`（關卡測試不應使用 `#[ignore]`）
  - `message`：`failed` 時為 panic message；`passed`、`not_run` 為 `null`
- 編譯失敗、編譯超時：全部測試為 `not_run`
- 執行中途被 kill（timeout／OOM／輸出超限）：已有結果的照實回傳，其餘（含被 kill 當下正在跑的那個）為 `not_run`
- 頂層 `passed = true` 當且僅當 `status == "ok"`

### 3.7 rustc 診斷解析（`diagnostics.rs`）

**輸入**：rustc 以 `--error-format=json` 輸出到 stderr，每行一個 JSON 物件。

**輸出結構**

```json
{
  "code": "E0382",
  "message": "borrow of moved value: `s`",
  "line": 5,
  "column": 20,
  "end_line": 5,
  "end_column": 21,
  "level": "error",
  "explanation_url": "https://doc.rust-lang.org/error_codes/E0382.html",
  "rendered": "error[E0382]: borrow of moved value: `s`\n --> /tmp/main.rs:5:20\n..."
}
```

**規則**

- `code`：`code.code`；無則 `null`（例如一般語法錯誤）
  - lint 警告的 `code.code` 是 lint 名（例 `unused_variables`），照樣填入
- `message`：頂層 `message` 字串
- 位置：取 `spans` 中 `is_primary = true` 的第一個 span（1-based）
  - `line` ← `line_start`、`column` ← `column_start`
  - `end_line` ← `line_end`、`end_column` ← `column_end`（rustc 的 `column_end` 是**不含**的結束位置，與 Monaco `endColumn` 語意相同，可直接使用）
- `level`：`error` | `warning` | `note` | `help`；`error: internal compiler error` 歸為 `error`
- `explanation_url`：`code` 符合 `^E\d{4}$` 時為 `https://doc.rust-lang.org/error_codes/{code}.html`，否則 `null`
- `rendered`：rustc JSON 的 `rendered` 原文；primary span 落在隱藏區段時為 `null`
- warning 一律保留（31 關以警告為線索），不因編譯成功而丟棄
  - 例外：Submit 時 primary span 落在隱藏區段的 warning 一律過濾不回傳（玩家無法修正）；同位置的 error 照舊回傳，位置欄位為 `null`
- 過濾：`message` 為 `aborting due to ...`、`... warning(s) emitted` 的摘要訊息，以及 `level = "failure-note"`（`For more information about this error...`）不回傳
- 非 JSON 行（例如連結器錯誤）→ 包成 `{code: null, level: "error", message: <原文>, rendered: <原文>}`，四個位置欄位與 `explanation_url` 皆為 `null`
- 排序：依 rustc 輸出順序；過濾後再取前 50 筆

**合併檔組成（Submit 模式）**

```
① 使用者程式碼
  ↓
② 換行
  ↓
③ 隱藏模組開頭
  ↓
④ 共用模擬器與地圖
  ↓
⑤ tests.rs
  ↓
⑥ 隱藏模組結尾
```

① 原樣放在最前面，第 1 行即編輯器第 1 行
② 使用者程式碼不以 `"\n"` 結尾時補一個 `"\n"`；已有換行則不再補，結尾空行原樣保留（避免黏到下一行，也不改變使用者行數）
③ 依序五行：`#[cfg(test)]`、`#[allow(dead_code)]`、`#[allow(unused_imports)]`、`mod __quest_hidden_tests {`、`    use super::*;`
  - `#[allow(unused_imports)]`：測試沒用到 `super::*` 的項目時不冒出隱藏區段的警告
④ 僅 `visual = "grid"`：`content/shared/grid_sim.rs` 全文 + `"\n"`，再接啟動時依 `[grid]` 產生的 `level_grid()` + `"\n"`（§2.2；省略的 `walls`、`gems`、`doors` 產生 `vec![]`）
  - 產生格式為單行，欄位順序與 `grid_sim.rs` 的 `Grid` 一致，座標轉成 tuple：
    ```rust
    fn level_grid() -> Grid { Grid { width: 5, height: 2, start: (0, 0), goal: (4, 1), walls: vec![(2, 0), (2, 1)], gems: vec![], doors: vec![] } }
    ```
⑤ 該關 `tests.rs` 全文（只含模組內容，不自帶 `mod`，見 curriculum.md §1.4）；不以 `"\n"` 結尾時補一個 `"\n"`，避免最後一行黏到結尾的 `}`
⑥ `}` + `"\n"`

- 模組名 `__quest_hidden_tests` 避免與玩家自己的 `mod tests` 衝突
- 模組內的模擬器與測試輔助項目會遮蔽 `use super::*` 引入的同名項目，玩家同名定義不會造成編譯錯誤

**行號對應（Submit 模式）**

- `user_line_count` = `user_code.lines().count()`（Rust `str::lines`，結尾換行不多算一行）；之後全部是隱藏區段
- primary span 的 `line_start <= user_line_count` → 原樣回傳
  - 若 `line_end` 跨進隱藏區段，`end_line` 截到 `user_line_count`、`end_column` 設為 `null`（前端標到行尾）
- `line_start` 落在隱藏區段 → 四個位置欄位與 `rendered` 全為 `null`（不洩漏隱藏測試位置），`message` 保留
- Run 模式只編譯使用者程式碼，無需對應

### 3.8 已知邊界情況

- 使用者自己寫了 `mod tests`：隱藏模組名為 `__quest_hidden_tests`，不衝突；玩家的 `#[test]` 會被編譯但不執行（§3.3 `--exact`）
- 使用者自己宣告 `mod __quest_hidden_tests`：E0428 照實回傳診斷，屬刻意行為不另處理
- 使用者把 `use super::*` 需要的項目設成私有：同一 crate 內 `super::*` 可見，無問題
- `fix-compile` 關卡 starter 本來就編不過：Run／Submit 回 `compile_error` 是預期行為
- 使用者程式碼含 `include_str!("/etc/passwd")`：只會讀到**容器內**的檔案，見 §4

### 3.9 Judge 設定（MVP 為程式內常數）

```rust
struct JudgeConfig {
    docker_bin: &'static str,         // "docker"；測試設為不存在的路徑以模擬 Docker 不可用
    image: &'static str,              // "rust-quest-runner:dev"
    max_concurrent: usize,            // 預設 max(1, CPU 核數 / 2)
    max_queue: usize,                 // 預設 max_concurrent * 4
    queue_wait_timeout: Duration,     // 10 s
    container_start_timeout: Duration,// 10 s
    compile_timeout: Duration,        // 15 s
    run_timeout: Duration,            // 5 s
    container_max_lifetime_secs: u64, // 60
    output_limit_bytes: usize,        // 64 KiB
    max_code_bytes: usize,            // 64 KiB
}
```

- 整合測試可直接建構不同值的 `JudgeConfig`（例 `max_concurrent = 0` 測 503、`docker_bin` 設為不存在的路徑測 daemon 無法連線）
- 所有 docker CLI 呼叫（version、image inspect、run、exec、kill、rm、ps）都經 `docker_bin` 執行
- 不開放環境變數覆寫（需求外不做）
- 各數值的保留理由見 decisions.md §13；未列在 `JudgeConfig` 的常數（body 128 KiB、編譯 stderr 1 MiB、診斷 50 筆、health 快取 10 秒、`Retry-After: 2`）寫在各自模組內

### 3.10 Docker 可用性檢查

- 啟動時與 `/api/health` 呼叫時執行：
  - `docker version --format '{{.Server.Version}}'`：daemon 是否可連（只看成功與否，版本字串不回傳）；`docker_bin` 無法執行（找不到執行檔）同樣視為 `docker_unreachable`
  - `docker image inspect rust-quest-runner:<tag>`：映像是否存在
- 結果快取 10 秒，避免 health 被頻繁呼叫時狂打 docker CLI
- 判題時 `docker run` 失敗或超過 10 秒（daemon 掛掉、資源不足）→ 回 `503 JUDGE_UNAVAILABLE`，並讓快取失效
- `/api/health` 只回 `status` 與 `judge.available`、`judge.reason`（api.md §5）

---

## 4. 安全

### 4.1 威脅模型

使用者送來的是**任意 Rust 程式碼**，且在**編譯期**（巨集）與**執行期**都可能有惡意行為。

- **無限迴圈／CPU 耗盡**：`loop {}`
- **Fork bomb**：`std::process::Command` 不斷產生子行程
- **記憶體耗盡**：`vec![0u8; 1 << 40]`、遞迴
- **輸出洪水**：無限 `println!`
- **讀檔**：執行期 `std::fs::read("~/.ssh/id_rsa")`；**編譯期** `include_str!("/etc/passwd")`
- **寫檔**：寫滿磁碟、改 host 檔案
- **網路**：對外連線、掃描內網、外洩資料
- **編譯器炸彈**：巨集遞迴展開、巨大常數陣列，讓 rustc 本身吃滿資源
- **API 暴露**：服務綁到 `0.0.0.0` 讓區網任何人都能執行程式碼

### 4.2 MVP 對策（Docker 沙箱）

- **CPU**：每容器 `--cpus 1`、編譯／執行分段 timeout，超時 `docker kill`
- **Fork bomb**：`--pids-limit 64`、`--init` 回收殭屍
- **記憶體**：`--memory 256m --memory-swap 256m`，OOM 由 cgroup 殺掉
- **輸出洪水**：host 端讀到 64 KiB 即 `docker kill`
- **讀 host 檔案**：容器只看得到映像內容 + 唯讀 `/work`（只含這次的 `main.rs`）；編譯期 `include_str!` 也只能讀容器內檔案
- **寫檔**：rootfs `--read-only`、`/tmp` 是 64m tmpfs、`--ulimit core=0`
- **網路**：`--network none`
- **提權**：`--user 1000:1000`、`--cap-drop ALL`、`no-new-privileges`、tmpfs `nosuid`、預設 seccomp
- **編譯器炸彈**：編譯 timeout + 記憶體上限同樣作用在 rustc
- **殘留**：容器 `--rm` + guard `docker rm -f` + 啟動時依 label 清理 + `sleep N` 自然結束
- **API 暴露**：預設綁 `127.0.0.1`；body 上限 128 KiB；Semaphore 限制總容器數

### 4.3 殘餘風險（誠實列出）

- **容器逃逸**
  - 容器與 host **共用 kernel**；kernel 或 runc 漏洞（例如歷史上的 CVE-2019-5736、CVE-2024-21626）可能讓容器內程式碼逃逸到 host
  - 已降低但未消除：非 root、無 capability、no-new-privileges、seccomp 大幅縮小攻擊面
  - macOS／Windows 的 Docker Desktop 多一層 VM，逃逸後落在 VM 而非實體 host，風險較低
- **Docker daemon 權限**
  - rootful Docker 的 daemon 以 root 執行；backend 所在帳號能呼叫 docker CLI ＝ 實質具備 host root 權限
  - 因此 **backend 本身若有可被利用的漏洞**（例如指令注入），後果等同 root 被拿下
  - 緩解：所有 docker 參數由程式組裝成 `Vec<String>` 傳給 `Command::args`，**不經 shell 拼字串**；使用者程式碼只以檔案形式進入容器，永不出現在指令列
- **資源型 DoS**
  - `max_concurrent × 256m` 記憶體、`max_concurrent` 核 CPU 是理論上限；Semaphore 設太大仍可能拖垮 host
  - Docker daemon 本身在高頻 create／remove 下也會成為瓶頸
- **隱藏測試洩漏**
  - panic 訊息會回傳給使用者，測試的 `assert_eq!` 訊息可能透露預期值；屬學習用途可接受
  - 使用者程式碼可在 Submit 時 `include_str!("/work/main.rs")` 讀到合併後的完整檔案（含 `tests.rs`）並印出；**已定案接受**（學習用途），對外開放前需重新評估
- **映像供應鏈**：基底映像若被汙染，所有判題都受影響；以 digest 固定版本緩解

### 4.4 後續強化選項（MVP 不做）

- **rootless Docker**：daemon 以一般使用者執行，逃逸後只拿到該使用者權限
- **gVisor（`--runtime runsc`）**：在使用者空間攔截 syscall，容器不直接碰 host kernel；代價是 syscall 較慢、相容性需驗證
- **userns-remap**：容器內 uid 對應到 host 上無權限的 uid 範圍
- **自訂 seccomp profile**：在預設基礎上再禁止 `fork` 以外不需要的 syscall
- **對外開放前必做**：反向代理 + TLS、per-IP rate limit（此時才用 429）、帳號機制、判題機與 API 主機分離

---

## 5. 效能

### 5.1 關卡內容：啟動時一次載入

- 啟動時讀完 `content/levels/`，建立 `LevelStore { by_id: HashMap<String, Arc<Level>>, ordered: Vec<Arc<Level>> }`
- `GET /api/levels`、`GET /api/levels/:id` 不碰檔案系統，**無 N+1 讀檔**
- 可選優化：啟動時把列表 JSON 預先序列化成 `Bytes`，請求時直接回傳（MVP 不做，量級不需要）
- 代價：改關卡內容需重啟 backend（hot reload 需求外不做）

### 5.2 熱路徑分析

- **GET /api/levels、/api/levels/:id**
  - 記憶體查表 + JSON 序列化，微秒等級；不是瓶頸
- **POST /run、/submit（真正的熱路徑）**
  - 每次判題的時間組成（估計值，需實測）：
    ```
    ① 排隊等待
      ↓
    ② 容器冷啟動
      ↓
    ③ 編譯
      ↓
    ④ 執行
      ↓
    ⑤ 刪除容器
    ```
    ① 0 ～ `queue_wait_timeout`，取決於負載
    ② 每次容器啟動多約 **0.3–1 秒**（`docker run -d` 建立 namespace、cgroup、掛 tmpfs）
    ③ 小型單檔 0.3–2 秒；`--test` 模式較慢
    ④ 一般 < 100 ms；上限 5 秒
    ⑤ `docker rm -f` 約 0.1–0.3 秒，可在回應送出後非同步執行（guard 內 `tokio::spawn`），不計入回應延遲
  - **實測基準**（使用者於 `rust-quest-runner:dev` 量測，單次 `docker run` 含編譯）：冷啟動約 **0.6 秒**、熱（映像與 page cache 已在記憶體）約 **0.26 秒**
  - 本規格採「殼容器 + 兩次 exec + rm」，比單次 `docker run` 多 3 次 docker CLI 往返，估計端到端落在 **約 0.5–1.5 秒**；實作後以 Docker 整合測試量測 p50／p95 並回填此處
- **併發與 tokio**
  - 全程用 `tokio::process`，等待子行程是非同步的，**不阻塞 worker thread**
  - `TempDir` 建立與寫檔是小量同步 I/O，可接受；若要嚴謹可包 `spawn_blocking`
  - 吞吐量上限 ≈ `max_concurrent / 平均判題秒數`；例：4 permit、1 秒 → 約 4 req/s
- **瓶頸**
  - host CPU（rustc）、Docker daemon 建立容器速度
  - 記憶體：`max_concurrent × 256m`

### 5.3 後續優化（MVP 不做）

- **預熱容器池 + `docker exec`**：預先開好 N 個殼容器，判題直接 `exec`，省下 0.3–1 秒冷啟動
  - 代價：容器重用需徹底清 `/tmp`、殺掉殘留行程，否則前一位使用者的狀態會汙染下一位；隔離性比拋棄式差，需要額外設計
- **結果快取**：以 `hash(level_id + mode + code)` 快取判題結果，重複送出同樣程式碼直接回傳
- **預先序列化列表 JSON**

---

## 6. 測試策略

### 6.1 測試分層

```
① 後端單元測試
  ↓
② 後端整合測試
  ↓
③ Docker 整合測試
  ↓
④ 內容測試
  ↓
⑤ BDD 情境
  ↓
⑥ 前端單元測試
  ↓
⑦ E2E
```

① `cargo test`；純邏輯，**不需 docker**
② `cargo test`；`tower::ServiceExt::oneshot` 打 Router，**不需 docker**
③ `cargo test -- --ignored`（或 `just test-docker`）；需 docker 與 runner 映像
④ 同 ③；驗證每關的 solution 與 starter
⑤ cucumber-rs 跑 `features/*.feature`；需 docker 的情境標 `@docker`
⑥ Vitest
⑦ Playwright（playwright-bdd）

### 6.2 各層測什麼

- **① 單元測試（不依賴 docker）**
  - `diagnostics.rs`：以 rustc 真實輸出存成 fixture（`tests/fixtures/rustc/*.jsonl`），測 E0382、語法錯誤（無 code）、lint 警告保留、連結器非 JSON 行、`aborting due to` 與 `failure-note` 過濾、primary span 對應 `end_line`／`end_column`、`rendered` 原文、Submit 行號落在隱藏區段時四個位置欄位與 `rendered` 為 `null`、span 跨進隱藏區段時截斷、隱藏區段的 warning 被過濾而 error 保留、50 筆上限
  - `test_output.rs`：全部通過、部分失敗含 panic 訊息（含 thread id 的前綴行被去除）、被 kill 時剩餘測試為 `not_run`、編譯失敗時全部 `not_run`、`__quest_hidden_tests::` 前綴去除、順序依 `tests.rs` 宣告順序
  - 合併檔組成：grid 關卡含共用模擬器與產生的 `level_grid()`、非 grid 關卡兩者都不含、隱藏模組開頭五行、`user_line_count` 計算、使用者程式碼最後一行無換行時補一個換行、已有換行時不多補
  - `level_grid()` 產生：§3.7 ④ 的單行格式、欄位值與 `[grid]` 相同、省略的陣列產生 `vec![]`
  - `docker.rs` 參數拆成獨立元素、`-v` 值為 `<暫存目錄>:/work:ro`、容器名以 `rq-judge-` 開頭且兩次產生不同、Run 與 Submit 的 rustc 參數都含 `-A dead_code` 與 `-F unsafe_code`
  - 輸出截斷：超過 64 KiB 保留前段、`stdout_truncated = true`、切在 UTF-8 字元中間時往前退
  - 暫存目錄權限：建立後目錄為 `0755`、檔案為 `0644`
  - `docker.rs` 指令組裝：**安全回歸測試**，斷言 rustc 參數含 `--edition 2024`；docker 參數必含 `--network none`、`--read-only`、`--cap-drop ALL`、`--user 1000:1000`、`--pids-limit 64`、`--memory-swap 256m`，且不含 `--privileged`、`docker.sock`
  - `content/loader.rs`：用測試用暫存目錄造關卡，驗證 §2.2 每一條規則違反時載入失敗（含 `id` 與目錄名不同、未知欄位、`chapter` 不在 `chapters.toml`、`chapters.toml` 缺檔或章節重複、`order` 重複、空 `story`）；省略欄位得到預設空陣列與 `null`；`hints` 省略得到 `[]`；測試名稱依宣告順序抽出
  - 結果判定：§3.6 ①–⑧ 全部分支（含編譯階段 OOM 判為 `memory_limit`、被 kill 時 `exit_code = null`）與 Submit 補充規則（exit 0 但缺結果、exit 非 0 但無 failed）
- **② 整合測試（不依賴 docker）**
  - `GET /api/levels`：`levels` 排序正確、`chapters` 只含有關卡的章節，且 JSON 中**不出現** `solution`、`tests` key
  - `GET /api/levels/:id`：存在、404 `LEVEL_NOT_FOUND`、不含 `solution`／`tests`
  - `POST run／submit`：非 JSON body → 400、缺 `Content-Type` → 400、缺 `code` → 400、`code` 超過 64 KiB → 413、不存在關卡 → 404、不存在關卡且 body 無效 → 400；錯誤 body 恰為 `{error: {code, message}}`
  - `JudgeConfig { max_concurrent: 0, max_queue: 0 }` → 503 `JUDGE_BUSY` 且帶 `Retry-After: 2`（不需 docker 即可測）；同時 `docker_bin` 不存在時仍回 `JUDGE_BUSY`
  - `docker_bin` 設為不存在的路徑 → `/api/health` 回 `degraded`、`judge.reason = docker_unreachable`；run／submit 回 503 `JUDGE_UNAVAILABLE` 且不帶 `Retry-After`（不需 docker 即可測）
- **③ Docker 整合測試（`#[ignore]`）**
  - Run：hello world 回 `ok` 與 stdout；`loop {}` 先印幾行再卡住 → 回 `timeout`、**stdout 保留已印出的行**，且**容器確實被移除**（`docker ps -a --filter label=...` 為空）；無限 `println!` 回 `output_limit`；fork bomb 不影響 host 且回錯誤狀態；`std::net::TcpStream::connect` 失敗
  - Submit：全通過（`ok`）、部分失敗（`test_failed`，含 panic message）、編譯錯誤的行號對應、玩家自寫的 `#[test]` 不被執行
  - Run：`panic!` 回 `runtime_panic`、exit 101
  - 映像名稱設成不存在的 tag → `/api/health` 回 `degraded`、`judge.reason = runner_image_missing`；run／submit 回 503 `JUDGE_UNAVAILABLE` 且不帶 `Retry-After`（需真的 daemon 才能區分「映像不存在」與「daemon 無法連線」，因此歸本層）
  - 執行前檢查 docker 可用，不可用時 `panic!` 並提示「請先 build runner 映像」，避免靜默通過
- **④ 內容測試（`#[ignore]`，CI 與新增關卡時必跑）**
  - 每一關：`solution.rs + tests.rs` → `status = ok`
  - 每一關：`starter.rs + tests.rs` → `status != ok`
    - `fix-compile` 關卡：必須是 `compile_error`
    - `build`／`debug` 關卡：`test_failed`（或依關卡設計為 `compile_error`）
  - 每一關：`solution.rs` 用 Run 模式可執行（有 `main` 的關卡）
  - grid 關卡：solution 的 Run stdout 套 `[grid]` 模擬須通關（Run 畫面與判題一致），模擬同時涵蓋 1,000 步上限
    - 做法：測試模組以 `include!("../../content/shared/grid_sim.rs")` 引入同一份模擬器，用 loader 解析出的 `[grid]` 建出 `Grid`，把 stdout 逐行交給 `assert_cleared`；不需另寫解析器
    - 判題用的 `level_grid()` 與這裡的 `Grid` 都從同一份 `[grid]` 產生，不需比對地圖
  - memory 關卡：每步 `line` 不超過對應檔案行數；`note` 以「（修正後）」開頭的步驟對照 `solution.rs`，其餘對照 `starter.rs`
- v1.2 確認：①–④ 每一項都只依賴已定的檔案與 runner 映像，沒有需要手寫第二份地圖或額外工具的項目
- **⑤ BDD（cucumber-rs）**
  - 以使用者語言描述：「送出正確解答會通過」「無限迴圈會在 5 秒內回報超時」「排隊滿時回 503」
  - `@docker` 標籤的情境在無 docker 環境以 tag filter 跳過，並在輸出中列出跳過數量
- **⑥ 前端單元測試（Vitest）**
  - grid stdout 指令解析器（含 1,000 步上限）、星等計算、progress store 解鎖規則與 localStorage 讀寫（含損壞資料）、草稿自動儲存、診斷 → Monaco marker 轉換
- **⑦ E2E（Playwright + playwright-bdd）**
  - 進入第一關 → 編輯 → Run 看到輸出 → Submit 通過 → 下一關解鎖 → 重新整理後進度保留
  - 預設打 mock API（`page.route`）以求穩定；另有一組 `@docker` 情境打真後端

### 6.3 執行指令（建議）

- `cargo test`：單元 + 不需 docker 的整合
- `cargo test -- --ignored`：docker 整合 + 內容測試
- `cargo test --test cucumber -- --tags 'not @docker'`
- `npm test`（Vitest）、`npm run e2e`（Playwright）；前端套件管理用 npm（`frontend/package-lock.json`）

---

## 7. 前端架構

畫面、互動與進度規則以 ux.md 為準；本節只描述程式結構。

### 7.1 路由（Vue Router）

- `/`：章節地圖（`LevelMapView`），依 chapter 分組，顯示鎖定狀態與星等
- `/level/:levelId`：關卡頁（`LevelView`）
  - `beforeEnter` guard：未解鎖 → redirect `/` 並 toast「這關還沒解鎖」（ux.md §1.2）
  - 不存在（API 404）→ 顯示找不到頁並提供回地圖連結
- 通關結算是關卡頁上的 Modal，不另開路由

### 7.2 狀態管理（Pinia）

- **`useLevelStore`**
  - `chapters: ChapterSummary[]`、`summaries: LevelSummary[]`、`details: Map<string, LevelDetail>`
  - `fetchList()`、`fetchDetail(id)`：已快取則不再請求
- **`useProgressStore`**（localStorage）
  - key 與結構照 ux.md §6.5：`rustquest.progress.v1`（`Progress`）、`rustquest.prefs.v1`（`Prefs`）
  - 星等：`stars: { pass, noHint, firstTry }` 三個布林，取歷史聯集（ux.md §6.1）
  - 本次遊玩計數：`current.hintsRevealed`、`current.failedSubmits`；只在 Submit 通過、結算星等後歸零（ux.md §6.1），進出關卡與重新整理都不重置
  - `Prefs`：只有 `theme`（ux.md §6.5）；播放速度只存在 store 記憶體
  - 解鎖規則照 ux.md §6.3，排序依 `(chapter, order)`
  - 讀取失敗或 JSON 損壞 → 以空進度啟動，不讓頁面壞掉
- **`useDraftStore`**（localStorage，ux.md §6.6）
  - key：`rustquest.draft.<levelId>`，值為程式碼字串
  - 編輯後 debounce 500 ms 寫入；進入關卡時有草稿就載入，沒有就用 `starter`
  - 「重設」改回 `starter` 並刪除草稿
- **`useAttemptStore`**（目前關卡的作答狀態）
  - `code`、`phase: 'idle' | 'running' | 'submitting'`（與 ux.md §3.1 狀態機同名：`running`＝Run 請求中、`submitting`＝Submit 請求中）
  - `lastRun: RunResponse | null`、`lastSubmit: SubmitResponse | null`
  - 切換關卡時以 `AbortController` 取消進行中的請求
  - Run／Submit 請求逾時 60 秒（前端常數）：逾時即 abort，歸為 `network_error`（ux.md §3.2）
- **版面記憶（只存記憶體，不寫 localStorage）**：平板抽屜「已自動展開過」與手機「已進過」的關卡 id 集合，放在 Pinia store；重新整理後重置（ux.md §2.2、§2.3）

### 7.3 元件樹

```
App
├── LevelMapView
│   └── ChapterSection
│       └── LevelNode（編號、名稱、kind 圖示、鎖頭、星等）
└── LevelView
    ├── LevelInfoPanel（左）
    │   ├── FerrisBubble（顯示 story，ux.md §1.4）
    │   ├── ConceptBlock（Markdown 渲染）
    │   └── HintList（逐條揭露，寫入 current.hintsRevealed；hints 為空時不渲染）
    ├── EditorPanel（中）
    │   ├── EditorToolbar（Run／Submit／Reset 回 starter）
    │   └── CodeEditor（Monaco 包裝，接收 diagnostics 設 markers）
    ├── ResultPanel（右）
    │   ├── VisualPanel
    │   │   ├── GridWorld（visual = grid）
    │   │   ├── MemoryView（visual = memory）
    │   │   └── PlaybackBar
    │   └── Console（輸出／編譯錯誤／測試結果三個分頁）
    ├── LevelCompleteDialog（星等、下一關）
    └── ConfirmDialog（看提示、重設的確認；海洋繪本風格 modal，不用 window.confirm，ux.md §6.2、§6.6）
```

### 7.4 Monaco 整合

- 使用 `monaco-editor` 內建 `rust` 語法高亮；以 Vite `?worker` 載入 editor worker
- `Diagnostic` → `monaco.editor.setModelMarkers`
  - `level`：`error → Error`、`warning → Warning`、`note／help → Info`
  - `line = null` 的診斷不畫 marker，只列在編譯錯誤分頁
  - 範圍直接對應：`startLineNumber = line`、`startColumn = column`、`endLineNumber = end_line`、`endColumn = end_column`
  - `end_column = null`（span 被截斷）時標到 `end_line` 行尾
  - 零寬度 span（`line = end_line` 且 `column = end_column`，rustc 常見於「缺分號」）：Monaco 不畫零寬 marker，依序改寫終點
    - `model.getWordAtPosition` 回傳的 token 結尾大於 `column`：`endColumn` 擴到該 token 結尾
    - 否則且不在行尾：`endColumn = column + 1`（至少 1 字元）
    - 位於行尾（`column` 等於該行 `maxColumn`）且該行非空：改標前一個字元（`startColumn = column - 1`）
    - 空行：照原值交給 Monaco，不另處理
- panic marker：Run 的 stderr 含 `main.rs:<行>:<欄>` 時，該行整行畫 error marker（`startColumn = 1`、`endColumn = 該行 maxColumn`），欄位只供錯誤卡片顯示
- 使用者再次編輯時清空 markers

### 7.5 可視化元件如何吃後端回傳

**GridWorld（`visual = grid`）**

- 地圖：`GET /api/levels/:id` 的 `grid`（`level.toml` `[grid]`，格式見 ux.md §4.2）
- 指令：`POST /run` 回傳的 `execution.stdout`；後端**不解析** stdout
- 前端 `parseGridCommands(stdout)` 依 ux.md §4.3 解析：`MOVE <方向>`、`PICK`、`OPEN <方向>`；不是指令嘗試的行只顯示在主控台
- 規則與 `content/shared/grid_sim.rs` 一致：撞牆、撞關閉的門、走出地圖、無效指令、`PICK` 空格、`OPEN` 沒有門都立即停止；通關＝撿完全部寶石且停在 `goal`
- **兩層上限**
  - 後端：stdout 最多 64 KiB，超過即終止並回 `execution.stdout_truncated = true`（`status = output_limit`）
  - 前端：指令步數上限 1,000 步（前端常數，與 `grid_sim.rs` 的 `MAX_STEPS` 相同），超過的指令不播放，停在第 1,000 步並提示「指令超過 1,000 步，後面的不播放了」（文案以 ux.md §4.5 為準）
- `stdout_truncated = true` 時：最後一行可能不完整，**丟棄最後一行**後播放，並提示「輸出過長已截斷」
- `status` 為 `timeout`、`memory_limit` 時：後端仍回傳已截取的 stdout，照常播放到中斷點，最後顯示對應狀態
- Submit 不驅動 GridWorld：Submit 的 stdout 是 libtest 輸出；可視化維持最後一次 Run 的結果
- 請求中由 `pending` prop（`phase !== 'idle'`）讓 GridWorld 回第 0 步並半透明；Submit 回來後 `pending` 解除，GridWorld 還原到上次 Run 播放結束的那一步（不重播），從未 Run 過則維持初始地圖（ux.md §4.1）

**MemoryView（`visual = memory`）**

- 資料來源：`GET /api/levels/:id` 的 `memory_snapshots`（關卡作者手寫的靜態快照，與使用者程式碼無關）
- 型別：ux.md §5.1 `MemorySnapshot`（api.md §6 有同一份 TS 型別）
- 以 `PlaybackBar` 逐步切換，`line` 同步編輯器行高亮（ux.md §5.4）；掛載時即同步第 1 步的行號，播完保留最後一步的行高亮
- 進關即顯示第 1 步、可播放；Run／Submit 結果不影響 MemoryView（ux.md §5.5）

### 7.6 星等計算（前端）

- 星等由前端計算：後端無狀態，不知道提示用量與送出次數；後端不回傳任何星等欄位
- 規則照 ux.md §6.1，三顆星各自判定：
  - ★ 通過：Submit 回 `passed = true`
  - ★ 沒看提示：通過時本次遊玩 `hintsRevealed = 0`
  - ★ 一次就過：通過時本次遊玩 `failedSubmits = 0`
- 計入失敗的 Submit：`status` 為 `compile_error`、`test_failed`、`timeout`、`memory_limit`、`output_limit`
- 不計入：`503 JUDGE_BUSY`、`503 JUDGE_UNAVAILABLE`、其他 4xx／5xx、網路錯誤
- 三顆星分別取歷史聯集，不會被較低成績覆蓋
