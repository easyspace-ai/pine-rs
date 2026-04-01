#!/usr/bin/env bash
# 运行黄金测试

set -u

NS=${1:-"all"}
PASS=0
FAIL=0

if [ "$NS" = "all" ]; then
  pattern="tests/golden/*.csv"
else
  pattern="tests/golden/${NS}*.csv"
fi

find_script() {
  local base="$1"
  find tests/scripts -name "${base}.pine" | head -n 1
}

for csv in $pattern; do
  [ -f "$csv" ] || continue

  base=$(basename "$csv" .csv)
  script=$(find_script "$base")
  [ -n "$script" ] || continue

  if cargo run -p pine-cli -- run "$script" --data "$csv" 2>/dev/null | \
     python3 tests/compare_golden.py "$csv"; then
    echo "✓ $csv"
    PASS=$((PASS+1))
  else
    echo "✗ $csv"
    FAIL=$((FAIL+1))
  fi
done

echo "═══ 黄金测试结果：$PASS 通过，$FAIL 失败 ═══"
[ "$FAIL" -eq 0 ]
