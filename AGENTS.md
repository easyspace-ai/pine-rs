# pine-rs · AGENTS.md

> 本文件是仓库内**唯一持续维护**的项目说明。  
> `AGENT.md`、`CLAUDE.md` 只做跳转，不维护独立规则；**链接必须使用相对路径**（见下文第 3 节），禁止写本机绝对路径，否则在其他机器或 CI 中会失效。

---

## 1. 当前项目定义

这个仓库包含两部分：

- **`pine-rs`**：Pine Script v6 本地执行内核（`crates/pine-*`、`pine-cli`）
- **`pine-tv`**：用于测试和验证内核的 Web 壳

当前阶段的唯一主目标是：

**把 `pine-rs` 做成稳定、可验证、可持续扩展的本地 Pine Script 内核。**

当前阶段**不是**要把 `pine-tv` 做成完整产品。

### 1.1 当前优先级（价值排序）

1. 内核正确性  
2. 测试与验收可信度（含黄金测试全链路）  
3. 内核缺陷修复与语义补齐  
4. `pine-tv` 仅做验证壳，不单独扩展产品需求  

### 1.2 关于 `pine-tv`

**当前是**：验证内核输出能否在 Web 中消费；基础图表、编辑器、运行入口；观察脚本与链路。

**当前不是**：主交付、进度判断基准、或能反向决定内核路线的产品层。

---

## 2. 整体进展与当前瓶颈

### 2.1 架构与骨架（进展）

Workspace 已具备完整内核骨架：`crates/` 下 lexer、parser、sema、eval、vm、runtime、`pine-builtin-macro`、`pine-stdlib`、`pine-output`、`pine-cli`、`pine-wasm` 等与 **`pine-tv`** 均已纳入工作区。说明在**架构层面**可以稳定迭代。

### 2.2 主线门禁与当前焦点（现状）

**Phase 3 关闭条件**见本文**第 12.3 节**：以 **`./scripts/dev_verify.sh --full` 全绿**为准（含 `tests/run_golden.sh`、`tests/vm_run_golden.sh`、`phase_acceptance` 等脚本所跑内容）。**该门禁须在每次合并/发布意图前保持通过**；一旦失败，应优先修复再继续扩张功能。

**当前阶段（Phase 4）**：在**不削弱**上述门禁的前提下，按**第 12.4 节**推进 `pine-vm` 与 `pine-eval` 的 parity 与能力扩展；可并行推进文档与仓库卫生（见第 3 节）。「代码能跑」仍须以**黄金与全量门禁**证明数值与行为可对齐本仓库基线。

---

## 3. Agent 入口、文档与仓库卫生

### 3.1 三套入口文件

根目录同时存在 `AGENTS.md`、`AGENT.md`、`CLAUDE.md` 时：

- **规则与任务以本文件（`AGENTS.md`）为准**。
- `AGENT.md` / `CLAUDE.md` 若保留，其唯一职责是**指向** `AGENTS.md`，且 Markdown 链接须写成**相对路径**，例如：`[AGENTS.md](./AGENTS.md)`。  
  **禁止**在跳转文件中使用本机绝对路径（例如 `/Users/.../AGENTS.md`），否则换机或 CI 会断链、工具可能读错上下文。

### 3.2 根目录调试残留

若仓库根目录存在 `debug_ast.rs` 等**不属于任何 crate** 的临时脚本：视为**卫生问题**——应删除，或迁入 `tests/scripts/regression/`（或 `examples/`）并配套正式回归方式，**不得**长期作为「隐形契约」依赖。

### 3.3 `three/` 目录

`three/` **不是**内核交付物的一部分；常见情形包含外部/试验性子项目（例如与 JS 转译、WASM 实验相关的目录）。**不作为** Phase 验收或进度依据。后续应二选一并在本文件或 `docs/GUIDE.md` 留一句定性：**正式收录（说明用途与边界）** 或 **删除/迁出主仓库**，避免新贡献者误以为属主线。

### 3.4 状态文件可信度

`.pine-rs-state/` 必须与真实验收一致。历史上若出现过「未过验证却标完成」，属于 **AI 自主开发的常见陷阱**：**禁止**把试验/原型记为完成；**禁止**用追加裸文本破坏 `completed_tasks.json` 的合法 JSON。完成后任务只能写在通过门禁之后，且须整体重写该 JSON。

