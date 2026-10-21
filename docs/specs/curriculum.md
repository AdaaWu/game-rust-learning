# Rust 闖關遊戲：課程與關卡設計規格（SDD 課程篇）

> 狀態：v1.6（同步 decisions.md §15 ⑱ 後的實作：第 9–13 章完整內容以 `content/` 為準、52／56／66 摘要對齊落檔、41 難度上調、63 註明落差；v1.5 同步 decisions.md §15 ⑬⑭：visual 新增 `timeline`、第 8 章 30／31／33 改用時間軸、§4.8 第 31 關改寫、49／67 已定；v1.4 新增第 9–13 章 34 關設計摘要，見 §3 第二季、§6.6）
> 範圍：課程結構、關卡內容、關卡資料格式中的課程欄位。前後端實作、判題沙箱不在本文件範圍。
> 相關文件：`docs/specs/ux.md`（`[grid]`、指令語法、`MemorySnapshot`、星等）、`docs/specs/architecture.md`（判題合併流程）、`docs/specs/decisions.md`

## 0. 結論摘要

- **13 章、67 關**，故事主線是螃蟹 Ferris 從沙灘甦醒，一路冒險到「群蟹大遷徙」；第二季（第 9–13 章、34–67 關）接著進入實務篇：集合、模組與錯誤、進階 trait、巨集，最後是全 debug 的「除錯試煉場」
  - 第 1–8 章（01–33）已落檔，完整內容以 `content/` 為準
  - 第 9–13 章（34–67）已落檔，完整內容以 `content/` 為唯一來源；§3 第二季保留設計摘要，與 `content/` 不一致時以 `content/` 為準
  - 第二季 34 關：build 7、fix-compile 5、debug 22；每關標難度 1–10（§6.6）
- **難度採螺旋式**：每個核心觀念先用 `build` 學會，再用 `fix-compile` 讀懂編譯器怎麼擋，最後用 `debug` 在執行期抓錯
- **MVP 首批 8 關**：01、02、05、09、11、19、28、31
  - 三種 kind 都有：build（01、05）、fix-compile（02、09、11）、debug（19、28、31）
  - visual 共四種（grid、memory、timeline、none）；MVP 用到其中三種：grid（01、02、05、19）、memory（09、11、28）、timeline（31）
- **memory_snapshots** 採 ux.md 的 `MemorySnapshot` 格式：每一步一張快照，含 stack frames、heap 區塊、指標（own／shared／mut）
- **grid 關卡**：地圖寫在 `level.toml` 的 `[grid]`（ux.md §4.2），判題用 `content/shared/grid_sim.rs` 共用模擬器
- **timeline 關卡**：玩家程式透過檔尾的時間軸小幫手 `mod timeline` 印出事件，畫成泳道圖（architecture.md §3.11、ux.md §11）；第 8 章 30、31、33 使用

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
- std 巨集一律寫完整路徑：`::std::assert!`、`::std::assert_eq!`、`::std::assert_ne!`、`::std::matches!`、`::std::panic!`、`::std::vec!`、`::std::format!` 等；玩家自訂的同名 `macro_rules!` 會延伸到隱藏模組（decisions.md ⑱）
- 測試輔助名稱避開判題器已用的名稱：`Grid`、`World`、`MAX_STEPS`、`simulate`、`assert_cleared`、`is_command_attempt`、`direction`、`invalid`、`level_grid`

### 1.5 共用模擬器 `content/shared/grid_sim.rs`

規則與 ux.md §4.3、§4.5 一致；前端 GridWorld 解析器改規則時兩邊一起改。

```rust
// grid 關卡共用模擬器，規則與 ux.md §4.3、§4.5 一致

// 明確引入 std 名稱，蓋過 use super::* 帶進來的玩家同名定義
use ::std::option::Option::{self, None, Some};
use ::std::result::Result::{self, Err, Ok};
use ::std::string::String;
use ::std::vec::Vec;

// 方法一律用完整語法呼叫，玩家自訂 trait 同名方法無法遮蔽

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
    !<str>::is_empty(line)
        && ::std::iter::Iterator::all(&mut <str>::chars(line), |c| {
            <char>::is_ascii_uppercase(&c) || c == '_' || c == ' '
        })
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
    ::std::format!("第 {} 步：看不懂 {}", step, line)
}

// 依指令模擬 Ferris，遇到第一個錯誤就停
fn simulate(grid: &Grid, cmds: &[String]) -> Result<World, String> {
    let mut pos = grid.start;
    let mut gems = <Vec<(i32, i32)> as ::std::clone::Clone>::clone(&grid.gems);
    let mut closed = <Vec<(i32, i32)> as ::std::clone::Clone>::clone(&grid.doors);
    let mut collected = 0;
    let mut step = 0;
    for raw in cmds {
        let line = <str>::trim(raw);
        if !is_command_attempt(line) {
            continue;
        }
        step += 1;
        if step > MAX_STEPS {
            return Err(::std::format!("指令超過 {} 步上限", MAX_STEPS));
        }
        let words: Vec<&str> = ::std::iter::Iterator::collect(<str>::split_whitespace(line));
        match <Vec<&str>>::as_slice(&words) {
            ["MOVE", dir] => {
                let (dx, dy) = <Option<(i32, i32)>>::ok_or_else(direction(dir), || invalid(step, line))?;
                let next = (pos.0 + dx, pos.1 + dy);
                if next.0 < 0 || next.1 < 0 || next.0 >= grid.width || next.1 >= grid.height {
                    return Err(::std::format!("第 {} 步：走出地圖", step));
                }
                if <[(i32, i32)]>::contains(&grid.walls, &next) {
                    return Err(::std::format!("第 {} 步：撞到牆 ({}, {})", step, next.0, next.1));
                }
                if <[(i32, i32)]>::contains(&closed, &next) {
                    return Err(::std::format!("第 {} 步：門還關著 ({}, {})", step, next.0, next.1));
                }
                pos = next;
            }
            ["PICK"] => {
                let found = ::std::iter::Iterator::position(&mut <[(i32, i32)]>::iter(&gems), |g| *g == pos);
                let idx = <Option<usize>>::ok_or_else(found, || ::std::format!("第 {} 步：這格沒有寶石", step))?;
                <Vec<(i32, i32)>>::remove(&mut gems, idx);
                collected += 1;
            }
            ["OPEN", dir] => {
                let (dx, dy) = <Option<(i32, i32)>>::ok_or_else(direction(dir), || invalid(step, line))?;
                let target = (pos.0 + dx, pos.1 + dy);
                let found = ::std::iter::Iterator::position(&mut <[(i32, i32)]>::iter(&closed), |d| *d == target);
                let idx = <Option<usize>>::ok_or_else(found, || ::std::format!("第 {} 步：那邊沒有門", step))?;
                <Vec<(i32, i32)>>::remove(&mut closed, idx);
            }
            _ => return Err(invalid(step, line)),
        }
    }
    Ok(World { pos, gems_collected: collected })
}

// 通關條件：撿完全部寶石且停在終點
fn assert_cleared(grid: &Grid, cmds: &[String]) {
    let world = <Result<World, String>>::unwrap_or_else(simulate(grid, cmds), |e| ::std::panic!("{}", e));
    ::std::assert_eq!(world.gems_collected, <[(i32, i32)]>::len(&grid.gems), "還有寶石沒撿到");
    ::std::assert_eq!(world.pos, grid.goal, "Ferris 沒有停在終點");
}
```

- 項目不加 `pub`：與 `tests.rs` 同在 `__quest_hidden_tests` 模組內，直接可見
- 玩家程式若有同名項目（例如自己的 `Grid`），模組內定義會遮蔽 `use super::*` 引入的名稱，不衝突
- 開頭明確 `use` std 的 `Option`、`Result`、`String`、`Vec` 與四個變體，玩家的 `type Result<T>` 之類同名定義不會改到模擬器與 grid 關卡的測試
- 方法一律用完整語法呼叫（例如 `<[(i32, i32)]>::contains(&grid.walls, &next)`），玩家自訂 trait 的同名方法無法遮蔽 std 方法

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
③ 只有 `visual = "grid"` 的關卡併入 `content/shared/grid_sim.rs`，再接判題器依 `[grid]` 產生的單行 `fn level_grid() -> Grid`（格式見 architecture.md §3.7 ④；省略的 `walls`、`gems`、`doors` 產生 `::std::vec![]`）
④ 該關的 `tests.rs` 全文（不以換行結尾時補一個換行）
⑤ `}`

- 測試名稱在 libtest 輸出為 `__quest_hidden_tests::<函式名>`，後端回傳前去掉前綴
- timeline 小幫手屬玩家程式碼，不走隱藏測試合併：loader 啟動時已把它接在 starter 尾端（§1.8），玩家送出的程式碼本來就含它，落在 ① 裡；③ 不併入任何 timeline 檔案

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

# 第二季（v1.4 新增）

[[chapters]]
chapter = 9
title = "集合工坊"

[[chapters]]
chapter = 10
title = "模組與錯誤之塔"

[[chapters]]
chapter = 11
title = "Trait 迷宮"

[[chapters]]
chapter = 12
title = "巨集與終局"

