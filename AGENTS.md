# pine-rs · AGENTS.md
# Pine Script v6 解释器 — 自主开发上下文

> 本文件是 Codex 的**唯一入口文件**。每次会话开始时自动读取。
> 所有开发决策、规范、命令都在这里。**不要跳过任何章节**。

---

## 0. 自主运行协议（最重要，先读这里）

你是 pine-rs 项目的**全权开发 Agent**。你的任务是：

1. 读取当前 Phase 的目标（见第 6 节）
2. 拆解成可执行的最小任务单元
3. 实现 → 测试 → 修复 → 提交，循环直到该任务通过
4. 任务完成后，**自动推进到下一个任务**，无需等待人工确认
5. 整个 Phase 完成后，**自动开始下一个 Phase**

### 停止条件（只有以下情况才停下来等待人类）

- `cargo test --workspace` 有失败，且你已经尝试修复超过 **3 次**仍未解决
- 遇到需要外部数据的决策（如：TradingView 参考输出的 CSV 文件缺失）
- 遇到架构层面的歧义，且 docs/ 目录下没有相关说明
- `git push` 失败（权限问题）

### 永不停止的情况

- 测试第一次失败 → 分析原因，修复，重跑
- 编译错误 → 修复，重试
- clippy 警告 → 修复，重试
- 快照 diff → 运行 `cargo insta accept`（新功能）或分析回归（已有功能）

---

## 1. 项目概览

**项目名称**：pine-rs  
**目标**：用 Rust 实现完整的 Pine Script v6 解释器运行时  
**核心文档**：`docs/GUIDE.md`（产品开发指导书，所有架构决策的依据）  
**当前 Phase**：读取 `.pine-rs-state/current_phase.txt`（不存在则从 Phase 1 开始）
**pinescript** `docs/pinescriptv6`(pinescript 官方相关文档，做为我们是否实现目标的参考)
```
pine-rs/
├── AGENTS.md                   ← 本文件
├── .Codex/
│   ├── agents/                 ← 子 Agent 定义
│   │   ├── tester.md
│   │   ├── reviewer.md
│   │   └── phase-driver.md
│   ├── commands/               ← 自定义命令
│   │   ├── run-phase.md
│   │   ├── run-tests.md
│   │   └── next-task.md
│   └── hooks/
│       └── post-commit.sh      ← 提交后自动运行测试
├── .pine-rs-state/             ← Agent 状态持久化
│   ├── current_phase.txt       ← 当前 Phase（1-6）
│   ├── current_task.txt        ← 当前任务描述
│   ├── completed_tasks.json    ← 已完成任务列表
│   └── blockers.txt            ← 阻塞项记录
├── crates/                     ← 各 crate 源码
├── tests/                      ← 测试脚本和黄金数据
└── docs/
    └── GUIDE.md                ← 产品开发指导书
    └── pinescriptv6             ← 官方pinescript v6说明书

```

---

## 2. 环境与构建命令

```bash
# 构建整个 workspace
cargo build --workspace

# 运行所有测试（主要验证命令）
cargo test --workspace

# 格式化
cargo fmt --all

# Lint（必须零警告）
cargo clippy --workspace -- -D warnings

# 快照测试审阅（新功能时接受，存在回归时审查）
cargo insta review
# 自动接受所有新快照（新功能开发时用）
cargo insta accept

# 运行单个 crate 的测试
cargo test -p pine-lexer
cargo test -p pine-parser
cargo test -p pine-sema
cargo test -p pine-eval
cargo test -p pine-stdlib

# 运行 CLI
cargo run -p pine-cli -- run <script.pine> --data <data.csv>
cargo run -p pine-cli -- check <script.pine>

# 性能基准
cargo bench -p pine-eval

# 完整验证（PR 前运行）
cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

---

## 3. 编码规范（强制执行）

### 绝对禁止

```rust
// ❌ 绝对禁止：library crate 中 unwrap/expect
some_option.unwrap()
some_result.expect("message")

// ❌ 绝对禁止：无注释的 unsafe
unsafe { ... }

// ❌ 绝对禁止：硬编码 max_bars_back
const MAX: usize = 500; // 应使用 RuntimeConfig.max_bars_back
```

### 必须遵守

```rust
// ✅ 库错误使用 thiserror
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token {0:?} at {1}")]
    UnexpectedToken(Token, Span),
}

