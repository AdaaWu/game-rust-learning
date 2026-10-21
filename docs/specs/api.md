# Rust Quest API 規格

> 狀態：v1.5（與其他四份規格同步升版，API 契約無異動；v1.4 `LevelVisual` 新增 `timeline`；決策見 decisions.md §15 ⑬）
> Base URL：`http://127.0.0.1:3000`（開發時前端經 Vite proxy 轉送 `/api`）
> 相關文件：`docs/specs/architecture.md`、`docs/specs/ux.md`（`GridDef`、`MemorySnapshot` 的來源）、`docs/specs/decisions.md`

---

## 0. 通用約定

- **格式**：request／response 一律 `application/json; charset=utf-8`
- **命名**：JSON 欄位一律 `snake_case`
- **無認證**：MVP 無登入、無 session，後端無狀態
- **不回傳的欄位**：任何端點都**不會**出現 `solution`、`tests` 欄位
- **Nullable**：文件中標 `| null` 的欄位一定會出現在 JSON 中，值可能是 `null`（不省略 key）
- **Body 上限**：整個 request body 128 KiB；`code` 欄位 64 KiB（UTF-8 位元組）
- **判題結果 vs. 錯誤**：使用者程式碼編譯失敗、測試失敗、超時都是**正常的判題結果**，回 `200`；只有請求本身有問題或服務無法判題時才回 4xx／5xx

### 0.1 統一錯誤格式

```json
{
  "error": {
    "code": "LEVEL_NOT_FOUND",
    "message": "找不到關卡：99-unknown"
  }
}
```

- `code`：機器判讀用，前端以此分支處理；**不要**依 `message` 判斷
- `message`：給人看的繁體中文說明；前端不直接顯示，玩家看到的文案以 ux.md §3.3 為準
- 不提供 `details`：前端分支只需要 `code`，等候秒數走 `Retry-After` header

### 0.2 錯誤碼總表

| HTTP | `error.code` | 觸發條件 | 前端處理（ux.md §3.1、§3.3） |
|---|---|---|---|
| 400 | `INVALID_REQUEST` | body 不是合法 JSON、`Content-Type` 不是 `application/json`、缺 `code`、`code` 非字串或為空字串 | `network_error` 卡片；開發模式 console.error |
| 404 | `LEVEL_NOT_FOUND` | `:id` 不存在 | `GET /api/levels/:id`：顯示「找不到這關」頁；run／submit：`network_error` 卡片 |
| 413 | `PAYLOAD_TOO_LARGE` | body 超過 128 KiB 或 `code` 超過 64 KiB | `payload_too_large` 卡片「程式碼太長了」 |
| 503 | `JUDGE_BUSY` | 判題排隊已滿，或排隊等候超時 | `server_busy`：依 `Retry-After` 倒數，Run 與 Submit 一起 disabled |
| 503 | `JUDGE_UNAVAILABLE` | Docker daemon 無法連線、runner 映像不存在、容器啟動失敗或啟動超過 10 秒 | `judge_unavailable` 卡片，不倒數 |
| 500 | `INTERNAL_ERROR` | 未預期錯誤 | `network_error` 卡片 |

- 只有 `JUDGE_BUSY` 附 `Retry-After: 2`（整數秒）；`JUDGE_UNAVAILABLE` 不附
- 檢查順序（多個條件同時成立時回先遇到的）：body 超過 128 KiB（413）→ JSON 格式與 `code` 欄位（400）→ `code` 超過 64 KiB（413）→ 關卡存在（404）→ 排隊名額與執行 permit（`JUDGE_BUSY`）→ Docker 可用性（`JUDGE_UNAVAILABLE`）
  - 例：關卡不存在且 body 無效回 400；滿載且映像不存在回 `JUDGE_BUSY`
- 前端分類只看 HTTP 狀態碼與 `error.code`：413 一律歸 `payload_too_large`（不讀 body）；503 依 `error.code` 分 `server_busy`／`judge_unavailable`；`Retry-After` 只接受整數秒（ux.md §3.3）
- 不存在的路徑與錯誤的 HTTP 方法沿用 axum 預設（404／405、空 body），不屬於本契約，前端不會主動觸發
- `429` 保留給之後的 per-client rate limit，MVP 不使用（理由見 architecture.md §3.4）

### 0.3 共用型別

