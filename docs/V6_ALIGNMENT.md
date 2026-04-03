# Pine Script v6 对齐总表（pine-rs）

> **用途**：对照 **TradingView Pine Script v6**（以仓库内 [`docs/pinescriptv6`](./pinescriptv6) 官方文档镜像为口径），持续追踪 **语法、语义、内置 API、策略与验证** 与官方的差距。  
> **风格**：类似 **`three/PineTS`** 文档站中的 **API Coverage**（按 namespace、函数粒度打勾）；本表优先保证 **可执行、可维护**，细到「每个函数一行」的矩阵由 §8 所述机制增量补齐。

- **产品与门禁**仍以 [`AGENTS.md`](../AGENTS.md) 为准；本表是 **「还要走多远」** 的路线图，**不单独降低** `dev_verify.sh --full` 标准。
- **North Star**：支持 **原生官方语法** 写就的指标与策略（社区脚本高比例 **无需改写** 即可 parse + sema + 执行）；**解析层避免 TV 不存在的专属语法成为「主路径」**（例如 UDF 应以官方 `name(params) =>` 为主，见 §1）。

---

## 状态图例

| 标记 | 含义 |
|------|------|
| ✅ | 与官方子集对齐且本仓库有 **可重复的验收**（单测 / 黄金 / parity） |
| 🟨 | **部分实现**或行为与文档仍有已知差距 |
| ⛔ | **未实现**或仅占位 |
| 🔜 | **明确延后**（在 AGENTS / GUIDE 中声明 phase 或豁免） |

---

## 1. 语言表面（Lexer / Parser / AST）

**目标**：与官方 v6 **词法与文法一致**；社区常见脚本结构可解析，**不因 pine-rs 扩展语法误导用户**。

| # | 主题 | 官方 v6（口径） | pine-rs 现状 | 优先级建议 |
|---|------|-----------------|-------------|------------|
| L1 | **用户函数 UDF** | `name(params) => expr` 或块；无 `fn` | ✅ `name(params) => expr/body` 已成为首要路径；`fn` 保留为兼容层 | **P0 完成** |
| L2 | **`switch`** | 表达式 + 缩进臂 **`pattern => body`**；可无 scrutinee；默认支 **`=>`** | ✅ 已支持 `pattern => body` 缩进臂；无 scrutinee switch；默认支 `=>` | **P0 完成** |
| L3 | **`for ... in`** | `for x in arr`、`for [i,v] in arr` 等 | ✅ 已支持 array 和 series 迭代，含元组解构 | **P1 完成** |
| L4 | **`import` 路径** | 如 `user/lib/1 as m`（非必选字符串路径） | ✅ Parser 已支持官方语法；执行层为 stub | **P1 完成** |
| L5 | **`export`** | 修饰整段声明（含参数表与 `=>`） | ✅ AST 形态与官方一致；执行层为 stub | **P1 完成** |
| L6 | **`enum`** | 独立枚举 | **无**对应 keyword/stmt | **P2** |
| L7 | **`else if`** | 手册常见 | ✅ Parser 正确处理 `else` + `if` 组合，不依赖 `elif` | **P1 完成** |
| L8 | **复合赋值** | `%= ` 等 | 部分未进 `AssignOp` | **P2** |
| L9 | **类型前缀** | `simple` / `series` / `const` 等 | **未完整建模** | **P1**（与 sema 强相关） |
| L10 | **绘图/对象类型名** | `label`、`line`、`box`… 作类型 | 多落 **User**；规则未钉死 | **P1** |
| L11 | **编译器指令** | `//@version=6` 等 | 多数字面 **`//` 丢弃**，无结构化 AST | **P2**（工具链） |
| L12 | **脚本入口** | `indicator()` / `strategy()` / `library()` | 多为普通调用；边界待脚本锁定 | **P1** |

**详细叙述与难度估计**：见（保留）[`GAP_ANALYSIS.md`](./GAP_ANALYSIS.md) §1–§4；**本表 §1 为执行层 checklist**。

---

## 2. 语义与运行时（sema / eval / vm）

| # | 主题 | 官方期望 | pine-rs 现状 | 标记 |
|---|------|----------|-------------|------|
| S1 | **`pine-sema` 真接入** | 类型、作用域、`var`/`varip`、history 规则与执行一致 | eval 主线 **未完全由 analyze 驱动** | 🟨 |
| S2 | **`switch` 运行时** | 与 TV 表达式 switch 一致 | ✅ 已支持有/无 scrutinee 的 switch；默认臂 `=>` | ✅ |
| S3 | **UDF 闭包 / 调用点隔离** | TV 规则 | 有调用点隔离实现；**与 TV 全语义仍有差距** | 🟨 |
| S4 | **UDT / `method`** | 与官方对象模型一致 | 解析/执行 **未闭环** | 🟨 |
| S5 | **`request.*`** | 多周期等 | AGENTS：Phase 5 前可 **na 占位**，与 TV **不等价** | 🔜 |
| S6 | **`pine-vm` 相对 eval** | 可选加速 | parity 见 AGENTS §12.4 | 🟨 |

---

## 3. 内置 API — 按 namespace（Pinets 式总览）

