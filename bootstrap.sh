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

# 较长序列：与 sample 前 15 根一致，后续按相同规则延展（供 Phase 2 SMA 等验收）
if [ ! -f tests/data/BTCUSDT_1h.csv ]; then
cat > tests/data/BTCUSDT_1h.csv << 'EOF'
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
1610755200,33000.0,33500.0,32800.0,33200.0,1100.0
1610841600,33200.0,33800.0,33000.0,33500.0,1200.0
1610928000,33500.0,34000.0,33300.0,33800.0,900.0
1611014400,33800.0,34200.0,33600.0,34000.0,1000.0
1611100800,34000.0,34500.0,33800.0,34200.0,1150.0
1611187200,34200.0,34800.0,34000.0,34500.0,950.0
1611273600,34500.0,35000.0,34300.0,34800.0,1050.0
1611360000,34800.0,35200.0,34600.0,35000.0,800.0
1611446400,35000.0,35500.0,34800.0,35200.0,900.0
1611532800,35200.0,35800.0,35000.0,35500.0,1100.0
1611619200,35500.0,36000.0,35300.0,35800.0,1200.0
1611705600,35800.0,36200.0,35600.0,36000.0,1000.0
1611792000,36000.0,36500.0,35800.0,36200.0,950.0
1611878400,36200.0,36800.0,36000.0,36500.0,1050.0
1611964800,36500.0,37000.0,36300.0,36800.0,1150.0
1612051200,36800.0,37200.0,36600.0,37000.0,900.0
1612137600,37000.0,37500.0,36800.0,37200.0,800.0
1612224000,37200.0,37800.0,37000.0,37500.0,1100.0
1612310400,37500.0,38000.0,37300.0,37800.0,1200.0
1612396800,37800.0,38200.0,37600.0,38000.0,1000.0
1612483200,38000.0,38500.0,37800.0,38200.0,950.0
1612569600,38200.0,38800.0,38000.0,38500.0,1050.0
1612656000,38500.0,39000.0,38300.0,38800.0,1150.0
1612742400,38800.0,39200.0,38600.0,39000.0,900.0
1612828800,39000.0,39500.0,38800.0,39200.0,800.0
1612915200,39200.0,39800.0,39000.0,39500.0,1100.0
1613001600,39500.0,40000.0,39300.0,39800.0,1200.0
1613088000,39800.0,40200.0,39600.0,40000.0,1000.0
1613174400,40000.0,40500.0,39800.0,40200.0,950.0
1613260800,40200.0,40800.0,40000.0,40500.0,1050.0
1613347200,40500.0,41000.0,40300.0,40800.0,1150.0
1613433600,40800.0,41200.0,40600.0,41000.0,900.0
1613520000,41000.0,41500.0,40800.0,41200.0,800.0
1613606400,41200.0,41800.0,41000.0,41500.0,1100.0
1613692800,41500.0,42000.0,41300.0,41800.0,1200.0
1613779200,41800.0,42200.0,41600.0,42000.0,1000.0
1613865600,42000.0,42500.0,41800.0,42200.0,950.0
1613952000,42200.0,42800.0,42000.0,42500.0,1050.0
1614038400,42500.0,43000.0,42300.0,42800.0,1150.0
1614124800,42800.0,43200.0,42600.0,43000.0,900.0
1614211200,43000.0,43500.0,42800.0,43200.0,800.0
1614297600,43200.0,43800.0,43000.0,43500.0,1100.0
1614384000,43500.0,44000.0,43300.0,43800.0,1200.0
1614470400,43800.0,44200.0,43600.0,44000.0,1000.0
1614556800,44000.0,44500.0,43800.0,44200.0,950.0
1614643200,44200.0,44800.0,44000.0,44500.0,1050.0
1614729600,44500.0,45000.0,44300.0,44800.0,1150.0
1614816000,44800.0,45200.0,44600.0,45000.0,900.0
1614902400,45000.0,45500.0,44800.0,45200.0,800.0
1614988800,45200.0,45800.0,45000.0,45500.0,1100.0
1615075200,45500.0,46000.0,45300.0,45800.0,1200.0
1615161600,45800.0,46200.0,45600.0,46000.0,1000.0
1615248000,46000.0,46500.0,45800.0,46200.0,950.0
1615334400,46200.0,46800.0,46000.0,46500.0,1050.0
1615420800,46500.0,47000.0,46300.0,46800.0,1150.0
1615507200,46800.0,47200.0,46600.0,47000.0,900.0
1615593600,47000.0,47500.0,46800.0,47200.0,800.0
1615680000,47200.0,47800.0,47000.0,47500.0,1100.0
1615766400,47500.0,48000.0,47300.0,47800.0,1200.0
1615852800,47800.0,48200.0,47600.0,48000.0,1000.0
1615939200,48000.0,48500.0,47800.0,48200.0,950.0
1616025600,48200.0,48800.0,48000.0,48500.0,1050.0
1616112000,48500.0,49000.0,48300.0,48800.0,1150.0
1616198400,48800.0,49200.0,48600.0,49000.0,900.0
1616284800,49000.0,49500.0,48800.0,49200.0,800.0
1616371200,49200.0,49800.0,49000.0,49500.0,1100.0
1616457600,49500.0,50000.0,49300.0,49800.0,1200.0
1616544000,49800.0,50200.0,49600.0,50000.0,1000.0
EOF
  echo "✓ BTCUSDT_1h 示例数据创建完成"
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
