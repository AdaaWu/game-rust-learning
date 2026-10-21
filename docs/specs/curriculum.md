# Rust 闖關遊戲：課程與關卡設計規格（SDD 課程篇）

> 狀態：v1.3（定稿；收尾第二輪驗收次要問題與工程師疑義，決策見 decisions.md §13–§15）
> 範圍：課程結構、關卡內容、關卡資料格式中的課程欄位。前後端實作、判題沙箱不在本文件範圍。
> 相關文件：`docs/specs/ux.md`（`[grid]`、指令語法、`MemorySnapshot`、星等）、`docs/specs/architecture.md`（判題合併流程）、`docs/specs/decisions.md`

## 0. 結論摘要

- **8 章、33 關**，故事主線是螃蟹 Ferris 從沙灘甦醒，一路冒險到「群蟹大遷徙」
- **難度採螺旋式**：每個核心觀念先用 `build` 學會，再用 `fix-compile` 讀懂編譯器怎麼擋，最後用 `debug` 在執行期抓錯
- **MVP 首批 8 關**：01、02、05、09、11、19、28、31
  - 三種 kind 都有：build（01、05）、fix-compile（02、09、11）、debug（19、28、31）
  - 三種 visual 都有：grid（01、02、05、19）、memory（09、11、28）、none（31）
- **memory_snapshots** 採 ux.md 的 `MemorySnapshot` 格式：每一步一張快照，含 stack frames、heap 區塊、指標（own／shared／mut）
- **grid 關卡**：地圖寫在 `level.toml` 的 `[grid]`（ux.md §4.2），判題用 `content/shared/grid_sim.rs` 共用模擬器

---

## 1. 共用約定

### 1.1 Ferris 指令集（stdout 一行一個指令）

指令語法與解析規則以 ux.md §4.3 為準，本節只列課程會用到的部分。

- `MOVE UP` / `MOVE DOWN` / `MOVE LEFT` / `MOVE RIGHT`：移動一格
- `PICK`：撿起腳下的寶石
- `OPEN UP` / `OPEN DOWN` / `OPEN LEFT` / `OPEN RIGHT`：打開該方向相鄰格的門
- 不是指令的行（例如含中文或小寫字母）只顯示在主控台，不影響 Ferris
- 撞牆、撞門、走出地圖、無效指令、`PICK` 空格、`OPEN` 沒有門：立即停止
- 指令最多 1,000 步，超過視為失敗

### 1.2 地圖格式（`level.toml` 的 `[grid]`）

欄位與驗證規則以 ux.md §4.2 為準。

- 座標一律 `[x, y]`：`x` 為欄（左到右）、`y` 為列（上到下），從 0 起算
- `width`、`height`：2–12
- `start`、`goal`：必填
- `walls`、`gems`、`doors`：座標陣列，可省略
- 通關條件：撿完全部 `gems`，且最後停在 `goal`

### 1.3 grid 關卡的判定方式

- 玩家實作「回傳指令列表」的函式（例如 `fn route() -> Vec<String>`）
- `main` 只負責把指令逐行印出，前端依 stdout 與 `[grid]` 做動畫
- 判題時後端把共用模擬器 `content/shared/grid_sim.rs` 與依 `[grid]` 產生的 `level_grid()` 併入隱藏測試模組，`tests.rs` 直接呼叫 `level_grid()`、`simulate()`、`assert_cleared()`
- 判定只看測試，不看 stdout，避免玩家「印對字但邏輯錯」也過關

### 1.4 tests.rs 撰寫規則

- `tests.rs` 只寫**模組內容**：不寫 `#[cfg(test)]`、`mod tests {}`、`use super::*;`，外層由判題器包成 `mod __quest_hidden_tests`（見 §1.6）
- 只呼叫「關卡說明裡公開的函式」，不呼叫玩家可能改簽名的內部函式
- 斷言訊息用繁體中文，直接說明 Ferris 發生什麼事
- grid 關卡
  - 不寫 `level_grid()`：判題器依 `[grid]` 產生（§1.6）；**關卡地圖**（玩家在畫面上看到的那張）只寫在 `level.toml` 一處，`tests.rs` 不重寫
  - 通關測試呼叫 `assert_cleared(&level_grid(), &route())`
  - 不在 `tests.rs` 內另寫模擬器
  - 例外：測試需要**關卡地圖以外的額外地圖**時（例如 19 關測「沒有寶石的島」），可自寫輔助函式產生測試專用地圖（19 關的 `grid_from`）；這些地圖不在畫面上出現，不違反單一來源。產生的 `level_grid()` 沒用到也不影響編譯
  - 若測試地圖中有一張與關卡地圖相同（19 關的 `gem_down_right`），畫面與判題的一致性由內容測試「solution 的 Run stdout 套 `[grid]` 須通關」保證（architecture.md §6.2 ④）
- 測試輔助名稱避開判題器已用的名稱：`Grid`、`World`、`MAX_STEPS`、`simulate`、`assert_cleared`、`is_command_attempt`、`direction`、`invalid`、`level_grid`

### 1.5 共用模擬器 `content/shared/grid_sim.rs`

規則與 ux.md §4.3、§4.5 一致；前端 GridWorld 解析器改規則時兩邊一起改。

```rust
// grid 關卡共用模擬器，規則與 ux.md §4.3、§4.5 一致

const MAX_STEPS: usize = 1000;

struct Grid {
    width: i32,
    height: i32,
    start: (i32, i32),
    goal: (i32, i32),
    walls: Vec<(i32, i32)>,
    gems: Vec<(i32, i32)>,
    doors: Vec<(i32, i32)>,
}

struct World {
    pos: (i32, i32),
    gems_collected: usize,
}

// 整行只含大寫字母、底線、空白才算指令嘗試
fn is_command_attempt(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c == ' ')
}

fn direction(word: &str) -> Option<(i32, i32)> {
    match word {
        "UP" => Some((0, -1)),
        "DOWN" => Some((0, 1)),
        "LEFT" => Some((-1, 0)),
        "RIGHT" => Some((1, 0)),
        _ => None,
    }
}

fn invalid(step: usize, line: &str) -> String {
    format!("第 {} 步：看不懂 {}", step, line)
}

// 依指令模擬 Ferris，遇到第一個錯誤就停
fn simulate(grid: &Grid, cmds: &[String]) -> Result<World, String> {
    let mut pos = grid.start;
    let mut gems = grid.gems.clone();
    let mut closed = grid.doors.clone();
    let mut collected = 0;
    let mut step = 0;
    for raw in cmds {
        let line = raw.trim();
        if !is_command_attempt(line) {
            continue;
        }
        step += 1;
        if step > MAX_STEPS {
            return Err(format!("指令超過 {} 步上限", MAX_STEPS));
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["MOVE", dir] => {
                let (dx, dy) = direction(dir).ok_or_else(|| invalid(step, line))?;
                let next = (pos.0 + dx, pos.1 + dy);
                if next.0 < 0 || next.1 < 0 || next.0 >= grid.width || next.1 >= grid.height {
                    return Err(format!("第 {} 步：走出地圖", step));
                }
                if grid.walls.contains(&next) {
                    return Err(format!("第 {} 步：撞到牆 ({}, {})", step, next.0, next.1));
                }
                if closed.contains(&next) {
                    return Err(format!("第 {} 步：門還關著 ({}, {})", step, next.0, next.1));
                }
                pos = next;
            }
            ["PICK"] => {
                let idx = gems
                    .iter()
                    .position(|g| *g == pos)
                    .ok_or_else(|| format!("第 {} 步：這格沒有寶石", step))?;
                gems.remove(idx);
                collected += 1;
            }
            ["OPEN", dir] => {
                let (dx, dy) = direction(dir).ok_or_else(|| invalid(step, line))?;
                let target = (pos.0 + dx, pos.1 + dy);
                let idx = closed
                    .iter()
                    .position(|d| *d == target)
                    .ok_or_else(|| format!("第 {} 步：那邊沒有門", step))?;
                closed.remove(idx);
            }
            _ => return Err(invalid(step, line)),
        }
    }
    Ok(World { pos, gems_collected: collected })
}

// 通關條件：撿完全部寶石且停在終點
fn assert_cleared(grid: &Grid, cmds: &[String]) {
    let world = simulate(grid, cmds).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(world.gems_collected, grid.gems.len(), "還有寶石沒撿到");
    assert_eq!(world.pos, grid.goal, "Ferris 沒有停在終點");
}
```

- 項目不加 `pub`：與 `tests.rs` 同在 `__quest_hidden_tests` 模組內，直接可見
- 玩家程式若有同名項目（例如自己的 `Grid`），模組內定義會遮蔽 `use super::*` 引入的名稱，不衝突

### 1.6 判題合併順序（Submit）

詳細流程見 architecture.md §3.7；關卡作者只需知道組合順序。

```
① 玩家程式碼
  ↓
② 隱藏模組開頭
  ↓
③ 共用模擬器與地圖
  ↓
④ tests.rs
  ↓
⑤ 隱藏模組結尾
```