**官方分面**：见 `docs/pinescriptv6/reference/functions/`（`general.md`、`ta.md`、`collections.md`、`drawing.md`、`strategy.md`、`request.md` 等）。

**pine-rs 实现入口**：`crates/pine-stdlib`（`ta`、`math`、`str`、`array`、`map`、`color`、`input`）；**绘图/策略**多在 `pine-output` 与 eval 管线。

| Namespace / 区域 | 官方参考（本仓库镜像） | pine-rs 现状摘要 | 验证方式 |
|-------------------|------------------------|------------------|----------|
| ** builtins / 全局 ** | `general.md`、变量与关键字 | OHLCV、`bar_index`、部分内建；与 TV 全集 **未闭合** | phase + 脚本 |
| **math.*** | 手册 math | **较大子集**已实现 | 单测 |
| **str.*** | 手册 string | **较大子集**已实现 | 单测 |
| **ta.*** | `ta.md` | **子集**；黄金已锁 **多支**关键函数（`run_golden`） | 🟨→✅（按函数补） |
| **array.*** | `collections.md` | **多函数**；`array.new_box` / `new_label` / `new_line` 等 **多数未做** | 注册表 + 规划黄金 |
| **map.*** | `collections.md` | **基础 CRUD** 子集 | 单测 |
| **color.*** | visuals + types | **子集** | 单测 |
| **input.*** | `input` | 常用 **input.* ** 已有 | 单测 |
| **绘图 plot / hline / shape…** | `drawing.md`、`plots.md` | `plot` 等与 **CLI/黄金** 打通；高级绘图 **未全覆盖** | 黄金 + pine-output |
| **label / line / box / table** | `objects`、drawing | **部分** / 对象模型未完整 | ⛔→🟨 |
| **strategy.*** | `strategy.md` | **非目标完整撮合**；信号级子集 **待规划** | AGENTS 第三层后 |
| **request.*** | `request.md` | **占位 / na** | 🔜 Phase 5+ |
| **alert / runtime.error** | 手册 | **未对齐** | ⛔ |

**细粒度「函数 × 行」矩阵**：见 [`FUNCTION_COVERAGE.md`](./FUNCTION_COVERAGE.md)（按 namespace 的函数级清单，随 Phase 5 持续更新）。

---

## 4. 指标 vs 策略

| 能力 | 说明 | 标记 |
|------|------|------|
| **指标（indicator）** | 以当前 **黄金 + pine-cli JSON outputs** 为主战场 | 🟨 |
| **策略（strategy）** | 含 `strategy.*`、仓位与回测语义；**超出「仅表达式求值」** | ⛔ / 🔜 |

---

## 5. 与 PineTS「API Coverage」的对应关系

- PineTS 使用 `builtin.json` 等维护 **函数级** ✅/空白状态。
- 本仓库已有启发式对比：**[`BUILTIN_GAP_REPORT.md`](./BUILTIN_GAP_REPORT.md)**（PineTS manifest vs `pine-stdlib` 注册启发式）。
- **推荐后续**：增加 **单一生成脚本**（从 `docs/pinescriptv6` 或 registry 导出 Markdown/JSON），在 CI 或手动发布 **「函数级 coverage 页」**，与本文件 **§3 联动**。

---

## 6. 验收与「可宣称兼容」

| 关卡 | 说明 |
|------|------|
| **门禁** | `./scripts/dev_verify.sh --full` **永远不降级**（AGENTS §12.3） |
| **语法** | TV 官方样例 / 社区片段 **parse + sema** 抽样集（待建目录或 fixtures） |
| **数值** | **黄金** + 容差约定（`compare_golden.py`） |
| **双路径** | eval / vm **parity**（AGENTS §12.4） |

---

## 7. 建议阶段路线（与 AGENTS 映射）

| 阶段（建议命名） | 重点 | 与当前 AGENTS |
|------------------|------|----------------|
| **4 — Syntax-native 指标基建** | §1 **L1–L5**（UDF `=>`、`switch` TV 形、`for..in`、`else if`、import/export 形态）；已完成的指标语义必须与官方一致；VM parity 保持但不扩张 | 当前 Phase **4** |
| **5 — API sweep** | §3 按 namespace 扩函数 + **函数级 coverage 表** | 语法对齐后接续 |
| **6 — Strategy subset** | 信号级 strategy、与 GUIDE 非目标 **显式修订** 后 | 长周期 |

---

## 8. 文档维护纪律

1. **本文件**随每条 PR 可改：语言项标 🟨→✅、namespace 行注「新增黄金」等。  
2. **大段叙事**仍可由 `GAP_ANALYSIS.md` 承担；**追踪 checklist 以本文件为准**。  
3. **AGENTS.md** 只保留 **门禁 + 阶段口号 + 指向本文件**，避免三份文档重复清单。  
4. **GUIDE.md** 与「不实现 X」类陈述若与 North Star 冲突，应 **改 GUIDE 并脚注日期**，以 AGENTS + 本表为 **当前产品意志**。

---

*版本：与仓库当前认知同步；语法或 registry 大改后须更新 §1 / §3。*
