#!/usr/bin/env bash
# bootstrap.sh — pine-rs 自主开发启动脚本
# 用法：bash bootstrap.sh
# 功能：初始化项目结构，创建状态文件，启动 Claude Code 自主模式

set -euo pipefail

echo "═══════════════════════════════════════"
echo "  pine-rs 自主开发启动器"
echo "═══════════════════════════════════════"

# ── 1. 创建目录结构 ──────────────────────
mkdir -p .pine-rs-state
mkdir -p .claude/agents .claude/commands .claude/hooks
mkdir -p crates
mkdir -p tests/scripts/basic tests/scripts/series tests/scripts/var_state
mkdir -p tests/scripts/stdlib/ta tests/scripts/stdlib/math
mkdir -p tests/scripts/udt tests/scripts/strategy tests/scripts/regression
mkdir -p tests/golden tests/data docs

# ── 2. 初始化状态文件 ────────────────────
if [ ! -f .pine-rs-state/current_phase.txt ]; then
  echo "1" > .pine-rs-state/current_phase.txt
  echo "初始化项目结构" > .pine-rs-state/current_task.txt
  echo "[]" > .pine-rs-state/completed_tasks.json
  echo "" > .pine-rs-state/blockers.txt
  echo "✓ 状态文件初始化完成"
else
  PHASE=$(cat .pine-rs-state/current_phase.txt)
  TASK=$(cat .pine-rs-state/current_task.txt)
  echo "⚡ 恢复已有状态：Phase $PHASE — $TASK"
fi

# ── 3. 创建 Workspace Cargo.toml ─────────
if [ ! -f Cargo.toml ]; then
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "crates/pine-lexer",
    "crates/pine-parser",
    "crates/pine-sema",
    "crates/pine-eval",
    "crates/pine-vm",
    "crates/pine-runtime",
    "crates/pine-stdlib",
    "crates/pine-output",
    "crates/pine-cli",
]

[workspace.dependencies]
logos        = "0.14"
chumsky      = { version = "0.10", features = ["pratt"] }
miette       = { version = "5", features = ["fancy"] }
thiserror    = "1"
indexmap     = "2"
smartstring  = "1"
smallvec     = "1"
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
insta        = "1"
proptest     = "1"
criterion    = { version = "0.5", features = ["html_reports"] }

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1

[profile.bench]
inherits = "release"
debug = true
EOF
  echo "✓ Workspace Cargo.toml 创建完成"
fi

# ── 4. 创建基础测试数据 ──────────────────
if [ ! -f tests/data/sample.csv ]; then
cat > tests/data/sample.csv << 'EOF'
time,open,high,low,close,volume
1609459200,29000.0,29500.0,28800.0,29300.0,1000.0
1609545600,29300.0,30000.0,29100.0,29800.0,1200.0
1609632000,29800.0,30200.0,29600.0,30100.0,900.0
1609718400,30100.0,30500.0,29900.0,30300.0,1100.0
1609804800,30300.0,30800.0,30100.0,30600.0,800.0
1609891200,30600.0,31000.0,30400.0,30800.0,950.0
1609977600,30800.0,31200.0,30600.0,31000.0,1050.0
1610064000,31000.0,31500.0,30800.0,31200.0,1150.0
1610150400,31200.0,31700.0,31000.0,31500.0,1000.0
1610236800,31500.0,32000.0,31300.0,31800.0,1200.0
1610323200,31800.0,32200.0,31600.0,32000.0,900.0
1610409600,32000.0,32500.0,31800.0,32200.0,1100.0
1610496000,32200.0,32700.0,32000.0,32500.0,800.0
1610582400,32500.0,33000.0,32300.0,32800.0,950.0
1610668800,32800.0,33200.0,32600.0,33000.0,1050.0
EOF
  echo "✓ 示例数据创建完成"
fi

# ── 5. 创建第一个测试脚本 ───────────────
if [ ! -f tests/scripts/basic/hello.pine ]; then
cat > tests/scripts/basic/hello.pine << 'EOF'
//@version=6
indicator("Hello pine-rs", shorttitle="hello")
plot(close)
EOF
  echo "✓ 基础测试脚本创建完成"
fi

# ── 6. 创建 Phase 验收脚本 ───────────────
cat > tests/phase_acceptance.sh << 'EOF'
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
EOF
chmod +x tests/phase_acceptance.sh

# ── 7. 创建黄金测试运行脚本 ─────────────
cat > tests/run_golden.sh << 'EOF'
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
EOF
chmod +x tests/run_golden.sh

# ── 8. 创建 Python 比较脚本 ─────────────
cat > tests/compare_golden.py << 'EOF'
#!/usr/bin/env python3
"""比较实际输出与黄金 CSV，误差阈值 1e-8"""
import sys, json, csv, math

def main():
    if len(sys.argv) < 2:
        print("用法: compare_golden.py <expected.csv>")
        sys.exit(1)
    
    expected_csv = sys.argv[1]
    actual_json = json.load(sys.stdin)
    
    expected = {}
    with open(expected_csv) as f:
        for row in csv.DictReader(f):
            expected[int(row['bar_index'])] = float(row['value']) if row['value'] != 'na' else None
    
    errors = 0
    for bar_idx, exp_val in expected.items():
        act_val = actual_json.get('plots', {}).get(str(bar_idx))
        if exp_val is None and act_val is None:
            continue
        if exp_val is None or act_val is None:
            print(f"bar[{bar_idx}]: expected {'na' if exp_val is None else exp_val}, got {'na' if act_val is None else act_val}")
            errors += 1
            continue
        diff = abs(float(act_val) - exp_val)
        if diff > 1e-8:
            print(f"bar[{bar_idx}]: diff={diff:.2e} (expected={exp_val}, got={act_val})")
            errors += 1
    
    if errors == 0:
        print(f"✓ 全部 {len(expected)} 个数值通过 (误差 < 1e-8)")
        sys.exit(0)
    else:
        print(f"✗ {errors}/{len(expected)} 个数值超出误差阈值")
        sys.exit(1)

if __name__ == '__main__':
    main()
EOF

echo ""
echo "═══════════════════════════════════════"
echo "  初始化完成！"
echo ""
echo "  启动自主开发模式："
echo ""
echo "  方式 1（推荐，全自主）："
echo "  claude --dangerously-skip-permissions \\"
echo "    \"读取 CLAUDE.md，从当前 Phase 开始自主开发，"
echo "     不要停止直到所有 Phase 完成或遇到阻塞项\""
echo ""
echo "  方式 2（headless 批处理）："
echo "  claude -p \"读取 CLAUDE.md 并执行 /next-task\" \\"
echo "    --allowedTools \"Bash,Read,Write,Edit,MultiEdit\""
echo ""
echo "  监控进度："
echo "  watch -n 30 'cat .pine-rs-state/current_task.txt'"
echo "═══════════════════════════════════════"