① 原樣放在檔案最前面，行號與編輯器一致
② 使用者程式碼不以換行結尾時補一個換行（已有則不補），再接五行：`#[cfg(test)]`、`#[allow(dead_code)]`、`#[allow(unused_imports)]`、`mod __quest_hidden_tests {`、`use super::*;`
③ 只有 `visual = "grid"` 的關卡併入 `content/shared/grid_sim.rs`，再接判題器依 `[grid]` 產生的單行 `fn level_grid() -> Grid`（格式見 architecture.md §3.7 ④；省略的 `walls`、`gems`、`doors` 產生 `vec![]`）
④ 該關的 `tests.rs` 全文（不以換行結尾時補一個換行）
⑤ `}`

- 測試名稱在 libtest 輸出為 `__quest_hidden_tests::<函式名>`，後端回傳前去掉前綴

### 1.7 章節名稱 `content/chapters.toml`

章節名稱的唯一來源；loader 啟動時載入，經 `GET /api/levels` 的 `chapters` 給前端（api.md §1、architecture.md §2.2）。

```toml
[[chapters]]
chapter = 1
title = "沙灘甦醒"

[[chapters]]
chapter = 2
title = "潮汐羅盤"

[[chapters]]
chapter = 3
title = "所有權之海"

[[chapters]]
chapter = 4
title = "生命週期遺跡"

[[chapters]]
chapter = 5
title = "漂流瓶"

[[chapters]]
chapter = 6
title = "工匠之島"

[[chapters]]
chapter = 7
title = "智慧指標洞窟"

[[chapters]]
chapter = 8
title = "群蟹大遷徙"
```

- `title` 與 §3 各章標題一致，不含括號內的主題說明
- 前端顯示為「第 N 章 title」；沒有關卡已載入的章節（MVP 的第 4、6 章）不會出現在 API 回應

### 1.8 `level.toml` 欄位與 story 寫法

- 必填：`id`（等於目錄名）、`chapter`、`order`、`title`、`kind`、`visual`、`story`、`concept`
- 選填：`hints`（省略即無提示，提示區不顯示）、`[grid]`（僅 grid）、`[[memory_snapshots]]`（僅 memory）
- 完整驗證規則見 architecture.md §2.2
- **`story` 就是 Ferris 對話泡泡的內容**（ux.md §1.4）
  - 用 Ferris 第一人稱（「我」）寫，3–4 行，每行一句
  - 可用行內 Markdown；不放程式碼區塊與標題（觀念說明放 `concept`）
  - 每章第一關的 `story` 兼任章節開場，交代新場景

---

## 2. 學習目標地圖

**一句話重點：每個觀念至少出現三次，依序是「會用」→「看懂編譯器拒絕」→「在執行期抓到錯」。**

### 2.1 觀念對應關卡

- **變數與不可變預設**：01、02、03、04
- **型別系統、struct、enum、match**：04、05、06、07、08
- **Ownership（所有權）**：每個值同時只有一個擁有者，擁有者離開作用域就釋放
  - 09、10、13、15
- **Borrowing（借用）**：同一時間「多個 `&T`」或「一個 `&mut T`」二選一
  - 10、11、12、13
- **Lifetimes（生命週期）**：編譯器追蹤「參考能活多久」，保證不會指向已釋放的記憶體
  - 14、15、16、17
- **Option / Result 錯誤處理**：沒有 null，可能失敗的事寫進型別
  - 18、19、20、21
- **Trait 與泛型**：22、23、24
- **Iterator 與 closure**：24、25、30
- **Smart pointer（Box / Rc / RefCell）**：26、27、28、29
- **並行安全（Send / Sync、Arc / Mutex、channel）**：30、31、32、33
- **與編譯器對話（讀 rustc 錯誤碼）**：02、04、07、09、11、12、14、15、23、26、30
- **執行期 debug（panic、overflow、死鎖、RefCell 雙重借用）**：17、19、21、25、28、29、31、33

### 2.2 為什麼這個順序能保證學會

```
① 語法與控制流程
  ↓
② 型別系統與 match
  ↓
③ 所有權與借用
  ↓
④ 生命週期
  ↓
⑤ Option / Result
  ↓
⑥ Trait、泛型、iterator
  ↓
⑦ Smart pointer
  ↓
⑧ 並行
```

- ① 先讓玩家能寫出會動的程式，建立成就感；`mut` 在第 2 關就出現，第一次體會「Rust 預設保守」
- ② enum + match 是後面 Option / Result 的基礎；窮舉檢查（E0004）讓玩家第一次感受「編譯器幫你想到漏掉的情況」
- ③ 所有權放在型別之後，因為要先懂 `String` 與 `Vec` 這類「擁有 heap 資料」的型別，move 才有意義
- ④ 生命週期是借用規則的延伸，必須在借用之後，否則 `'a` 只是背語法
- ⑤ 錯誤處理放在所有權之後，`?` 與 `Option<&T>` 才不會卡在借用問題
- ⑥ trait 與泛型需要 enum、借用都熟，iterator 的 closure 捕捉也牽涉借用
- ⑦ smart pointer 是「打破單一擁有者規則的合法出口」，玩家必須先被規則限制過，才懂 Rc / RefCell 的取捨
- ⑧ 並行是 Rust 精神的總驗收：Send / Sync 就是所有權與借用規則延伸到執行緒；`Arc<Mutex<T>>` 是 `Rc<RefCell<T>>` 的執行緒安全版，⑦ 的 28 關與 ⑧ 的 31 關刻意做成對照

---

## 3. 章節結構（8 章、33 關）

**一句話重點：前兩章以 build 為主，三、四章混入 fix-compile，五章以後以 debug 與 boss 為主。**

格式：`編號 slug｜標題｜kind｜visual｜教學觀念｜趣味點`

### 第 1 章 沙灘甦醒（基礎語法）

- `01 hello-ferris`｜第一步｜build｜grid｜fn、`Vec<String>`、`main` 印出指令｜第一次讓 Ferris 動起來 **[MVP]**
- `02 mut-matters`｜黏住的腳｜fix-compile｜grid｜不可變預設、`mut`（E0384、E0596）｜Ferris 被「不可變」黏在原地 **[MVP]**
- `03 loop-the-shore`｜海岸巡邏｜build｜grid｜`for`、range、`while`、`loop` 帶回傳值｜用迴圈走 Z 字形，用迴圈取代重複的指令
- `04 type-tides`｜潮汐表｜fix-compile｜grid｜型別推論、整數型別、`as` 轉型（E0308）｜潮汐高度是 `u8`，步數是 `i32`，對不上就出不了海

### 第 2 章 潮汐羅盤（型別系統）

- `05 compass-enum`｜神奇羅盤｜build｜grid｜`enum`、`match`、`#[derive]`｜用羅盤方向組出路線 **[MVP]**
- `06 ferris-struct`｜Ferris 的狀態欄｜build｜grid｜`struct`、`impl`、`&self` / `&mut self`｜Ferris 有了血量與背包
- `07 exhaustive-match`｜羅盤多了斜角｜fix-compile｜grid｜窮舉檢查（E0004）｜新增 `Diagonal` 方向後，所有 match 都被編譯器點名
- `08 pattern-treasure`｜寶石鑑定｜build｜grid｜解構、guard、帶資料的 enum（`Tile::Gem(u32)`）｜不同寶石不同分數，挑最高分路線

### 第 3 章 所有權之海（Ownership 與 Borrowing）

- `09 move-the-gem`｜交出去的背包｜fix-compile｜memory｜move 語意（E0382）｜背包交給別人後就不能再翻 **[MVP]**
- `10 borrow-dont-take`｜借看不給拿｜build｜memory｜`&T`、`&mut T` 參數｜海鷗只能看寶石，不能帶走
- `11 one-writer-rule`｜一寫多讀｜fix-compile｜memory｜借用規則（E0502）｜記住第一樣寶物時，背包不能被擴充 **[MVP]**
- `12 two-claws`｜兩隻鉗子｜fix-compile｜memory｜同時兩個 `&mut`（E0499）｜左右鉗同時抓同一顆寶石
- `13 slice-the-reef`｜切開珊瑚礁｜build｜memory｜`&str` vs `String`、slice、`Copy` vs `Clone`｜只切地圖的一段給隊友看

### 第 4 章 生命週期遺跡（Lifetimes）

- `14 missing-lifetime`｜古碑上的兩個名字｜fix-compile｜memory｜生命週期標註 `'a`（E0106）｜回傳較長的名字，編譯器問「它從哪來」
- `15 dangling-relic`｜帶不出的寶物｜fix-compile｜memory｜回傳區域變數參考（E0515、E0597）｜遺跡關門，寶物跟著消失
- `16 relic-struct`｜導遊卷軸｜build｜memory｜struct 持有參考 `Guide<'a>`、省略規則｜卷軸只指向原文，不複製
- `17 boss-ancient-tablet`｜BOSS：古碑解讀｜debug｜memory｜UTF-8 byte index、`char_indices`｜古碑是中文寫的，切字串切到半個字就 panic

### 第 5 章 漂流瓶（Option / Result）

- `18 maybe-gem`｜也許有寶石｜build｜grid｜`Option`、`if let`、`match`｜寶箱可能是空的
- `19 question-mark`｜沒有寶石的島｜debug｜grid｜`unwrap` panic、`?` 運算子｜地圖沒寶石時 Ferris 直接暈倒 **[MVP]**
- `20 bottle-message`｜漂流瓶密語｜build｜none｜`Result`、`parse`、自訂 error enum、`From`｜解析瓶中信的指令文字，格式錯要說清楚錯在哪
- `21 overflow-current`｜逆流｜debug｜grid｜`usize` 減法 overflow、`checked_sub`｜往左走到座標 -1，整個海流倒灌

