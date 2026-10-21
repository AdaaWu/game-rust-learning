# Rust Quest

[![CI](https://github.com/AdaaWu/game-rust-learning/actions/workflows/ci.yml/badge.svg)](https://github.com/AdaaWu/game-rust-learning/actions/workflows/ci.yml)

Vue 3 + Axum 的 Rust 學習闖關網頁：規劃 53 關，目前 33 關／8 章。類 Codecademy 的線上編輯器，搭配 Ferris 地圖、記憶體可視化與 debug 關卡，判題在 Docker 沙箱內執行。

## 快速開始

前置需求：Rust 1.99.1（含 rustfmt、clippy）、Node 22 或 24 LTS、Docker。

```
① 建置判題映像
  ↓
② 啟動後端
  ↓
③ 啟動前端
```

① `docker build -t rust-quest-runner:dev runner/`（Dockerfile 變更後重建）
② `cd backend && cargo run`（綁定 `127.0.0.1:3000`）
③ `cd frontend && npm install && npm run dev`，打開 http://localhost:5173

Node 26 可執行，但 vite dev server 偶發 V8 abort，遇到時改用 LTS。

## 測試指令

- 後端：`cargo test`（略過需要 Docker 的測試）
- 後端 Docker：`cargo test -- --ignored`（需先建好判題映像）
- 後端格式與 lint：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`
- 前端單元：`npm test`
- 前端型別與建置：`npm run build`
- 前端 e2e：`npm run e2e`（首次先跑 `npx playwright install chromium`）

## 文件導覽

- `docs/specs/`：API、架構、課綱、UX 與決策紀錄
- `features/`：BDD 情境（`ui_*.feature` 給 Playwright，其餘給 cucumber-rs）
- `docs/PLAN.md`：開發進度與已定決策
- `docs/roles/`：各角色職責與派工範本

## 安全提醒

僅供本機學習使用：判題會執行使用者送出的程式碼，對外部署前必須改用 rootless Docker 或 gVisor 等更強的隔離。