**`Diagnostic`**

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
  "rendered": "error[E0382]: borrow of moved value: `s`\n --> /tmp/main.rs:5:20\n  |\n...（rustc 人類可讀全文）\n"
}
```

- `code`：`string | null`；rustc 錯誤碼（`E0382`）或 lint 名（`unused_variables`）
- `message`：`string`
- `line`、`column`：`integer | null`；rustc primary span 的起點，1-based，對應**使用者程式碼**
- `end_line`、`end_column`：`integer | null`；primary span 的終點，`end_column` 為**不含**的結束欄位（與 Monaco `endColumn` 同語意）
- 四個位置欄位在以下情況為 `null`：錯誤落在隱藏測試區段、無位置資訊（例如連結器錯誤）
- 兩種 `null` 的區分（前端依此顯示「隱藏測試」或「無位置」，ux.md §3.4）
  - `line = null` 且 `rendered = null`：隱藏測試區段（只有這種情況 `rendered` 會是 null）
  - `line = null` 且 `rendered` 有值：無位置資訊
- 例外：span 從使用者程式碼跨進隱藏測試區段時，`end_line` 截在使用者程式碼最後一行、`end_column` 為 `null`（前端標到行尾）
- `level`：`"error" | "warning" | "note" | "help"`
- `explanation_url`：`string | null`；僅 `E\d{4}` 格式的 code 有值
- `rendered`：`string | null`；rustc JSON 的 `rendered` 欄位（含程式碼片段、`help:` 建議的人類可讀原文）
  - 非 JSON 行（例如連結器錯誤）為該行原文
  - primary span 落在隱藏測試區段時為 `null`（原文含隱藏區段行號）
- warning 一律保留回傳（關卡可能以警告當線索）
  - 例外：Submit 時位於隱藏測試區段的 warning 不回傳（玩家無法修正）；隱藏區段的 error 照常回傳，位置欄位為 `null`

**`CompileResult`**

```json
{
  "success": false,
  "timed_out": false,
  "diagnostics": []
}
```

- `success`：`boolean`；rustc exit 0 時為 `true`
- `timed_out`：`boolean`；編譯超過 15 秒被終止時為 `true`（此時 `status = "timeout"`、`execution = null`）
- `diagnostics`：`Diagnostic[]`，最多 50 筆；編譯成功時仍可能有 warning
- 編譯階段 rustc 被 OOM 終止：`status = "memory_limit"`、`success = false`、`timed_out = false`、`diagnostics` 為終止前已解析的部分（通常為空）、`execution = null`；Submit 的 `tests` 全部 `not_run`

**`ExecutionResult`**

```json
{
  "exit_code": 0,
  "stdout": "Hello, Ferris!\n",
  "stderr": "",
  "timed_out": false,
  "stdout_truncated": false,
  "stderr_truncated": false
}
```

- `exit_code`：`integer | null`；程式被終止時一律為 `null`，包含 host 端 kill（timeout、output_limit）與 OOM（容器回報的 137 不原樣回傳）
- `stdout`、`stderr`：`string`；各自上限 64 KiB，非 UTF-8 位元組以 `U+FFFD` 取代
  - **超時、OOM、輸出超限時仍回傳已截取的內容**，不會是空字串
- `timed_out`：`boolean`
- `stdout_truncated`、`stderr_truncated`：`boolean`；超過 64 KiB 時為 `true`，內容保留前 64 KiB（切在 UTF-8 字元中間時往前退），最後一行可能不完整
- 編譯失敗或編譯超時時，回應中的 `execution` 整個為 `null`

**`TestResult`**

```json
{
  "name": "ferris_reaches_exit",
  "status": "failed",
  "message": "assertion `left == right` failed: Ferris 沒有停在終點\n  left: (0, 1)\n right: (4, 1)"
}
```

- `name`：`string`；已去除 `__quest_hidden_tests::` 前綴
- `status`：`"passed" | "failed" | "not_run"`
  - `not_run`：編譯失敗，或執行中途因超時、OOM、輸出超限被終止，這個測試沒有產生結果
- `message`：`string | null`；`failed` 時為 panic 訊息，`passed`、`not_run` 時為 `null`
  - 已去除 `thread '...' (<thread id>) panicked at <路徑>:<行>:<欄>:` 這一行與 `note: run with RUST_BACKTRACE` 提示行
- 單一測試不另設 `passed` 布林；頂層的 `passed` 見 §4

**`JudgeStatus`**（Run 與 Submit 共用）

- `ok`：Run＝exit 0；Submit＝所有測試通過
- `compile_error`：編譯失敗
- `runtime_panic`：**僅 Run**；exit 非 0（panic 為 exit 101，`std::process::exit(n)` 非 0 也歸此類）
- `test_failed`：**僅 Submit**；至少一個測試失敗
- `timeout`：編譯超過 15 秒或執行超過 5 秒；以 `compile.timed_out`、`execution.timed_out` 區分
- `memory_limit`：超過 256 MB 被 OOM 終止；編譯或執行階段都可能發生（以 `execution` 是否為 null 區分）
- `output_limit`：stdout 或 stderr 超過 64 KiB 被終止
  - Run 與 Submit 共用同一套判定，但 Submit 時 libtest 預設捕捉測試內的輸出，實務上極少觸發；BDD 情境只針對 Run
- 判定優先順序：`timeout` → `memory_limit` → `compile_error` → `output_limit` → `runtime_panic`／`test_failed` → `ok`（細節見 architecture.md §3.6）
- 程式 abort（例：stack overflow，exit 134）：Run 為 `runtime_panic`；Submit 為 `test_failed`，已完成的測試照實、正在跑與之後的測試為 `not_run`、沒有任何 `failed`；`execution.exit_code = 134`，`execution.stderr` 含 `has overflowed its stack` 訊息（architecture.md §3.6）
- 七種值都是正式狀態，前端各有對應畫面（ux.md §3.3）

**`LevelKind`**：`"build" | "fix-compile" | "debug"`

**`LevelVisual`**：`"grid" | "memory" | "timeline" | "none"`

- `timeline`：Run 的 stdout 含 `@tl|` 事件行，前端畫並行時間軸（事件格式見 architecture.md §3.11，畫法見 ux.md §11）；沒有專屬資料欄位

**`GridDef`**（來源：`level.toml` 的 `[grid]`，格式以 ux.md §4.2 為準）

```json
{
  "width": 5,
  "height": 3,
  "start": [0, 0],
  "goal": [4, 1],
  "walls": [[2, 0], [2, 1]],
  "gems": [],
  "doors": []
}
```

- 座標一律 `[x, y]`，`x` 為欄、`y` 為列，從 0 起算
- `walls`、`gems`、`doors`：`level.toml` 省略時回傳空陣列，不回 `null`
- 後端啟動時已依 ux.md §4.2 驗證，前端仍保留防呆

**`MemorySnapshot`**（來源：`level.toml` 的 `[[memory_snapshots]]`，型別以 ux.md §5.1 為準）

```json
{
  "line": 3,
  "note": "擁有權移動到 count_items 的參數，report 的 bag 失效",
  "frames": [
    {
      "id": "report",
      "fn": "report",
      "vars": [
        { "id": "report.bag", "name": "bag", "type": "Vec<String>", "value": null, "state": "moved" }
      ]
    },
    {
      "id": "count_items",
      "fn": "count_items",
      "vars": [
        { "id": "count_items.bag", "name": "bag", "type": "Vec<String>", "value": "len=3 cap=3", "state": "live" }
      ]
    }
  ],
  "heap": [
    { "id": "h1", "type": "[String; 3]", "value": "珍珠 | 貝殼 | 海星", "state": "live" }
  ],
  "pointers": [
    { "from": "count_items.bag", "to": "h1", "kind": "own" }
  ],
  "error": null
}
```

錯誤步驟範例（只列 `error`）：

```json
{ "error": { "code": "E0382", "target": "report.bag" } }
```

- 陣列索引即步數（第 0 筆是第 1 步），不另帶 `step`
- `frames`、`heap`、`pointers`、`frames[].vars`：一定是陣列；TOML 省略時回 `[]`
- `vars[].value`：`string | null`；TOML 省略時回 `null`
- `vars[].state`：`"live" | "moved" | "dropped"`；`heap[].state`：`"live" | "freed"`
- `pointers[].kind`：`"own" | "shared" | "mut"`；`from` 為變數 `id`，`to` 為變數或 heap 區塊的 `id`
- `error`：`{ code, target } | null`；TOML 省略時回 `null`
  - `code`：`string | null`，rustc 錯誤碼（`E\d{4}`）；執行期錯誤為 `null`
  - `target`：同一步存在的變數或 heap 區塊 `id`

---

## 1. GET /api/levels

取得章節與關卡列表（不含程式碼、測試、解答）。

### Request

- 無參數

### Response `200`

```json
{
  "chapters": [
    { "chapter": 1, "title": "沙灘甦醒" },
    { "chapter": 3, "title": "所有權之海" }
  ],
  "levels": [
    {
      "id": "01-hello-ferris",
      "chapter": 1,
      "order": 1,
      "title": "第一步",
      "kind": "build",
      "visual": "grid"
    },
    {
      "id": "02-mut-matters",
      "chapter": 1,
      "order": 2,
      "title": "黏住的腳",
      "kind": "fix-compile",
      "visual": "grid"
    },
    {
      "id": "09-move-the-gem",
      "chapter": 3,
      "order": 9,
      "title": "交出去的背包",
      "kind": "fix-compile",
      "visual": "memory"
    }
  ]
}
```

- `chapters`：`ChapterSummary[]`，來源 `content/chapters.toml`（architecture.md §2.2）
  - 只列出至少有一關已載入的章節，依 `chapter` 升冪
  - `chapter`：整數，與關卡的 `chapter` 對應；`title`：章節名稱，前端顯示為「第 N 章 title」
  - 範例只列其中兩章；實際回傳 `chapters.toml` 中所有至少有一關已載入的章節
- `levels` 排序：`chapter` 升冪，再依 `order` 升冪（後端排好，前端不需再排）
- `chapter`、`order`：整數；`order` 沿用課程正式編號（全域遞增），只載入部分關卡時編號會跳號
- 解鎖規則由前端依此順序計算（ux.md §6.3）

### 錯誤

- 無業務錯誤；僅可能 `500 INTERNAL_ERROR`

---

## 2. GET /api/levels/:id

取得單一關卡詳細資料。

### Request

- Path：`id`（`string`），例：`01-hello-ferris`

### Response `200`

`visual = "grid"` 範例：

```json
{
  "id": "01-hello-ferris",
  "chapter": 1,
  "order": 1,
  "title": "第一步",
  "kind": "build",
  "visual": "grid",
  "story": "我在沙灘上醒來，殼上還沾著海草。\n……",
  "concept": "## 函式與回傳值\n\n- `fn route() -> Vec<String>`：`->` 後面是回傳型別\n……",
  "hints": [
    "出口在第 2 列最右邊，中間有一排岩石擋住",
    "先往下走到最底列，再一路往右，最後往上一格",
    "完整路線：MOVE DOWN 兩次、MOVE RIGHT 四次、MOVE UP 一次"
  ],
  "starter": "// 回傳 Ferris 要執行的指令\nfn route() -> Vec<String> {\n    ……\n}\n",
  "grid": {
    "width": 5,
    "height": 3,
    "start": [0, 0],
    "goal": [4, 1],
    "walls": [[2, 0], [2, 1]],
    "gems": [],
    "doors": []
  },
  "memory_snapshots": null
}
```

`visual = "memory"` 範例（只列與上例不同的欄位）：

```json
{
  "id": "09-move-the-gem",
  "visual": "memory",
  "grid": null,
  "memory_snapshots": [
    {
      "line": 8,
      "note": "bag 擁有 heap 上的三個字串",
      "frames": [
        {
          "id": "report",
          "fn": "report",
          "vars": [
            { "id": "report.bag", "name": "bag", "type": "Vec<String>", "value": "len=3 cap=3", "state": "live" }
          ]
        }
      ],
      "heap": [
        { "id": "h1", "type": "[String; 3]", "value": "珍珠 | 貝殼 | 海星", "state": "live" }
      ],
      "pointers": [
        { "from": "report.bag", "to": "h1", "kind": "own" }
      ],
      "error": null
    },
    {
      "line": 3,
      "note": "擁有權移動到 count_items 的參數，report 的 bag 失效",
      "frames": [
        {
          "id": "report",
          "fn": "report",
          "vars": [
            { "id": "report.bag", "name": "bag", "type": "Vec<String>", "value": null, "state": "moved" }
          ]
        },
        {
          "id": "count_items",
          "fn": "count_items",
          "vars": [
            { "id": "count_items.bag", "name": "bag", "type": "Vec<String>", "value": "len=3 cap=3", "state": "live" }
          ]
        }
      ],
      "heap": [
        { "id": "h1", "type": "[String; 3]", "value": "珍珠 | 貝殼 | 海星", "state": "live" }
      ],
      "pointers": [
        { "from": "count_items.bag", "to": "h1", "kind": "own" }
      ],
      "error": null
    }
  ]
}
```

- `story`、`concept`：Markdown 字串，前端負責渲染（需做 XSS 防護，例如 DOMPurify）
- `hints`：`string[]`，全部一次給；逐條揭露由前端控制；可為空陣列（`level.toml` 省略時）
- `starter`：起始程式碼；前端「重設」按鈕用此值
- `grid`：`GridDef | null`；`visual = "grid"` 時必有值，其他為 `null`（§0.3）
- `memory_snapshots`：`MemorySnapshot[] | null`；`visual = "memory"` 時必有值，其他為 `null`（§0.3）
- `visual = "timeline"`：不新增欄位，`grid`、`memory_snapshots` 皆為 `null`
  - `starter` 尾端已含時間軸小幫手 `mod timeline`（loader 啟動時把 `content/shared/timeline.rs` 接在 `starter.rs` 之後，architecture.md §2.2）；「重設」取回的也是含小幫手的版本
  - 範例（只列與 grid 範例不同的欄位，`starter` 節錄）：

```json
{
  "id": "33-boss-great-migration",
  "visual": "timeline",
  "starter": "use std::sync::mpsc;\n……\nfn main() {\n    ……\n}\n\n// ===== 時間軸小幫手：把事件印給右邊的時間軸看，不改變程式行為（不用改） =====\nmod timeline {\n……\n}\n",
  "grid": null,
  "memory_snapshots": null
}
```
- **絕不包含** `solution`、`tests`

### 錯誤

- `404 LEVEL_NOT_FOUND`

```json
{
  "error": {
    "code": "LEVEL_NOT_FOUND",
    "message": "找不到關卡：99-unknown"
  }
}
```

---

## 3. POST /api/levels/:id/run

只編譯並執行使用者的 `main`，回傳輸出。不跑隱藏測試。

### Request

```json
{
  "code": "fn main() {\n    println!(\"MOVE DOWN\");\n    println!(\"MOVE DOWN\");\n}\n"
}
```

- `code`：`string`，必填，不可為空字串，上限 64 KiB

### Response `200`

**`status`**：`JudgeStatus`，本端點可能值為 `ok`、`compile_error`、`runtime_panic`、`timeout`、`memory_limit`、`output_limit`

**範例：成功**（`01-hello-ferris` 的解答）

```json
{
  "status": "ok",
  "compile": {
    "success": true,
    "timed_out": false,
    "diagnostics": []
  },
  "execution": {
    "exit_code": 0,
    "stdout": "MOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE UP\n",
    "stderr": "",
    "timed_out": false,
    "stdout_truncated": false,
    "stderr_truncated": false
  }
}
```

**範例：編譯錯誤**（`09-move-the-gem` 的 starter）

```json
{
  "status": "compile_error",
  "compile": {
    "success": false,
    "timed_out": false,
    "diagnostics": [
      {
        "code": "E0382",
        "message": "borrow of moved value: `bag`",
        "line": 10,
        "column": 17,
        "end_line": 10,
        "end_column": 20,
        "level": "error",
        "explanation_url": "https://doc.rust-lang.org/error_codes/E0382.html",
        "rendered": "error[E0382]: borrow of moved value: `bag`\n  --> /tmp/main.rs:10:17\n   |\n 8 |     let bag = vec![String::from(\"珍珠\"), String::from(\"貝殼\"), String::from(\"海星\")];\n   |         --- move occurs because `bag` has type `Vec<String>`, which does not implement the `Copy` trait\n 9 |     let n = count_items(bag);\n   |                         --- value moved here\n10 |     let first = bag[0].clone();\n   |                 ^^^ value borrowed here after move\n...\n"
      }
    ]
  },
  "execution": null
}
```

**範例：執行超時**

```json
{
  "status": "timeout",
  "compile": {
    "success": true,
    "timed_out": false,
    "diagnostics": []
  },
  "execution": {
    "exit_code": null,
    "stdout": "MOVE DOWN\nMOVE DOWN\nMOVE DOWN\n",
    "stderr": "",
    "timed_out": true,
    "stdout_truncated": false,
    "stderr_truncated": false
  }
}
```

**範例：panic**（`28-refcell-panic` 的 starter）

```json
{
  "status": "runtime_panic",
  "compile": {
    "success": true,
    "timed_out": false,
    "diagnostics": []
  },
  "execution": {
    "exit_code": 101,
    "stdout": "",
    "stderr": "\nthread 'main' (12) panicked at /tmp/main.rs:15:11:\nRefCell already borrowed\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n",
    "timed_out": false,
    "stdout_truncated": false,
    "stderr_truncated": false
  }
}
```

- `visual = "grid"` 的關卡：前端從 `execution.stdout` 解析指令驅動 GridWorld（語法見 ux.md §4.3）
  - 後端 stdout 上限 64 KiB，超過回 `stdout_truncated = true`、`status = "output_limit"`；前端另設指令步數上限 1,000 步
  - `status` 為 `timeout`、`memory_limit`、`output_limit` 時 `execution.stdout` 仍含終止前已輸出的指令，可播放到中斷點
- Run 不回傳 `stdout` 以外的 grid 判定結果；通關只看 Submit
- `visual = "timeline"` 的關卡：前端從 `execution.stdout` 解析 `@tl|` 開頭的事件行驅動 TimelineView（格式見 architecture.md §3.11）
  - 後端不解析、不過濾事件行；上限、截斷、`timeout` 時保留已輸出內容的規則與 grid 相同
  - 範例（33 starter 死鎖，節錄；行號為 starter 實際行號，事件交錯每次 Run 不同）：`status = "timeout"`、`execution.stdout` 為 `"@tl|start|36|分身 0|\n@tl|lock_wait|39|分身 0|窄橋 0\n……\n@tl|recv_wait|58|main|回報\n……\n@tl|lock_wait|39|分身 0|窄橋 1\n@tl|lock_wait|39|分身 1|窄橋 0\n"`
  - Submit 時小幫手不印事件（`cfg!(test)`），Submit 的 stdout 不含事件行
- panic 位置格式為 `/tmp/main.rs:<行>:<欄>`，行號與編輯器一致（Run 只編譯使用者程式碼）
- 編譯超時範例：`status = "timeout"`、`compile.timed_out = true`、`compile.success = false`、`execution = null`
- 沒有 `main` 的程式碼會得到 rustc `E0601` 的 `compile_error`

### 錯誤

- `400 INVALID_REQUEST`、`404 LEVEL_NOT_FOUND`、`413 PAYLOAD_TOO_LARGE`
- `503 JUDGE_BUSY`（header `Retry-After: 2`）

```json
{
  "error": {
    "code": "JUDGE_BUSY",
    "message": "目前判題人數過多，請稍後再試"
  }
}
```

- `503 JUDGE_UNAVAILABLE`（不附 `Retry-After`）

```json
{
  "error": {
    "code": "JUDGE_UNAVAILABLE",
    "message": "判題服務未就緒：找不到 runner 映像"
  }
}
```

  - 四種原因共用同一個 `code`，只差在 `message`：Docker daemon 無法連線（含 docker CLI 無法執行）、runner 映像不存在、`docker run` 失敗、容器啟動超過 10 秒；回應不帶原因代碼（前端顯示同一句說明，ux.md §3.3）

---

## 4. POST /api/levels/:id/submit

把使用者程式碼與隱藏測試合併後編譯並執行，回傳每個測試的結果（合併順序見 architecture.md §3.7）。

### Request

```json
{
  "code": "use std::cell::RefCell;\nuse std::rc::Rc;\n……\n"
}
```

- 規則同 `/run`

### Response `200`

**`status`**：`JudgeStatus`，本端點可能值為 `ok`、`compile_error`、`test_failed`、`timeout`、`memory_limit`、`output_limit`

- 不會出現 `runtime_panic`：測試中的 panic 歸類為 `test_failed`

**範例：全部通過**（`28-refcell-panic` 的解答）

```json
{
  "status": "ok",
  "passed": true,
  "compile": {
    "success": true,
    "timed_out": false,
    "diagnostics": []
  },
  "execution": {
    "exit_code": 0,
    "stdout": "\nrunning 3 tests\ntest __quest_hidden_tests::deposit_new_item ... ok\ntest __quest_hidden_tests::duplicate_is_skipped ... ok\ntest __quest_hidden_tests::gull_shares_the_chest ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n",
    "stderr": "",
    "timed_out": false,
    "stdout_truncated": false,
    "stderr_truncated": false
  },
  "tests": [
    { "name": "deposit_new_item", "status": "passed", "message": null },
    { "name": "duplicate_is_skipped", "status": "passed", "message": null },
    { "name": "gull_shares_the_chest", "status": "passed", "message": null }
  ]
}
```

**範例：部分失敗，含 warning**（`31-shared-counter` 的 starter，即 GET 回傳的 `starter`，尾端含 timeline 小幫手）

```json
{
  "status": "test_failed",
  "passed": false,
  "compile": {
    "success": true,
    "timed_out": false,
    "diagnostics": [
      {
        "code": "unused_assignments",
        "message": "value assigned to `sum` is never read",
        "line": 8,
        "column": 5,
        "end_line": 8,
        "end_column": 16,
        "level": "warning",
        "explanation_url": null,
        "rendered": "warning: value assigned to `sum` is never read\n --> /tmp/main.rs:8:5\n  |\n8 |     sum += gems;\n  |     ^^^^^^^^^^^\n  |\n  = help: maybe it is overwritten before being read?\n...\n"
      }
    ]
  },
  "execution": {
    "exit_code": 101,
    "stdout": "\nrunning 5 tests\ntest __quest_hidden_tests::hundred_crabs ... FAILED\ntest __quest_hidden_tests::hundred_crabs_share_one_counter ... FAILED\ntest __quest_hidden_tests::no_crab ... ok\ntest __quest_hidden_tests::one_crab_one_haul ... FAILED\ntest __quest_hidden_tests::three_crabs ... FAILED\n\nfailures:\n...\n",
    "stderr": "",
    "timed_out": false,
    "stdout_truncated": false,
    "stderr_truncated": false
  },
  "tests": [
    {
      "name": "one_crab_one_haul",
      "status": "failed",
      "message": "assertion `left == right` failed: 螃蟹交了 5 顆寶石，計數器卻沒有變\n  left: 10\n right: 15"
    },
    {
      "name": "hundred_crabs_share_one_counter",
      "status": "failed",
      "message": "assertion `left == right` failed: 一百隻螃蟹同時交寶石，有寶石在途中消失了\n  left: 0\n right: 5050"
    },
    {
      "name": "three_crabs",
      "status": "failed",
      "message": "assertion `left == right` failed: 三隻螃蟹的寶石沒有全部算進去\n  left: 0\n right: 15"
    },
    {
      "name": "hundred_crabs",
      "status": "failed",
      "message": "assertion `left == right` failed: 有寶石在途中消失了\n  left: 0\n right: 5050"
    },
    { "name": "no_crab", "status": "passed", "message": null }
  ]
}
```

**範例：編譯錯誤（錯誤落在隱藏測試區段）**（`05-compass-enum`：玩家以為 `route` 沒用到 `delta` 就把整個函式刪掉後送出）

- 玩家程式碼共 29 行，自己的 `main` 與 `route` 都不呼叫 `delta`；只有隱藏測試 `delta_matches_direction` 呼叫 4 次
- rustc 1.99.1（edition 2024，runner 映像內）實測回 4 筆 E0425，primary span 全部位於隱藏區段，因此四個位置欄位與 `rendered` 都是 `null`（隱藏區段的實際行號隨合併檔開頭與共用模擬器變動，規格不寫死）
- `aborting due to 4 previous errors` 與 `failure-note` 已依 architecture.md §3.7 過濾

```json
{
  "status": "compile_error",
  "passed": false,
  "compile": {
    "success": false,
    "timed_out": false,
    "diagnostics": [
      {
        "code": "E0425",
        "message": "cannot find function `delta` in this scope",
        "line": null,
        "column": null,
        "end_line": null,
        "end_column": null,
        "level": "error",
        "explanation_url": "https://doc.rust-lang.org/error_codes/E0425.html",
        "rendered": null
      },
      {
        "code": "E0425",
        "message": "cannot find function `delta` in this scope",
        "line": null,
        "column": null,
        "end_line": null,
        "end_column": null,
        "level": "error",
        "explanation_url": "https://doc.rust-lang.org/error_codes/E0425.html",
        "rendered": null
      },
      {
        "code": "E0425",
        "message": "cannot find function `delta` in this scope",
        "line": null,
        "column": null,
        "end_line": null,
        "end_column": null,
        "level": "error",
        "explanation_url": "https://doc.rust-lang.org/error_codes/E0425.html",
        "rendered": null
      },
      {
        "code": "E0425",
        "message": "cannot find function `delta` in this scope",
        "line": null,
        "column": null,
        "end_line": null,
        "end_column": null,
        "level": "error",
        "explanation_url": "https://doc.rust-lang.org/error_codes/E0425.html",
        "rendered": null
      }
    ]
  },
  "execution": null,
  "tests": [
    { "name": "command_text", "status": "not_run", "message": null },
    { "name": "delta_matches_direction", "status": "not_run", "message": null },
    { "name": "route_reaches_temple", "status": "not_run", "message": null }
  ]
}
```

**範例：執行超時**

```json
{
  "status": "timeout",
  "passed": false,
  "compile": {
    "success": true,
    "timed_out": false,
    "diagnostics": []
  },
  "execution": {
    "exit_code": null,
    "stdout": "\nrunning 3 tests\ntest __quest_hidden_tests::deposit_new_item ... ok\ntest __quest_hidden_tests::duplicate_is_skipped ... ok\ntest __quest_hidden_tests::gull_shares_the_chest ... ",
    "stderr": "",
    "timed_out": true,
    "stdout_truncated": false,
    "stderr_truncated": false
  },
  "tests": [
    { "name": "deposit_new_item", "status": "passed", "message": null },
    { "name": "duplicate_is_skipped", "status": "passed", "message": null },
    { "name": "gull_shares_the_chest", "status": "not_run", "message": null }
  ]
}
```

**欄位說明**

- `passed`：`boolean`；等同 `status == "ok"`，給前端快速判斷；只有 `tests` 每一筆都是 `passed` 時為 `true`
- 執行結束但結果不完整時一律 `test_failed`（timeout、memory_limit、output_limit 仍優先，architecture.md §3.6）
  - exit 0 但有測試沒有結果：缺的測試為 `not_run`
  - exit 非 0 但沒有 `failed` 的測試（例如測試 harness 崩潰）：沒有結果的測試為 `not_run`
- `tests`：`TestResult[]`；永遠列出該關**全部**隱藏測試，順序依 `tests.rs` 宣告順序
  - 編譯失敗、編譯超時：全部為 `not_run`
  - 執行中途被終止：已有結果的照實回傳，其餘（含終止當下正在跑的那個）為 `not_run`
  - 玩家自己寫的 `#[test]` 不執行、不列入