---

## 4. 仓库结构（主线相关）

```text
pine-rs/
├── AGENTS.md
├── AGENT.md              # 可选：仅跳转至 ./AGENTS.md
├── CLAUDE.md             # 可选：仅跳转至 ./AGENTS.md
├── START_AUTONOMOUS.md
├── .pine-rs-state/
├── crates/
│   ├── pine-lexer/
│   ├── pine-parser/
│   ├── pine-sema/
│   ├── pine-eval/
│   ├── pine-vm/
│   ├── pine-runtime/
│   ├── pine-builtin-macro/
│   ├── pine-stdlib/
│   ├── pine-output/
│   ├── pine-cli/
│   └── pine-wasm/
├── pine-tv/
├── tests/
└── three/                # 试验/外部参考，非内核验收范围（见上文 3.3 节）
```

**职责边界**：

- `crates/pine-*` 与 `pine-cli`：内核主线。  
- `pine-tv`：验证壳。  
- **不允许**用 `pine-tv` 的产品需求反向定义内核「完成」。

---

## 5. 开发原则

### 5.1 主线原则

- 先修内核，再谈壳层体验。  
- **先修验证链路（含黄金）**，再声明功能完成。  
- 先修状态文件与本文档口径一致，再推进 Phase 编号与对外叙事。

### 5.2 文档原则

- 长期维护的**项目入口**只有本文件；产品与架构细节以 `docs/GUIDE.md` 为权威。  
- `docs/GUIDE.md` 与代码冲突时，优先修代码或在该文档中**明确**偏差与原因。

### 5.3 状态文件原则

- `.pine-rs-state/` 只记内核主线，不记 `pine-tv` 演示进度。  
- `current_phase.txt`：当前 Phase 编号。  
- `current_task.txt`：当前主任务（应对齐本文第 8 节分层）。  
- `completed_tasks.json`：始终合法 JSON。  
- `blockers.txt`：仅真实需要人工介入的阻塞。

---

## 6. 当前 Phase 口径

当前以**内核主线**计（与 `.pine-rs-state/current_phase.txt` 对齐）：

- **当前 Phase：`4`**  
- **当前目标**：保持 **`./scripts/dev_verify.sh --full` 全绿**（第 12.3 节 C1）；在此基础上按**第 12.4 节**推进 `pine-vm` 纵深与 eval 的 parity，不降低黄金与门禁标准。

### Phase 1

目标：词法与语法可稳定解析。  
验收：`cargo test -p pine-lexer`，`cargo test -p pine-parser`。

### Phase 2

目标：基础执行与 series 语义可运行。  
验收：`cargo test --workspace`；`sma_manual` 类等算例可运行且可对比。

### Phase 3（已关闭）

目标：标准库与 CLI 输出进入**可验证**状态。  
验收：本文**第 12.3 节** checklist（C1–C6）及 **`dev_verify.sh --full`**；黄金脚本与 CSV、`compare_golden.py`、`pine-cli` JSON `outputs` 键对齐；`ta.*` 极值族与 `for` 等有回归锁定。

### Phase 4–6

**Phase 4 为当前工作重心**（VM parity、语言子集扩展等）；Phase 5–6 仍为后续计划。仓库内试验代码**不自动视为** Phase 4「已完成」，须带独立验收与门禁。

---

## 7. 编码与验证要求

### 7.1 强制命令

- `cargo fmt --all`  
- `cargo clippy --workspace -- -D warnings`  
- `cargo test --workspace`  

### 7.2 库代码限制

- library crate 禁止 `unwrap()` / `expect()`。  
- `unsafe` 须注释说明。  
- `max_bars_back` 禁止硬编码。  
- `na` 传播须走统一逻辑。

### 7.3 每次改动后的验证顺序

1. 格式化与 lint  
2. 全量测试  
3. 涉及数值结果：跑黄金测试  
4. 涉及输出格式：核对 CLI / schema / 对比脚本是否一致  

---

## 8. 下一阶段任务分层（按优先级）

下列三层：**默认先保高层**；任何工作不得削弱 **`./scripts/dev_verify.sh --full`**（含 `run_golden` 与 VM 黄金）。

### 第一层：门禁不退化（持续）

