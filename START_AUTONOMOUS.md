# pine-rs 自主开发启动提示

使用本仓库时，统一先读取 [AGENTS.md](./AGENTS.md)。

建议启动顺序：

```bash
cat AGENTS.md
cat .pine-rs-state/current_phase.txt
cat .pine-rs-state/current_task.txt
cat .pine-rs-state/completed_tasks.json
cargo test --workspace
```

当前基线：

- 主线只看 `pine-rs` 内核
- `pine-tv` 仅作为验证壳
- 当前重点是修复黄金测试链路并稳定 Phase 3
