#!/usr/bin/env bash
# VM 黄金测试：使用 pine-vm 执行黄金脚本并对比输出

set -u

PASS=0
FAIL=0

# Colors for output (only if terminal supports it)
if [ -t 1 ]; then
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    NC='\033[0m'
else
    GREEN=''
    RED=''
    NC=''
fi

echo "═══ VM 黄金测试开始 ═══"
echo ""

# Run VM golden tests through cargo
cd "$(dirname "$0")/.."

# Run the specific vm_golden_test with detailed output
echo "运行 VM 黄金测试..."
if cargo test -p pine-vm --test vm_golden_test 2>&1 | grep -E "^test.*ok$|^test.*FAILED$"; then
    PASS=$((PASS+1))
    echo -e "${GREEN}✓${NC} VM golden tests passed"
else
    FAIL=$((FAIL+1))
    echo -e "${RED}✗${NC} VM golden tests failed"
fi

echo ""
echo "═══ VM 黄金测试结果：$PASS 通过，$FAIL 失败 ═══"
[ "$FAIL" -eq 0 ]
