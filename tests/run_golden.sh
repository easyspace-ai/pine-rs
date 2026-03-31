#!/usr/bin/env bash
# 运行黄金测试
NS=${1:-"all"}
PASS=0; FAIL=0

for csv in tests/golden/${NS}*.csv; do
  [ -f "$csv" ] || continue
  SCRIPT="${csv/golden/scripts}"
  SCRIPT="${SCRIPT/.csv/.pine}"
  [ -f "$SCRIPT" ] || continue
  
  if cargo run -p pine-cli -- run "$SCRIPT" --data tests/data/sample.csv 2>/dev/null | \
     python3 tests/compare_golden.py "$csv" 2>/dev/null; then
    echo "✓ $csv"
    PASS=$((PASS+1))
  else
    echo "✗ $csv"
    FAIL=$((FAIL+1))
  fi
done

echo "═══ 黄金测试结果：$PASS 通过，$FAIL 失败 ═══"
[ $FAIL -eq 0 ]