[[chapters]]
chapter = 13
title = "除錯試煉場"
```

- `title` 與 §3 各章標題一致，不含括號內的主題說明
- 前端顯示為「第 N 章 title」；沒有關卡已載入的章節（MVP 的第 4、6 章）不會出現在 API 回應

### 1.8 `level.toml` 欄位與 story 寫法

- 必填：`id`（等於目錄名）、`chapter`、`order`、`title`、`kind`、`visual`、`story`、`concept`
  - `kind`：`build | fix-compile | debug`
  - `visual`：`grid | memory | timeline | none`
- 選填：`hints`（省略即無提示，提示區不顯示）、`[grid]`（僅 grid）、`[[memory_snapshots]]`（僅 memory）
- 完整驗證規則見 architecture.md §2.2
- **難度、debug 階梯只是文件標註**（§3 第二季標頭、§6.6），不寫進 `level.toml`：loader 拒絕未知欄位，寫了會啟動失敗
- **timeline 關卡**（architecture.md §2.2、§3.11）
  - 不可有 `[grid]` 與 `memory_snapshots`；沒有專屬設定表，鎖與 channel 的名稱寫在小幫手呼叫的參數裡
  - 小幫手 `mod timeline` 只有一份 `content/shared/timeline.rs`；loader 啟動時把它接在 `starter.rs` 與 `solution.rs` 尾端（中間空一行）
  - 關卡作者**不要**把小幫手貼進 `starter.rs`、`solution.rs`；兩個檔案直接呼叫 `timeline::lane`、`lock`、`send`、`recv`、`note`
  - 每條要畫成泳道的執行緒在開頭寫 `let _lane = timeline::lane(...)`（不可寫 `let _ =`，會立刻結束泳道）；`lane` 須在該執行緒第一個事件之前呼叫
  - 泳道名稱即識別：同一關內各執行緒的泳道名稱須唯一（例：`分身 0`、`分身 1`）；沒有呼叫 `lane` 的執行緒以 std 執行緒名當泳道（主執行緒為 `main`），不參與「全部執行緒都結束了」的判斷（ux.md §11.6）
  - 取巧反例 fixture 也不自帶小幫手，內容測試比照 loader 接上（architecture.md §6.2）
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
- **與編譯器對話（讀 rustc 錯誤碼）**：02、04、07、09、11、12、14、15、23、26、30；第二季 36、40、47、48、55
- **執行期 debug（panic、overflow、死鎖、RefCell 雙重借用）**：17、19、21、25、28、29、31、33；第二季見下方「debug 能力」

第二季（v1.4）新增觀念

- **集合（HashMap、HashSet、entry、無序性）**：34、35、39、54、67
- **排序與 closure key（穩定排序、多鍵）**：35、38、39
- **走訪時修改集合（E0502、retain）**：37、39
- **標準 trait 契約（Display、FromStr、Hash／Eq）**：36、49、50、52、59
- **模組與可見性（mod、pub、E0603）**：40、45
- **錯誤處理進階（自訂 Error、From 鏈、Box\<dyn Error\>、不吞錯誤、交易性）**：41、43、44、45、67
- **撰寫測試（#[test]、邊界案例、突變檢查）**：42、45
- **關聯型別、自訂 Iterator、運算子多載**：36（`type Err`）、46、52
- **Fn／FnMut／FnOnce 與 closure 捕捉**：47、51、52
- **trait object 與泛型取捨（dyn compatible）**：48、52
- **巨集（macro_rules!、規則順序、hygiene、重複求值、隱藏控制流）**：53、54、55、56、57、58、59
- **數值陷阱（off-by-one、underflow、`as` 截斷、浮點比較）**：60、61、62、67
- **複習型 debug（惰性、clone 複本、lifetime 設計、RefCell＋Rc 循環、鎖持有過久）**：63、64、65、66、67
- **debug 能力階梯（§6.6 D1–D5）**
  - D1 單一 bug＋明確 panic：37、49、56、60
  - D2 單一 bug＋只有錯誤結果：38、43、50、57、61、62
  - D3 需看 warning 或可視化線索：44、51、58、63、64、65
  - D4 多個 bug 疊加：39、67
  - D5 修好一個才浮現下一個：45、52、59、66、67

- 已知缺口：「撰寫測試」與「模組與可見性」只出現 2 次，低於「每個觀念至少三次」，處理方式見 §6.6 末段

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
  ↓
⑨ 集合
  ↓
⑩ 模組與錯誤
  ↓
⑪ 進階 trait
  ↓
⑫ 巨集
  ↓
⑬ 除錯試煉
```

- ① 先讓玩家能寫出會動的程式，建立成就感；`mut` 在第 2 關就出現，第一次體會「Rust 預設保守」
- ② enum + match 是後面 Option / Result 的基礎；窮舉檢查（E0004）讓玩家第一次感受「編譯器幫你想到漏掉的情況」
- ③ 所有權放在型別之後，因為要先懂 `String` 與 `Vec` 這類「擁有 heap 資料」的型別，move 才有意義
- ④ 生命週期是借用規則的延伸，必須在借用之後，否則 `'a` 只是背語法
- ⑤ 錯誤處理放在所有權之後，`?` 與 `Option<&T>` 才不會卡在借用問題
- ⑥ trait 與泛型需要 enum、借用都熟，iterator 的 closure 捕捉也牽涉借用
- ⑦ smart pointer 是「打破單一擁有者規則的合法出口」，玩家必須先被規則限制過，才懂 Rc / RefCell 的取捨
- ⑧ 並行是 Rust 精神的總驗收：Send / Sync 就是所有權與借用規則延伸到執行緒；`Arc<Mutex<T>>` 是 `Rc<RefCell<T>>` 的執行緒安全版，⑦ 的 28 關與 ⑧ 的 31 關刻意做成對照
- ⑨ 集合是實務篇開場：需要 Option（`get`）、closure（排序 key）、trait（Display／FromStr），也讓第 3 章的借用規則以 E0502 再出現一次
- ⑩ 模組與錯誤放在集合之後：錯誤型別要實作 Display（⑨ 剛學），測試撰寫需要先會讀 debug 線索
- ⑪ 進階 trait 回頭補第 6 章沒講的關聯型別、Fn 家族與 dyn 取捨，Hash／Eq 契約需要 ⑨ 的 HashSet
- ⑫ 巨集放最後一個主題：巨集展開的程式要用到前面所有語法，巨集 bug（重複求值、隱藏 return）也需要 debug 經驗
- ⑬ 全 debug 章節綜合第 1–12 章，難度逐關遞增，作為整季總驗收

---

## 3. 章節結構（13 章、67 關）

**一句話重點：前兩章以 build 為主，三、四章混入 fix-compile，五章以後以 debug 與 boss 為主；第二季 debug 比例逐章提高，第 13 章全為 debug。**

- 第 1–8 章（01–33）為第一季，以下單行摘要
- 第 9–13 章（34–67）為第二季，採展開格式，見本節「第二季」各小節

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

- `30 send-the-crabs`｜派出分身｜fix-compile｜timeline｜`thread::spawn`、`move` closure（E0373）、`Send` / `Sync`、`Rc` 不是 `Send`（E0277）→ `Arc`；`roll_call`、`dispatch` 回傳 `Vec<JoinHandle<String>>` 由呼叫端 join，`pack` 把地圖包成可共用的 `Arc<Map>`｜Ferris 分身出發，帶錯指標就出不了門；修好後時間軸上每隻分身一條泳道，誰先結束不一定
- `31 shared-counter`｜消失的寶石｜debug｜timeline｜`Arc<Mutex<T>>`、MutexGuard、複製出值的 bug、`Mutex<T>` 是 `Sync`；bug 在抽出的 `add_haul(&Mutex<u32>, u32)`｜一百隻螃蟹採集，總數卻是 0；時間軸上每隻螃蟹都「讀到 0」，且標註落在放開計數器之後 **[MVP]**
- `32 message-relay`｜螃蟹傳令兵｜build｜grid｜`mpsc::channel`、`tx.clone()`、drop 原始 `tx` 關閉 channel、`recv_timeout`、依編號排序｜三隻螃蟹偵察員用傳聲管回報路段，指揮部不能為迷路的螃蟹等到天黑
- `33 boss-great-migration`｜BOSS：大遷徙｜debug｜timeline｜死鎖（鎖順序）、channel 複習、綜合所有觀念｜三隻分身各站上窄橋，兩隻互等對方讓路，指揮官也在等不會來的回報；時間軸以 ✗ 標出「分身 0 等窄橋 1（被分身 1 佔著）」的循環等待，修好鎖順序但沒 drop(tx) 時 `main` 停在等回報、其他執行緒都已結束（移除 `[grid]`，放棄 Ferris 地圖演出）

### 第二季共用約定（第 9–13 章）

**一句話重點：每關寫清楚「教什麼、怎麼失敗、畫面怎麼呈現、取巧怎麼擋」，debug 關另寫觸發條件與線索。**

- 程式只用 std、`rustc --edition 2024`、不可用 `unsafe`（判題 `-F unsafe_code`）
- 本節列出的錯誤碼已用 rustc 1.99.1 以最小重現實測：36（E0277 ×2）、40（E0603 ×2）、47（E0596、E0507）、48（E0038）、55（E0425）、37 說明中的 E0502；49 的 stack overflow 訊息、51 的兩個 warning、58 的提早 return 行為也已實測
- 每關欄位
  - 標頭：`編號 slug`｜標題｜kind｜visual｜難度（1–10，依據見 §6.6）
  - 觀念、公開函式（tests 會呼叫，簽名定案後不可改）、starter 失敗方式、畫面、趣味點、反取巧
  - debug 關另寫：**觸發條件**（什麼輸入才會出錯）、**線索**（panic 訊息、warning、可視化異常、測試失敗訊息）、**難度階梯**（§6.6 的 D1–D5）
- 反取巧共通手法（下文以代號引用）
  - 【自建】測試自己建立多組輸入（含邊界與關卡地圖沒有的資料），擋寫死答案與特判輸入
  - 【型別】測試以特定型別、trait 或路徑呼叫（例如 `"R".parse::<Step>()`），沒照觀念寫就編譯不過；依 decisions ⑪，這類只靠介面擋下的取巧**不寫反例 fixture**，反例改針對能編譯的邏輯取巧
  - 【行為】斷言觀念特有的行為（呼叫次數、求值次數、`strong_count`、`Weak::upgrade`、HashMap 重建 20 次結果一致）
  - 【突變】把 tests.rs 內的錯誤版本傳給玩家寫的檢查函式，必須被抓到（42、45 關）
  - 【看門狗】可能卡住的關卡用 `recv_timeout` 在測試內判定失敗（同 33 關）