// ✅ 用户可见错误使用 miette
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("type mismatch")]
pub struct TypeError {
    #[label("expected {expected}, found {found}")]
    pub span: SourceSpan,
    pub expected: PineType,
    pub found: PineType,
}

// ✅ na 传播：所有算术走 na_ops 模块
use crate::na_ops::add; // 不要内联 na 检查

// ✅ Series 对齐：if/else 两分支都必须 push
// 见 docs/SERIES_SEMANTICS.md
```

### 命名规范

| 对象 | 规范 | 示例 |
|------|------|------|
| AST 节点 | PascalCase | `BinExpr`, `IfStmt` |
| IR 操作码 | SCREAMING_SNAKE | `PUSH_SERIES` |
| 内置函数 | `ns_name` | `ta_sma`, `math_abs` |
| Series 类型 | `*Buf/*Id` | `SeriesBuf<f64>` |
| 测试函数 | `test_*` | `test_ema_initial_na` |

---

## 4. 测试规范

### 每次代码变更后的验证顺序

```bash
# Step 1: 格式 + lint
cargo fmt --all && cargo clippy --workspace -- -D warnings

# Step 2: 单元测试
cargo test --workspace

# Step 3: 快照处理
cargo insta review   # 有新快照时审阅；回归时拒绝

# Step 4（涉及 ta.* 或 series 语义时）: 黄金测试
bash tests/run_golden.sh
```

### 新增功能的 TDD 流程

1. 先在 `tests/scripts/` 写 `.pine` 测试脚本
2. 运行 `cargo test` → 确认测试失败（红灯）
3. 实现功能代码
4. 运行 `cargo test` → 确认通过（绿灯）
5. 运行 `cargo insta accept` → 提交快照
6. 对 `ta.*` 函数：在 `tests/golden/` 补充预期 CSV

### 不允许的操作

- 不允许跳过失败的测试（`#[ignore]`）
- 不允许 `unwrap()` 在 library crate 中
- 不允许快照有未审阅的 diff 就提交

---

## 5. Git 工作流

```bash
# 每完成一个原子任务就提交（至少每小时一次）
git add -A
git commit -m "feat(pine-eval): implement if/else series alignment enforcement"

# Commit 消息格式（Conventional Commits）
# feat(crate): 新功能
# fix(crate): bug 修复
# test(crate): 新增测试
# refactor(crate): 重构（无行为变化）
# perf(crate): 性能优化
# docs: 文档更新

# 分支策略
# main   → 始终可发布
# dev    → 当前 Phase 集成
# feat/* → 功能开发（从 dev 检出）
```

---

## 6. Phase 任务清单（自主执行的核心）

### 如何读取当前状态

```bash
cat .pine-rs-state/current_phase.txt    # 当前 Phase
cat .pine-rs-state/current_task.txt     # 当前任务
cat .pine-rs-state/completed_tasks.json # 已完成列表
```

### 如何更新状态

```bash
# 完成一个任务后
echo '任务描述' >> .pine-rs-state/completed_tasks.json

# 开始下一个任务
echo '新任务描述' > .pine-rs-state/current_task.txt

# 推进到下一个 Phase
echo '2' > .pine-rs-state/current_phase.txt
```

---

### Phase 1：Lexer + Parser（目标：任意 v6 脚本可解析）

**完成标准**：`cargo test -p pine-lexer && cargo test -p pine-parser` 全部通过，
且 `tests/scripts/` 下所有 `.pine` 文件可以无错解析。

#### 任务清单

- [ ] **P1-T1**：创建 Workspace 结构（Cargo.toml + 所有 crate 骨架）
- [ ] **P1-T2**：pine-lexer：Token 枚举定义（含所有关键字、操作符、字面量）
- [ ] **P1-T3**：pine-lexer：logos 驱动的 Lexer 实现
- [ ] **P1-T4**：pine-lexer：缩进处理（INDENT/DEDENT/NEWLINE 虚拟 token）
- [ ] **P1-T5**：pine-lexer：颜色字面量 `#RRGGBBAA` 解析
- [ ] **P1-T6**：pine-lexer：snapshot 测试覆盖所有 Token 类型
- [ ] **P1-T7**：pine-parser：AST 节点定义（Stmt + Expr 完整枚举，含 v6 新语法）
- [ ] **P1-T8**：pine-parser：语句解析（VarDecl/Assign/If/For/While/FnDef/TypeDef）
- [ ] **P1-T9**：pine-parser：Pratt 表达式解析（17 级优先级）
- [ ] **P1-T10**：pine-parser：v6 新语法（switch/type/method/??/import/export）
- [ ] **P1-T11**：pine-parser：错误恢复（多错误报告）
- [ ] **P1-T12**：pine-parser：snapshot 测试覆盖所有语法结构
- [ ] **P1-T13**：pine-parser：AST Display impl（调试打印）

**Phase 1 验收命令**：
```bash
cargo test -p pine-lexer && \
cargo test -p pine-parser && \
cargo run -p pine-cli -- check tests/scripts/basic/hello.pine
```

---

### Phase 2：核心执行引擎（目标：SMA 手算结果与 TV 一致）

**完成标准**：`tests/scripts/series/sma_manual.pine` 执行结果与
`tests/golden/sma_manual.csv` 误差 < 1e-10。

#### 任务清单

- [ ] **P2-T1**：pine-runtime/value.rs：Value 枚举（含 Na 传播规则）
- [ ] **P2-T2**：pine-runtime/na_ops.rs：所有算术/比较运算的 Na 传播集中实现
- [ ] **P2-T3**：pine-runtime/series.rs：SeriesBuf<T> 实现（push/index/max_len）
- [ ] **P2-T4**：pine-runtime/config.rs：RuntimeConfig（max_bars_back 等）
- [ ] **P2-T5**：pine-runtime/context.rs：ExecutionContext 骨架
- [ ] **P2-T6**：pine-sema/types.rs：PineType 枚举 + series 传染规则
- [ ] **P2-T7**：pine-sema/scope.rs：作用域 + 符号表（indexmap 实现）
- [ ] **P2-T8**：pine-sema/infer.rs：类型推断（不动点迭代）
- [ ] **P2-T9**：pine-sema：Series 标注 Pass（标记哪些变量需要 SeriesBuf）
- [ ] **P2-T10**：pine-sema：var/varip 提升 Pass
- [ ] **P2-T11**：pine-eval/eval_expr.rs：基础表达式求值（字面量/变量/二元运算/Na）
- [ ] **P2-T12**：pine-eval/eval_stmt.rs：VarDecl/Assign/If/For/While
- [ ] **P2-T13**：pine-eval/eval_stmt.rs：if/else series 对齐强制执行
- [ ] **P2-T14**：pine-eval/fn_call.rs：用户函数调用 + call-site series 隔离
- [ ] **P2-T15**：pine-eval/runner.rs：bar-by-bar 主循环
- [ ] **P2-T16**：pine-runtime：var/varip 持久化状态
- [ ] **P2-T17**：写测试脚本 `tests/scripts/series/sma_manual.pine`
- [ ] **P2-T18**：运行黄金测试，误差 < 1e-10

**Phase 2 验收命令**：
```bash
cargo test --workspace && \
cargo run -p pine-cli -- run tests/scripts/series/sma_manual.pine \
  --data tests/data/BTCUSDT_1h.csv | \
  python3 tests/compare_golden.py tests/golden/sma_manual.csv
```

---

### Phase 3：内置标准库 P1（目标：ta.*/math.* 通过黄金测试）

