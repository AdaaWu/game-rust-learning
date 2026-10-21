#!/usr/bin/env bash
# 在 rust-quest-runner:dev 內產生 stack overflow abort（exit 134）的 Run 與 Submit 真實輸出
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
FX=$(dirname "$HERE")
W=$(mktemp -d)
mkdir -p "$W/src" "$W/out"

cp "$HERE/abort/user.rs" "$W/src/run.rs"
{
  cat "$HERE/abort/user.rs"
  printf '%s\n' '#[cfg(test)]' '#[allow(dead_code)]' '#[allow(unused_imports)]' 'mod __quest_hidden_tests {' '    use super::*;'
  cat "$HERE/abort/tests.rs"
  printf '}\n'
} > "$W/src/submit.rs"

chmod 755 "$W" "$W/src"; chmod 644 "$W"/src/*
# 與判題相同：docker exec 直接執行，不經 shell（避免 shell 加印 Aborted）
C=rq-fixture-abort-$$
docker run -d --rm --name "$C" --network none --memory 512m --read-only --tmpfs /tmp:rw,exec,size=128m \
  --user 1000:1000 -v "$W/src":/work:ro rust-quest-runner:dev sleep 120 > /dev/null
trap 'docker kill "$C" > /dev/null 2>&1 || true; rm -rf "$W"' EXIT
O=$W/out
docker exec "$C" sh -c 'cp /work/run.rs /tmp/run.rs && rustc --edition 2024 --error-format=json -A dead_code -F unsafe_code -o /tmp/run /tmp/run.rs'
docker exec "$C" sh -c 'cp /work/submit.rs /tmp/submit.rs && rustc --edition 2024 --error-format=json -A dead_code -F unsafe_code --test -o /tmp/submit /tmp/submit.rs'
set +e
docker exec "$C" /tmp/run > "$O/run_stdout.txt" 2> "$O/run_stderr.txt"
echo $? > "$O/run_exit.txt"
H=__quest_hidden_tests
docker exec "$C" /tmp/submit --test-threads=1 --exact $H::a_first $H::b_overflows $H::c_never > "$O/submit_stdout.txt" 2> "$O/submit_stderr.txt"
echo $? > "$O/submit_exit.txt"
set -e

mkdir -p "$FX/abort"
for f in run_stdout.txt run_stderr.txt run_exit.txt submit_stdout.txt submit_stderr.txt submit_exit.txt; do
  cp "$W/out/$f" "$FX/abort/$f"
done
echo "abort fixtures 已產生"