### 第 6 章 工匠之島（Trait、泛型、Iterator、Closure）

- `22 trait-crafts`｜會走路的都是夥伴｜build｜grid｜trait、`impl Trait for`、預設方法、`Box<dyn Trait>` 與 `&dyn Trait`、靜態與動態分派｜Ferris 與海鷗都實作 `Mover`，海鷗一次走兩格、仍會被岩石擋；`relay` 讓不同夥伴排成同一支接力隊伍
- `23 generic-bag`｜萬用寶物袋｜fix-compile｜none｜泛型與 trait bound（E0369、E0277）｜找最大值時編譯器問「這東西能比大小嗎」
- `24 iterator-tide`｜一行退潮｜build｜grid｜`iter` / `map` / `filter` / `collect`、closure｜用一條 iterator 鏈算出路線
- `25 lazy-current`｜不動的計數器｜debug｜none｜iterator 惰性求值、`#[must_use]` 警告、`FnMut` 捕捉｜`map` 沒有 `collect`，寶石一顆都沒算

### 第 7 章 智慧指標洞窟（Box / Rc / RefCell）

- `26 box-the-cave`｜無限深的洞窟｜fix-compile｜memory｜遞迴型別（E0072）、`Box`｜洞窟裡還有洞窟，大小算不出來
- `27 shared-shell`｜共用貝殼屋｜build｜memory｜`Rc`、`strong_count`、`Rc::clone` 只加計數｜三隻螃蟹住同一個殼
- `28 refcell-panic`｜寶物箱卡住了｜debug｜memory｜`RefCell` 執行期借用檢查、雙重借用 panic｜邊看箱子邊放東西，箱子鎖死 **[MVP]**
- `29 leaky-cycle`｜永不消失的漩渦｜debug｜memory｜`Rc` 循環參考洩漏、`Weak`｜兩個島互相指著對方，永遠不會被釋放

### 第 8 章 群蟹大遷徙（並行安全）

- `30 send-the-crabs`｜派出分身｜fix-compile｜none｜`thread::spawn`、`move` closure（E0373）、`Send` / `Sync`、`Rc` 不是 `Send`（E0277）→ `Arc`；`roll_call`、`dispatch` 回傳 `Vec<JoinHandle<String>>` 由呼叫端 join，`pack` 把地圖包成可共用的 `Arc<Map>`｜Ferris 分身出發，帶錯指標就出不了門
- `31 shared-counter`｜消失的寶石｜debug｜none｜`Arc<Mutex<T>>`、MutexGuard、複製出值的 bug、`Mutex<T>` 是 `Sync`；bug 在抽出的 `add_haul(&Mutex<u32>, u32)`｜一百隻螃蟹採集，總數卻是 0 **[MVP]**
- `32 message-relay`｜螃蟹傳令兵｜build｜grid｜`mpsc::channel`、`tx.clone()`、drop 原始 `tx` 關閉 channel、`recv_timeout`、依編號排序｜三隻螃蟹偵察員用傳聲管回報路段，指揮部不能為迷路的螃蟹等到天黑
- `33 boss-great-migration`｜BOSS：大遷徙｜debug｜grid｜死鎖（鎖順序）、channel 複習、綜合所有觀念｜三隻分身各站上窄橋，兩隻互等對方讓路，指揮官也在等不會來的回報

---

## 4. MVP 首批 8 關完整內容

**一句話重點：每關 starter 失敗的原因，就是該關要教的觀念本身。**

MVP 失敗原因總覽：

- 01：路線只走一步，測試判定沒到出口（build 的起點）
- 02：缺 `mut`，觸發 E0384、E0596（不可變預設）
- 05：函式本體是 `todo!()`，測試 panic（enum + match）
- 09：`bag` 被 move 後又使用，觸發 E0382（ownership）
- 11：持有 `&bag[0]` 期間呼叫 `push`，觸發 E0502（borrowing）
- 19：`unwrap()` 在找不到寶石時 panic（Option 與 `?`）
- 28：`Ref` 還活著時呼叫 `borrow_mut()`，RefCell panic（執行期借用規則）
- 31：從 MutexGuard 複製出 `u32` 後只改到複本，總數永遠 0（Mutex 的正確用法）

> 編譯環境：`rustc --edition 2024 --test -A dead_code -F unsafe_code`（rustc 1.99.1）。MVP 8 關已依 §1.6 合併順序實測（當時開頭四行；v1.3 加的 `#[allow(unused_imports)]` 只抑制警告，不影響結果）：solution 全部測試通過、starter 依各關所述失敗；正式納入前仍由內容測試（architecture.md §6.2 ④）在 runner 映像內再跑一次。

2024 edition 相容性檢查：

- 未使用 `gen` 作為識別字（2024 保留字）
- 無 `extern` 區塊、無 `static mut`、無 `unsafe fn`
- 無 RPIT（`-> impl Trait`），不受捕獲規則變更影響
- closure 與 match pattern 未使用 `ref` / `&` 搭配預設綁定模式（2024 match ergonomics 限制）
- `todo!()`、`panic!` 都有明確的期望型別，不依賴 never type fallback
- 31 關改用尾端運算式回傳 MutexGuard 解參考值，順帶教 2024 尾端暫存值規則
- 28 關 concept 補充 2024 `if let` 暫存值釋放時機

---

### 4.1 `content/levels/01-hello-ferris/`

#### level.toml

```toml
id = "01-hello-ferris"
chapter = 1
order = 1
title = "第一步"
kind = "build"
visual = "grid"

story = """
我在沙灘上醒來，殼上還沾著海草。
遠方的岩縫透出一道光，那應該是通往內陸的出口。
可是我不知道怎麼走，我只聽得懂你寫的指令，帶我過去好嗎？
"""

concept = """
## 函式與回傳值

- `fn route() -> Vec<String>`：`->` 後面是回傳型別
- 函式最後一行**不加分號**的運算式，就是回傳值
- `Vec<String>`：可以變長的字串清單
- `cmds.push(String::from("MOVE RIGHT"))`：往清單尾端加一個指令

## main 與輸出

- `main` 是程式進入點
- `println!("{}", c)` 把指令印到 stdout，畫面上的 Ferris 會照著動
"""

hints = [
  "出口在第 2 列最右邊，中間有一排岩石擋住",
  "先往下走到最底列，再一路往右，最後往上一格",
  "完整路線：MOVE DOWN 兩次、MOVE RIGHT 四次、MOVE UP 一次",
]

[grid]
width  = 5
height = 3
start  = [0, 0]
goal   = [4, 1]
walls  = [[2, 0], [2, 1]]
```

#### starter.rs

```rust
// 回傳 Ferris 要執行的指令
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.push(String::from("MOVE DOWN"));
    // TODO 繼續加指令，帶 Ferris 走到終點
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
```

#### tests.rs

```rust
#[test]
fn ferris_reaches_exit() {
    assert_cleared(&level_grid(), &route());
}
```

#### solution.rs

```rust
// 回傳 Ferris 要執行的指令
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.push(String::from("MOVE DOWN"));
    cmds.push(String::from("MOVE DOWN"));
    for _ in 0..4 {
        cmds.push(String::from("MOVE RIGHT"));
    }
    cmds.push(String::from("MOVE UP"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
```

- **starter 失敗原因**：只走一步停在 (0, 1)，`ferris_reaches_exit` 斷言「Ferris 沒有停在終點」失敗

---

### 4.2 `content/levels/02-mut-matters/`

#### level.toml

```toml
id = "02-mut-matters"
chapter = 1
order = 2
title = "黏住的腳"
kind = "fix-compile"
visual = "grid"

story = """
前方四格有一顆閃亮的寶石，寶石後面就是出口！
我寫好了「往右走四步、撿起寶石、再走一步」的程式，卻一步也動不了。
海鷗在旁邊笑：「你的變數被宣告成不能改的啦！」
"""

concept = """
## Rust 的變數預設不可變

- `let x = 0;` 之後不能再改 `x`
- 要改就寫 `let mut x = 0;`
- 這是刻意的設計：看到 `mut` 就知道「這個值會變」，沒有 `mut` 就能放心它不會被偷改

## 讀懂這兩個錯誤

- **E0384** cannot assign twice to immutable variable：對不可變變數第二次賦值
- **E0596** cannot borrow as mutable：`push` 需要修改 `Vec`，但它不是 `mut`
- 編譯器的 `help:` 那一行通常直接告訴你要加在哪裡
"""

hints = [
  "看編譯錯誤裡的變數名稱，有哪兩個變數後來被修改了？",
  "steps 會 += 1，cmds 會 push，兩個都需要能被修改",
  "把 let steps 改成 let mut steps，把 let cmds 改成 let mut cmds",
]

[grid]
width  = 6
height = 2
start  = [0, 0]
goal   = [5, 0]
walls  = [[0, 1], [1, 1], [2, 1], [3, 1], [4, 1], [5, 1]]
gems   = [[4, 0]]
```

#### starter.rs

```rust
// 往右走四步撿起寶石，再走一步到終點
fn route() -> Vec<String> {
    let cmds = Vec::new();
    let steps = 0;
    while steps < 4 {
        cmds.push(String::from("MOVE RIGHT"));
        steps += 1;
    }
    cmds.push(String::from("PICK"));
    cmds.push(String::from("MOVE RIGHT"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
```