- 每關至少一個取巧反例 fixture（`backend/tests/fixtures/cheats/<level-id>/`），須能編譯並由測試斷言擋下
- 「Submit 時測試原始碼可讀」為已定決策：玩家照抄 tests.rs 的特定輸入特判，靠【自建】多組輸入降低效益，不另擋

### 第 9 章 集合工坊（HashMap、HashSet、Display／FromStr、排序）

章節開場：大遷徙結束，螃蟹們在新大陸開了工坊與市集，Ferris 開始管帳、管地圖、管貨。

- `34 isle-atlas`｜島嶼地名冊｜build｜grid｜難度 6.0
  - 觀念：`HashMap::new`／`insert`／`get` 回傳 `Option<&V>`；`HashSet` 去重與 `contains`；HashMap 不保證走訪順序
  - 公開函式：`build_atlas(entries: &[(&str, (i32, i32))]) -> HashMap<String, (i32, i32)>`、`first_visits(stops: &[&str], seen: &mut HashSet<String>) -> Vec<String>`（略過已在 `seen` 的地名，保留第一次出現的順序）、`locate(atlas, stops) -> Vec<(i32, i32)>`（查不到的地名略過）；`walk(from, to)` 由 starter 提供
  - starter：三個函式 `todo!()`，Run 直接 panic
  - 畫面：每個停靠點都 `PICK`；沒去重時 Ferris 回到撿過的島，模擬器停在「這格沒有寶石」
  - 趣味：海鷗同一座島喊三次，還喊了不存在的「龍蝦島」
  - 反取巧
    - 寫死路線：【自建】地名冊與停靠清單
    - 不用 HashSet：`seen` 由測試傳入且預先放入一個地名，斷言該地名被略過、呼叫後 `seen` 含全部地名
    - `locate` 用 `iter().find` 查 HashMap：行為相同，列已知殘留，concept 說明 O(1) 與 O(n) 取捨

- `35 gem-ledger`｜寶石帳本｜build｜memory｜難度 6.5
  - 觀念：`entry(k).or_insert(0)` 回傳 `&mut V`、`or_default()`、一次查找完成「沒有就建、有就改」；HashMap 無序 → 轉成 Vec 用 `sort_by` 加 closure key 排序（`b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))` 或 `sort_by_key` 搭 `Reverse`）
  - 公開函式：`tally(finds: &[&str]) -> HashMap<String, u32>`、`group_by_isle(finds: &[(&str, &str)]) -> HashMap<String, Vec<String>>`、`leaderboard(ledger: &HashMap<String, u32>) -> Vec<(String, u32)>`（數量多到少，同數量依名稱）
  - starter：`todo!()`
  - 畫面：heap 顯示帳本；`entry` 那一步出現 `mut` 指標從 `count` 指進帳本，`+= 1` 後 heap 的值更新；最後一步排行榜 Vec 已排序
  - 趣味：商人說這本帳每次翻開順序都不一樣
  - 反取巧
    - 【自建】同名多次、空輸入、同數量多筆
    - 依 HashMap 走訪順序輸出：同一份資料以不同插入順序建 20 個新 HashMap 呼叫 `leaderboard`，結果須全部相同且等於預期【行為】
    - `get` 後再 `insert` 兩次查找：行為相同，列已知殘留，concept 說明效能差異

- `36 bottle-codec`｜瓶中密碼機｜fix-compile｜grid｜難度 6.5
  - 觀念：`impl FromStr`（關聯型別 `type Err`）、`impl fmt::Display`（`write!`）、實作 Display 就免費得到 `to_string()`、`parse::<T>()` 靠 FromStr
  - 公開型別：`enum Step`（四方向與撿寶石）、`StepError::BadToken(String)`；短記號 `R`、`L`、`U`、`D`、`*` 解析成 Step，Display 印成 `MOVE RIGHT`、`PICK`
  - starter 失敗：已有固有方法 `Step::from_text`、`Step::render`，但 `route()` 寫 `tok.parse::<Step>()` 與 `format!("{step}")` → **E0277 ×2**（`Step: FromStr` not satisfied、`Step` doesn't implement Display）
  - 解法：把兩個固有方法搬進兩個 trait impl（不是單行修正）
  - 畫面：修好前編譯失敗；修好後瓶中密語翻成指令，Ferris 照走
  - 趣味：瓶中信用速記寫，Ferris 要一台雙向翻譯機
  - 反取巧
    - 改呼叫端用固有方法繞過 trait：測試直接寫 `"R".parse::<Step>()`、`format!("{}", s)`、`s.to_string()`【型別】
    - FromStr 亂收：斷言 `"X".parse::<Step>()` 為 `Err(StepError::BadToken("X"))`
    - Display 寫死：五種 Step 逐一斷言輸出

- `37 net-mending`｜補網的螃蟹｜debug｜memory｜難度 6.5｜階梯 D1
  - 觀念：走訪集合時修改它。直接 `for f in &net { net.remove(..) }` 會被 **E0502** 擋下；改用索引 `for i in 0..net.len()` 繞過編譯器，`remove(i)` 讓後面元素左移、長度變短；正解 `retain`（concept 一併介紹 `HashMap::retain`、先收集再修改）
  - 公開函式：`mend(net: &mut Vec<Fish>)` 移除所有腐壞的魚，保留其他魚的順序
  - starter bug：`for i in 0..net.len()` 裡遇到腐壞就 `net.remove(i)`；範圍在迴圈開始時就算好，移除後索引照常前進
  - 觸發條件：網裡有任何一條腐壞的魚，迴圈範圍仍用舊長度；連續兩條腐壞時還會漏刪
  - 線索：panic `index out of bounds: the len is 3 but the index is 3`，指向迴圈那一行
  - 畫面：heap 顯示網子 `[爛, 爛, 好, 爛]`；`remove(0)` 後整排左移，`i = 1` 指向原本第 3 條，第 2 條漏網；最後一步 `i` 超出長度，error 標在網子區塊
  - 趣味：上一隻螃蟹被編譯器罵了之後改用索引「繞過去」，結果網破了
  - 反取巧
    - 【自建】連續 4 條腐壞、全爛、空網、最後一條爛、沒有爛魚
    - 跑兩遍迴圈的補丁：連續 4 條腐壞的案例擋下
    - `swap_remove` 打亂順序：斷言好魚順序不變
    - 新建 Vec 再指派回去：合法解，不算取巧

- `38 rank-twice`｜排了兩次的排行榜｜debug｜memory｜難度 7.0｜階梯 D2
  - 觀念：`sort_by_key` 是穩定排序；多鍵排序分兩趟時，**後排的鍵才是主鍵**；正解是一趟 `sort_by(...then_with(...))` 或 tuple key `(Reverse(score), name)`
  - 公開函式：`rank(crabs: &mut Vec<Crab>)`：分數高到低，同分依名字
  - starter bug：先 `sort_by_key(分數)` 再 `sort_by_key(名字)`，結果只照名字排
  - 觸發條件：任何分數與名字順序不一致的資料（關卡範例資料就會觸發）
  - 線索：沒有 panic；測試失敗訊息顯示「第 1 名應該是 9 分的珍珠，實際是 3 分的阿貝」；Run 主控台印出的排行榜明顯不對
  - 畫面：heap 顯示 Vec 在第一趟後、第二趟後的內容，第二趟把第一趟的結果蓋掉
  - 趣味：頒獎典禮上冠軍是分數最低的螃蟹
  - 反取巧
    - 【自建】同分多筆、全部同分、單筆、空
    - 只反轉名字順序剛好對上範例：多組隨機排列輸入擋下

- `39 boss-market-day`｜BOSS：市集結算日｜debug｜grid｜難度 8.0｜階梯 D4
  - 綜合：FromStr 解析交易紀錄、entry 統計、排序決定走訪順序、retain 清掉售完攤位
  - 公開函式：`parse_trades(log: &str) -> Result<Vec<Trade>, TradeError>`、`stock(trades) -> HashMap<String, u32>`、`visit_order(stock) -> Vec<String>`、`route()`
  - starter 三個 bug（story 不點名）
    - ① 統計寫 `*e.or_insert(1) += 1`，每個攤位多 1
    - ② `visit_order` 直接走訪 HashMap，沒排序
    - ③ 清除售完攤位用索引 `remove`，連續售完時漏刪
  - 觸發條件：② 每次執行都可能不同；① 只在攤位第一次出現時多算；③ 需要相鄰兩個售完攤位
  - 線索
    - 同一份程式連按兩次 Run，Ferris 走的路線不一樣（②）
    - Ferris 停在售完攤位 `PICK` 失敗「這格沒有寶石」（①③）
    - 測試訊息逐項點出數量、順序、殘留攤位
  - 畫面：地圖用牆把攤位隔開，只有正確順序的 L 形路線可通
  - 反取巧
    - 【自建】交易紀錄、相鄰售完攤位
    - 順序：20 個新 HashMap 重跑結果一致且等於預期【行為】
    - 寫死路線：【自建】擋下

### 第 10 章 模組與錯誤之塔（mod／pub、自訂 Error、From 鏈、寫測試）

章節開場：市集北邊有一座高塔，每層都有守門員，只放行「說得清楚錯在哪」的螃蟹。

