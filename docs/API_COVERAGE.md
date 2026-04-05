# Pine Script API — 维度覆盖（pine-rs）

**逐函数、与 PineTS 同结构的详细表**：[`docs/api-coverage/README.md`](./api-coverage/README.md)（自 PineTS 复制正文，并多一列 **pine-rs**；更新方式见该目录 `README.md`）。

与 [`three/PineTS/docs/api-coverage.md`](../three/PineTS/docs/api-coverage.md) **相同的分类维度**，下面从 **namespace / 能力面** 概括 **当前 Rust 内核（`crates/pine-*` + `pine-cli`）**；本仓库维护的 **已实现函数短清单** 见 [`FUNCTION_COVERAGE.md`](./FUNCTION_COVERAGE.md)，与官方差距总表见 [`V6_ALIGNMENT.md`](./V6_ALIGNMENT.md) §3。

---

## 图例（与 PineTS 一致）

- **✅** 已实现，且有单测 / 黄金测试 / parity 等可重复验收  
- **✔️** 已实现子集或仅有底层模型，**与 TradingView 全集或未与 eval 全线贯通**，仍需补全或加强测试  
- **❌** 未实现（或仅占位、无可用语义）  
- **⏳** 已排期但刻意延后（见 [`AGENTS.md`](../AGENTS.md)，例如 `request.*`）

---

## 维度对照表

| 维度（PineTS） | 状态 | pine-rs 现状（简要） |
|----------------|------|----------------------|
| **Input** | ✅ | `input.*` 常用子集在 [`pine-stdlib`](../crates/pine-stdlib/src/input.rs) 注册并单测覆盖（见 `FUNCTION_COVERAGE.md`） |
| **Math** | ✅ | `math.*` 较大子集；含 `math.log` / `math.log10`（与下栏「Log」不同） |
| **Technical Analysis** | ✔️ | `ta.*` 已注册 46 个函数，核心指标已有较广黄金覆盖；距官方 `ta` 全集仍有缺口 |
| **Array** | ✔️ | `array.*` 已注册 42 个函数；标量数组子集较完整，但 `array.new_box` / `new_label` / `new_line` 等对象数组仍未做 |
| **Box** | ✔️ | [`pine-output`](../crates/pine-output/src/drawing.rs) 有 box 对象与更新语义；脚本侧 **`box.new` / `box.*` 未与 eval 全线贯通** |
| **Chart** | ❌ | `chart.*` namespace **未实现** |
| **Color** | ✅ | `color.*` 子集在 `pine-stdlib`，单测覆盖 |
| **Label** | ✔️ | 输出层含 label 模型与 `label.style_*` 等解析；**`label.new` 等绘图 API 执行层未完整** |
| **Line** | ✔️ | 同 Label：数据结构为主，**`line.new` / `line.set_*` 等未完整对外** |
| **Linefill** | ❌ | **`linefill.*` 未实现** |
| **Log** | ❌ | 手册中 **`log.*`（调试日志）** 未对齐（注意：**`math.log` 已实现**，归入 Math） |
| **Map** | ✅ | `map.*` 已有 11 个基础函数，CRUD 主路径可用 |
| **Matrix** | ❌ | 运行时或有矩阵相关 **Value 占位**；**`matrix.*` 标准库未注册** |
| **Plot** | ✔️ | 已注册 `plot` / `hline` / `bgcolor` / `fill` / `plotshape` / `plotchar` / `plotarrow`；其中 `plot()` 的端到端验证最完整 |
| **Request** | ⏳ | 按 **`AGENTS.md`**：`request.*` **占位 / 与 TV 不等价**，Phase 5+ 再推进 |
| **String** | ✅ | `str.*` 已注册 22 个函数，已不是少量占位状态 |
| **Strategy** | ✔️ | [`strategy.rs`](../crates/pine-stdlib/src/strategy.rs) 当前提供 6 个信号级函数与常量；**完整撮合与 TV 策略语义非当前目标** |
| **Table** | ✔️ | `pine-output` 含 table 模型；**`table.new` / `table.cell` 等脚本 API 未完整** |
| **Syminfo** | ❌ | **`syminfo.*` 无系统实现** |
| **Runtime** | ✔️ | 如 **`bar_index`**、OHLCV 等随 runner 注入；**`runtime.error`、`timenow` 等与官方 runtime 命名空间仍大半未对齐** |
| **Polyline** | ❌ | **`polyline.*` 未实现** |
| **Others** | ✔️ | 混杂项：`indicator()` / `library()` 等 **多为普通调用形态**，真入库与 **alert / alertcondition / fill** 等 **未闭环**；与 PineTS manifest 的粗对比见 [`BUILTIN_GAP_REPORT.md`](./BUILTIN_GAP_REPORT.md) |

---

## 和 PineTS 文档站的关系

- PineTS 在 `api-coverage/*.md` 里按 namespace **逐项勾函数**；本仓库镜像目录 **`docs/api-coverage/`** 保留 PineTS **Status** 列，并增加 **pine-rs** 列（由 `scripts/annotate_api_coverage_pinets.py` 生成，可手改纠错）。这些逐函数页偏向“和 PineTS 对照”，不等同于 registry 实时快照。  
- 另一份 **仅 pine-rs 已实现项** 的清单：**`FUNCTION_COVERAGE.md`**；与 PineTS builtin 的启发式差异见 **`BUILTIN_GAP_REPORT.md`**。  
- 本页 **只作维度总览**，细节以 **`docs/api-coverage/`** 与 **`FUNCTION_COVERAGE.md`** 为准。

---

## 维护说明

- 合并 **新增 namespace 或大批新内置函数** 时：更新 **`FUNCTION_COVERAGE.md`**，并视情况调整本表对应行与 **`V6_ALIGNMENT.md` §3**。  
- 验收仍 **`./scripts/dev_verify.sh` / `--full`** 为准（见 `AGENTS.md`）。