- 不回傳 `summary`：通過／失敗／未執行數由前端從 `tests` 計數；`tests.length` 即該關隱藏測試總數
- 星等由前端計算（規則見 ux.md §6.1）；後端不回傳星等相關欄位
- 測試**原始碼**不回傳；測試函式名稱與 panic 訊息會回傳

### 錯誤

- 同 `/run`：`400`、`404`、`413`、`503 JUDGE_BUSY`、`503 JUDGE_UNAVAILABLE`

---

## 5. GET /api/health

服務健康檢查。Docker 狀態快取 10 秒。

### Response `200`（正常）

```json
{
  "status": "ok",
  "judge": { "available": true, "reason": null }
}
```

### Response `200`（降級）

```json
{
  "status": "degraded",
  "judge": { "available": false, "reason": "runner_image_missing" }
}
```

- `status`：`"ok" | "degraded"`；`judge.available = false` 時為 `degraded`
  - `degraded`：關卡可瀏覽，但 run／submit 會回 `503 JUDGE_UNAVAILABLE`
- **降級時仍回 `200`**：服務本身活著，關卡瀏覽可用；監控請讀 `status` 欄位判斷
- `judge.reason`：`null | "docker_unreachable" | "runner_image_missing"`；`available = true` 時為 `null`
- 只回前端用得到的欄位；映像名、併發數、排隊數等不回傳（映像名是程式內常數 `rust-quest-runner:dev`，見 architecture.md §3.9）
- 前端進入關卡頁時呼叫一次，`degraded` 時停用 Run／Submit 並顯示說明（ux.md §3.3）