- `40 tower-gates`｜塔門的通行證｜fix-compile｜grid｜難度 6.5
  - 觀念：`mod`、項目預設私有、`pub`、`pub(crate)`、巢狀模組、`use` 路徑；最小可見性原則
  - 公開 API：`tower::plan() -> Vec<String>`、`tower::stairs::climb(floors: &[Floor]) -> Vec<String>`；`tower::door_code()` 應維持私有
  - starter 失敗：main 呼叫 `tower::plan()` 與 `tower::stairs::climb(..)` → **E0603 ×2**（function `plan` is private、module `stairs` is private）
  - 畫面：每層樓是地圖的一段，修好後 Ferris 爬到塔頂
  - 趣味：守門寄居蟹只認 `pub` 通行證
  - 反取巧
    - 拆掉 mod 把函式搬到最外層：測試以完整路徑呼叫【型別】
    - 寫死 `climb`：【自建】樓層
    - 全部改成 `pub`：執行期測試無法分辨，列已知殘留；concept 與 hints 說明最小可見性

- `41 error-ladder`｜錯誤的梯子｜build｜memory｜難度 7.5
  - 觀念：自訂 error enum 實作 `Display` 與 `std::error::Error`（含 `source()`）；兩層 `From` 讓 `?` 一路往上轉；應用層用 `Box<dyn Error>` 收任何錯誤、`downcast_ref` 取回原型別
  - 公開函式：`read_floor(s) -> Result<u32, FloorError>`（`FloorError::BadNumber(ParseIntError)`、`Missing`）、`climb(plan) -> Result<Vec<String>, ClimbError>`（`ClimbError::Floor(FloorError)`、`TooHigh(u32)`）、`run(plan) -> Result<Vec<String>, Box<dyn Error>>`
  - starter：enum 已宣告，impl 與函式 `todo!()`
  - 畫面：三層 frame（run → climb → read_floor）；錯誤在最內層產生，每次 `?` 往外一層、包一層，最後在 run 被 Box 進 heap
  - 趣味：塔的樓梯有一層刻成國字「三」
  - 反取巧
    - 全部轉成 String：測試 `match` 具體變體、`source()` 必須為 `Some` 且 `downcast_ref::<FloorError>()` 成功
    - Display 寫死：斷言訊息含層號與原字串
    - 【自建】合法與各種錯誤的計畫

- `42 test-the-tide`｜幫潮汐表寫測試｜build｜grid｜難度 7.5
  - 觀念：`#[test]`、`#[cfg(test)] mod tests`、`assert_eq!`／`assert!` 帶訊息、邊界案例（0 距離、負方向、只動一軸）；好測試的定義是「在錯的程式上會失敗」
  - 公開函式：`check_steps(steps: fn((i32, i32), (i32, i32)) -> Vec<String>)`（玩家寫，錯了就 panic）、`steps_to(from, to)`（starter 有 bug：dx 為負時 `0..dx` 是空範圍，不會往左與往上）
  - 玩家任務：① 寫 `check_steps`；② 修好 `steps_to`。starter 附 `#[cfg(test)] mod tests` 示範 `#[test] fn ... { check_steps(steps_to) }`，`main` 先呼叫 `check_steps(steps_to)` 再印路線，Run 就能看到自己的測試抓到 bug
  - 判題方式
    - Submit 以 `--exact` 只跑隱藏測試，玩家的 `#[test]` 會編譯但不執行（architecture §3.3），所以改把 `check_steps` 當受測物
    - ① 傳入 tests.rs 的參考實作，`check_steps` 不可 panic
    - ② 逐一傳入 tests.rs 的突變版本（往左變往右、忽略 y、多走一步、0 距離多一步、先垂直後水平），每個都必須 panic；用 `std::panic::catch_unwind`，期間以 `set_hook` 暫時靜音，失敗訊息點名沒抓到的突變
    - ③ 玩家的 `steps_to` 通過隱藏斷言
    - ④ `assert_cleared(&level_grid(), &route())`
    - 已以 rustc 1.99.1 實測 `catch_unwind` 搭 fn 指標可行
  - 畫面：地圖需要往左與往上，starter 的 Ferris 走到一半就停在錯的地方
  - 趣味：海鷗說「我測過了」，結果牠只測了往右
  - 反取巧
    - `check_steps` 一律 panic：① 擋
    - `check_steps` 空函式：②【突變】擋
    - 比對 fn 指標 `steps as usize == steps_to as usize`：① 傳入的是參考實作 → 擋
    - 以玩家自己的 `steps_to` 當對照（差分測試）：列已知殘留；玩家已修好 bug，concept 說明「拿自己的實作當標準答案」的盲點

- `43 swallowed-key`｜被揉掉的收據｜debug｜grid｜難度 7.5｜階梯 D2
  - 觀念：`let _ = expr` 丟掉 `Result`，連 `unused_must_use` 警告一起關掉；錯誤被吞之後程式在錯的前提下繼續；正解 `?`、`match`／`or_else` 做備援
  - 公開函式：`try_key(door, key) -> Result<(), LockError>`（提供，正確）、`unlock(door, cmds: &mut Vec<String>) -> Result<(), TowerError>`（先試主鑰匙，失敗改試備用鑰匙，成功才 push `OPEN`，兩把都錯回 `Err(TowerError::Locked { floor })`）、`climb(floors) -> Result<Vec<String>, TowerError>`
  - starter bug：主鑰匙失敗後寫 `let _ = try_key(door, &door.spare);` 再回 `Ok(())`
  - 觸發條件：主鑰匙錯、備用鑰匙對（第 2 層）；兩把都錯時錯誤完全消失
  - 線索：沒有 panic、**也沒有 warning**；Ferris 走到第 2 層門前停下「門還關著」；測試訊息「兩把鑰匙都錯，應該回報 Locked」
  - 趣味：Ferris 把錯誤收據揉掉丟進海裡
  - 反取巧
    - 一律 push `OPEN`：【自建】兩把都錯的門，斷言回 `Err(Locked)` 且 `cmds` 沒有新增 `OPEN`
    - 改成 panic：斷言回傳 `Err`，panic 即測試失敗
    - 寫死路線：【自建】樓層

- `44 half-built-route`｜蓋到一半的樓梯｜debug｜grid｜難度 8.0｜階梯 D3
  - 觀念：錯誤處理的「交易性」：失敗時不可留下半套狀態；先驗證再修改，或在區域變數組好再一次 `extend`
  - 公開函式：`append_floor(floor: &str, cmds: &mut Vec<String>) -> Result<(), TowerError>`、`route()`（主計畫失敗就接備援計畫）
  - starter bug：`append_floor` 邊解析邊 push，遇到壞字元才回 `Err`，前半段指令已留在 `cmds`
  - 觸發條件：一層樓的指令中間（不是開頭）有壞字元
  - 線索：沒有 panic；畫面上 Ferris 先走了幾步主計畫，再接備援計畫，撞牆；主控台印出的指令前幾行和備援計畫對不上
  - 趣味：樓梯蓋了一半就被丟著，下一隊螃蟹踩空
  - 反取巧
    - 【自建】壞字元在開頭、中間、結尾；斷言 `Err` 時 `cmds` 與呼叫前完全相同【行為】
    - 備援計畫寫死成從頭重走：斷言 `append_floor` 本身的交易性，不只看 route

- `45 boss-tower-summit`｜BOSS：塔頂的審判｜debug｜grid｜難度 8.5｜階梯 D5
  - 綜合：模組（`mod parser`、`mod planner`，可見性已正確）、自訂錯誤與 From、`?`、不吞錯誤、交易性、寫測試
  - starter 三個 bug，修好一個才浮現下一個
    - ① `parser` 用 `.unwrap_or_default()` 把壞數字吞成 0 層，route 是空的
    - ② 修好 ① 後錯誤浮現，但 `impl From<ParseIntError> for TowerError` 轉成 `Missing` 而非 `BadNumber`
    - ③ starter 內玩家先前寫的 `check_parse` 只測成功路徑，抓不到錯誤路徑（測試本身也有 bug）
  - 觸發條件：計畫含壞數字；③ 只在隱藏【突變】測試中觸發
  - 線索：先看到 Ferris 原地不動（①）；修好後測試訊息「錯誤種類應為 BadNumber」（②）；最後是「你的 check_parse 沒抓到：把壞數字當 0」（③）
  - 反取巧
    - 【自建】各種計畫；`match` 錯誤變體
    - `check_parse` 以【突變】驗證（同 42 的判題方式）
    - 寫死路線：【自建】擋下

### 第 11 章 Trait 迷宮（關聯型別、Fn 家族、dyn 取捨、trait 契約）

章節開場：塔頂的門後是一座迷宮，牆上刻著各種 trait，只有遵守契約的夥伴走得出去。

- `46 patrol-iterator`｜巡邏隊的節拍器｜build｜grid｜難度 7.0
  - 觀念：關聯型別：`impl Iterator for Patrol { type Item = Offset; fn next(&mut self) -> Option<Offset> }`、`impl Add<Offset> for Pos { type Output = Pos }`；關聯型別與泛型參數的差別（一個型別只有一種 Item）；實作 `next` 就免費得到 `take`、`map`、`collect`；無限 iterator 與惰性
  - 公開函式：`Patrol::new(pattern: Vec<Offset>)`（循環節拍，空 pattern 時 `next` 回 `None`）、`waypoints(start: Pos, patrol: Patrol, n: usize) -> Vec<Pos>`（用 `+`）、`route()`
  - starter：`todo!()`
  - 畫面：Ferris 依循環節拍巡邏 n 拍，經過寶石格
  - 趣味：巡邏隊永遠不停，Ferris 只需要前 n 拍
  - 反取巧
    - 預先展開成 Vec 再 `into_iter()`：`Patrol::new(..).nth(1_000_000)` 必須是 `Some` 且等於循環位置【行為】
    - Add 只加 x：【自建】含負值與 y 方向
    - 改用方法 `pos.shift(o)`：測試寫 `p + o`【型別】