#### tests.rs

```rust
#[test]
fn ferris_picks_the_gem() {
    assert_cleared(&level_grid(), &route());
}
```

#### solution.rs

```rust
// 往右走四步撿起寶石，再走一步到終點
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    let mut steps = 0;
    while steps < 4 {
        cmds.push(String::from("MOVE RIGHT"));
        steps += 1;
    }
    cmds.push(String::from("PICK"));
    cmds.push(String::from("MOVE RIGHT"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
```

- **starter 失敗原因**：編譯失敗，E0384（`steps += 1`）與 E0596（`cmds.push`），邏輯本身正確，只差 `mut`
- 地圖下排全是牆，視覺上是一條走廊（`height` 最小為 2）

---

### 4.3 `content/levels/05-compass-enum/`

#### level.toml

```toml
id = "05-compass-enum"
chapter = 2
order = 5
title = "神奇羅盤"
kind = "build"
visual = "grid"

story = """
我在退潮的沙洲撿到一個古老羅盤，上面只有四個方向。
傳說只要把羅盤方向翻譯成指令，就能找到通往潮汐神殿的路。
路線羅盤已經指好了，翻譯的部分就拜託你了。
"""

concept = """
## enum：列出所有可能

- `enum Dir { Up, Down, Left, Right }`：方向只會是這四種之一
- 用 `Dir::Up` 取得其中一個值

## match：每種情況都要處理

- `match d { Dir::Up => ..., Dir::Down => ..., ... }`
- match 是運算式，可以直接當回傳值
- 少寫任何一個分支，編譯器都會拒絕（下一章會正面遇到）

## derive

- `#[derive(Debug, Clone, Copy, PartialEq)]` 讓編譯器自動幫 enum 產生「可印出、可複製、可比較」的能力
"""

hints = [
  "todo!() 會在執行時 panic，要把它換成真正的 match",
  "command 的每個分支回傳字串，例如 Dir::Up => \"MOVE UP\"，最後記得轉成 String",
  "delta 回傳 (x 位移, y 位移)：往上是 (0, -1)，往右是 (1, 0)",
]

[grid]
width  = 4
height = 3
start  = [0, 0]
goal   = [3, 2]
walls  = [[3, 0], [0, 1], [0, 2]]
```

#### starter.rs

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// 把方向翻譯成 Ferris 的指令
fn command(d: Dir) -> String {
    todo!("用 match 回傳 MOVE UP / MOVE DOWN / MOVE LEFT / MOVE RIGHT")
}

// 方向對應的 (x, y) 位移
fn delta(d: Dir) -> (i32, i32) {
    todo!("用 match 回傳位移")
}

fn route() -> Vec<String> {
    let plan = [Dir::Right, Dir::Right, Dir::Down, Dir::Down, Dir::Right];
    plan.iter().map(|d| command(*d)).collect()
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
```

#### tests.rs

```rust
#[test]
fn command_text() {
    assert_eq!(command(Dir::Up), "MOVE UP");
    assert_eq!(command(Dir::Down), "MOVE DOWN");
    assert_eq!(command(Dir::Left), "MOVE LEFT");
    assert_eq!(command(Dir::Right), "MOVE RIGHT");
}

#[test]
fn delta_matches_direction() {
    assert_eq!(delta(Dir::Up), (0, -1));
    assert_eq!(delta(Dir::Down), (0, 1));
    assert_eq!(delta(Dir::Left), (-1, 0));
    assert_eq!(delta(Dir::Right), (1, 0));
}

#[test]
fn route_reaches_temple() {
    assert_cleared(&level_grid(), &route());
}
```

#### solution.rs

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// 把方向翻譯成 Ferris 的指令
fn command(d: Dir) -> String {
    let text = match d {
        Dir::Up => "MOVE UP",
        Dir::Down => "MOVE DOWN",
        Dir::Left => "MOVE LEFT",
        Dir::Right => "MOVE RIGHT",
    };
    text.to_string()
}

// 方向對應的 (x, y) 位移
fn delta(d: Dir) -> (i32, i32) {
    match d {
        Dir::Up => (0, -1),
        Dir::Down => (0, 1),
        Dir::Left => (-1, 0),
        Dir::Right => (1, 0),
    }
}

fn route() -> Vec<String> {
    let plan = [Dir::Right, Dir::Right, Dir::Down, Dir::Down, Dir::Right];
    plan.iter().map(|d| command(*d)).collect()
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
```

- **starter 失敗原因**：能編譯（`todo!()` 型別是 `!`，另有未使用參數警告），三個測試都在呼叫 `command` / `delta` 時 panic
- `delta` 採 `(x, y)` 與 `[grid]` 座標一致：`y` 往下為正

---

### 4.4 `content/levels/09-move-the-gem/`

#### level.toml

```toml
id = "09-move-the-gem"
chapter = 3
order = 9
title = "交出去的背包"
kind = "fix-compile"
visual = "memory"

story = """
我把背包交給海鷗清點數量。
牠數完說「三樣」，就連背包一起飛走了。
我回頭想看第一樣寶物是什麼，手上卻空空如也……
"""

concept = """
## Ownership 三條規則

- 每個值都有一個**擁有者（owner）**
- 同一時間只能有**一個**擁有者
- 擁有者離開作用域時，值會被**釋放（drop）**

## Move（移動）

- `count_items(bag)` 把 `Vec<String>` 傳進函式，擁有權就**移動**給參數
- 函式結束時參數被 drop，heap 上的資料跟著釋放
- 之後再用 `bag` 就是 **E0382** use / borrow of moved value

## 三種修法與取捨

- **調整順序**：先讀 `bag[0]`，再交出去（本關解答）
- **`bag.clone()`**：複製一份 heap 資料，代價是多一次配置與複製
- **改傳 `&bag`**：只借不給，下一關的主題
"""

hints = [
  "錯誤訊息說 bag 在哪一行被 move 了？之後又在哪一行被使用？",
  "count_items(bag) 之後，bag 已經不屬於 report 了",
  "把 let first = bag[0].clone(); 移到 count_items(bag) 之前",
]

[[memory_snapshots]]
line = 8
note = "bag 擁有 heap 上的三個字串"
heap = [
  { id = "h1", type = "[String; 3]", value = "珍珠 | 貝殼 | 海星", state = "live" },
]
pointers = [
  { from = "report.bag", to = "h1", kind = "own" },
]

[[memory_snapshots.frames]]
id = "report"
fn = "report"
vars = [
  { id = "report.bag", name = "bag", type = "Vec<String>", value = "len=3 cap=3", state = "live" },
]

[[memory_snapshots]]
line = 3
note = "擁有權移動到 count_items 的參數，report 的 bag 失效"
heap = [
  { id = "h1", type = "[String; 3]", value = "珍珠 | 貝殼 | 海星", state = "live" },
]
pointers = [
  { from = "count_items.bag", to = "h1", kind = "own" },
]

[[memory_snapshots.frames]]
id = "report"
fn = "report"
vars = [
  { id = "report.bag", name = "bag", type = "Vec<String>", state = "moved" },
]

[[memory_snapshots.frames]]
id = "count_items"
fn = "count_items"
vars = [
  { id = "count_items.bag", name = "bag", type = "Vec<String>", value = "len=3 cap=3", state = "live" },
]

[[memory_snapshots]]
line = 9
note = "count_items 結束，參數被 drop，heap 資料釋放"
heap = [
  { id = "h1", type = "[String; 3]", value = "珍珠 | 貝殼 | 海星", state = "freed" },
]
pointers = []

[[memory_snapshots.frames]]
id = "report"
fn = "report"
vars = [
  { id = "report.bag", name = "bag", type = "Vec<String>", state = "moved" },
  { id = "report.n", name = "n", type = "usize", value = "3", state = "live" },
]

[[memory_snapshots]]
line = 10
note = "再讀 `bag[0]`，編譯器回報 **E0382** borrow of moved value: `bag`"
error = { code = "E0382", target = "report.bag" }
heap = []
pointers = []

[[memory_snapshots.frames]]
id = "report"
fn = "report"
vars = [
  { id = "report.bag", name = "bag", type = "Vec<String>", state = "moved" },
  { id = "report.n", name = "n", type = "usize", value = "3", state = "live" },
]
```

#### starter.rs

```rust
// 計算背包裡有幾樣東西
fn count_items(bag: Vec<String>) -> usize {
    bag.len()
}

// 回傳 (數量, 第一樣東西)
fn report() -> (usize, String) {
    let bag = vec![String::from("珍珠"), String::from("貝殼"), String::from("海星")];
    let n = count_items(bag);
    let first = bag[0].clone();
    (n, first)
}

fn main() {
    let (n, first) = report();
    println!("背包有 {} 樣，第一樣是 {}", n, first);
}
```

#### tests.rs

```rust
#[test]
fn report_counts_and_names_first() {
    let (n, first) = report();
    assert_eq!(n, 3, "背包數量不對");
    assert_eq!(first, "珍珠", "第一樣寶物不對");
}
```

#### solution.rs

```rust
// 計算背包裡有幾樣東西
fn count_items(bag: Vec<String>) -> usize {
    bag.len()
}

// 回傳 (數量, 第一樣東西)
fn report() -> (usize, String) {
    let bag = vec![String::from("珍珠"), String::from("貝殼"), String::from("海星")];
    let first = bag[0].clone();
    let n = count_items(bag);
    (n, first)
}

fn main() {
    let (n, first) = report();
    println!("背包有 {} 樣，第一樣是 {}", n, first);
}
```

- **starter 失敗原因**：編譯失敗 E0382，`bag` 在第 9 行 move 進 `count_items`，第 10 行又被借用
- **測試刻意不呼叫 `count_items`**：玩家改成 `&[String]` 或 `clone()` 都能過，三種修法都被接受

---

### 4.5 `content/levels/11-one-writer-rule/`

#### level.toml

```toml
id = "11-one-writer-rule"
chapter = 3
order = 11
title = "一寫多讀"
kind = "fix-compile"
visual = "memory"

story = """
我想先記住背包裡第一樣寶物的名字，再把剛撿到的紅寶石放進去。
很合理的兩個動作，編譯器卻說不行。
海鷗歪著頭：「你手指還指著第一格，背包就要搬家了耶。」
"""

concept = """
## 借用規則

