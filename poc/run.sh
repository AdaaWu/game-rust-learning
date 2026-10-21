#!/usr/bin/env bash
# 一鍵啟動 POC：server 127.0.0.1:3100、web 127.0.0.1:5180，Ctrl+C 一起結束
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

docker image inspect rust-quest-runner:dev >/dev/null 2>&1 || {
  echo "找不到映像 rust-quest-runner:dev，請先在 runner/ 建置"; exit 1;
}

echo "▶ 編譯 server…"
(cd "$ROOT/server" && cargo build --quiet)
[ -d "$ROOT/web/node_modules" ] || (cd "$ROOT/web" && npm install)

pids=()
cleanup() {
  trap - INT TERM EXIT
  echo; echo "■ 關閉中…"
  for p in ${pids[@]+"${pids[@]}"}; do
    pkill -P "$p" 2>/dev/null || true
    kill "$p" 2>/dev/null || true
  done
  wait 2>/dev/null || true
  left=$(docker ps -q --filter "name=rq-poc-")
  [ -n "$left" ] && docker kill $left >/dev/null 2>&1 || true
}
trap cleanup INT TERM EXIT

POC_ADDR=127.0.0.1:3100 "$ROOT/server/target/debug/rust_quest_poc_server" &
pids+=($!)
(cd "$ROOT/web" && exec npx vite --host 127.0.0.1 --port 5180 --strictPort) &
pids+=($!)

echo "✓ 打開 http://127.0.0.1:5180 （Ctrl+C 結束）"
# macOS 內建 bash 3.2 沒有 wait -n，改輪詢任一行程結束
while kill -0 "${pids[0]}" 2>/dev/null && kill -0 "${pids[1]}" 2>/dev/null; do sleep 1; done
