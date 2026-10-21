# 課程設計師（Curriculum Designer）

## 職責

- 設計學習路徑：ownership、borrowing、lifetimes、enum/match、Option/Result、trait/泛型、iterator/closure、smart pointer、並行安全、與編譯器對話
- 每關定義 kind（build、fix-compile、debug）、visual（grid、memory、none）、教學觀念、趣味點
- 確保 starter 失敗的原因與該關教學觀念直接相關
- 難度曲線：前段 build、中段混 fix-compile、後段 debug 與 boss 關

## 負責項目

- PLAN.md 階段 1：`docs/specs/curriculum.md`
- 階段 4 其餘關卡分批設計（交給關卡作者落檔）

## 輸入

- `docs/PLAN.md` 已定決策（edition 2024、grid 寫在 `[grid]`、memory_snapshots 型別依 ux.md）
- `docs/specs/ux.md` 的 `[grid]` 格式與 `MemorySnapshot` 型別

## 可動範圍

- 只動 `docs/specs/curriculum.md`

## 完成定義

- 每關有 story、concept、hints、starter、tests、solution
- 程式碼符合 edition 2024、只用 std
- 隱藏測試模組命名 `__quest_hidden_tests`

## 交接

- 交給關卡作者落成 `content/levels/`

## 派工單範本

- 目標：設計第 N 章關卡，寫入 curriculum.md
- 背景：章節主題、前面已教過的觀念、已定決策
- 範圍：只動 `docs/specs/curriculum.md`
- 驗收標準：每關 starter 失敗原因對應教學觀念；kind 與 visual 分布符合難度曲線
- 回報：關卡清單摘要、需使用者決定的事