**完成标准**：`tests/golden/ta_*.csv` 和 `tests/golden/math_*.csv` 全部通过，
误差 < 1e-8。

#### 任务清单

- [ ] **P3-T1**：pine-stdlib/registry.rs：FunctionRegistry + hash dispatch
- [ ] **P3-T2**：pine-stdlib/ta.rs：ta.sma（含 na 初始化行为）
- [ ] **P3-T3**：pine-stdlib/ta.rs：ta.ema / ta.rma / ta.wma
- [ ] **P3-T4**：pine-stdlib/ta.rs：ta.rsi（Wilder 平滑）
- [ ] **P3-T5**：pine-stdlib/ta.rs：ta.macd / ta.bbands / ta.stoch
- [ ] **P3-T6**：pine-stdlib/ta.rs：ta.atr / ta.tr / ta.cci / ta.mom
- [ ] **P3-T7**：pine-stdlib/ta.rs：ta.highest/lowest/highestbars/lowestbars
- [ ] **P3-T8**：pine-stdlib/ta.rs：ta.crossover / ta.crossunder / ta.barssince
- [ ] **P3-T9**：pine-stdlib/math.rs：全部 math.* 函数
- [ ] **P3-T10**：pine-cli：CSV 数据加载（DataFeed trait 实现）
- [ ] **P3-T11**：pine-cli：`run` 子命令（JSON 输出）
- [ ] **P3-T12**：pine-cli：`check` 子命令（miette 错误输出）
- [ ] **P3-T13**：为每个 ta.* 函数添加黄金测试脚本 + 预期 CSV