- 同一時間，二選一：
  - 任意多個**共享借用** `&T`（只能讀）
  - 恰好一個**可變借用** `&mut T`（可讀可寫）
- 借用的有效範圍：從建立到**最後一次使用**為止（NLL，Non-Lexical Lifetimes：依實際使用位置判斷借用何時結束）

## 為什麼 push 不能和 &bag[0] 共存

- `push` 可能讓 `Vec` 容量不足而**重新配置**：搬到 heap 的新位置，舊位置釋放
- `first` 還指著舊位置，就會變成懸空參考（dangling reference）
- 其他語言要到執行期才出事，Rust 在編譯期用 **E0502** 擋下

## 修法

- 不要「拿著參考」跨過修改：先 `clone()` 成自己擁有的 `String`
- 或把讀取移到修改之後
"""

hints = [
  "錯誤訊息標了三個位置：哪裡借出、哪裡要可變借用、哪裡還在用第一個借用",
  "first 是 &String，它在 push 之後才被使用，所以借用一直延續到 push 之後",
  "把 let first = &bag[0]; 改成 let first = bag[0].clone();，最後直接回傳 first",
]

[[memory_snapshots]]
line = 3
note = "first 共享借用 bag 的第一格"
heap = [
  { id = "h1", type = "[String; 1]", value = "珍珠", state = "live" },
]
pointers = [
  { from = "main.bag", to = "h1", kind = "own" },
  { from = "first_then_add.bag", to = "main.bag", kind = "mut" },
  { from = "first_then_add.first", to = "h1", kind = "shared" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.bag", name = "bag", type = "Vec<String>", value = "len=1 cap=1", state = "live" },
]

[[memory_snapshots.frames]]
id = "first_then_add"
fn = "first_then_add"
vars = [
  { id = "first_then_add.bag", name = "bag", type = "&mut Vec<String>", state = "live" },
  { id = "first_then_add.first", name = "first", type = "&String", state = "live" },
]

[[memory_snapshots]]
line = 4
note = "push 需要擴充容量，buffer 搬家，first 會指向已釋放的位置；編譯器回報 **E0502** cannot borrow `*bag` as mutable because it is also borrowed as immutable"
error = { code = "E0502", target = "first_then_add.first" }
heap = [
  { id = "h1", type = "[String; 1]", value = "珍珠", state = "freed" },
  { id = "h2", type = "[String; 4]", value = "珍珠 | 紅寶石", state = "live" },
]
pointers = [
  { from = "main.bag", to = "h2", kind = "own" },
  { from = "first_then_add.bag", to = "main.bag", kind = "mut" },
  { from = "first_then_add.first", to = "h1", kind = "shared" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.bag", name = "bag", type = "Vec<String>", value = "len=2 cap=4", state = "live" },
]

[[memory_snapshots.frames]]
id = "first_then_add"
fn = "first_then_add"
vars = [
  { id = "first_then_add.bag", name = "bag", type = "&mut Vec<String>", state = "live" },
  { id = "first_then_add.first", name = "first", type = "&String", state = "live" },
]

[[memory_snapshots]]
line = 4
note = "（修正後）first 是 clone 出來的字串，擁有自己的 heap 資料，push 搬家不影響它"
heap = [
  { id = "h2", type = "[String; 4]", value = "珍珠 | 紅寶石", state = "live" },
  { id = "h3", type = "str", value = "珍珠", state = "live" },
]
pointers = [
  { from = "main.bag", to = "h2", kind = "own" },
  { from = "first_then_add.bag", to = "main.bag", kind = "mut" },
  { from = "first_then_add.first", to = "h3", kind = "own" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.bag", name = "bag", type = "Vec<String>", value = "len=2 cap=4", state = "live" },
]

[[memory_snapshots.frames]]
id = "first_then_add"
fn = "first_then_add"
vars = [
  { id = "first_then_add.bag", name = "bag", type = "&mut Vec<String>", state = "live" },
  { id = "first_then_add.first", name = "first", type = "String", state = "live" },
]
```

#### starter.rs

```rust
// 先記下第一樣寶物，再放入紅寶石
fn first_then_add(bag: &mut Vec<String>) -> String {
    let first = &bag[0];
    bag.push(String::from("紅寶石"));
    first.clone()
}

fn main() {
    let mut bag = vec![String::from("珍珠")];
    let first = first_then_add(&mut bag);
    println!("{} {:?}", first, bag);
}
```

#### tests.rs

```rust
#[test]
fn remembers_first_and_adds_ruby() {
    let mut bag = vec![String::from("珍珠")];
    let first = first_then_add(&mut bag);
    assert_eq!(first, "珍珠");
    assert_eq!(bag, vec![String::from("珍珠"), String::from("紅寶石")]);
}

#[test]
fn works_with_longer_bag() {
    let mut bag = vec![String::from("海星"), String::from("貝殼")];
    assert_eq!(first_then_add(&mut bag), "海星");
    assert_eq!(bag.len(), 3, "紅寶石沒有放進背包");
}
```

#### solution.rs

```rust
// 先記下第一樣寶物，再放入紅寶石
fn first_then_add(bag: &mut Vec<String>) -> String {
    let first = bag[0].clone();
    bag.push(String::from("紅寶石"));
    first
}

fn main() {
    let mut bag = vec![String::from("珍珠")];
    let first = first_then_add(&mut bag);
    println!("{} {:?}", first, bag);
}
```

- **starter 失敗原因**：編譯失敗 E0502，`first` 的共享借用延續到第 5 行，中間第 4 行要可變借用

---

### 4.6 `content/levels/19-question-mark/`

#### level.toml

```toml
id = "19-question-mark"
chapter = 5
order = 19
title = "沒有寶石的島"
kind = "debug"
visual = "grid"

story = """
我寫了一個自動導航程式：找到寶石、規劃路線、出發。
前幾座島都很順利，直到登上一座光禿禿的小島，程式直接崩潰，我原地暈倒。
導航程式應該冷靜地回報「這裡沒有寶石」，而不是倒下。幫我修好它！
"""

concept = """
## Option：可能沒有值

- `Option<T>` 只有兩種：`Some(值)` 或 `None`
- Rust 沒有 null，「可能沒有」一定寫在型別上

## unwrap 的風險

- `unwrap()` 遇到 `None` 會 **panic**，程式直接中止
- 適合「確定不可能是 None」的地方，不適合處理正常會發生的情況

## ? 運算子

- 在回傳 `Option` 的函式裡，`x?` 的意思是：
  - `x` 是 `Some(v)` 就取出 `v` 繼續
  - `x` 是 `None` 就**立刻回傳 None**
- 同樣適用於 `Result`，下一關會用到

## Debug 心法