- `47 closure-trio`｜三種鉗法｜fix-compile｜memory｜難度 7.5
  - 觀念：closure 依捕捉方式自動實作 `Fn`（唯讀借用）、`FnMut`（可變借用）、`FnOnce`（取走所有權）；bound 選「最寬鬆且夠用」；`Fn` 可當 `FnMut` 用，`FnMut` 可當 `FnOnce` 用
  - 公開函式：`repeat<F>(n: u32, f: F)`、`deliver<F>(f: F) -> String`
  - starter 失敗：兩者 bound 都寫 `Fn`，呼叫端 closure 會 push 進 `log`、會 move 出 `gem` → **E0596**（captured variable in a `Fn` closure）、**E0507**（cannot move out of `gem`）
  - 解法：`repeat` 改 `FnMut` 並加 `mut f`；`deliver` 改 `FnOnce`（兩處、兩種不同修法）
  - 畫面：closure 本身是 frame 裡的變數：`push_log` 以 `mut` 指標指向 `log`、`hand_over` 以 `own` 指標擁有 `gem`（`gem` 標 moved）；error 步驟標在 `log` 與 `gem`
  - 趣味：Ferris 的三種鉗法：看、捏、交出去
  - 反取巧
    - 改呼叫端不 push 或改用 clone：測試自寫 `let mut n = 0; repeat(3, || n += 1)` 與 `deliver(move || s)`（`s` 為 String）【型別】
    - 在 `Fn` closure 內用 `Cell` 繞過：測試的 closure 是自己寫的 `n += 1`，`Fn` bound 仍編譯不過
    - 呼叫次數錯：斷言計數等於 n，含 n = 0【行為】

- `48 mixed-squad`｜混編小隊｜fix-compile｜grid｜難度 7.5
  - 觀念：trait object 與泛型的取捨：泛型單型化、靜態分派、可內聯，但一個 Vec 只能裝同一型別；`Box<dyn Trait>` 動態分派（vtable）、可混裝，但 trait 必須 dyn compatible（不能有泛型方法、不能回傳 `Self`）；`where Self: Sized` 把單一方法排除在 vtable 外
  - 公開 API：`trait Walker { fn name(&self) -> String; fn steps(&self) -> Vec<String>; fn steps_with<F: Fn(&str) -> String>(&self, f: F) -> Vec<String> { 預設實作 } }`、`relay(squad: &[Box<dyn Walker>]) -> Vec<String>`、熱路徑 `lead<W: Walker>(w: &W)`
  - starter 失敗：`Vec<Box<dyn Walker>>` → **E0038**（`Walker` is not dyn compatible）
  - 解法擇一：`steps_with` 加 `where Self: Sized`，或參數改 `&dyn Fn(&str) -> String`；concept 比較兩者代價
  - 畫面：Ferris、海鷗、寄居蟹排成接力隊，各走一段
  - 趣味：三種夥伴步伐不同，卻要排進同一支隊伍
  - 反取巧
    - 改成 enum 包三種夥伴：測試定義自己的 `struct TestCrab` 實作 `Walker` 後放進 `Vec<Box<dyn Walker>>` 呼叫 `relay`【型別】
    - `steps_with` 為預設方法，測試型別只實作必要方法；測試呼叫寫 `w.steps_with(&|s: &str| ..)`，兩種修法都相容
    - 刪掉 `steps_with`：測試在具體型別上呼叫它【型別】

- `49 echo-rune`｜回音符文｜debug｜grid｜難度 7.5｜階梯 D1
  - 觀念：`Display` 裡呼叫 `self.to_string()` 或 `write!(f, "{}", self)` 會再呼叫自己的 `fmt` → 無限遞迴；應印欄位而不是印自己
  - 公開型別：`Rune` 的 Display 印出指令文字，`route()` 用 `to_string()` 組指令
  - starter bug：`write!(f, "{}", self.to_string())`
  - 觸發條件：任何一次格式化 `Rune`
  - 線索：Run 主控台 `thread 'main' has overflowed its stack`、`fatal runtime error: stack overflow, aborting`；rustc 沒有警告
  - 畫面：Ferris 一步都沒走，主控台出現 stack overflow
  - 趣味：符文唸出自己的名字，回音停不下來
  - 反取巧
    - 【自建】每種符文逐一斷言輸出
    - 繞過 Display 改用固有方法：測試寫 `format!("{}", rune)`【型別】
  - 判題注意（已定，architecture.md §3.6、decisions.md §15 ⑭）
    - stack overflow 會讓整個測試程式 abort（exit 134）；libtest 依**測試名稱字母序**執行，宣告順序無效
    - 會 abort 的測試（格式化 `Rune` 的測試）名稱一律以 `zz_` 開頭，排在字母序最後，其他測試才有結果
    - Run 判 `runtime_panic`；Submit 判 `test_failed`，已印出 ok 的測試為 `passed`，當下 abort 的與之後的為 `not_run`；前端在測試結果分頁顯示 stderr 的 overflow 訊息（ux.md §3.3）

- `50 twin-isles`｜雙胞胎島｜debug｜memory｜難度 8.0｜階梯 D2
  - 觀念：`Hash` 與 `Eq` 的契約：`a == b` 必須推得 `hash(a) == hash(b)`；手寫 `PartialEq`（名稱不分大小寫）卻 `#[derive(Hash)]`（分大小寫）會讓 HashSet 出現「相等卻重複」、`contains` 查不到
  - 公開函式：`Isle::new(name)`、`chart(sightings: &[&str]) -> HashSet<Isle>`
  - starter bug：`#[derive(Hash)]` 與手寫的不分大小寫 `PartialEq` 並存
  - 觸發條件：同一座島以不同大小寫出現
  - 線索：沒有 panic；`Coral`、`CORAL`、`coral` 算成三座島；測試訊息「應該只有 2 座島，實際 4 座」
  - 畫面：heap 畫出 HashSet 的桶，`Coral` 在桶 2、`coral` 在桶 5，同一座島住兩個桶；error 標在重複的區塊；修正後同名落在同一桶
  - 趣味：海鷗大小寫不分，地圖上冒出一堆雙胞胎島
  - 反取巧
    - 建構時把名稱轉小寫：斷言 `Isle::new("Coral").name == "Coral"`
    - 改用 Vec 去重：`chart` 回傳 `HashSet<Isle>`【型別】；測試另建 `HashMap<Isle, u32>` 以不同大小寫查詢
    - 把 `PartialEq` 改成分大小寫：斷言 `Isle::new("A") == Isle::new("a")`
    - `Hash` 寫成常數：行為正確但退化，列已知殘留，concept 說明效能代價

- `51 copied-counter`｜數不到的步數｜debug｜grid｜難度 8.5｜階梯 D3
  - 觀念：`move` closure 捕捉 `Copy` 值（`i32`）時拿到的是副本，closure 內 `+= 1` 改的是副本，外面的計數不變；正解：不加 `move` 讓它可變借用，或把計數放進 closure 的回傳值
  - 公開函式：`patrol(beats: &[Offset], on_beat: impl FnMut()) -> Vec<String>`、`count_beats(beats) -> u32`、`route()`（依步數決定最後在哪個寶石格停下）
  - starter bug：`let mut steps = 0; patrol(beats, move || steps += 1); steps`
  - 觸發條件：一定觸發，但只表現為「數字錯」
  - 線索：rustc 兩個 warning：`value captured by steps is never read`、`value assigned to steps is never read`；Ferris 在起點附近就停下（步數為 0 選錯分支）
  - 趣味：計步器每次都說「今天走了 0 步」
  - 反取巧
    - 【自建】多組節拍；斷言 `count_beats` 精確值
    - 用 `beats.len()` 直接回傳而不經 closure：`patrol` 只在特定節拍呼叫 `on_beat`（例如原地不動的節拍不算），測試以含原地節拍的輸入擋下

- `52 boss-labyrinth-heart`｜BOSS：迷宮之心｜debug｜grid｜難度 9.0｜階梯 D5
  - 綜合：自訂 Iterator、Add、trait object、Display、closure
  - starter 三個 bug，修好一個才浮現下一個
    - ① 自訂 Iterator 的 `next` 先遞增索引再取值，第一拍被跳過
    - ② 修好 ① 後，海鷗的固有方法 `stride()` 遮蔽了 `Walker::stride()`：直接呼叫走 1 格，經 `Box<dyn Walker>` 走 2 格，兩段路線接不起來
    - ③ `impl Add` 的 y 寫成 `self.y - o.dy`，上下顛倒：指令由 `Display` 印出所以路線正確，但 Ferris 自以為的位置錯了，腳下有寶石也不撿
  - 觸發條件：① 任何節拍序列（第一拍一定被跳過）；② 隊伍裡有海鷗，且同一段路線同時經過直接呼叫與 `Box<dyn Walker>`；③ 第一次垂直移動之後，自以為的 y 就與實際相反；第一顆寶石在垂直移動之前，所以只有後半段的寶石撿不到
  - 線索：①「第一步應該是 MOVE RIGHT」；② 畫面上海鷗第一段只飛了一半，與接力段的步幅不同；③ 路線走對了，但後半段經過寶石卻沒撿
  - 反取巧：【自建】節拍與隊伍；Add 用 x、y 各自不同的值；測試同時以具體型別與 `dyn` 呼叫並斷言結果一致

### 第 12 章 巨集與終局（macro_rules!）

章節開場：迷宮出口是一間咒語工坊，古老的捲軸會在「編譯時」把短咒語展開成長長的指令。