**Phase 3 验收命令**：
```bash
cargo test -p pine-stdlib && \
bash tests/run_golden.sh ta && \
bash tests/run_golden.sh math
```

---

### Phase 4：完整语言特性（目标：90% 社区脚本可运行）

#### 任务清单

- [ ] **P4-T1**：pine-sema + pine-eval：UDT（type 定义、字段访问）
- [ ] **P4-T2**：pine-sema + pine-eval：method 绑定与调用
- [ ] **P4-T3**：pine-runtime + pine-stdlib：array<T> 完整实现
- [ ] **P4-T4**：pine-runtime + pine-stdlib：matrix<T> 实现
- [ ] **P4-T5**：pine-runtime + pine-stdlib：map<K,V> 实现
- [ ] **P4-T6**：pine-stdlib/str.rs：str.* 完整函数
- [ ] **P4-T7**：pine-stdlib/array.rs：array.* 完整方法
- [ ] **P4-T8**：pine-eval：switch/case/default 语句
- [ ] **P4-T9**：pine-eval：import/export/library 模块系统
- [ ] **P4-T10**：pine-stdlib/color.rs：color.* 函数
- [ ] **P4-T11**：UDT 和 array 专项测试套件

---

### Phase 5：输出层与策略（目标：策略信号与 TV 一致）

#### 任务清单

- [ ] **P5-T1**：pine-output/plot.rs：plot/plotshape/plotchar/plotarrow
- [ ] **P5-T2**：pine-output/drawing.rs：label.new + label.set_*（生命周期管理）
- [ ] **P5-T3**：pine-output/drawing.rs：box.new + box.set_*
- [ ] **P5-T4**：pine-output/drawing.rs：table.new + table.cell
- [ ] **P5-T5**：pine-output/strategy.rs：strategy.entry/exit/close
- [ ] **P5-T6**：pine-output/strategy.rs：pyramiding/commission/slippage 配置
- [ ] **P5-T7**：pine-output：hline/bgcolor/fill
- [ ] **P5-T8**：alertcondition 实现
- [ ] **P5-T9**：JSON 输出格式定稿 + schema 文档
- [ ] **P5-T10**：策略信号黄金测试

---

### Phase 6：性能优化 + VM（目标：10万 bar < 100ms）

#### 任务清单

- [ ] **P6-T1**：SeriesBuf<f64> 特化（消除 Value 装箱）
- [ ] **P6-T2**：变量访问改为 slot index（借鉴 Rhai）
- [ ] **P6-T3**：内置函数 hash dispatch + Bloom filter（借鉴 Rhai）
- [ ] **P6-T4**：pine-vm/compiler.rs：Typed HIR → 字节码编译器
- [ ] **P6-T5**：pine-vm/vm.rs：栈式 VM 执行引擎
- [ ] **P6-T6**：Rayon 并行（多脚本/多股票）
- [ ] **P6-T7**：criterion 性能基准套件
- [ ] **P6-T8**：验收：10万 bar + EMA/RSI/MACD 组合指标 < 100ms

---

## 7. 已知最难的地方（必读）

### 难点 1：Series 对齐（最容易出错）

```rust
// ❌ 错误：if 分支只在 true 时 push
if cond {
    my_series.push(value_a); // 只有 cond=true 时 push
}
// 此后 my_series[1] 的偏移量会错位！

// ✅ 正确：两个分支都必须 push
if cond {
    my_series.push(value_a);
} else {
    my_series.push(Value::Na); // false 分支也要 push Na
}
```

详见 `docs/SERIES_SEMANTICS.md`。

