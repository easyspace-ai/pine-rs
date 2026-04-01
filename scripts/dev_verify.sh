#!/usr/bin/env bash
# 内核改动后的标准验证（与 AGENTS.md 一致）。
#   ./scripts/dev_verify.sh         格式化 + clippy + 全量测试 + CLI smoke
#   ./scripts/dev_verify.sh --full  另含 Phase 1/2 验收与黄金测试

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FULL=0
if [[ "${1:-}" == "--full" ]]; then
  FULL=1
fi

echo "==> cargo fmt --all"
cargo fmt --all

echo "==> cargo clippy --workspace -- -D warnings"
cargo clippy --workspace -- -D warnings

echo "==> cargo test --workspace"
cargo test --workspace

echo "==> pine-cli check (smoke)"
cargo run -p pine-cli --quiet -- check tests/scripts/basic/hello.pine

if [[ "$FULL" -eq 1 ]]; then
  echo "==> phase acceptance 1"
  bash tests/phase_acceptance.sh 1
  echo "==> phase acceptance 2"
  bash tests/phase_acceptance.sh 2
  echo "==> golden tests"
  bash tests/run_golden.sh
fi

echo "OK: dev_verify.sh completed (full=$FULL)"