- `53 macro-scroll`｜咒語捲軸｜build｜grid｜難度 7.0
  - 觀念：`macro_rules!`、matcher 與 transcriber、片段指示符 `ident`、`expr`、`stringify!`、多條規則依序比對、`$( ... ),*` 重複；巨集與函式的差別（參數數量可變、接受語法片段、編譯期展開）
  - 公開巨集：`steps!(DIR, n)` 展開成 n 個 `MOVE DIR`（`steps!(PICK, n)` 為 n 個 `PICK`，需放在通用規則之前）、`route![RIGHT 2, PICK 1, DOWN 3]`
  - starter：巨集規則本體為 `todo!()`
  - 畫面：捲軸唸出咒語，Ferris 照走
  - 反取巧
    - `route!` 寫死成 `vec!`：測試自己呼叫 `route![LEFT 3, PICK 1, UP 2]`、`steps!(DOWN, 0)`【自建】【型別】
    - 只接受字面值：測試寫 `let k = 2; steps!(UP, k)`
    - 巨集內呼叫輔助函式：合法，不算取巧

- `54 atlas-literal`｜一行寫完的海圖｜build｜grid｜難度 7.5
  - 觀念：重複搭配分隔符與 `$(,)?` 允許結尾逗號；巨集展開成區塊運算式；遞迴巨集（`max_of!` 拆頭與剩餘）
  - 公開巨集：`atlas!{ "珊瑚島" => (1, 0), ... }` 回傳 `HashMap<String, (i32, i32)>`（呼應第 9 章）、`max_of!(a, b, c, ...)`
  - starter：兩個巨集各只有一條吃下任何輸入的佔位 matcher（`($($unwritten:tt)*)`），展開成 `unwritten_atlas()`／`unwritten_max()`，兩個函式本體為 `todo!()`；Run 直接 panic、測試全部失敗
  - 畫面：Ferris 依海圖走到最遠的島
  - 反取巧
    - 【自建】空海圖 `atlas!{}`、結尾逗號、單一元素 `max_of!(3)`、多元素
    - 以 f64（只有 `PartialOrd`）呼叫 `max_of!`：陣列加 `iter().max()` 的寫法因 f64 沒有 `Ord` 無法編譯，強制遞迴寫法（只靠型別擋下，不寫反例）
    - 已知殘留：以 `$(..),+` 展開成陣列再 `reduce` 的寫法可通過；仍須寫出重複 matcher 與 transcriber，測試無法從行為分辨遞迴與陣列寫法

- `55 hygiene-spell`｜失效的咒語｜fix-compile｜grid｜難度 7.5
  - 觀念：巨集衛生（hygiene）：巨集內的名稱和呼叫端同名變數互不相通；要操作呼叫端變數，必須把名稱當參數傳入
  - 公開巨集：`walk!(bag, DIR)`、`grab!(bag)`
  - starter 失敗：巨集本體直接寫 `cmds.push(..)` → **E0425**（cannot find value `cmds` in this scope），每個呼叫點都報
  - 解法：巨集改收 `$bag:ident`，並改所有呼叫點（不是單行修正）
  - 畫面：修好前編譯失敗；修好後 Ferris 照咒語走
  - 反取巧
    - 測試用不叫 `cmds` 的變數名呼叫 `walk!(pack, LEFT)`【型別】
    - 改用 `static mut`：`-F unsafe_code` 擋下

- `56 wrong-spell-order`｜搶先生效的咒語｜debug｜grid｜難度 8.0｜階梯 D2
  - 觀念：`macro_rules!` 由上而下比對，第一條吻合的規則就生效；特殊規則要放在通用規則之前
  - starter bug：`steps!` 的通用規則 `($dir:ident, $n:expr)` 放在 `(PICK, $n:expr)` 之前，`PICK` 被展開成 `MOVE PICK`
  - 觸發條件：只有撿寶石的咒語觸發
  - 線索：沒有 panic；模擬器停在「第 N 步：看不懂 `MOVE PICK`」，主控台也印出 `MOVE PICK`（屬錯誤輸出，故為 D2 而非 D1）
  - 反取巧：【自建】`steps!(PICK, 2)`、`steps!(UP, 1)`；特判 `stringify!` 結果改寫字串：合法，不算取巧

- `57 double-tide`｜兩次漲潮｜debug｜grid｜難度 8.0｜階梯 D2
  - 觀念：`$e:expr` 是程式碼不是值：transcriber 寫兩次 `$e` 就求值兩次，副作用（`next()`、`+= 1`）重複；正解是展開開頭 `let v = $e;` 只求值一次
  - starter bug：`log_step!(cmds, tide.next())` 展開成 `println!(.., $e)` 加 `cmds.push($e)`
  - 觸發條件：參數帶副作用
  - 線索：沒有 panic；主控台印出的指令與 Ferris 實際走的交錯錯開，每隔一步少一步，Ferris 撞牆
  - 反取巧
    - 呼叫端先 `let c = ..` 再傳：測試自己用帶副作用的運算式呼叫巨集，斷言只執行一次【行為】
    - 【自建】潮汐序列

- `58 hidden-return`｜咒語裡的暗門｜debug｜grid｜難度 8.5｜階梯 D3
  - 觀念：巨集展開的 `return` 屬於**呼叫端函式**：`step_or_bail!` 遇錯時 `return cmds`，整個 `route()` 提早結束；巨集隱藏控制流程的風險；正解：回報錯誤或略過該步而不是 return
  - 規格：遇到壞咒語時略過該步並記錄到 `skipped`，其餘照走
  - starter bug：`Err(_) => return $cmds`
  - 觸發條件：序列中間有一個壞咒語
  - 線索：沒有 panic、沒有 warning；畫面上 Ferris 走到一半就停，停點恰好在壞咒語前
  - 反取巧：【自建】壞咒語在開頭、中間、結尾、連續多個；斷言 `skipped` 數量與其餘指令完整

- `59 boss-spell-forge`｜BOSS：咒語熔爐｜debug｜grid｜難度 9.0｜階梯 D5
  - 綜合：巨集（規則順序、重複求值、衛生）、第 9–11 章觀念
  - starter 三個 bug，修好一個才浮現下一個
    - ① `forge!` 的提早 `return` 讓路線只剩第一段
    - ② 修好 ① 後，`$e` 被求值兩次，`tally` 的 entry 計數翻倍
    - ③ 計數正確後，`Display` 印出的指令多一個結尾空白，`route()` 用 `==` 比對指令名稱時失敗而跳過寶石
  - 觸發條件：① 咒語有兩段以上（只有一段時看不出來）；② 傳給 `forge!` 的運算式有副作用（例：`tally` 的 entry 計數）；③ 路線上有寶石要撿
  - 線索：① 畫面提早停止；② 測試訊息「珍珠應為 2 顆，實際 4 顆」；③ Ferris 經過寶石卻沒撿
  - 反取巧：【自建】咒語序列；帶副作用的參數斷言求值次數

### 第 13 章 除錯試煉場（綜合 debug）

章節開場：咒語熔爐後方是一座試煉場，每一關都是「看起來能動的程式」，Ferris 要靠線索找出藏起來的 bug。

- 全部 debug；難度逐關遞增、無回落（§6.6）
- 不給「這關考什麼」，只給故事與線索；hints 第一條仍為提問式

- `60 edge-of-map`｜地圖的邊緣｜debug｜grid｜難度 6.5｜階梯 D1
  - 觀念：off-by-one 與 `usize` 邊界：`len() - 1` 在空集合 underflow；`0..=n` 多一格
  - 公開函式：`last_gem(gems: &[(usize, usize)]) -> Option<(usize, usize)>`、`scan_row(width: usize) -> Vec<String>`
  - starter bug：`let i = gems.len() - 1;`
  - 觸發條件：空的寶石清單（關卡範例第二座島沒有寶石）
  - 線索：panic `attempt to subtract with overflow`，行號指向 `len() - 1`
  - 反取巧：【自建】空、一個、多個；改成 `if len == 0 { return None }` 是合法解；寫死：【自建】擋

- `61 truncated-tide`｜被截斷的潮位｜debug｜grid｜難度 7.0｜階梯 D2
  - 觀念：`as` 轉型無聲截斷：`300_u32 as u8 == 44`；正解 `u8::try_from(x)` 並處理錯誤
  - 公開函式：`tide_level(raw: u32) -> Result<u8, TideError>`、`route()`（潮位超過上限要繞路）
  - starter bug：`raw as u8`
  - 觸發條件：潮位 ≥ 256
  - 線索：沒有 panic；Ferris 以為潮位很低直接涉水，走進被淹沒的格子（牆）
  - 反取巧：【自建】255、256、300、u32::MAX；斷言 256 以上回 `Err`；只特判 300：多組輸入擋下

- `62 float-shoal`｜浮點沙洲｜debug｜grid｜難度 7.5｜階梯 D2
  - 觀念：浮點比較：`0.1 + 0.2 != 0.3`；用誤差範圍 `(a - b).abs() < EPS`，或改用整數單位（公分）
  - 公開函式：`reached(depth: f64, target: f64) -> bool`、`dive_route(steps: &[f64]) -> Vec<String>`
  - starter bug：`depth == target`
  - 觸發條件：深度由多個小數累加
  - 線索：沒有 panic；Ferris 到了寶石格卻不 `PICK`，主控台印出 `0.30000000000000004`
  - 反取巧：【自建】多組累加序列；斷言相差很大的值不算到達（擋 `true` 常數與過大的 EPS）

- `63 lazy-lantern`｜不亮的燈籠｜debug｜grid｜難度 8.0｜階梯 D3
  - 觀念：iterator 惰性（複習 25 關）：`map` 內做副作用卻沒消費；`unused_must_use` 警告是線索；`for` 或 `for_each` 才會執行
  - 公開函式：`light_all(lanterns: &mut [Lantern])`、`route()`（只走點亮的路）
  - starter bug：`lanterns.iter_mut().map(|l| l.lit = true);`
  - 觸發條件：一定觸發
  - 線索：同一行兩則 rustc warning
    - `unused_must_use`：``unused `Map` that must be used``，附註直接寫出 `iterators are lazy and do nothing unless consumed`
    - `map_unit_fn`：`Iterator::map` 的 closure 回傳 `()`，建議改用 `for_each`
    - 畫面：Ferris 原地不動（沒有點亮的路）
  - 反取巧：【自建】燈籠數 0、1、多個；斷言每一個都點亮【行為】