### 难点 2：函数 call-site series 隔离

```rust
// Pine Script：
// f(close)   和   f(high)   各自维护独立的 series 历史
// 两次调用的内部 var 状态完全隔离

// 实现：call-site key = (fn_name, call_site_span_id)
// ExecutionContext 为每个 key 维护独立的 SeriesSlotMap
```

### 难点 3：na 传播

```
na + 1     == na    （不是 1）
na * 0     == na    （不是 0）
na == na   == false  （不是 true，类似 NaN）
na(na)     == true
nz(na, 0)  == 0
```

所有运算必须经过 `na_ops` 模块，不要内联 na 检查。

### 难点 4：ta.* 函数的初始化行为

```pine
// ta.ema(close, 14) 的前 13 根 bar 值为 na
// 第 14 根 bar 才开始有值（用简单平均初始化）
// 这个行为必须与 TradingView 完全一致
```

对照 `docs/GUIDE.md` 第 5.6 节和 pine-lang 源码（批判性参考）。

---

## 8. 借鉴参考（按优先级）

| 来源 | 借鉴内容 | 注意 |
|------|----------|------|
| `docs/GUIDE.md` | 所有架构决策 | 权威文档 |
| Rhai 源码 | Dynamic 布局、slot index、hash dispatch | 无 series，动态类型 |
| pine-lang (xewkf) | Series/na/var 思路、ta.* 函数逻辑 | v4/v5，可能有 bug，必须用黄金测试验证 |
| chumsky 官方示例 | Pratt parsing、缩进处理 | 直接参考 |

---

## 9. 上下文管理（防止 context 爆满）

在 70% context 时开始注意，85% 时使用 /compact，90% 以上必须 /clear。

### 主动管理策略

- **完成每个 Phase 的 Task** 后：`/compact`，保留状态文件
- **开始新 Phase** 前：`/clear`，然后重新读取 AGENTS.md
- **长时间调试同一个 bug** 超过 3 次后：`/clear`，重新分析
- 状态持久化在 `.pine-rs-state/`，不依赖 context 记忆

### 会话恢复

每次新会话开始时执行：

```bash
# 读取当前状态
cat .pine-rs-state/current_phase.txt
cat .pine-rs-state/current_task.txt
cat .pine-rs-state/completed_tasks.json | python3 -m json.tool | tail -20

# 运行测试确认当前状态
cargo test --workspace 2>&1 | tail -30
```

---

## 10. 子 Agent 协作模式

运行多个 Codex 会话并行可以加速开发——使用 Writer/Reviewer 模式：一个 Agent 写代码，另一个审查；或者一个写测试，另一个写通过测试的代码。

### 推荐并行模式

```
主 Agent（Opus）：
  → 读取 Phase 任务清单
  → 拆解并分配给子 Agent
  → 聚合结果，运行集成测试
  → 提交到 dev 分支

子 Agent A（Sonnet）：实现模块代码
子 Agent B（Sonnet）：同步编写测试
子 Agent C（Haiku）：运行 lint + 格式化
```

### .Codex/agents/ 中的子 Agent 定义

- `tester.md`：专注写测试脚本和黄金测试 CSV
- `reviewer.md`：专注 code review，检查 na/series 语义
- `phase-driver.md`：专注 Phase 推进逻辑，管理 .pine-rs-state/

---

## 11. 快速参考：关键文件路径

```
docs/GUIDE.md                   ← 产品开发指导书（权威）
docs/SERIES_SEMANTICS.md        ← Series 语义详细说明
docs/NA_RULES.md                ← na 传播规则
.pine-rs-state/current_phase.txt
.pine-rs-state/completed_tasks.json
tests/scripts/                  ← .pine 测试脚本
tests/golden/                   ← 预期 CSV 输出
tests/run_golden.sh             ← 黄金测试运行脚本
crates/pine-runtime/src/series.rs    ← SeriesBuf 核心
crates/pine-runtime/src/na_ops.rs   ← na 传播（不要内联）
crates/pine-eval/src/runner.rs      ← bar-by-bar 主循环
crates/pine-stdlib/src/ta.rs        ← ta.* 内置函数
```

---

*pine-rs AGENTS.md v1.0 — 与 docs/GUIDE.md 配套使用*
