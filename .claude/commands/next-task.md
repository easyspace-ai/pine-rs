---
# .claude/commands/next-task.md
description: 查看并开始下一个任务
---

执行以下步骤：

1. 读取 `.pine-rs-state/current_phase.txt` 获取当前 Phase
2. 读取 `.pine-rs-state/completed_tasks.json` 获取已完成任务
3. 对照 `CLAUDE.md` 第 6 节的任务清单，找到下一个未完成的任务
4. 将下一个任务写入 `.pine-rs-state/current_task.txt`
5. 输出任务描述和预期完成标准
6. 立即开始执行该任务
