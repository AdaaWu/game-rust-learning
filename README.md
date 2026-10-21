# Rust Quest

Rust Quest 是一個依章節逐關進行的 Rust 學習闖關網頁：前端 Vue 3、後端 Axum。玩家在瀏覽器寫 Rust，後端用 Docker 沙箱編譯與執行判題。

> **僅供本機學習使用。** 判題會執行使用者送出的程式碼，對外部署前必須改用 rootless Docker 或 gVisor 等更強的隔離。

## 目錄結構

- `backend/`：Axum API 與判題器（Rust）
- `frontend/`：Vue 3 + Vite + TS + Pinia + Monaco
- `content/`：關卡內容（`chapters.toml`、`levels/`、`shared/`）
- `runner/`：判題映像 `rust-quest-runner:dev` 的 Dockerfile
- `features/`：BDD 情境（`ui_*.feature` 給 Playwright，其餘給 cucumber-rs）
- `poc/`：早期概念驗證，可獨立啟動
- `docs/`：規格、計畫與角色說明

## 前置需求

- Rust 1.99.1（含 rustfmt、clippy）
- Node：`^22.22.2 || ^24.15.0 || ^26.0.0`（見 `frontend/package.json` 的 `engines`）
- Docker：本機可執行 `docker` 指令

建議使用 Node 22 或 24 LTS。Node 26 可執行，但 vite dev server 偶發 V8 abort（e2e 曾遇到），遇到時改用 LTS。

## 啟動步驟

```
① 建置判題映像
  ↓
② 啟動後端
  ↓
③ 啟動前端
```

① 建置判題映像（只需一次，Dockerfile 變更後重建）

```bash
docker build -t rust-quest-runner:dev runner/
```

② 啟動後端：綁定 `127.0.0.1:3000`，啟動時載入 `content/`

```bash
source $HOME/.cargo/env
cd backend
cargo run
```

③ 啟動前端：dev server 把 `/api` proxy 到 `http://127.0.0.1:3000`

```bash
cd frontend
npm install
npm run dev
```

打開 http://localhost:5173 即可開始。

## 測試指令

**後端**（在 `backend/` 執行）

- `cargo test`：單元、整合與 cucumber 情境，略過需要 Docker 的測試
- `cargo test -- --ignored`：只跑需要 Docker 與真實 `content/` 的測試（含 cucumber 的 `@docker` 情境），需先建好判題映像
- `cargo test -- --include-ignored`：全部一起跑
- `cargo test --test cucumber`：只跑 cucumber 情境；可加 `-- --tags '<表示式>'` 自訂篩選
- `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`：格式與 lint

**前端**（在 `frontend/` 執行）

- `npm test`：vitest 單元測試
- `npm run typecheck`：vue-tsc 型別檢查
- `npm run build`：型別檢查加正式建置
- `npm run e2e`：bddgen 產生步驟後跑 Playwright；自動在 5182 埠起 vite dev server，若 5182 已有服務則直接沿用

e2e 首次執行前需安裝瀏覽器：`npx playwright install chromium`。

## 埠號分配

- `3000`：後端 API（`127.0.0.1`）
- `5173`：前端 dev server（vite 預設，被占用時會自動往上找）
- `5182`：e2e 用的 vite dev server
- `5180`：POC 前端
- `3100`：POC 後端

## POC 啟動

```bash
poc/run.sh
```

需先建好 `rust-quest-runner:dev`。啟動後打開 http://127.0.0.1:5180，`Ctrl+C` 一起關閉。

## 開發方式

```
① SDD 規格
  ↓
② BDD 情境
  ↓
③ TDD 實作
```

① 先在 `docs/specs/` 寫規格（API、架構、課綱、UX、決策）
② 把驗收行為寫成 `features/*.feature`
③ 先寫測試跑紅燈，再實作到綠燈

- 進度與已定決策：`docs/PLAN.md`
- 各角色職責與派工範本：`docs/roles/`

## 安全提醒

- 判題在 Docker 沙箱內執行，`docker.sock` 不掛進任何容器，後端跑在 host
- 本專案只設計給本機學習使用
- 對外部署前需改用 rootless Docker 或 gVisor，並重新評估資源限制與網路隔離