- `64 stale-copy`｜改錯的分身｜debug｜memory｜難度 8.5｜階梯 D3
  - 觀念：clone 太多：`for mut crab in crabs.clone()` 修改的是複本，原資料不變；正解 `iter_mut()`
  - 公開函式：`heal_all(crabs: &mut Vec<Crab>)`
  - starter bug：迴圈走訪 `crabs.clone()`
  - 觸發條件：一定觸發，但程式沒有任何錯誤訊息
  - 線索：沒有 panic、沒有 warning；記憶體圖出現兩塊 heap，修改落在複本那塊、函式結束複本 freed；測試訊息「血量應為 10，實際 3」
  - 反取巧：【自建】多隻螃蟹；斷言原 Vec 被改；先 clone 再整個指派回去：合法，不算取巧

- `65 owned-guide`｜過期的導遊卷軸｜debug｜memory｜難度 9.0｜階梯 D3
  - 觀念：lifetime 設計：`Guide { notes: String }` 建構時把資料複製進去，之後原資料更新 Guide 看不到；改成 `Guide<'a> { notes: &'a str }` 或在需要時才借用（複習 16 關）；取捨：持有參考就受原資料生命週期限制
  - 公開型別：`Guide::new(..)`、`Guide::next_stop(&self, ..)`；地圖資料在 Guide 建立後會更新航道
  - starter bug：`Guide::new(notes: &str)` 內 `notes.to_string()` 存一份快照
  - 觸發條件：建立 Guide 之後才改航道
  - 線索：沒有 panic；記憶體圖中原航道與 Guide 內的副本是兩塊 heap，內容不同；Ferris 照舊航道走撞到新建的堤防（牆）
  - 反取巧：【自建】先建 Guide 再改資料；測試以 `Guide<'_>` 與 `std::ptr::eq` 檢查 Guide 內參考指向原資料【型別】【行為】
  - 設計提醒：此關修正需調整 struct 與呼叫順序，難度高於前一關的單點修正

- `66 tangled-shells`｜糾纏的貝殼｜debug｜memory｜難度 9.5｜階梯 D5
  - 觀念：RefCell 執行期 panic 與 Rc 循環洩漏疊加（複習 28、29 關）
  - starter 兩個 bug，修好一個才浮現下一個
    - ① 分珍珠的 `share` 開頭把 `pearls.borrow()` 存進 `left`，持有到函式結束，迴圈內又 `pearls.borrow_mut()` → panic `already borrowed: BorrowMutError`
    - ② 修好 ① 後，鄰居互相以 `Rc` 指向，drop 後 `strong_count` 不歸零（洩漏）
  - 觸發條件：① 有鄰居且有珍珠時呼叫 `share`（`link` 本身沒有 bug）；② 兩個貝殼互為鄰居後離開作用域
  - 線索：① panic 訊息；② 沒有任何訊息，只有測試「島嶼應該已被釋放」與記憶體圖中 freed 沒出現
  - 反取巧：【行為】斷言 `Weak::upgrade()` 為 `None`、`strong_count` 精確值；以 `clone()` 整份資料避開 RefCell：斷言兩個鄰居看到同一份更新（`Rc::ptr_eq`）

- `67 boss-final-trial`｜BOSS：最終試煉｜debug｜grid｜難度 10｜階梯 D4＋D5
  - 綜合：並行、錯誤處理、集合、型別轉換、iterator
  - 情境：指揮部開執行緒派三隊螃蟹偵察，彙整回報後排出最終路線
  - starter 四個 bug，部分疊加、部分要修好前一個才浮現
    - ① 指揮部持有 `board` 的 MutexGuard 時呼叫 `rx.recv()`，偵察隊要上鎖寫回報 → 鎖持有過久造成死鎖
    - ② 修好 ① 後，解析回報用 `.ok()` 把錯誤吞掉，壞回報的那一隊消失
    - ③ 路線依 HashMap 走訪順序合併，每次不同
    - ④ 座標從 `i64` `as usize` 轉換，負座標變成極大值
  - 觸發條件：① 任何一次執行（偵察隊一定要上鎖寫回報）；② 回報中有一則格式錯誤（第 2 隊）；③ 有兩隊以上的回報要合併；④ 回報含負座標
  - 線索：① Run timeout、Submit 看門狗訊息「指揮部等不到回報」；② 測試「應回報第 2 隊的錯誤」；③ 連按兩次 Run 路線不同；④ Ferris 走出地圖
  - 複習「模組與可見性」（decisions.md §15 ⑭）：程式拆成 `mod scout`（偵察隊：解析與回報）與 `mod command`（指揮部：彙整與排路線）兩個模組，測試以模組路徑呼叫；只作結構複習，不設可見性 bug
  - 複習「撰寫測試」（decisions.md §15 ⑭）：玩家須補一個檢查函式（例：檢查路線每一格都在地圖內、相鄰兩格只差一步），starter 本體為 `todo!()`；tests.rs 以【突變】把錯誤路線傳給它，必須被抓到
    - 檢查函式只由測試呼叫、不在主流程，不改變 ①–④ 的浮現順序；名稱與簽名由關卡作者定案，簽名定案後不可改
  - 反取巧：【看門狗】；【自建】回報；【突變】；20 次重跑一致；測試自開執行緒並 assert `Send`；已知殘留同 33 關（不開執行緒的串行版）
  - visual 只用 grid（decisions.md §15 ⑬）：一關只有一種 visual；死鎖已在 33 以時間軸教過，本關靠 Run timeout、主控台與看門狗訊息

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

> v1.5：visual 改 `timeline`（decisions.md §15 ⑬）。`starter.rs`、`solution.rs` 只寫玩家程式碼；時間軸小幫手 `mod timeline` 由 loader 接在尾端（§1.8），下方程式碼不含它。下方片段已用 rustc 1.99.1 `--edition 2024 -A dead_code -F unsafe_code` 接上 architecture.md §3.11 參考實作後實測（一般與 `--test` 兩種）。

#### level.toml

```toml
id = "31-shared-counter"
chapter = 8
order = 31
title = "消失的寶石"
kind = "debug"
visual = "timeline"

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

## 時間軸小幫手

- 檔案最下方的 `mod timeline` 會把上鎖、解鎖印給右邊的時間軸看，不改變程式行為
- `timeline::lock(&m, "計數器")` 和 `m.lock()` 用法相同，只多一個名字；回傳的 guard 一樣要 `.unwrap()`，離開作用域才解鎖
- `timeline::note(...)` 在時間軸上插一面小旗子，`timeline::lane(...)` 替這條執行緒取名

## 對照第 28 關

- `Rc<RefCell<T>>`：單執行緒，借用檢查在執行期
- `Arc<Mutex<T>>`：多執行緒，存取權用鎖控制

## 2024 edition：尾端暫存值提早釋放

- 函式最後一行 `*timeline::lock(&total, "計數器").unwrap()` 產生的 guard 是暫存值（小幫手的 guard 包著 `MutexGuard`，規則相同）
- 2024 起，尾端運算式的暫存值會在區域變數之前釋放，guard 不會比 `total` 活得久
- 2021 同一行會觸發 E0597，必須先存進變數再回傳

## Debug 心法

- 編譯器的警告也是線索：value assigned is never read
- 看時間軸：每隻螃蟹的小旗子「讀到 n」寫的是多少？小旗子插在放開計數器之前還是之後？
"""

hints = [
  "編譯器有一則警告說某個變數被賦值後從來沒被讀過，是 add_haul 裡的哪一個？它和計數器是同一個東西嗎？",
  "let mut sum = *timeline::lock(total, \"計數器\").unwrap(); 的 sum 是 u32 複本，不是計數器本身；時間軸上它在拿到鎖的同一行就放開了",
  "把 guard 留在變數裡：let mut sum = timeline::lock(total, \"計數器\").unwrap(); 再寫 *sum += gems;（小旗子改印 *sum）",
  "讀出舊值和寫回新值要在同一次 lock() 內完成（原子性）：分兩次上鎖，中間別的螃蟹可能插隊，寶石照樣會消失",
]
```

#### starter.rs

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// 一隻螃蟹撿完一片海灘，把數量加進共用的總數
fn add_haul(total: &Mutex<u32>, gems: u32) {
    let mut sum = *timeline::lock(total, "計數器").unwrap();
    timeline::note(&format!("讀到 {}", sum));
    sum += gems;
}

