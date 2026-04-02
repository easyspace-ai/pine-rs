# 参考项目：PineTS、pinecone、pine-lang

本文记录仓库内 `three/` 目录下三个外部参考项目的定位，以及它们对 **pine-rs** 后续开发的参考价值。

- 项目目标、Phase 与验收口径以根目录 [AGENTS.md](../AGENTS.md) 为准；本文不定义完成标准。
- 参考源码路径（本仓库内）；若随子模块/拷贝更新，以实际目录为准：`three/PineTS`、`three/pinecone`、`three/pine-lang`。

---

## 1. PineTS（TypeScript：转译 + JavaScript 运行时）

**定位：** 将 Pine 转译为可在 Node / 浏览器中执行的 JS，并实现大量 `ta`、`math`、`array` 等 namespace；配套文档与测试覆盖较完整。

### 对 pine-rs 的参考价值

| 领域 | 说明 |
|------|------|
| **语义与架构** | `docs/architecture/` 中对「Series 优先、历史存储与下标方向、按 bar 增量计算、调用状态隔离」等有系统描述，与内核需面对的问题一致，适合作为**对照清单**，而非照搬实现。 |
| **标准库覆盖面** | `docs/api-coverage/`、`docs/api-coverage/pinescript-v6/builtin.json` 等清单式进度，可用于比对 **ta / math / plot** 等缺口与优先级；**不替代**黄金测试与 AGENTS 中的 Phase 定义。 |
| **回归与基线** | `tests/compatibility/`：基于指标代码 + 历史数据生成 `.expect.json`、锁定行为，与仓库内 `tests/run_golden.sh` / 黄金测试**思路相近**。可借鉴：浮点特殊值序列化、将「兼容性回归」与「正确性证明」分层等流程。 |
| **代码组织** | `src/namespaces/*` 按领域/方法拆分，对扩展 `pine-stdlib` 的**目录与命名**有参考意义。 |

### 局限

- 执行模型是**转译到 JS**，与 pine-rs 的 **Rust 词法/语义/求值** 不同，不可直接移植代码。
- 许可证为 AGPL / 商业双许可；若涉及大段复制需谨慎；**阅读架构与测试思路**通常无妨。

---

## 2. pinecone（Rust：解释器 + WASM）

**定位：** Rust workspace，与 pine-rs 的 crate 划分高度相似：`pine-lexer`、`pine-parser`、`pine-ast`、`pine-interpreter`、`pine-builtins`、`pine-builtin-macro`、`pine-wasm`、`pine-reference` 等。

### 对 pine-rs 的参考价值

| 领域 | 说明 |
|------|------|
| **crate 边界** | lexer / parser / ast / 解释器 / builtins / wasm 的分层，可用于复盘 pine-rs 各 `pine-*` crate 的职责，减少后续拆分或合并时的试错。 |
| **内置函数扩展** | `pine-builtin-macro` 与 `pine-builtins` 的组合，对应规模化注册 builtins、降低样板代码；在**大批量实现 ta/math** 时可作设计对标（实现仍应自主或与许可证要求一致）。 |
| **分发形态** | `pine-wasm` 体现「同一语义进浏览器」的路径，与长期 WASM 方向可作技术预研参照。 |
| **测试与对照** | `pine-reference` 等命名提示的参考/对照测试思路，与黄金基线可互相启发。 |

### 局限

- 文档与语义目标以 README 所称版本为准，**不等于** TradingView Pine v6 权威语义。
- 具体算法与实现需注意版权与「净室」边界；架构与 API 形状参考更安全。

---

## 3. pine-lang（Rust `nom` 解析 + 周边工具）

**定位：** 较小 workspace，核心为 `pine` crate（基于 `nom` 的解析等）；另有 `pine-ws`、`pine-doc`；仓库中还存在 `pine-vscode`、`pine-ls`、`pine-py` 等**未完全纳入**顶层 `Cargo.toml` members 的子项目。根目录 README 可能滞后，以各 crate 源码为准。

### 对 pine-rs 的参考价值

| 领域 | 说明 |
|------|------|
| **解析策略** | 可作为「`nom` 组合子风格」与当前 **手写 lexer + parser** 的**小样本对照**（与 pinecone、pine-rs 形成三角比较）。 |
| **工具链** | LSP、VS Code、WS、文档生成等，更适合记在 **Phase 4+ 或生态层**；当前 AGENTS 不以 `pine-tv` 或编辑器体验为主交付。 |

### 局限

- 体量和 v6 完整度可能不如 PineTS/pinecone；适合**局部借鉴**（例如解析写法、工具链骨架），不宜当作语义权威。

---

## 与当前主线的对齐建议（Phase 3 语境）

1. **测试与可信度：** 优先吸收 PineTS **兼容性/基线**流程与 **API 覆盖清单**方法，强化黄金测试可读性与 stdlib 缺口盘点。
2. **架构与扩展：** 对照 pinecone 的 **crate 划分** 与 **builtins + macro + wasm**，在扩容标准库或考虑 wasm 时减少设计往返。
3. **按需深入 pine-lang：** 仅在明确要做解析器实验或 editor 集成时投入时间。

可选后续工作（不在本文定义范围）：从 PineTS 的 builtin 清单与 pine-rs 现有黄金用例做**差集表**；或画 **pinecone crates ↔ pine-rs crates** 映射表，服务于 Phase 3「可验证边界」梳理。

---

## 「已迁入 / 计划中」对照（Wave × 来源 × pine-rs 落点）

| Wave | 来源 | pine-rs 落点 | 状态（约） |
|------|------|----------------|------------|
| 1 | PineTS 状态隔离 + 本仓 `ExecutionContext` | `pine-runtime::ExecutionContext`（`var_scoped`、`SeriesKey`、`push_to_series` 按 bar 去重）、`pine-eval` 与运行时桥接、稳定调用点 intern（全源码 span）、`lex_with_indentation` 全局字节 span | **已接入主线**（持续打磨） |
| 2 | pinecone `BuiltinFunction` 思路 | `crates/pine-builtin-macro`（`#[pine_builtin]`）+ `pine-stdlib`：`math` 中大量单参函数已宏注册（`exp`/`log`/三角/双曲/`round` 族/`isna`/`tostring` 等），变参与特殊语义仍手写 | **基建 + 扩大试点** |
| 3 | PineTS 兼容性流程 | `tests/compatibility/`、`scripts/run_compat_smoke.sh`、`scripts/report_builtin_gap.py` → `docs/BUILTIN_GAP_REPORT.md`；真实基线示例：`expect/pinets_math_abs.expect.json`（来自 PineTS `math.abs`）+ `data/BTCUSDC_1d_pinets_abs_window.csv`、`run_pinets_abs_compare.sh` | **脚手架**（非黄金门禁） |
| 4 | pinecone `pine-wasm` | `crates/pine-wasm`（`wasm32`：`checkScript` / `runScriptJson`）；`pine-eval` 的 `parallel` feature 供 WASM 关 rayon | **薄导出已建** |
| 文档 | 三家对照 | 本文 + `docs/GAP_ANALYSIS.md` | **随验证更新** |

---

## 修订记录

- 2026-04-02：初稿（基于 `three/` 目录现状整理）。
- 2026-04-01：补 Wave 对照表与已迁入项（ builtins 宏、compat 脚本、pine-wasm、lexer 全局 span）。