- 先看測試失敗的 panic 訊息，再對照程式找出是哪個 `unwrap` 爆掉
"""

hints = [
  "失敗的測試是哪兩個？它們的地圖有什麼共同點？",
  "panic 訊息是 called `Option::unwrap()` on a `None` value，去找 plan 裡的 unwrap",
  "plan 本身就回傳 Option，把三個 .unwrap() 改成 ? 就好",
]

[grid]
width  = 5
height = 3
start  = [0, 0]
goal   = [4, 2]
gems   = [[3, 2]]
```

#### starter.rs

```rust
// 找出字元在地圖上的位置 (x, y)
fn find(map: &[&str], target: char) -> Option<(usize, usize)> {
    for (y, line) in map.iter().enumerate() {
        if let Some(x) = line.chars().position(|ch| ch == target) {
            return Some((x, y));
        }
    }
    None
}

// 從 from 走到 to：先上下、再左右
fn walk(cmds: &mut Vec<String>, from: (usize, usize), to: (usize, usize)) {
    let vertical = if to.1 > from.1 { "MOVE DOWN" } else { "MOVE UP" };
    for _ in 0..from.1.abs_diff(to.1) {
        cmds.push(vertical.to_string());
    }
    let horizontal = if to.0 > from.0 { "MOVE RIGHT" } else { "MOVE LEFT" };
    for _ in 0..from.0.abs_diff(to.0) {
        cmds.push(horizontal.to_string());
    }
}

// 規劃先撿寶石再到出口的路線，地圖缺 F、G、E 任一個時回傳 None
fn plan(map: &[&str]) -> Option<Vec<String>> {
    let ferris = find(map, 'F').unwrap();
    let gem = find(map, 'G').unwrap();
    let exit = find(map, 'E').unwrap();
    let mut cmds = Vec::new();
    walk(&mut cmds, ferris, gem);
    cmds.push("PICK".to_string());
    walk(&mut cmds, gem, exit);
    Some(cmds)
}

fn main() {
    let map = ["F....", ".....", "...GE"];
    match plan(&map) {
        Some(cmds) => {
            for c in cmds {
                println!("{}", c);
            }
        }
        None => println!("這座島沒有寶石"),
    }
}
```

#### tests.rs

```rust
// 把測試用的字元地圖轉成 Grid
fn grid_from(map: &[&str]) -> Grid {
    let mut grid = Grid {
        width: map[0].chars().count() as i32,
        height: map.len() as i32,
        start: (0, 0),
        goal: (0, 0),
        walls: vec![],
        gems: vec![],
        doors: vec![],
    };
    for (y, line) in map.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let p = (x as i32, y as i32);
            match ch {
                'F' => grid.start = p,
                'E' => grid.goal = p,
                'G' => grid.gems.push(p),
                _ => {}
            }
        }
    }
    grid
}

#[test]
fn gem_down_right() {
    let map = ["F....", ".....", "...GE"];
    let cmds = plan(&map).expect("有寶石時應該回傳路線");
    assert_cleared(&grid_from(&map), &cmds);
}

#[test]
fn gem_up_left() {
    let map = ["EG...", ".....", "....F"];
    let cmds = plan(&map).expect("有寶石時應該回傳路線");
    assert_cleared(&grid_from(&map), &cmds);
}

#[test]
fn island_without_gem() {
    let map = ["F....", "....E"];
    assert!(plan(&map).is_none(), "沒有寶石時應該回傳 None");
}

#[test]
fn island_without_ferris() {
    let map = ["....G", "E...."];
    assert!(plan(&map).is_none(), "地圖上沒有 Ferris 時應該回傳 None");
}
```

- 測試地圖與玩家 `main` 的地圖一樣用字元表示（`F` Ferris、`G` 寶石、`E` 出口），由 `grid_from` 轉成 `Grid` 後交給共用模擬器
- `gem_down_right` 的地圖與 `[grid]` 相同，確保畫面與判題一致
- 本關測試需要多張地圖，不使用判題器產生的 `level_grid()`

#### solution.rs

```rust
// 找出字元在地圖上的位置 (x, y)
fn find(map: &[&str], target: char) -> Option<(usize, usize)> {
    for (y, line) in map.iter().enumerate() {
        if let Some(x) = line.chars().position(|ch| ch == target) {
            return Some((x, y));
        }
    }
    None
}

// 從 from 走到 to：先上下、再左右
fn walk(cmds: &mut Vec<String>, from: (usize, usize), to: (usize, usize)) {
    let vertical = if to.1 > from.1 { "MOVE DOWN" } else { "MOVE UP" };
    for _ in 0..from.1.abs_diff(to.1) {
        cmds.push(vertical.to_string());
    }
    let horizontal = if to.0 > from.0 { "MOVE RIGHT" } else { "MOVE LEFT" };
    for _ in 0..from.0.abs_diff(to.0) {
        cmds.push(horizontal.to_string());
    }
}

// 規劃先撿寶石再到出口的路線，地圖缺 F、G、E 任一個時回傳 None
fn plan(map: &[&str]) -> Option<Vec<String>> {
    let ferris = find(map, 'F')?;
    let gem = find(map, 'G')?;
    let exit = find(map, 'E')?;
    let mut cmds = Vec::new();
    walk(&mut cmds, ferris, gem);
    cmds.push("PICK".to_string());
    walk(&mut cmds, gem, exit);
    Some(cmds)
}

fn main() {
    let map = ["F....", ".....", "...GE"];
    match plan(&map) {
        Some(cmds) => {
            for c in cmds {
                println!("{}", c);
            }
        }
        None => println!("這座島沒有寶石"),
    }
}
```

- **starter 失敗原因**：能編譯，前兩個測試通過；`island_without_gem`、`island_without_ferris` 因 `unwrap()` panic 而失敗
- 找不到寶石時 `main` 印出的「這座島沒有寶石」不是指令，只顯示在主控台
- `usize::abs_diff` 需要 Rust 1.60 以上

---

### 4.7 `content/levels/28-refcell-panic/`

#### level.toml

```toml
id = "28-refcell-panic"
chapter = 7
order = 28
title = "寶物箱卡住了"
kind = "debug"
visual = "memory"

story = """
我和海鷗合買了一個寶物箱，用 Rc 讓我們都能拿著鑰匙。
因為兩個都要往裡面放東西，箱子外面又套了一層 RefCell。
規則很簡單：重複的寶物不放，新的才放。
可是我第一次放海星，箱子就「喀」一聲卡死了。
"""

concept = """
## Rc<RefCell<T>>：共享又可修改

- `Rc<T>`：多個擁有者共享同一份資料，`Rc::clone` 只加**參考計數**
- `RefCell<T>`：**interior mutability（內部可變性）**，透過共享參考也能修改內容
- 合起來就是「多人共用、大家都能改」

## 代價：借用規則延後到執行期

- `borrow()` 拿到 `Ref`（共享借用），`borrow_mut()` 拿到 `RefMut`（可變借用）
- 規則和編譯期一樣：多個 `Ref` 或一個 `RefMut`
- 違反時不是編譯錯誤，而是**執行時 panic**（BorrowMutError）
- `Ref` 要等變數離開作用域（或 `drop`）才歸還

## Debug 心法

- 找出 panic 當下「還活著的 Ref」是哪個變數
- 縮短借用：用完立刻結束，不要把 `Ref` 存在變數裡跨過 `borrow_mut()`

## 2024 edition：if let 暫存值提早釋放