1. **全量门禁绿**：`./scripts/dev_verify.sh` 与 **`--full`** 在主干上保持通过；新增黄金或改 CLI 输出格式时同步脚本与文档。  
2. **入口与状态一致**：`AGENT.md` / `CLAUDE.md` / `START_AUTONOMOUS.md` 等以相对路径指向 `./AGENTS.md`；`.pine-rs-state/` 与本文、与真实验收一致，禁止虚假完成。  
3. **Phase 3 已达成项的维护**：黄金配对、列名与 `compare_golden.py` 约定、`for` 与极值族回归等（见第 12.3 节）**不得回退**；回归失败优先修实现而非放宽阈值。

### 第二层：Phase 4 内核纵深（当前主投入）

1. **VM 与 eval 脚本级一致性**（第 12.4 节）：在**同一批真实 `.pine` 脚本**上对齐 VM 与 `pine-eval` 输出（数值与 plot/系列键），优先扩展黄金已覆盖脚本的双路径 parity；再审慎扩展 VM 指令集与可编译子集。  
2. **`pine-vm` 工程质量**：调试输出仅走显式开关；保持 `cargo clippy --workspace -- -D warnings`；按需收敛测试里的告警噪声。  
3. **仓库卫生**（穿插）：根目录 `debug_ast.rs`、`three/` 的定性或迁出（第 3 节）——不替代第 1 条主线。

### 第三层：`pine-tv` 与长期能力（与 Phase 4 可并行，不降门禁）

- **Phase 4（内核）**：例如 UDT（`type` / method）、`array.*`、`map.*`、`str.*` 等扩展——支撑更复杂社区脚本；仍以本仓库测试与文档为验收，不以壳层观感为准。  
- **`pine-tv` 产品化 Phase A**：在**内核输出已稳定可消费**的前提下，可将验证壳升级为更完整的三栏 Playground、打通 `/api/run` 端到端等——与 Phase 4 **可并行**，因壳消费的是已稳定契约；**不得**用壳需求倒逼降低黄金门禁。

**显式不优先**（与原文一致，避免 scope 漂移）：

- 不把 `pine-tv` 界面当成 Phase 3 完成标准。  
- **暂不优先**：`library()` 导出表与 `import` 真文件加载闭环；嵌套 UDF / 高阶可调用值；把未验收 VM/并行/Web 演示直接记为「项目完成」。

---

## 9. 推荐命令

仓库根目录执行：

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

## 10. 状态更新规则

- 仅在实际通过验证后，才写入 `completed_tasks.json`。  
- `completed_tasks.json` 必须**整体合法重写**。  
- 试验、探索、原型：不写入完成列表。  
- 跨 Phase 但未通过该 Phase 验收：不提前记成完成。

---

## 11. 一句话基线

当前阶段要把 **`pine-rs` 做成稳定可信的本地 Pine Script 内核**；`pine-tv` 是验证壳，**不是**现阶段核心交付物。

---

## 12. Phase 3 工作规则（「四条」+ VM 门槛）

本节是 **Phase 3 落地与排期** 的操作规则：**先满足本节，再系统性扩张 `pine-vm` 能力与 Phase 4 范围**；避免无验收尺子的扩张。

### 12.1 适用范围与显式延后

- **范围**：**单脚本**指标/策略语义；与 Pine v6 / TradingView **可核对的数值与行为**对齐，以**本仓库黄金测试 + 文档**为证据，不以 `pine-tv` 外观为准。  
- **显式延后（Phase 3 不要求完成）**：  
  - `library()` **导出表**与 `import` **真实文件加载**闭环；  
  - **嵌套 UDF**、带 AST 体的 **高阶可调用值**（当前 `Closure` 类能力）；  
  - 把 **VM / 并行 / Web 演示** 等试验代码直接记为「Phase 3 已完成」。

### 12.2 「四条」优先事项 — 定义与完成标准（DoD）

以下四条按顺序推进；每一条的 **DoD** 为门禁，未完成则不调低 Phase 3 标准去赶别的线。

1. **状态与说明一致**  
   - **内容**：`.pine-rs-state/` 与本文口径一致；`completed_tasks.json` 合法且不与「未完成却标完成」冲突。  
   - **DoD**：`current_phase.txt`、`current_task.txt` 反映当前焦点（与第 8 节一致）；无虚假完成。

