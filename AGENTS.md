# pine-rs · AGENTS.md

> 本文件是仓库内唯一持续维护的项目说明。
> `AGENT.md`、`CLAUDE.md` 只做跳转，不再维护独立规则。

---

## 1. 当前项目定义

这个仓库包含两部分：

- `pine-rs`：Pine Script v6 本地执行内核
- `pine-tv`：用于测试和验证内核的 Web 壳

当前阶段的唯一主目标是：

**把 `pine-rs` 做成稳定、可验证、可持续扩展的本地 Pine Script 内核。**

当前阶段不是要把 `pine-tv` 做成完整产品。

### 当前优先级

1. 内核正确性
2. 测试与验收可信度
3. 内核缺陷修复与语义补齐
4. `pine-tv` 仅做验证壳，不单独扩展产品需求

### 关于 `pine-tv`

`pine-tv` 当前定位：

- 验证内核输出是否能在 Web 中消费
- 提供基础图表、编辑器、运行入口
- 帮助观察脚本结果与交互链路

`pine-tv` 当前**不是**：

- 本阶段的主交付
- 进度判断基准
- 可以反向决定内核路线的上层产品

---

## 2. 仓库结构

```text
pine-rs/
├── AGENTS.md
├── AGENT.md
├── CLAUDE.md
├── START_AUTONOMOUS.md
├── .pine-rs-state/
├── crates/
│   ├── pine-lexer/
│   ├── pine-parser/
│   ├── pine-sema/
│   ├── pine-eval/
│   ├── pine-vm/
│   ├── pine-runtime/
│   ├── pine-stdlib/
│   ├── pine-output/
│   └── pine-cli/
├── pine-tv/
└── tests/
```

职责边界：

- `crates/pine-*` 和 `pine-cli` 属于内核主线
- `pine-tv` 属于验证壳
- 不允许把 `pine-tv` 的需求当成内核完成标准

---

## 3. 当前开发原则

### 主线原则

- 先修内核，再谈壳层体验
- 先修验证链路，再声明功能完成
- 先修状态文件和文档一致性，再继续推进 Phase

### 文档原则

- 只有 `AGENTS.md` 是长期维护入口
- `docs/GUIDE.md` 是产品与架构权威说明
- 当 `docs/GUIDE.md` 与代码不一致时，优先修代码或补明确说明

### 状态文件原则

`.pine-rs-state/` 只记录**内核主线**，不记录 `pine-tv` 的演示或临时试验进展。

状态文件要求：

- `current_phase.txt`：只写当前内核 Phase 编号
- `current_task.txt`：只写当前内核主任务
- `completed_tasks.json`：必须始终是有效 JSON
- `blockers.txt`：只记录真正需要人介入的阻塞

禁止事项：

- 禁止把验证中的原型任务记成已完成
- 禁止把 Phase 4-6 的试验性代码直接记为“正式完成”
- 禁止用追加裸文本的方式修改 `completed_tasks.json`

---

## 4. 当前 Phase 口径

当前状态以“内核主线”计：

- 当前 Phase：`3`
- 当前目标：修复黄金测试链路，确认 Phase 3 的完成边界

### Phase 1

目标：词法与语法可稳定解析

验收基准：

- `cargo test -p pine-lexer`
- `cargo test -p pine-parser`

### Phase 2

目标：基础执行与 series 语义可运行

验收基准：

- `cargo test --workspace`
- `sma_manual` 类核心算例可运行且结果可对比

### Phase 3

目标：标准库与 CLI 输出进入“可验证”状态

当前真正未完成的部分，不是“再加更多函数”，而是：

- 黄金测试输入输出格式统一
- 验收脚本和实际输出结构统一
- 对 `ta.*`、`math.*`、plot 输出做可信核对

### Phase 4-6

这些仍然存在于长期路线里，但当前只视为**后续计划**，不以仓库中已有原型代码自动视为完成。

---

## 5. 编码与验证要求

### 强制要求

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

### 库代码限制

- library crate 中禁止 `unwrap()` / `expect()`
- `unsafe` 必须带明确注释
- `max_bars_back` 禁止硬编码
- `na` 传播必须走统一逻辑

### 每次改动后的验证顺序

1. 格式化与 lint
2. 全量测试
3. 涉及数值结果时，跑黄金测试
4. 涉及输出格式时，核对 CLI / schema / 测试脚本是否一致

---

## 6. 接下来优先做什么

当前最近一段时间，只做下面这些：

1. 修正状态文件与说明文件
2. 修复黄金测试链路
3. 修复 `for` 循环相关问题
4. 校准 Phase 3 完成标准
5. 在此基础上继续内核功能补强

明确不优先做的事：

- 不以 `pine-tv` 的界面扩展为当前重点
- 不以“看起来像 TradingView”为当前重点
- 不把未验收的 VM / 并行 / Web 演示代码直接视为项目完成

---

## 7. 推荐命令

一键验证（脚本在仓库根目录执行）：

```bash
./scripts/dev_verify.sh        # fmt + clippy + test --workspace + hello.pine check
./scripts/dev_verify.sh --full # 另含 phase_acceptance 1/2 与 run_golden.sh
```

等价拆分：

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace

cargo run -p pine-cli -- check tests/scripts/basic/hello.pine
cargo run -p pine-cli -- run tests/scripts/series/sma_manual.pine --data tests/data/BTCUSDT_1h.csv

bash tests/phase_acceptance.sh 1
bash tests/phase_acceptance.sh 2
bash tests/run_golden.sh
```

---

## 8. 状态更新规则

更新 `.pine-rs-state/` 时遵守：

- 只有在“通过验证”后，才能把任务写入 `completed_tasks.json`
- `completed_tasks.json` 必须整体改写，保持合法 JSON
- 如果只是试验、探索、原型，不写入完成列表
- 如果任务跨 Phase，但还没通过该 Phase 验收，不提前记成完成

---

## 9. 一句话基线

这个仓库当前阶段的主目标，是把 `pine-rs` 做成稳定可信的本地 Pine Script 内核；`pine-tv` 只是用于测试与验证的 Web 壳，不作为当前阶段的核心交付。