- `if let Some(x) = c.borrow().items.first() { .. } else { c.borrow_mut() .. }`
- 2021：scrutinee 的 `Ref` 活到整個 `else` 結束，`else` 裡 `borrow_mut()` 會 panic
- 2024：`Ref` 在進入 `else` 前就釋放，不再 panic
- 但 `then` 區塊內仍持有 `Ref`，在那裡 `borrow_mut()` 依然 panic
"""

hints = [
  "panic 訊息提到 already borrowed，代表 borrow_mut 時還有人在借",
  "view 是 chest.borrow() 回傳的 Ref，它活到函式最後一行",
  "先算出 exists 這個 bool，讓 borrow() 在同一行結束；需要時再 borrow_mut 放入，最後重新 borrow() 取長度",
]

[[memory_snapshots]]
line = 21
note = "chest 與 gull 共享同一個寶物箱，strong = 2"
heap = [
  { id = "h1", type = "RcBox<RefCell<Chest>>", value = "items: [珍珠] · strong=2 weak=0 · 借用：無", state = "live" },
]
pointers = [
  { from = "main.chest", to = "h1", kind = "own" },
  { from = "main.gull", to = "h1", kind = "own" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.chest", name = "chest", type = "Rc<RefCell<Chest>>", state = "live" },
  { id = "main.gull", name = "gull", type = "Rc<RefCell<Chest>>", state = "live" },
]

[[memory_snapshots]]
line = 11
note = "view 取得共享借用，RefCell 記下 1 個共享借用"
heap = [
  { id = "h1", type = "RcBox<RefCell<Chest>>", value = "items: [珍珠] · strong=2 weak=0 · 借用：1 個共享", state = "live" },
]
pointers = [
  { from = "main.chest", to = "h1", kind = "own" },
  { from = "main.gull", to = "h1", kind = "own" },
  { from = "deposit.chest", to = "main.chest", kind = "shared" },
  { from = "deposit.view", to = "h1", kind = "shared" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.chest", name = "chest", type = "Rc<RefCell<Chest>>", state = "live" },
  { id = "main.gull", name = "gull", type = "Rc<RefCell<Chest>>", state = "live" },
]

[[memory_snapshots.frames]]
id = "deposit"
fn = "deposit"
vars = [
  { id = "deposit.chest", name = "chest", type = "&Rc<RefCell<Chest>>", state = "live" },
  { id = "deposit.view", name = "view", type = "Ref<Chest>", state = "live" },
]

[[memory_snapshots]]
line = 15
note = "view 還沒歸還就要 `borrow_mut()`，執行期 panic：RefCell already borrowed"
error = { target = "deposit.view" }
heap = [
  { id = "h1", type = "RcBox<RefCell<Chest>>", value = "items: [珍珠] · strong=2 weak=0 · 借用：1 個共享", state = "live" },
]
pointers = [
  { from = "main.chest", to = "h1", kind = "own" },
  { from = "main.gull", to = "h1", kind = "own" },
  { from = "deposit.chest", to = "main.chest", kind = "shared" },
  { from = "deposit.view", to = "h1", kind = "shared" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.chest", name = "chest", type = "Rc<RefCell<Chest>>", state = "live" },
  { id = "main.gull", name = "gull", type = "Rc<RefCell<Chest>>", state = "live" },
]

[[memory_snapshots.frames]]
id = "deposit"
fn = "deposit"
vars = [
  { id = "deposit.chest", name = "chest", type = "&Rc<RefCell<Chest>>", state = "live" },
  { id = "deposit.view", name = "view", type = "Ref<Chest>", state = "live" },
]

[[memory_snapshots]]
line = 13
note = "（修正後）共享借用在算出 exists 那行就歸還，再取得可變借用放入海星"
heap = [
  { id = "h1", type = "RcBox<RefCell<Chest>>", value = "items: [珍珠, 海星] · strong=2 weak=0 · 借用：可變", state = "live" },
]
pointers = [
  { from = "main.chest", to = "h1", kind = "own" },
  { from = "main.gull", to = "h1", kind = "own" },
  { from = "deposit.chest", to = "main.chest", kind = "shared" },
]

[[memory_snapshots.frames]]
id = "main"
fn = "main"
vars = [
  { id = "main.chest", name = "chest", type = "Rc<RefCell<Chest>>", state = "live" },
  { id = "main.gull", name = "gull", type = "Rc<RefCell<Chest>>", state = "live" },
]

[[memory_snapshots.frames]]
id = "deposit"
fn = "deposit"
vars = [
  { id = "deposit.chest", name = "chest", type = "&Rc<RefCell<Chest>>", state = "live" },
  { id = "deposit.exists", name = "exists", type = "bool", value = "false", state = "live" },
]
```

#### starter.rs

```rust
use std::cell::RefCell;
use std::rc::Rc;

// Ferris 與海鷗共用的寶物箱
struct Chest {
    items: Vec<String>,
}

// 放入寶物（重複就不放），回傳箱內數量
fn deposit(chest: &Rc<RefCell<Chest>>, item: &str) -> usize {
    let view = chest.borrow();
    if view.items.iter().any(|i| i.as_str() == item) {
        return view.items.len();
    }
    chest.borrow_mut().items.push(item.to_string());
    view.items.len()
}

fn main() {
    let chest = Rc::new(RefCell::new(Chest { items: vec![String::from("珍珠")] }));
    let gull = Rc::clone(&chest);
    println!("箱內 {} 樣", deposit(&chest, "海星"));
    println!("箱內 {} 樣", deposit(&gull, "海星"));
}
```

#### tests.rs

```rust
fn chest_with(items: &[&str]) -> Rc<RefCell<Chest>> {
    let items = items.iter().map(|s| s.to_string()).collect();
    Rc::new(RefCell::new(Chest { items }))
}

#[test]
fn deposit_new_item() {
    let chest = chest_with(&["珍珠"]);
    assert_eq!(deposit(&chest, "海星"), 2);
    assert_eq!(chest.borrow().items, vec![String::from("珍珠"), String::from("海星")]);
}

#[test]
fn duplicate_is_skipped() {
    let chest = chest_with(&["珍珠"]);
    assert_eq!(deposit(&chest, "珍珠"), 1, "重複的寶物不應該放入");
}

#[test]
fn gull_shares_the_chest() {
    let chest = chest_with(&[]);
    let gull = Rc::clone(&chest);
    deposit(&gull, "海星");
    deposit(&chest, "貝殼");
    assert_eq!(Rc::strong_count(&chest), 2);
    assert_eq!(gull.borrow().items.len(), 2, "海鷗看到的箱子內容不對");
}
```

#### solution.rs

```rust
use std::cell::RefCell;
use std::rc::Rc;

// Ferris 與海鷗共用的寶物箱
struct Chest {
    items: Vec<String>,
}

// 放入寶物（重複就不放），回傳箱內數量
fn deposit(chest: &Rc<RefCell<Chest>>, item: &str) -> usize {
    let exists = chest.borrow().items.iter().any(|i| i.as_str() == item);
    if !exists {
        chest.borrow_mut().items.push(item.to_string());
    }
    chest.borrow().items.len()
}

fn main() {
    let chest = Rc::new(RefCell::new(Chest { items: vec![String::from("珍珠")] }));
    let gull = Rc::clone(&chest);
    println!("箱內 {} 樣", deposit(&chest, "海星"));
    println!("箱內 {} 樣", deposit(&gull, "海星"));
}
```

- **starter 失敗原因**：能編譯；`deposit_new_item`、`gull_shares_the_chest` 在 `borrow_mut()` 時 panic，`duplicate_is_skipped` 走提早 return 路徑而通過；panic 訊息為 `RefCell already borrowed`
- 這個「部分測試通過」是刻意的：讓玩家發現 bug 只在「放入新寶物」路徑出現

---

### 4.8 `content/levels/31-shared-counter/`

#### level.toml

```toml
id = "31-shared-counter"
chapter = 8
order = 31
title = "消失的寶石"
kind = "debug"
visual = "none"

story = """
大遷徙前夕，我派出一百隻分身，每隻負責一片海灘。
大家撿完寶石，就呼叫 `add_haul` 把數量加進共用的計數器。
所有分身都平安回來了，計數器上卻寫著：0。
海鷗盯著空空的寶物箱：「寶石該不會被你們吃掉了吧？」
"""

concept = """
## Arc<Mutex<T>>：跨執行緒共享又可修改

- `Arc<T>`：**原子參考計數**，`Rc` 的執行緒安全版（實作 `Send` + `Sync`）
- `Mutex<T>`：同一時間只允許一個執行緒存取內容
- `lock()` 回傳 `MutexGuard<T>`，guard 存活期間持有鎖，離開作用域自動解鎖

## 這關的陷阱：複製出來改

- `*guard` 對 `u32` 這種 `Copy` 型別，會**複製**一份值出來
- 對複本 `+=` 不會影響 Mutex 裡的值
- 要透過 guard 本身修改：`*guard += n`

## 為什麼 &Mutex 能交給一百條執行緒

- `Mutex<T>` 只要 `T: Send` 就是 `Sync`：多條執行緒同時拿著 `&Mutex<u32>` 是安全的
- 真正碰到內容前一定要先 `lock()`，同一時間只有一條執行緒拿得到 guard
- 所以 `add_haul` 只需要 `&Mutex<u32>`，不用知道外面是 `Arc` 還是 `thread::scope` 借來的

## 對照第 28 關

- `Rc<RefCell<T>>`：單執行緒，借用檢查在執行期
- `Arc<Mutex<T>>`：多執行緒，存取權用鎖控制

## 2024 edition：尾端暫存值提早釋放

- 函式最後一行 `*total.lock().unwrap()` 產生的 MutexGuard 是暫存值
- 2024 起，尾端運算式的暫存值會在區域變數之前釋放，guard 不會比 `total` 活得久
- 2021 同一行會觸發 E0597，必須先存進變數再回傳

## Debug 心法

