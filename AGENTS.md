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
6. **Pine v6 / TradingView 语义基线（单脚本）**：在不要求 `library`/`import` 真加载、不要求嵌套 UDF 的前提下，用黄金测试与官方文档把 **`ta.*`、`math.*`、`na` 传播、series、`plot` 输出**等修到**可核对、可重复证明**；优先修已发现的数值与语义偏差，不一次性吞尽所有边缘语法。

明确不优先做的事：

- 不以 `pine-tv` 的界面扩展为当前重点
- 不以“**图表壳**看起来像 TradingView”为交付标准；**内核行为**以官方 Pine v6 与黄金测试为准
- 不把未验收的 VM / 并行 / Web 演示代码直接视为项目完成
- **暂不优先**：`library()` 导出表与 `import` **真文件加载**闭环、**嵌套 UDF / 高阶函数**（例如带 AST 的可调用 `Closure`）；待上述基线与黄金链路稳定后再展开

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

---

## 10. Phase 3 工作规则（「四条」+ VM 门槛）

本节是 **Phase 3 落地与排期** 的操作规则：**先满足本节，再系统性推进 `pine-vm` 能力**；避免无验收尺子的 VM 扩张。

### 10.1 适用范围与显式延后

- **范围**：**单脚本**指标/策略语义；与 Pine v6 / TradingView **可核对的数值与行为**对齐，以**本仓库黄金测试 + 文档**为证据，不以 `pine-tv` 的外观或手动画图为准。
- **显式延后（Phase 3 不要求完成）**：
  - `library()` **导出表**与 `import` **真实文件加载**闭环；
  - **嵌套 UDF**、带 AST 体的 **高阶可调用值**（当前 `Closure` 类能力）；
  - 把 **VM / 并行 / Web 演示** 等试验代码直接记为「Phase 3 已完成」。

### 10.2 「四条」优先事项 — 定义与完成标准（DoD）

以下四条按顺序推进；每一条的 **DoD** 为门禁，未完成则不调低 Phase 3 标准去赶别的线。

1. **状态与说明一致**  
   - **内容**：`.pine-rs-state/` 与 `AGENTS.md` 口径一致；`completed_tasks.json` 合法 JSON，且不与「未完成却标完成」冲突。  
   - **DoD**：`current_phase.txt`、`current_task.txt` 反映本节 10.2 的当前焦点；无虚假完成的条目。

2. **黄金测试链路可靠**  
   - **内容**：`pine-cli run`、黄金 CSV、`tests/compare_golden.py`、目录 `tests/golden/` 与 `tests/scripts/` 的配对约定明确且无「 silently skip」。  
   - **DoD**：`./scripts/dev_verify.sh --full` 中 **`bash tests/run_golden.sh` 全绿**；若增删黄金文件，须同时更新对比脚本或文档中的约定。  
   - **参考命令**：`./scripts/dev_verify.sh --full`（见 §7）。

3. **`ta.*` / `plot`（及与显示相关的输出）黄金覆盖**  
   - **内容**：已实现且对外承诺的 **`ta.*`、关键 `math.*`**（若该函数是指标核心路径）及 **`plot` 写入的序列**，应有对应黄金样例或在 §10.3 checklist / 豁免表中写明「刻意不覆盖」的例外与原因。  
   - **DoD**：对计划内函数（含 `highest`/`lowest`/`highestbars`/`lowestbars` 等缺口项）**补齐或修复** `.pine` + `tests/golden/*.csv` + `tests/compare_golden.py` 可追溯对比；新增函数时**默认**同步黄金或加豁免说明。  
   - **注意**：仅靠 `cargo test -p pine-stdlib` **不代替**黄金门禁；数值语义以黄金为准。

4. **`for` 循环语义与回归**  
   - **内容**：`for` / `for…in` 与指标中常见写法在 **bar 循环 + series** 下行为正确，且不引入与黄金不一致的静默错误。  
   - **DoD**：相关缺陷有 **单元测试或黄金脚本** 锁定；`./scripts/dev_verify.sh --full` 仍通过。

### 10.3 Phase 3 关闭 checklist（可宣布「基线达成」前须满足）

以下是对 §10.2 的 **可勾选清单**；**未同时满足前，不宣布 Phase 3（单脚本 TV 语义基线）结束**。

| # | 项 | 说明 |
|---|----|------|
| C1 | `dev_verify --full` | `./scripts/dev_verify.sh --full` 通过（含 phase_acceptance 与 `run_golden`）。 |
| C2 | 黄金配对 | `tests/golden/<basename>.csv` 必须在 `tests/scripts/**/<basename>.pine` 有配对；`run_golden.sh` 对缺脚本 **报错计失败**，不得静默跳过。 |
| C3 | 数值容差 | 黄金对比仅用 `tests/compare_golden.py` 默认阈值（当前 `1e-8`）；若改容差须改该脚本并在此表或 `docs/GUIDE.md` 中说明。 |
| C4 | 列名对齐 | 黄金 CSV 中系列列与 `pine-cli --format json` 的 `outputs` 键一致，或符合 `compare_golden.py` 的列名匹配规则。 |
| C5 | `ta.*` 极值族黄金 | `ta.highest` / `ta.lowest` / `ta.highestbars` / `ta.lowestbars` 均有黄金样例（与同段 `tests/data` 约定一致）。 |
| C6 | `for` 回归 | `pine-eval` 等对 **inclusive `for` + `:=` 累加**等有锁定用例；`cargo test --workspace` 通过。 |

**豁免**：某函数刻意不做黄金时，须在 `docs/GUIDE.md` 或本节追加「豁免 + 原因 + 替代验证」。

### 10.4 VM（`pine-vm`）推进门槛

- **前提**：§10.2 四条 DoD 已满足，且 **`tests/run_golden.sh` 可作为回归基线**。  
- **首轮目标（建议）**：VM 路径与现有 **`pine-eval` 路径对同一批黄金脚本输出一致**（parity），再扩展指令集与语言子集。  
- **禁止**：在未与黄金对齐的情况下，将 VM 标为「主执行引擎」或替代 eval 的**唯一**验收路径而不设 parity。  
- **文档**：Phase 4–6 在 §4 中仍为后续计划；VM 的阶段性完成需独立验收（parity + 本仓库强制命令 green），**不继承** Phase 3 的完成声明。

### 10.5 执行顺序（与你方约定的「先规则、后四条」）

1. 以 **本节 10.1–10.4** 为规则基线（本文档即为规则来源）。  
2. 按 **10.2 条 1 → 4** 实施开发与修复。  
3. 勾选并维护 **Phase 3 checklist**（§10.3）。  
4. 再按 **§10.4** 启动 VM 的 parity 驱动开发。

若规则与 `docs/GUIDE.md` 冲突，**先改实现或改 GUIDE 并在此处引用**，避免多套标准。