---

## 6. 前端 TypeScript 型別（對照用）

```ts
type LevelKind = 'build' | 'fix-compile' | 'debug'
type LevelVisual = 'grid' | 'memory' | 'timeline' | 'none'

interface ChapterSummary { chapter: number; title: string }

interface LevelSummary {
  id: string; chapter: number; order: number
  title: string; kind: LevelKind; visual: LevelVisual
}

interface LevelListResponse { chapters: ChapterSummary[]; levels: LevelSummary[] }

interface LevelDetail extends LevelSummary {
  story: string; concept: string; hints: string[]
  starter: string
  grid: GridDef | null
  memory_snapshots: MemorySnapshot[] | null
}

// timeline 沒有 API 欄位；事件型別 TimelineEvent 是前端解析 stdout 的內部型別（ux.md §11.4），不屬於本契約

// 與 ux.md §4.2 [grid] 一致；座標 [x, y]
type Point = [number, number]

interface GridDef {
  width: number; height: number
  start: Point; goal: Point
  walls: Point[]; gems: Point[]; doors: Point[]
}

// 與 ux.md §5.1 一致
interface MemorySnapshot {
  line: number
  note: string
  frames: StackFrame[]
  heap: HeapBlock[]
  pointers: Pointer[]
  error: SnapshotError | null
}

interface SnapshotError { code: string | null; target: string }

interface StackFrame { id: string; fn: string; vars: StackVar[] }

interface StackVar {
  id: string; name: string; type: string
  value: string | null
  state: 'live' | 'moved' | 'dropped'
}

interface HeapBlock { id: string; type: string; value: string; state: 'live' | 'freed' }

interface Pointer { from: string; to: string; kind: 'own' | 'shared' | 'mut' }

interface Diagnostic {
  code: string | null; message: string
  line: number | null; column: number | null
  end_line: number | null; end_column: number | null
  level: 'error' | 'warning' | 'note' | 'help'
  explanation_url: string | null
  rendered: string | null
}

interface CompileResult {
  success: boolean; timed_out: boolean; diagnostics: Diagnostic[]
}

interface ExecutionResult {
  exit_code: number | null; stdout: string; stderr: string
  timed_out: boolean; stdout_truncated: boolean; stderr_truncated: boolean
}

type JudgeStatus =
  | 'ok' | 'compile_error' | 'runtime_panic' | 'test_failed'
  | 'timeout' | 'memory_limit' | 'output_limit'

interface RunResponse {
  status: Exclude<JudgeStatus, 'test_failed'>
  compile: CompileResult
  execution: ExecutionResult | null
}

type TestStatus = 'passed' | 'failed' | 'not_run'

interface TestResult { name: string; status: TestStatus; message: string | null }

interface SubmitResponse {
  status: Exclude<JudgeStatus, 'runtime_panic'>
  passed: boolean
  compile: CompileResult
  execution: ExecutionResult | null
  tests: TestResult[]
}

interface HealthResponse {
  status: 'ok' | 'degraded'
  judge: { available: boolean; reason: 'docker_unreachable' | 'runner_image_missing' | null }
}

interface ApiError {
  error: { code: string; message: string }
}
```