- 編譯器的警告也是線索：value assigned is never read
"""

hints = [
  "編譯器有一則警告說某個變數被賦值後從來沒被讀過，是 add_haul 裡的哪一個？它和計數器是同一個東西嗎？",
  "let mut sum = *total.lock().unwrap(); 的 sum 是 u32 複本，不是計數器本身",
  "把 guard 留在變數裡：let mut sum = total.lock().unwrap(); 再寫 *sum += gems;",
]
```

#### starter.rs

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// 一隻螃蟹撿完一片海灘，把數量加進共用的總數
fn add_haul(total: &Mutex<u32>, gems: u32) {
    let mut sum = *total.lock().unwrap();
    sum += gems;
}

// 每片海灘派一隻螃蟹，全部回來後讀出總數
fn gather(beaches: Vec<u32>) -> u32 {
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for gems in beaches {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || add_haul(&total, gems)));
    }
    for h in handles {
        h.join().unwrap();
    }
    // 2024 edition：尾端暫存的 guard 會先於 total 釋放
    *total.lock().unwrap()
}

fn main() {
    println!("總共 {} 顆寶石", gather(vec![3, 5, 7]));
}
```

#### tests.rs

```rust
// &Mutex 能同時交給很多條執行緒，因為 Mutex<u32> 是 Sync
fn assert_sync<T: Sync>(_: &T) {}

#[test]
fn one_crab_one_haul() {
    let total = std::sync::Mutex::new(10u32);
    add_haul(&total, 5);
    assert_eq!(*total.lock().unwrap(), 15, "螃蟹交了 5 顆寶石，計數器卻沒有變");
}

#[test]
fn hundred_crabs_share_one_counter() {
    let total = std::sync::Mutex::new(0u32);
    assert_sync(&total);
    // 分四波出發，每波 25 隻螃蟹同時把寶石加進同一個計數器
    for wave in 0..4u32 {
        std::thread::scope(|s| {
            for crab in 1..=25u32 {
                let total = &total;
                s.spawn(move || add_haul(total, wave * 25 + crab));
            }
        });
    }
    assert_eq!(*total.lock().unwrap(), 5050, "一百隻螃蟹同時交寶石，有寶石在途中消失了");
}

#[test]
fn three_crabs() {
    assert_eq!(gather(vec![3, 5, 7]), 15, "三隻螃蟹的寶石沒有全部算進去");
}

#[test]
fn hundred_crabs() {
    let beaches: Vec<u32> = (1..=100).collect();
    assert_eq!(gather(beaches), 5050, "有寶石在途中消失了");
}

#[test]
fn no_crab() {
    assert_eq!(gather(Vec::new()), 0, "沒有螃蟹出門，計數器卻不是 0");
}
```

#### solution.rs

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// 一隻螃蟹撿完一片海灘，把數量加進共用的總數
fn add_haul(total: &Mutex<u32>, gems: u32) {
    let mut sum = total.lock().unwrap();
    *sum += gems;
}

// 每片海灘派一隻螃蟹，全部回來後讀出總數
fn gather(beaches: Vec<u32>) -> u32 {
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for gems in beaches {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || add_haul(&total, gems)));
    }
    for h in handles {
        h.join().unwrap();
    }
    // 2024 edition：尾端暫存的 guard 會先於 total 釋放
    *total.lock().unwrap()
}

fn main() {
    println!("總共 {} 顆寶石", gather(vec![3, 5, 7]));
}
```

- **starter 失敗原因**：能編譯，有 2 筆警告（rustc 1.99.1 實測）：`unused_variables`（第 6 行 `sum` 被賦值卻沒被使用）與 `unused_assignments`（第 7 行 `sum += gems` 的值沒被讀取）；`one_crab_one_haul`、`hundred_crabs_share_one_counter`、`three_crabs`、`hundred_crabs` 因計數器沒變而失敗、`no_crab` 通過
- `add_haul` 只收 `&Mutex<u32>`：測試直接呼叫它，並用 `thread::scope` 讓一百條執行緒共用同一個計數器，改寫 `gather` 繞過 bug 無法通過
- 尾端運算式 `*total.lock().unwrap()` 依賴 2024 edition 規則：尾端暫存值（MutexGuard）在區域變數 `total` 之前釋放；同一行在 2021 edition 會觸發 E0597

---

## 5. memory_snapshots 欄位設計

**一句話重點：型別以 ux.md §5.1 的 `MemorySnapshot` 為準；本節只說明在 `level.toml` 裡怎麼寫。**

### 5.1 TOML 寫法

```
① [[memory_snapshots]]
  ↓
② line、note、heap、pointers
  ↓
③ [[memory_snapshots.frames]]
```

① 每一步一個 `[[memory_snapshots]]`，陣列順序即播放順序（第 0 筆是第 1 步）
② 先寫該步的純量與 inline table 陣列；`heap`、`pointers` 沒有內容時可寫 `[]` 或省略（省略視為 `[]`）
③ 每個 stack frame 一個 `[[memory_snapshots.frames]]`，由外到內排列，最後一個是目前執行中的函式；`vars` 為 inline table 陣列

### 5.2 欄位對照（ux.md §5.1）

- 快照：`line`（1 起算，對應編輯器行號）、`note`（支援行內 markdown）、`frames`、`heap`、`pointers`、`error`
- `frames[]`：`id`、`fn`、`vars`
- `vars[]`：`id`（全域唯一，慣例 `<frame id>.<變數名>`）、`name`、`type`、`value`、`state`（`live`／`moved`／`dropped`）
  - `value` 可省略：TOML 沒有 null，省略即代表 `null`（API 回傳 `"value": null`）
- `heap[]`：`id`（`h1`、`h2`…）、`type`、`value`、`state`（`live`／`freed`）
- `pointers[]`：`from`（變數 `id`）、`to`（變數 `id` 或 heap `id`）、`kind`（`own`／`shared`／`mut`）
- `error`：`code`（`E\d{4}` 錯誤碼；執行期錯誤省略）、`target`（出事的變數 `id` 或 heap `id`，須存在於同一步）
  - 只在「錯誤發生的那一步」寫，其他步省略；省略即代表 `null`
  - 寫法：`error = { code = "E0382", target = "report.bag" }`，放在 `[[memory_snapshots.frames]]` 之前
- 借用狀態不另開欄位，前端由 `pointers` 推導（ux.md §5.1）

### 5.3 寫作慣例

- **錯誤訊息寫進 `note`**：例如「編譯器回報 **E0382** …」「執行期 panic：RefCell already borrowed」
- **錯誤位置寫進 `error`**：`target` 指向「規則被違反的那個東西」，例如被 move 後又使用的變數、懸空的參考、還沒歸還的 `Ref`；`note` 負責解釋原因
- **修正後的步驟**：`note` 以「（修正後）」開頭，`line` 對應 solution 的行號
- **Rc／RefCell 計數**：寫進 heap 的 `value`，例如 `items: [珍珠] · strong=2 weak=0 · 借用：1 個共享`
- **指向 heap 的某個元素**：`to` 指向整個 heap 區塊，不指向元素
- **已釋放的 heap 區塊**：該步標 `freed`，下一步起不再列出

### 5.4 設計取捨

- **用「完整快照」而非「差異事件」**：每步獨立、前端可任意跳步；代價是 TOML 較長，但每關通常 3–5 步，可接受
- **快照是手寫的教學腳本，不是真實執行追蹤**：fix-compile 關卡的 starter 根本無法執行，只能手寫；前端進關即可播放，不依賴 Run 結果（ux.md §5.5）；代價是必須人工維持行號正確，由內容測試檢查 `line` 不超過檔案行數（architecture.md §6.2 ④）；`pointers` 端點與 `error.target` 由 loader 驗證存在
- **TOML 採 inline table 陣列**：每個 inline table 單行，符合 TOML 1.0 規範

---

## 6. 難度曲線與趣味性設計

**一句話重點：每章「學會 → 被擋 → 抓錯 → Boss」，獎勵綁在 Rust 觀念上，不綁在打字速度上。**

### 6.1 難度曲線

```
① 第 1–2 章 build 為主
  ↓
② 第 3–4 章混入 fix-compile
  ↓
③ 第 5–7 章 debug 為主
  ↓
④ 第 8 章綜合 Boss
```

- ① 8 關裡 5 關 build；每關只動 3–10 行，建立「程式能控制 Ferris」的回饋迴圈
- ② fix-compile 佔一半；起始碼故意只有「一個觀念」的錯，錯誤訊息就是教材
- ③ debug 為主；程式能編譯但行為錯，玩家要讀測試失敗訊息與 panic 位置
- ④ Boss 關同時出現多種錯誤，不給「這關考什麼」的提示，只給故事

### 6.2 解鎖

- 規則以 ux.md §6.3 為準：同章節前一關至少 ★1 開下一關；上一章所有關卡至少 ★1 開下一章
- 排序依 `(chapter, order)`；關卡沿用正式編號，MVP 只載入 8 關時照樣依此排序

### 6.3 星等評分

規則以 ux.md §6.1 為準，三顆星各自判定、由前端計算。

- ★ 通過：任一次 Submit 全部測試通過
- ★ 沒看提示：通過時，本次遊玩沒有揭露任何提示
- ★ 一次就過：通過時，本次遊玩沒有失敗的 Submit（Run 不計；系統錯誤不計，見 ux.md §6.1）
- 不另設依 kind 而異的挑戰條件（不看指令數、編譯次數、修改行數）

### 6.4 Ferris 劇情推進

- 不另做章節過場：每章第一關的 `story` 兼任章節開場（§1.8）；整章關卡都通過後解鎖下一章（ux.md §6.3）
- 海鷗是固定配角：負責吐槽與暗示，提示文字可以用牠的口吻
- 第 7 章開始，鼓勵玩家在程式裡印出自己的台詞：不是指令的行只顯示在主控台，不影響 Ferris
- 最終章所有在前面章節遇過的角色（海鷗、寶物箱、古碑）一起出現在大遷徙地圖

### 6.5 趣味性原則

- **錯誤要有畫面**：E0382 時 Ferris 伸手抓空；RefCell panic 時寶物箱卡死；溢位時海流倒灌
- **測試訊息是劇情**：斷言訊息寫「Ferris 撞到岩石」，不寫「assertion failed」
- **一關只教一件事**：Boss 關以外，starter 只放一個與本關觀念相關的錯
- **讓玩家先犯錯**：fix-compile 與 debug 關卡的起始碼「看起來很合理」，讓玩家體會規則的價值

---

## 7. 已定事項

原本的未決項目都已定案，理由見 `docs/specs/decisions.md`。

- **`[grid]` 欄位**：採 ux.md §4.2 格式（`width`、`height`、`start`、`goal`、`walls`、`gems`、`doors`），不使用字元地圖與 `par`
- **關卡編號**：MVP 沿用正式編號（01、02、05…），解鎖依 `(chapter, order)`
- **測試模擬器**：抽成 `content/shared/grid_sim.rs`，判題時併入 `__quest_hidden_tests`（§1.5、§1.6）
- **星等條件**：只有通過、沒看提示、一次就過三項；不記錄送出編譯次數與修改行數
- **死鎖關卡（33）**：Run 靠判題執行 timeout（5 秒）終止，回報 `timeout`；Submit 由測試內看門狗逐測試判定互等（decisions.md ⑪）
- **編譯警告**：判題回傳保留 rustc warning，31 關的警告線索會顯示給玩家
- **grid 地圖單一來源**：`level_grid()` 由判題器依 `[grid]` 產生，`tests.rs` 不再手寫（§1.4、§1.6）
- **記憶體圖錯誤標記**：`memory_snapshots` 的錯誤步驟以 `error` 標出出事位置（§5.2）
- **章節名稱**：寫在 `content/chapters.toml`（§1.7）
- **Ferris 對話**：`story` 即泡泡內容，第一人稱撰寫；MVP 8 關已改寫（§1.8）
- **測試專用地圖**：關卡地圖不在 `tests.rs` 重寫；額外測試地圖可自寫（§1.4，19 關）