// 每片海灘派一隻螃蟹，全部回來後讀出總數
fn gather(beaches: Vec<u32>) -> u32 {
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for (i, gems) in beaches.into_iter().enumerate() {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || {
            let _lane = timeline::lane(&format!("螃蟹 {}", i));
            add_haul(&total, gems);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    // 2024 edition：尾端暫存的 guard 會先於 total 釋放
    *timeline::lock(&total, "計數器").unwrap()
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
    let mut sum = timeline::lock(total, "計數器").unwrap();
    timeline::note(&format!("讀到 {}", *sum));
    *sum += gems;
}

// 每片海灘派一隻螃蟹，全部回來後讀出總數
fn gather(beaches: Vec<u32>) -> u32 {
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for (i, gems) in beaches.into_iter().enumerate() {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || {
            let _lane = timeline::lane(&format!("螃蟹 {}", i));
            add_haul(&total, gems);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    // 2024 edition：尾端暫存的 guard 會先於 total 釋放
    *timeline::lock(&total, "計數器").unwrap()
}

fn main() {
    println!("總共 {} 顆寶石", gather(vec![3, 5, 7]));
}
```

- **starter 失敗原因**：能編譯，有 1 筆警告（rustc 1.99.1 實測）：`unused_assignments`（第 8 行 `sum += gems` 的值沒被讀取）；`one_crab_one_haul`、`hundred_crabs_share_one_counter`、`three_crabs`、`hundred_crabs` 因計數器沒變而失敗、`no_crab` 通過
  - 第 7 行 `note` 讀了 `sum`，所以原本的 `unused_variables` 不再出現；`note` 必須放在 `sum += gems` 之前，`unused_assignments` 才保留（hint 1 的線索），內容測試釘住
- **時間軸（starter Run）**：三條螃蟹泳道各自「拿到 計數器 → 放開 計數器 → 🚩讀到 0」，小旗子都在放開之後；`main` 泳道沒有 `lane`、沒有開始與結束標記，只在最後讀總數時上鎖一次（不參與「全部執行緒都結束了」判斷，ux.md §11.6）
- **時間軸（solution Run）**：小旗子落在持有區段內，三隻依拿到鎖的先後讀到 0、前一隻加完的數（例：0、3、10，順序每次可能不同）
- `add_haul` 只收 `&Mutex<u32>`：測試直接呼叫它，並用 `thread::scope` 讓一百條執行緒共用同一個計數器，改寫 `gather` 繞過 bug 無法通過；Submit 時小幫手以 `cfg!(test)` 靜音，不影響測試
- 尾端運算式 `*timeline::lock(&total, "計數器").unwrap()` 依賴 2024 edition 規則：尾端暫存值在區域變數 `total` 之前釋放；同一行在 2021 edition 會觸發 E0597（已實測）

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
- 第二季延伸：第 9–12 章 debug 比例逐章提高（3/6、3/6、4/7、4/7），章末 Boss 皆為 debug；第 13 章 8 關全為 debug；逐關難度見 §6.6

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
- 第 8 章所有在前面章節遇過的角色（海鷗、寶物箱、古碑）一起出現在大遷徙的 story 裡；33 改用時間軸後不再有大遷徙地圖（decisions.md §15 ⑬）

### 6.5 趣味性原則

- **錯誤要有畫面**：E0382 時 Ferris 伸手抓空；RefCell panic 時寶物箱卡死；溢位時海流倒灌
- **測試訊息是劇情**：斷言訊息寫「Ferris 撞到岩石」，不寫「assertion failed」
- **一關只教一件事**：Boss 關以外，starter 只放一個與本關觀念相關的錯
- **讓玩家先犯錯**：fix-compile 與 debug 關卡的起始碼「看起來很合理」，讓玩家體會規則的價值

### 6.6 難度曲線表（第 9–13 章）

**一句話重點：難度 1–10、以 0.5 為級距；第二季最低 6.0（等於第 8 章前段），第 13 章逐關遞增無回落。**

難度刻度定錨（第一季只為定錨標示，不回改內容）

- 第 8 章：30 為 6.0、31 為 6.0、32 為 6.5、33 為 7.5
- 判定依據：新觀念數、需修改的位置數、線索的明顯程度（D1–D5）、是否需綜合前章

debug 難度階梯

```
① D1 明確 panic
  ↓
② D2 只有錯誤結果
  ↓
③ D3 看 warning 或畫面
  ↓
④ D4 多個 bug 疊加
  ↓
⑤ D5 修好一個才浮現下一個
```

- ① 單一 bug，panic 訊息與行號直接指到出事處
- ② 單一 bug，沒有 panic，只有測試失敗訊息或錯誤輸出
- ③ 沒有 panic，主要線索是 rustc warning、記憶體圖或 grid 動畫的異常
- ④ 同時存在多個獨立 bug，測試失敗訊息分別指向不同症狀
- ⑤ 前一個 bug 遮住後一個，修好才看到新的失敗

第 8 → 9 章銜接

- 第 9 章是實務篇開場，主題比並行容易，但起點 6.0 不低於第 8 章前段
- 不比第 1–3 章簡單：每關同時用到 Option、closure、trait 等第 5–6 章觀念
- 第 9 章的兩個 debug 從 D1、D2 開始，讓 debug 階梯在第二季重新起步

逐關難度與遞增理由

- 第 9 章 集合工坊
  - 34 isle-atlas 6.0：新集合 API，build，三個小函式
  - 35 gem-ledger 6.5：entry 回傳 `&mut V` 加上多鍵排序
  - 36 bottle-codec 6.5：fix-compile，但要寫兩個 trait impl
  - 37 net-mending 6.5：第一個 debug，D1 明確 panic
  - 38 rank-twice 7.0：D2，沒有 panic，要懂穩定排序
  - 39 boss-market-day 8.0：D4，三個 bug 加上不確定的走訪順序
- 第 10 章 模組與錯誤之塔
  - 40 tower-gates 6.5：章節開場，E0603 錯誤訊息直白
  - 41 error-ladder 7.5：三層錯誤型別與 From 鏈；實測工作量偏重（starter 有 6 個 `impl` 待寫：`Display`、`Error`、`From` 各兩個，另加 3 個函式），由 7.0 上調，與 42 同級
  - 42 test-the-tide 7.5：新技能，要自己想邊界案例
  - 43 swallowed-key 7.5：D2，`let _` 讓 warning 也消失
  - 44 half-built-route 8.0：D3，要從畫面看出半套狀態
  - 45 boss-tower-summit 8.5：D5，三個 bug 依序浮現，含測試本身的 bug
- 第 11 章 Trait 迷宮
  - 46 patrol-iterator 7.0：兩個關聯型別、無限 iterator
  - 47 closure-trio 7.5：兩處不同的 bound 修法
  - 48 mixed-squad 7.5：要在兩種 dyn compatible 修法間取捨
  - 49 echo-rune 7.5：D1，stack overflow 訊息明確但沒有行號
  - 50 twin-isles 8.0：D2，要懂 Hash 與 Eq 的契約
  - 51 copied-counter 8.5：D3，唯一線索是 warning
  - 52 boss-labyrinth-heart 9.0：D5，三個 bug 依序浮現
- 第 12 章 巨集與終局
  - 53 macro-scroll 7.0：新語法，build
  - 54 atlas-literal 7.5：重複、結尾逗號、遞迴
  - 55 hygiene-spell 7.5：fix-compile，要改巨集簽名與所有呼叫點
  - 56 wrong-spell-order 8.0：D2，沒有 panic，模擬器訊息直接顯示 `MOVE PICK`（原標 D1，但 D1 定義為明確 panic，改標 D2；不放寬 D1 定義，以免 37、49、60 的 D1 意義變動）
  - 57 double-tide 8.0：D2，要理解 `$e` 是程式碼
  - 58 hidden-return 8.5：D3，沒有任何訊息，只有提早停止的畫面
  - 59 boss-spell-forge 9.0：D5
- 第 13 章 除錯試煉場（逐關遞增，無回落）
  - 60 edge-of-map 6.5：D1，panic 直接指到 `len() - 1`
  - 61 truncated-tide 7.0：D2，`as` 截斷沒有任何訊息
  - 62 float-shoal 7.5：D2，要從主控台數值看出浮點誤差
  - 63 lazy-lantern 8.0：D3，線索只有 warning
    - 實測體感偏易：rustc 附註直接寫出 `iterators are lazy`，等於點名原因
    - 數值維持 8.0：第 13 章須逐關遞增，降到 7.5 會與 62 同級；關卡順序已落檔，若要真正反映體感，須調換 62／63 的 `order`（動 `content/`），列為已知落差
  - 64 stale-copy 8.5：D3，沒有 warning，要讀記憶體圖
  - 65 owned-guide 9.0：D3，要改 struct 設計與生命週期
  - 66 tangled-shells 9.5：D5，panic 修好後浮現無訊息的洩漏
  - 67 boss-final-trial 10：D4＋D5，死鎖遮住其餘三個 bug

避免第一季 09、11、12 連續單行修正的平台期

- 第二季每個 fix-compile 都需要兩處以上修改或新增 impl：36 兩個 trait impl、40 兩個可見性加上保留私有、47 兩種不同 bound、48 設計取捨、55 巨集簽名加所有呼叫點
- 唯一連續的 fix-compile（47、48）修法類型不同：47 改 bound，48 在兩種 dyn compatible 修法間取捨，visual 也不同
- debug 關依 D1 → D5 階梯推進，每章第一個 debug 從較低階梯起步，章末 Boss 落在 D4 或 D5
- 章節開場關（34、40、46、53、60）允許比前一章 Boss 低，但不低於 6.0；第 13 章內嚴格遞增

觀念覆蓋補強（已定，decisions.md §15 ⑭）

- 「撰寫測試」原本只在 42、45 出現：67 加一個玩家須補的檢查函式，以【突變】驗證（§3 第 13 章 67）
- 「模組與可見性」原本只在 40、45 出現：67 的程式拆成 `mod scout`、`mod command` 兩個模組，不設可見性 bug，只作結構複習（§3 第 13 章 67）

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
- **timeline 可視化**：一關只有一種 visual，timeline 關卡不帶 `[grid]` 與 `memory_snapshots`；第 8 章 30、31、33 用 timeline，32 維持 grid，67 只用 grid（decisions.md §15 ⑬⑭）
- **33 Boss 以時間軸呈現循環等待**：移除 `[grid]`、放棄 Ferris 地圖演出；時間軸以 ✗ 標出兩隻分身互等窄橋，`main` 停在等回報（§3 第 8 章、ux.md §11.6）
- **會 abort 的測試**：libtest 依名稱字母序執行，會 abort 的測試以 `zz_` 開頭；Run 判 `runtime_panic`、Submit 判 `test_failed`（49 關，architecture.md §3.6）
