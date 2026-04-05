#!/usr/bin/env bash
# 运行黄金测试：每个 tests/golden/*.csv 必须在 tests/scripts 下存在同名 *.pine，否则失败（禁止静默跳过）。

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
  if [ -z "$script" ]; then
    echo "✗ $csv: 未找到 tests/scripts/**/${base}.pine"
    FAIL=$((FAIL+1))
    continue
  fi

  if cargo run -p pine-cli -- run "$script" --data "$csv" --engine eval 2>/dev/null | \
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
