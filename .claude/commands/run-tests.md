---
# .claude/commands/run-tests.md
description: 运行完整测试套件并处理结果
---

按顺序执行：

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo insta review --accept-all  # 只在新功能时用；有回归时用 cargo insta review
```

如果有测试失败：
1. 分析失败原因（读取错误信息）
2. 定位问题代码
3. 修复
4. 重新运行测试
5. 重复直到全部通过

如果 3 次修复后仍失败，将问题记录到 `.pine-rs-state/blockers.txt` 并停止。
