#!/usr/bin/env bash
# 验收指定 Phase 的完成标准

set -euo pipefail

PHASE=${1:-1}

case $PHASE in
  1)
    echo "验收 Phase 1: Lexer + Parser"
    cargo test -p pine-lexer && cargo test -p pine-parser
    ;;
  2)
    echo "验收 Phase 2: 核心执行引擎"
    cargo test --workspace --exclude pine-tv
    # 运行 SMA 黄金测试（如果数据存在）
    if [ -f tests/golden/sma_manual.csv ]; then
      cargo run -p pine-cli -- run tests/scripts/series/sma_manual.pine \
        --data tests/golden/sma_manual.csv 2>/dev/null | \
        python3 tests/compare_golden.py tests/golden/sma_manual.csv
    fi
    ;;
  3)
    echo "验收 Phase 3: 内置标准库"
    cargo test -p pine-stdlib
    bash tests/run_golden.sh
    ;;
  *)
    echo "验收 Phase $PHASE"
    cargo test --workspace --exclude pine-tv
    ;;
esac
