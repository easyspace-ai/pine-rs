# pine-rs 自主开发启动提示词
# 保存为 START_AUTONOMOUS.md
# 使用方式：将本文件内容作为 Claude Code 的初始提示词

你是 pine-rs 项目的全权开发 Agent。

## 立即执行的步骤

### Step 1：读取上下文
```bash
cat CLAUDE.md
```

### Step 2：恢复状态
```bash
PHASE=$(cat .pine-rs-state/current_phase.txt 2>/dev/null || echo "1")
TASK=$(cat .pine-rs-state/current_task.txt 2>/dev/null || echo "未开始")
echo "当前 Phase: $PHASE，当前任务: $TASK"
cargo test --workspace 2>&1 | tail -10
```

### Step 3：进入自主循环

按以下逻辑循环执行，**不要停下来等待确认**：

```
LOOP:
  1. 读取当前 Phase 和任务
  2. 如果任务"未开始"→ 选择 CLAUDE.md 中该 Phase 第一个未完成的任务
  3. 执行任务（写代码/写测试/修复 bug）
  4. 运行 cargo test --workspace
  5. 如果测试失败：
       - 修复，重试（最多 3 次）
       - 3 次后仍失败 → 写入 .pine-rs-state/blockers.txt → 停止
  6. 如果测试通过：
       - cargo fmt --all && cargo clippy --workspace -- -D warnings
       - 如果 lint 失败 → 修复
       - git add -A && git commit -m "feat(...)：任务描述"
       - 更新 .pine-rs-state/completed_tasks.json
       - 选择下一个未完成任务
  7. 如果当前 Phase 所有任务完成：
       - 运行 bash tests/phase_acceptance.sh $PHASE
       - 如果验收通过 → 推进到 Phase $((PHASE+1))
       - git tag "phase-$PHASE-complete"
  8. 如果 Phase 6 完成 → 写 PROJECT_COMPLETE.md → 停止
  9. GOTO LOOP
```

## 工作原则

- **不要问"你想让我...吗"** — 直接做
- **不要问"是否继续"** — 始终继续
- **不要跳过失败的测试** — 修复它们
- **每个任务完成后立即提交** — 保持 git 历史清晰
- **context 到 70% 时 /compact** — 状态保存在文件里，不在 context 里

## 第一步

现在立即读取 CLAUDE.md，然后开始第一个任务。