2. **黄金测试链路可靠**  
   - **内容**：`pine-cli run`、黄金 CSV、`tests/compare_golden.py`、`tests/golden/` 与 `tests/scripts/` 的配对约定明确且无 **silently skip**。  
   - **DoD**：`./scripts/dev_verify.sh --full` 中 **`bash tests/run_golden.sh` 全绿**；增删黄金文件须同步对比脚本或本文 / `docs/GUIDE.md` 的约定。  
   - **参考命令**：见第 9 节。

3. **`ta.*` / `plot`（及与显示相关的输出）黄金覆盖**  
   - **内容**：已实现且对外承诺的 **`ta.*`、关键 `math.*`**（若该函数是指标核心路径）及 **`plot` 写入的序列**，应有黄金样例，或在第 12.3 节 checklist / 豁免表中写明「刻意不覆盖」及原因。  
   - **DoD**：计划内函数（含 `highest` / `lowest` / `highestbars` / `lowestbars` 等）**补齐或修复** `.pine` + `tests/golden/*.csv` + `tests/compare_golden.py` 可追溯对比；新增函数时**默认**同步黄金或加豁免说明。  
   - **注意**：仅靠 `cargo test -p pine-stdlib` **不代替**黄金门禁；数值语义以黄金为准。

4. **`for` 循环语义与回归**  
   - **内容**：`for` / `for…in` 在 **bar 循环 + series** 下行为正确，且不引入与黄金不一致的静默错误。  
   - **DoD**：相关缺陷有 **单元测试或黄金脚本** 锁定；`./scripts/dev_verify.sh --full` 仍通过。

### 12.3 Phase 3 关闭 checklist（宣布「基线达成」前须满足）

未同时满足前，**不宣布** Phase 3（单脚本 TV 语义基线）结束。

| # | 项 | 说明 |
|---|----|------|
| C1 | `dev_verify --full` | `./scripts/dev_verify.sh --full` 通过（含 phase_acceptance 与 `run_golden`）。 |
| C2 | 黄金配对 | `tests/golden/<basename>.csv` 须在 `tests/scripts/**/<basename>.pine` 有配对；`run_golden.sh` 对缺脚本 **报错计失败**，不得静默跳过。 |
| C3 | 数值容差 | 黄金对比仅用 `tests/compare_golden.py` 默认阈值（当前 `1e-8`）；若改容差须改脚本并在本表或 `docs/GUIDE.md` 说明。 |
| C4 | 列名对齐 | 黄金 CSV 系列列与 `pine-cli --format json` 的 `outputs` 键一致，或符合 `compare_golden.py` 的列名规则。 |
| C5 | `ta.*` 极值族黄金 | `ta.highest` / `ta.lowest` / `ta.highestbars` / `ta.lowestbars` 均有黄金样例（与同段 `tests/data` 约定一致）。 |
| C6 | `for` 回归 | 对 **inclusive `for` + `:=` 累加**等有锁定用例；`cargo test --workspace` 通过。 |

**豁免**：某函数刻意不做黄金时，须在 `docs/GUIDE.md` 或本节追加「豁免 + 原因 + 替代验证」。

### 12.4 VM（`pine-vm`）推进门槛

- **前提**：第 12.2 节四条 DoD 已满足，且 **`tests/run_golden.sh` 可作为回归基线**。  
- **首轮目标（建议）**：VM 路径与现有 **`pine-eval` 路径对同一批黄金脚本输出一致**（parity），再扩展指令集与语言子集。  
- **禁止**：未与黄金对齐时，将 VM 标为「主执行引擎」或替代 eval 的**唯一**验收路径而不设 parity。  
- **文档**：Phase 4–6 在第 6 节仍为后续计划；VM 的阶段性完成须独立验收（parity + 本仓库强制命令 green），**不继承** Phase 3 的完成声明。

### 12.5 执行顺序

1. 以 **12.1–12.4** 为规则基线。  
2. 按 **12.2** 条 **1 → 4** 实施。  
3. 勾选并维护 **12.3** checklist。  
4. 再按 **12.4** 启动 VM 的 parity 驱动开发。

若规则与 `docs/GUIDE.md` 冲突，**先改实现或改 GUIDE 并在此引用**，避免多套标准。
