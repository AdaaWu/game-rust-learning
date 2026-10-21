#!/usr/bin/env bash
# 在 rust-quest-runner:dev 內重產 rustc 與 libtest fixture（v1.3 指令與五行隱藏模組開頭）
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
FX=$(dirname "$HERE")
LV=$FX/content/levels
W=$(mktemp -d)
mkdir -p "$W/src" "$W/out"

# 依 architecture §3.7 合併：缺結尾換行才補、五行開頭、tests.rs 缺換行也補、結尾 }
merge() {
  {
    cat "$1"; [ -z "$(tail -c1 "$1")" ] || printf '\n'
    printf '%s\n' '#[cfg(test)]' '#[allow(dead_code)]' '#[allow(unused_imports)]' 'mod __quest_hidden_tests {' '    use super::*;'
    cat "$2"; [ -z "$(tail -c1 "$2")" ] || printf '\n'
    printf '}\n'
  } > "$3"
}

cp "$HERE"/run/*.rs "$W/src/"
merge "$HERE/submit/user_main_only.rs" "$LV/02-plain-sum/tests.rs" "$W/src/hidden_submit.rs"
merge "$HERE/submit/user_warn.rs" "$HERE/submit/hidden_warn_tests.rs" "$W/src/warn_submit.rs"
merge "$LV/02-plain-sum/solution.rs" "$LV/02-plain-sum/tests.rs" "$W/src/sum_pass.rs"
merge "$LV/02-plain-sum/starter.rs" "$LV/02-plain-sum/tests.rs" "$W/src/sum_fail.rs"
merge "$HERE/submit/mixed_user.rs" "$HERE/submit/mixed_tests.rs" "$W/src/mixed.rs"
merge "$HERE/submit/user_main_only.rs" "$HERE/submit/hang_tests.rs" "$W/src/hang.rs"

cat > "$W/src/inner.sh" <<'IN'
set -u
cd /tmp
for f in e0382 syntax warn nomain many multiline; do
  cp /work/$f.rs /tmp/main.rs
  rustc --edition 2024 --error-format=json -A dead_code -o /tmp/main /tmp/main.rs 2> /out/$f.jsonl
done
for f in hidden_submit warn_submit sum_pass sum_fail mixed hang; do
  cp /work/$f.rs /tmp/main.rs
  rustc --edition 2024 --error-format=json -A dead_code --test -o /tmp/$f /tmp/main.rs 2> /out/$f.jsonl
done
H=__quest_hidden_tests
/tmp/sum_pass --test-threads=1 --exact $H::sums_three $H::empty_is_zero $H::all_ones > /out/all_pass.txt
/tmp/sum_fail --test-threads=1 --exact $H::sums_three $H::empty_is_zero $H::all_ones > /out/partial_fail.txt
/tmp/mixed --test-threads=1 > /out/mixed_with_player_test.txt
timeout -s KILL 3 /tmp/hang --test-threads=1 --exact $H::a_first $H::b_second $H::c_hangs $H::d_never > /out/killed_midway.txt
exit 0
IN
chmod 755 "$W" "$W/src"; chmod 644 "$W"/src/*; chmod 777 "$W/out"
docker run --rm --network none --memory 512m --read-only --tmpfs /tmp:rw,exec,size=128m \
  --user 1000:1000 -v "$W/src":/work:ro -v "$W/out":/out rust-quest-runner:dev sh /work/inner.sh

O=$W/out
cp "$O/e0382.jsonl" "$FX/rustc/run_e0382_move.jsonl"
cp "$O/syntax.jsonl" "$FX/rustc/run_syntax_error.jsonl"
cp "$O/warn.jsonl" "$FX/rustc/run_warnings_only.jsonl"
cp "$O/nomain.jsonl" "$FX/rustc/run_e0601_no_main.jsonl"
cp "$O/many.jsonl" "$FX/rustc/run_sixty_errors.jsonl"
cp "$O/multiline.jsonl" "$FX/rustc/run_multiline_span.jsonl"
cp "$O/hidden_submit.jsonl" "$FX/rustc/submit_hidden_e0425.jsonl"
cp "$O/warn_submit.jsonl" "$FX/rustc/submit_user_and_hidden_warning.jsonl"
for f in all_pass partial_fail mixed_with_player_test killed_midway; do cp "$O/$f.txt" "$FX/libtest/$f.txt"; done

# 合成 fixture（由真實輸出改寫）
sed -E "s/^(thread '[^']*') \([0-9]+\) panicked/\1 panicked/" "$O/partial_fail.txt" > "$FX/libtest/partial_fail_no_thread_id.txt"
{ head -1 "$O/e0382.jsonl"; echo 'error: linking with `cc` failed: exit status: 1'; tail -2 "$O/e0382.jsonl"; } > "$FX/rustc/synthetic_non_json_line.jsonl"
python3 - "$O/syntax.jsonl" "$FX/rustc/synthetic_ice_level.jsonl" <<'PY'
import sys, json
d = json.loads(open(sys.argv[1]).read().splitlines()[0])
d["level"] = "error: internal compiler error"
open(sys.argv[2], "w").write(json.dumps(d, ensure_ascii=False) + "\n")
PY
rm -rf "$W"
echo "fixtures 已重產"
