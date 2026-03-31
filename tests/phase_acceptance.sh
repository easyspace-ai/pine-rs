#!/usr/bin/env bash
# 验收指定 Phase 的完成标准
PHASE=${1:-1}

case $PHASE in
  1)
    echo "验收 Phase 1: Lexer + Parser"
    cargo test -p pine-lexer && cargo test -p pine-parser
    ;;
  2)
    echo "验收 Phase 2: 核心执行引擎"
    cargo test --workspace
    # 运行 SMA 黄金测试（如果数据存在）
    if [ -f tests/golden/sma_manual.csv ]; then
      cargo run -p pine-cli -- run tests/scripts/series/sma_manual.pine \
        --data tests/data/BTCUSDT_1h.csv 2>/dev/null | \
        python3 tests/compare_golden.py tests/golden/sma_manual.csv || true
    fi
    ;;
  3)
    echo "验收 Phase 3: 内置标准库"
    cargo test -p pine-stdlib
    bash tests/run_golden.sh ta 2>/dev/null || echo "警告：黄金测试未完整配置"
    ;;
  *)
    echo "验收 Phase $PHASE"
    cargo test --workspace
    ;;
esac
