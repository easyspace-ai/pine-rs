**pine-rs**

Pine Script v6 解释器

产品开发指导书

  ---------------------------- ------------------------------------------
  **版本**                     v1.0

  **状态**                     正式发布

  **语言**                     Rust 2021 Edition

  **目标版本**                 Pine Script v6
  ---------------------------- ------------------------------------------

**前言**

本文档是 pine-rs 项目的唯一权威开发指导书，涵盖技术选型依据、完整架构设计、模块划分职责、编码规范、测试体系和分阶段交付计划。所有开发决策均应以本文档为准；若文档与代码不一致，优先修正代码。

文档约定：

- 粗体代码片段（如 SeriesBuf\<T\>）代表正式的类型/函数名，应与代码库保持一致

- 标注「关键」的内容是已知最容易出错的地方，务必重视

- 标注「决策」的内容说明为何选择当前方案而非备选方案

**01**

**项目背景与目标**

*Background & Goals*

**1.1 项目定位**

pine-rs 是一个用 Rust 从零实现的 Pine Script v6 完整运行时，目标是在 TradingView 平台之外独立执行 Pine Script 策略与指标脚本。核心应用场景包括：

- 量化策略的离线历史回测（本地 OHLCV CSV / 数据库接入）

- 实时信号生成服务（对接行情 WebSocket 推送，输出策略指令）

- 嵌入第三方 Rust 应用的脚本引擎（策略参数动态热更新）

- Pine Script 开发工具链（LSP、语法高亮、错误诊断）

**1.2 非目标（明确排除）**

- 不实现 TradingView 图表渲染层

- 不实现 request.security 的实时多周期数据拉取（Phase 5 前以 na 占位）

- 不实现策略撮合引擎（只输出信号，由宿主应用负责订单模拟）

- 不追求 100% 语义兼容（以 TradingView 文档为准，不逆向未文档化的边缘行为）

**1.2.1 与官方 v6 的对齐追踪**

- **逐项 backlog、状态图例与阶段路线**（North Star：社区脚本尽量少改即可跑）见 [`V6_ALIGNMENT.md`](./V6_ALIGNMENT.md)。  
- §1.2 所列 **非目标** 主要指 **宿主侧能力**（图表渲染、实时 `request.security` 拉取、真实撮合等），**不**否定在解析与内置 API 上向官方 v6 文档收敛；二者独立。

**1.3 成功指标**

  ----------------------------------------------------------------------------------------
  **指标**                 **目标值**                            **验证方式**
  ------------------------ ------------------------------------- -------------------------
  语法覆盖率               Pine Script v6 全部已文档化语法结构   解析器 snapshot 测试集

  数值精度                 与 TradingView 输出误差 \< 1e-8       黄金测试 CSV diff

  社区脚本兼容率           90% 公开 v6 脚本无 panic              随机抽样 100 个脚本运行

  执行性能（Phase 6）      10 万 bar + 复杂指标 \< 100ms         criterion benchmark

  API 稳定性               Phase 5 后公开 API 语义版本锁定       semver + changelog
  ----------------------------------------------------------------------------------------

**02**

**技术选型**

*Technology Decisions*

**2.1 语言与工具链**

  ----------------------------------------------------------------------------------
  **组件**        **选型**              **理由**
  --------------- --------------------- --------------------------------------------
  编程语言        Rust 2021 Edition     内存安全、零成本抽象、无 GC 停顿、生态成熟

  MSRV            1.75.0                稳定版特性集足够，不追 nightly

  包管理          Cargo Workspace       多 crate 独立发布，依赖隔离

  CI              GitHub Actions        免费额度充足，matrix 测试多平台

  代码格式        rustfmt（默认配置）   无争议，强制统一

  Lint            clippy -D warnings    不允许任何警告提交
  ----------------------------------------------------------------------------------

**2.2 解析器技术选型决策**

这是整个项目最关键的技术决策之一，经过以下候选方案对比：

  ---------------------------------------------------------------------------------------------------------------------------------------------------------------------
  **方案**          **优点**                                                 **缺点**                                                      **结论**
  ----------------- -------------------------------------------------------- ------------------------------------------------------------- ----------------------------
  手写递归下降      最快、最灵活、编译错误清晰                               开发周期长，缩进处理需自行实现                                备选（Phase 6 优化可迁移）

  nom               性能接近手写，成熟稳定                                   缩进敏感语法需大量状态机，v6 扩展困难（pine-lang 前车之鉴）   不采用

  chumsky（采用）   原生缩进支持、内置 Pratt 解析、内置错误恢复、Span 泛型   性能约为手写的 80%，类型错误较难读                            采用

  lalrpop           LR 文法描述清晰                                          不支持缩进敏感语法                                            不采用
  ---------------------------------------------------------------------------------------------------------------------------------------------------------------------

> **关键 pine-lang（xewkf）使用 nom 实现了 v4/v5，在 v6 引入 type/method/map 后无法继续演进而停止维护。**
>
> 这是选择 chumsky 而非 nom 的直接依据。

Lexer 与 Parser 的组合方案：

- Lexer：logos（正则驱动，生成 Token 流，速度接近手写），负责 INDENT / DEDENT / NEWLINE 虚拟 token 的插入

- Parser：chumsky（消费 Token 流，输出带 Span 的 AST），使用 pratt() 组合子处理运算符优先级

- 集成层：logos-chumsky 提供零拷贝 TokenStream 适配器，直接桥接两库

**2.3 运行时关键库选型**

  -------------------------------------------------------------------------------------------------
  **用途**               **选型**                 **版本**   **说明**
  ---------------------- ------------------------ ---------- --------------------------------------
  错误诊断（用户可见）   miette                   5.x        rich 错误报告，源码位置标注

  错误类型（库内部）     thiserror                1.x        零开销错误枚举派生

  有序 HashMap           indexmap                 2.x        作用域符号表，保持声明顺序

  短字符串优化           smartstring              1.x        ≤23 字节栈内联，Pine string 多为短串

  小 Vec 优化            smallvec                 1.x        函数参数列表、短 series 窗口

  序列化输出             serde + serde_json       1.x        输出 plot/signal 为 JSON

  快照测试               insta                    1.x        AST 和执行结果快照，防止静默回归

  属性测试               proptest                 1.x        na 传播、数值边界的随机验证

  基准测试               criterion                0.5        eval 主循环、SeriesBuf 热路径

  Fuzz 测试              cargo-fuzz (libFuzzer)   latest     Lexer/Parser 不 panic 保证
  -------------------------------------------------------------------------------------------------

**2.4 参考项目与借鉴策略**

  ------------------------------------------------------------------------------------------------------------------------------------------------------------------
  **参考项目**        **借鉴内容**                                                                  **注意事项**
  ------------------- ----------------------------------------------------------------------------- ----------------------------------------------------------------
  Rhai                Dynamic 类型内存布局、变量预计算偏移、immutable string、no-scope-chain 设计   不能借鉴：无 series 概念，动态类型与 Pine 静态推断冲突

  pine-lang (xewkf)   Series/na/var 的处理思路、ta.\* 内置函数初始化逻辑、Workspace 分层结构        代码停留 v4/v5，不可直接复用；每处借鉴必须用黄金测试验证正确性

  chumsky 官方示例    Pratt parsing 模式、缩进处理、错误恢复 strategy                               直接参考，无风险
  ------------------------------------------------------------------------------------------------------------------------------------------------------------------

**03**

**架构设计**

*Architecture Design*

**3.1 总体架构**

pine-rs 采用 Cargo Workspace 多 crate 结构，按编译管道严格分层，每层只依赖下方层次：

> 源码文本（UTF-8 .pine 文件）
>
> │
>
> ▼
>
> ┌─────────────────────────┐
>
> │ pine-lexer │ Token 流（含虚拟 INDENT/DEDENT/NEWLINE）
>
> └────────────┬────────────┘
>
> │
>
> ▼
>
> ┌─────────────────────────┐
>
> │ pine-parser │ 带 Span 的 AST（完整覆盖 v6 语法）
>
> └────────────┬────────────┘
>
> │
>
> ▼
>
> ┌─────────────────────────┐
>
> │ pine-sema │ Typed HIR（类型推断、series 标注、var 提升）
>
> └────────────┬────────────┘
>
> │
>
> ┌───────┴────────┐
>
> ▼ ▼
>
> ┌──────────┐ ┌──────────┐
>
> │ pine-eval│ │ pine-vm │ Phase 6: 字节码编译 + 栈式 VM
>
> └────┬─────┘ └──────────┘
>
> │
>
> ▼
>
> ┌─────────────────────────┐
>
> │ pine-runtime │ 执行上下文、SeriesBuf、var 状态、RuntimeConfig
>
> │ pine-stdlib │ ta.\* / math.\* / str.\* / array.\* / matrix.\*
>
> │ pine-output │ plot / label / box / table / strategy 输出模型
>
> └─────────────────────────┘
>
> │
>
> ▼
>
> ┌─────────────────────────┐
>
> │ pine-cli │ CLI 工具：run / check / bench 子命令
>
> └─────────────────────────┘

**3.2 Workspace 目录结构**

> pine-rs/
>
> ├── AGENT.md ← AI 代理行为守则（Claude Code 自动读取）
>
> ├── Cargo.toml ← workspace manifest
>
> ├── crates/
>
> │ ├── pine-lexer/ ← logos 驱动的 Lexer
>
> │ │ ├── src/lib.rs
>
> │ │ └── src/token.rs ← Token 枚举定义
>
> │ ├── pine-parser/ ← chumsky 递归下降 Parser
>
> │ │ ├── src/ast.rs ← AST 节点定义（完整 v6）
>
> │ │ ├── src/parser.rs ← 顶层 parser 入口
>
> │ │ └── src/expr.rs ← Pratt 表达式解析
>
> │ ├── pine-sema/ ← 语义分析
>
> │ │ ├── src/types.rs ← PineType 枚举
>
> │ │ ├── src/scope.rs ← 作用域 + 符号表
>
> │ │ └── src/infer.rs ← 类型推断 + series 标注
>
> │ ├── pine-eval/ ← 树遍历解释器
>
> │ │ ├── src/eval_stmt.rs ← 语句求值
>
> │ │ ├── src/eval_expr.rs ← 表达式求值
>
> │ │ └── src/runner.rs ← bar-by-bar 主循环
>
> │ ├── pine-vm/ ← （Phase 6）字节码 VM
>
> │ ├── pine-runtime/
>
> │ │ ├── src/context.rs ← ExecutionContext
>
> │ │ ├── src/series.rs ← SeriesBuf\<T\>
>
> │ │ ├── src/value.rs ← Value 枚举
>
> │ │ └── src/config.rs ← RuntimeConfig
>
> │ ├── pine-stdlib/
>
> │ │ ├── src/ta.rs ← ta.sma / ema / rsi / macd \...
>
> │ │ ├── src/math.rs
>
> │ │ ├── src/str.rs
>
> │ │ ├── src/array.rs
>
> │ │ └── src/registry.rs ← 函数注册表（hash dispatch）
>
> │ ├── pine-output/
>
> │ │ ├── src/plot.rs
>
> │ │ ├── src/drawing.rs ← label / box / table
>
> │ │ └── src/strategy.rs ← strategy.entry / exit
>
> │ └── pine-cli/
>
> │ └── src/main.rs
>
> ├── tests/
>
> │ ├── snapshots/ ← insta 快照文件（提交到 Git）
>
> │ ├── scripts/ ← .pine 测试脚本
>
> │ │ ├── basic/ ← 变量、运算符、控制流
>
> │ │ ├── series/ ← 序列语义边界案例
>
> │ │ ├── var_state/ ← var/varip 持久化
>
> │ │ ├── stdlib/ ← 内置函数精度验证
>
> │ │ ├── udt/ ← type/method
>
> │ │ ├── strategy/ ← 策略信号
>
> │ │ └── regression/ ← bug 复现脚本
>
> │ └── golden/ ← 黄金测试 expected CSV
>
> └── docs/
>
> ├── SERIES_SEMANTICS.md ← series 语义详细说明
>
> ├── NA_RULES.md ← na 传播规则
>
> └── BUILTIN_COVERAGE.md ← 内置函数覆盖状态

**04**

**核心语义设计**

*Core Semantics*

**4.1 Pine Script 执行模型**

这是 pine-rs 与普通解释器最本质的差异，必须深刻理解：

> **关键 Pine Script 不是\"执行一次\"的脚本。脚本会对每一根 K 线（bar）完整执行一遍，**
>
> 从最老的 bar\[0\] 到最新的 bar\[N\]，形成一个大循环。每个变量自动维护其历史序列。
>
> 这个模型的所有实现细节都必须正确，否则指标计算结果无法与 TradingView 对齐。

主循环伪代码：

> for bar_index in 0..=N {
>
> ctx.inject_bar(ohlcv\[bar_index\]); // 注入 open/high/low/close/volume/time
>
> ctx.restore_var_state(); // 恢复 var/varip 变量
>
> eval_script(&script, &mut ctx); // 执行所有顶层语句
>
> ctx.push_all_series(); // 所有 series 缓冲区 push 当前值
>
> ctx.collect_output(bar_index); // 收集 plot/label/signal 输出
>
> }

**4.2 Value 枚举设计**

运行时值是整个系统的核心数据类型，设计直接影响性能和内存布局：

> #\[repr(u8)\]
>
> pub enum Value {
>
> Na, // 缺失值，所有算术传染性返回 Na
>
> Int(i64),
>
> Float(f64),
>
> Bool(bool),
>
> Str(smartstring::SmartString), // ≤23B 栈内联，长串 heap
>
> Color(u32), // RRGGBBAA packed
>
> Series(SeriesId), // 对 SeriesBuf 的句柄，非实际值
>
> Array(Rc\<RefCell\<Vec\<Value\>\>\>),
>
> Matrix(Rc\<RefCell\<MatrixData\>\>),
>
> Map(Rc\<RefCell\<IndexMap\<Value,Value\>\>\>),
>
> Object(Rc\<RefCell\<UDTInstance\>\>), // type 关键字定义的实例
>
> Callable(FnRef), // 函数引用
>
> }
>
> **注意 Na 的特殊规则：**
>
> na + 1 == Na na \* 0 == Na na == na → false（类似 NaN）
>
> na(x) → bool nz(x, default) → x if !Na else default
>
> 所有二元运算符在任一操作数为 Na 时均返回 Na，比较运算符也不例外。

**4.3 SeriesBuf 设计**

每个 series 变量在运行时对应一个 SeriesBuf，是 pine-rs 最重要的数据结构：

> pub struct SeriesBuf\<T\> {
>
> buf: VecDeque\<T\>,
>
> max_len: usize, // 由 RuntimeConfig.max_bars_back 控制，默认 500
>
> }
>
> impl\<T: Default + Clone\> SeriesBuf\<T\> {
>
> // 每 bar 结束时调用，将当前值压入历史
>
> pub fn push(&mut self, val: T) {
>
> self.buf.push_front(val);
>
> if self.buf.len() \> self.max_len {
>
> self.buf.pop_back();
>
> }
>
> }
>
> // close\[n\]：读取 n 根 bar 前的值，越界返回 T::default()（对应 na）
>
> pub fn index(&self, n: usize) -\> Option\<&T\> {
>
> self.buf.get(n)
>
> }
>
> }

**4.4 变量类型与持久化规则**

  ---------------------------------------------------------------------------------------------------------
  **关键字**   **初始化时机**                 **跨 bar 持久**   **实时 tick 更新**   **用法场景**
  ------------ ------------------------------ ----------------- -------------------- ----------------------
  （无）       每 bar 重新初始化              否                否                   普通计算变量

  var          仅在 bar_index == 0 时初始化   是                否                   累加器、状态机

  varip        仅在 bar_index == 0 时初始化   是                是                   实时 tick 级别的状态
  ---------------------------------------------------------------------------------------------------------

> **关键 Series 对齐约束（最易出错）：**
>
> if/else 的两个分支必须都向所有 live series push 值，即使该分支未被执行。
>
> 违反此约束会导致 close\[1\] 等历史访问偏移错误，且该 bug 极难追踪。
>
> 实现方案：sema 阶段标记所有 live series；eval 阶段在 if/else 退出前强制对齐。

**4.5 类型系统**

  -----------------------------------------------------------------------------------------------
  **类型分类**    **具体类型**                                  **说明**
  --------------- --------------------------------------------- ---------------------------------
  简单类型        int, float, bool, string, color               编译期已知，可栈分配

  Series 类型     series int, series float, series bool, \...   具有历史缓冲区，按 bar 推进

  集合类型        array\<T\>, matrix\<T\>, map\<K,V\>           v6 新增，堆分配

  用户类型        type Foo（UDT）                               v6 新增，字段 + method

  特殊类型        na, void, label, box, table, line             na 多态，drawing 对象
  -----------------------------------------------------------------------------------------------

Series 传染规则：若任一操作数为 series 类型，结果也为 series 类型。类型推断需实现不动点迭代（循环变量可能触发类型升级）。

**05**

**模块详细规格**

*Module Specifications*

**5.1 pine-lexer**

  -----------------------------------------------------------------------------
  **职责项**      **说明**
  --------------- -------------------------------------------------------------
  输入 / 输出     输入 UTF-8 源码字符串 → 输出 Vec\<(Token, Span)\>

  缩进处理        追踪缩进层级栈，插入虚拟 INDENT / DEDENT token（类 Python）

  字符串字面量    处理单双引号、转义序列 \\n \\t \\\\

  颜色字面量      解析 #RGB / #RGBA / #RRGGBB / #RRGGBBAA，转为 Color(u32)

  数字字面量      整型 / 浮点（支持 1_000_000 分隔符）

  错误处理        非法字符不 panic，生成 Token::Error 并继续
  -----------------------------------------------------------------------------

关键 Token 类型（部分）：

> INDENT / DEDENT / NEWLINE // 缩进控制（虚拟）
>
> IF / ELSE / ELIF / FOR / TO / BY / WHILE / BREAK / CONTINUE
>
> VAR / VARIP / TYPE / METHOD / IMPORT / EXPORT / LIBRARY
>
> SWITCH / CASE / DEFAULT // v6 新增
>
> IDENT(String) / INT(i64) / FLOAT(f64) / BOOL(bool)
>
> PLUS / MINUS / STAR / SLASH / PERCENT / HAT
>
> EQ / NEQ / LT / LE / GT / GE
>
> AND / OR / NOT
>
> LBRACKET / RBRACKET / LPAREN / RPAREN / LBRACE / RBRACE
>
> DOT / COMMA / COLON / ARROW / QMARK / QMARKQMARK // ?: 三元 / ?? na 合并
>
> COLONEQ / PLUSEQ / MINUSEQ / STAREQ / SLASHEQ // 复合赋值

**5.2 pine-parser**

  ------------------------------------------------------------------------------
  **职责项**      **说明**
  --------------- --------------------------------------------------------------
  输入 / 输出     输入 Token 流 → 输出 SyntaxTree（带 Span 的 AST）

  解析策略        手写递归下降 + chumsky 组合子；表达式部分用 pratt()

  错误恢复        语句边界同步，尽量报告多个错误；生成部分 AST

  v6 新语法       完整支持 type/method/import/export/library/switch/map/matrix

  Span 设计       所有 AST 节点携带 (file_id, byte_start, byte_end)
  ------------------------------------------------------------------------------

AST 核心节点（部分）：

> pub enum Stmt {
>
> VarDecl { name: Ident, kind: VarKind, type_ann: Option\<PineType\>, init: Expr, span: Span },
>
> Assign { target: AssignTarget, op: AssignOp, value: Expr, span: Span },
>
> If { cond: Expr, then: Block, elif: Vec\<(Expr,Block)\>, else\_: Option\<Block\>, span: Span },
>
> For { var: Ident, from: Expr, to: Expr, by: Option\<Expr\>, body: Block, span: Span },
>
> While { cond: Expr, body: Block, span: Span },
>
> Switch { expr: Expr, cases: Vec\<SwitchCase\>, default: Option\<Block\>, span: Span },
>
> FnDef { name: Ident, params: Vec\<Param\>, ret: Option\<PineType\>, body: Block, span: Span },
>
> TypeDef { name: Ident, fields: Vec\<Field\>, span: Span },
>
> MethodDef { type\_: Ident, name: Ident, params: Vec\<Param\>, body: Block, span: Span },
>
> Return { value: Option\<Expr\>, span: Span },
>
> Expr (Expr),
>
> }
>
> pub enum Expr {
>
> Literal(Lit, Span),
>
> Ident(Ident, Span),
>
> BinOp { op: BinOp, lhs: Box\<Expr\>, rhs: Box\<Expr\>, span: Span },
>
> UnaryOp { op: UnaryOp, operand: Box\<Expr\>, span: Span },
>
> Ternary { cond: Box\<Expr\>, then: Box\<Expr\>, else\_: Box\<Expr\>, span: Span },
>
> NaCoalesce { lhs: Box\<Expr\>, rhs: Box\<Expr\>, span: Span }, // x ?? y
>
> Index { base: Box\<Expr\>, offset: Box\<Expr\>, span: Span }, // close\[1\]
>
> FieldAccess { base: Box\<Expr\>, field: Ident, span: Span }, // obj.field
>
> MethodCall { base: Box\<Expr\>, method: Ident, args: Vec\<Arg\>, span: Span },
>
> FnCall { func: Box\<Expr\>, args: Vec\<Arg\>, span: Span },
>
> ArrayLit(Vec\<Expr\>, Span),
>
> }

**5.3 pine-sema**

  --------------------------------------------------------------------------------
  **分析 Pass**         **职责**
  --------------------- ----------------------------------------------------------
  Pass 1：收集声明      扫描顶层 fn/type 定义，建立预声明符号表（处理前向引用）

  Pass 2：类型推断      递归推断所有表达式类型；series 传染规则不动点迭代

  Pass 3：Series 标注   标记每个变量是否需要 SeriesBuf；标注 series 对齐要求

  Pass 4：var 提升      标记 var/varip 变量，记录其在 ExecutionContext 中的 slot

  Pass 5：内置解析      解析 ta.sma 等限定名到 FnRef；验证参数类型
  --------------------------------------------------------------------------------

**5.4 pine-eval**

树遍历解释器，Phase 1-5 的执行核心。

  --------------------------------------------------------------------------------------
  **模块**        **职责**
  --------------- ----------------------------------------------------------------------
  runner.rs       bar-by-bar 主循环；bar 注入；series push 协调；输出收集

  eval_stmt.rs    VarDecl / Assign / If / For / While / Switch / FnDef / Return 的求值

  eval_expr.rs    BinOp / UnaryOp / FnCall / Index / FieldAccess / Ternary 的求值

  fn_call.rs      用户函数调用：call-site series map 隔离；参数绑定；返回值

  na_ops.rs       Na 传播规则的集中实现（所有算术/比较运算经此模块）
  --------------------------------------------------------------------------------------

> **关键 函数调用中的 series 隔离（极易出错）：**
>
> 用户自定义函数内部的 var 变量和 series 缓冲区必须按调用点（call-site）隔离。
>
> 同一函数被调用两次（如 f(close) 和 f(high)），两次调用的 series 状态完全独立。
>
> 实现方式：为每个调用点在 ExecutionContext 中维护独立的 series slot map。

**5.5 pine-runtime**

  ---------------------------------------------------------------------------------------------------------------------
  **组件**           **关键设计**
  ------------------ --------------------------------------------------------------------------------------------------
  ExecutionContext   持有 SeriesBuf 注册表（SeriesId → SeriesBuf）、var slot 表、call-site series map、当前 bar OHLCV

  SeriesBuf\<T\>     VecDeque\<T\>，max_len 由 RuntimeConfig.max_bars_back 决定（默认 500，可覆盖）

  RuntimeConfig      max_bars_back: usize、max_labels: usize、max_boxes: usize、calc_bars_count: Option\<usize\>

  DataFeed trait     抽象数据源接口：fn bar_count(&self) -\> usize; fn bar(&self, i: usize) -\> OhlcvBar;
  ---------------------------------------------------------------------------------------------------------------------

**5.6 pine-stdlib（内置函数库）**

所有内置函数通过 FunctionRegistry 统一注册，支持 hash dispatch（函数名预计算 hash，O(1) 查找）。

  -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
  **命名空间**   **函数列表**                                                                                                                                                                                     **优先级**   **精度要求**
  -------------- ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ ------------ --------------------
  ta.\*          sma, ema, rma, wma, vwma, highest, lowest, highestbars, lowestbars, rsi, macd, bbands, stoch, atr, tr, cci, cmo, mom, change, crossover, crossunder, barssince, valuewhen, pivothigh, pivotlow   P1           与 TV 误差 \< 1e-8

  math.\*        abs, ceil, floor, round, sqrt, pow, log, log10, exp, sin, cos, tan, asin, acos, atan, min, max, sign, random, todegrees, toradians                                                               P1           标准库精度

  str.\*         tostring, tonumber, format, length, substring, contains, startswith, endswith, split, replace, lower, upper, match, pos                                                                          P2           字符串操作

  array.\*       new, push, pop, shift, unshift, get, set, size, sort, slice, join, concat, includes, indexof, lastindexof, remove, insert, clear, copy, sum, avg, min, max, stdev, variance                      P2           ---

  matrix.\*      new, get, set, rows, cols, add, mult, transpose, pinv, det, eigenvalues                                                                                                                          P3           数值线代

  map.\*         new, get, put, contains, keys, values, remove, clear, size                                                                                                                                       P3           ---

  color.\*       new, r, g, b, t, from_gradient, rgb, hsv                                                                                                                                                         P2           ---

  request.\*     security（多时间周期）                                                                                                                                                                           P4           Phase 5 后实现
  -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

新增内置函数的标准流程：

1.  在 pine-stdlib/src/registry.rs 中注册函数签名（名称、参数类型、返回类型）

2.  在对应模块（ta.rs / math.rs / \...）实现函数体，遵循 na 传播规则

3.  在 tests/scripts/stdlib/\<namespace\>/ 新建 .pine 测试脚本

4.  运行 cargo test -p pine-stdlib，提交 insta 快照

5.  对比 TradingView 输出，确认数值精度（黄金测试）

**5.7 pine-output（输出模型）**

  ---------------------------------------------------------------------------------------------------------
  **输出类型**     **关键字段**                                         **限制**
  ---------------- ---------------------------------------------------- -----------------------------------
  PlotSeries       title, color, linewidth, style, display, offset      每脚本最多 64 个 plot

  LabelObject      x, y, text, color, style, size, tooltip              最多 500 个（RuntimeConfig 可调）

  BoxObject        left, top, right, bottom, border_color, bgcolor      最多 500 个

  TableObject      columns×rows cells，每格 text/bgcolor/text_color     持久存在，不计数限制

  StrategySignal   direction(long/short), qty, comment, alert_message   每 bar 可多个

  AlertCondition   condition(bool series), title, message               声明式，不是命令式
  ---------------------------------------------------------------------------------------------------------

**06**

**开发规范**

*Development Standards*

**6.1 代码风格**

  -------------------------------------------------------------------------------------------------------
  **规则**        **要求**                                         **工具**
  --------------- ------------------------------------------------ --------------------------------------
  格式化          所有代码必须通过 cargo fmt \--check              rustfmt（默认配置）

  Lint            零警告，cargo clippy \-- -D warnings             clippy

  unwrap/expect   library crate 中绝对禁止；CLI 和测试代码中可用   clippy::unwrap_used（library crate）

  unsafe          必须附带 // SAFETY: 注释，需 PR Review           clippy::undocumented_unsafe_blocks

  错误类型        库内部用 thiserror；用户可见错误用 miette        ---

  文档注释        所有 pub 函数/类型必须有 /// 文档                cargo doc \--no-deps
  -------------------------------------------------------------------------------------------------------

**6.2 命名约定**

  -----------------------------------------------------------------------------------------------
  **对象类型**             **规范**                   **示例**
  ------------------------ -------------------------- -------------------------------------------
  AST 节点                 PascalCase + 功能后缀      BinExpr, FnCallExpr, IfStmt

  IR 操作码                SCREAMING_SNAKE_CASE       PUSH_SERIES, CALL_BUILTIN, LOAD_VAR

  内置函数对应 Rust 函数   snake_case，命名空间前缀   ta_sma, math_abs, str_format

  Series 相关类型          带 Series/Buf/Id 后缀      SeriesBuf\<T\>, SeriesId, SeriesRegistry

  错误类型                 PascalCase + Error 后缀    ParseError, SemanticError, RuntimeError

  测试函数                 test\_ 前缀 + 被测功能     test_ema_initial_na, test_var_persistence
  -----------------------------------------------------------------------------------------------

**6.3 Git 工作流**

  ------------------------------------------------------------------------------------
  **分支**              **规则**
  --------------------- --------------------------------------------------------------
  main                  始终可发布；所有 CI 必须绿灯；只接受来自 dev 的 squash merge

  dev                   Phase 内日常集成分支；接受来自 feat/\* / fix/\* 的 PR

  feat/\<n\>-\<name\>   功能分支，从 dev 检出，如 feat/03-sema-type-inference

  fix/\<issue\>         Bug 修复分支，写法 fix/42-series-alignment

  bench/\<n\>           性能优化分支
  ------------------------------------------------------------------------------------

Commit 消息格式（Conventional Commits）：

> feat(pine-sema): add series alignment enforcement for if/else
>
> fix(pine-eval): var state not restored correctly in nested fn calls
>
> test(pine-stdlib): add golden test for ta.rsi vs TradingView output
>
> perf(pine-eval): replace HashMap with IndexMap in scope lookup
>
> docs(SERIES_SEMANTICS): document varip intra-bar update behavior

**6.4 PR 合并标准（Checklist）**

> **提示 每个 PR 合并前必须满足以下全部条件：**

6.  cargo fmt \--check 通过

7.  cargo clippy \--workspace \-- -D warnings 无警告

8.  cargo test \--workspace 全部通过

9.  cargo insta test 无未审阅 snapshot diff（cargo insta review 处理完毕）

10. 涉及新功能：附带单元测试或集成测试

11. 涉及 series 语义变更：附带黄金测试，且对比 TV 输出通过

12. 涉及公开 API 变更：更新 CHANGELOG.md 和文档注释

13. 涉及 SeriesBuf/ExecutionContext：更新 docs/SERIES_SEMANTICS.md

**6.5 Agent（Claude Code）操作守则**

当使用 AI 代码助手在此项目工作时，强制遵守以下规则：

- 任务开始前必须读取 AGENT.md，获取完整上下文

- 修改 pine-eval 任何代码前，先在 tests/scripts/regression/ 写好复现 .pine 脚本

- 不得在 library crate 中使用 unwrap() / expect()，违者 clippy 会报错

- 新增内置函数必须运行 cargo test -p pine-stdlib 并审阅快照

- 修改 SeriesBuf 必须更新 docs/SERIES_SEMANTICS.md

- 不得假设 na 语义\"应该\"如何------以 TradingView 实测为准

- 对 pine-lang（xewkf）的代码借鉴必须附黄金测试验证，不可直接复制

**07**

**测试体系**

*Testing Strategy*

**7.1 测试分层**

  -------------------------------------------------------------------------------------------------------------------
  **层次**     **工具**          **覆盖范围**                                    **运行时机**   **验收标准**
  ------------ ----------------- ----------------------------------------------- -------------- ---------------------
  单元测试     #\[test\]         各 crate 内部函数，100% pub API                 push / PR      全部通过

  快照测试     insta             Lexer token 流、Parser AST、Sema 类型标注输出   push / PR      无未审阅 diff

  集成测试     harness（自建）   完整 .pine 脚本从输入到输出端到端               PR             全部通过

  黄金测试     CSV diff          与 TradingView 参考输出对比，误差 \< 1e-8       PR to main     全部通过

  Fuzz 测试    cargo-fuzz        Lexer + Parser 对任意输入不 panic               每日 CI        300 秒内无崩溃

  属性测试     proptest          na 传播规则、数值边界（NaN/Inf/i64 溢出）       push / PR      1000 次随机全部通过

  性能基准     criterion         eval 主循环、SeriesBuf::push、ta.ema 热路径     每周 CI        回归不超过 10%
  -------------------------------------------------------------------------------------------------------------------

**7.2 快照测试规范**

快照文件（tests/snapshots/）必须提交到 Git，作为语法/语义变更的审计记录：

> // 示例：Parser 快照测试
>
> #\[test\]
>
> fn test_parse_sma_crossover() {
>
> let src = include_str!(\"../scripts/basic/sma_cross.pine\");
>
> let ast = pine_parser::parse(src).expect(\"parse ok\");
>
> insta::assert_debug_snapshot!(ast);
>
> }
>
> // 快照更新工作流：
>
> // 1. 运行 cargo test（出现 snapshot diff）
>
> // 2. 运行 cargo insta review（交互式审阅每个 diff）
>
> // 3. 接受合理变更 / 拒绝意外变更
>
> // 4. 提交快照文件

**7.3 黄金测试脚本集**

tests/scripts/ 下按以下分类维护测试脚本，每个脚本在 tests/golden/ 中有对应 expected.csv：

  -----------------------------------------------------------------------------------------------------------
  **目录**        **内容**                                                 **关键验证点**
  --------------- -------------------------------------------------------- ----------------------------------
  basic/          变量赋值、四则运算、字符串、颜色、bool 逻辑              基础值语义正确

  series/         历史访问 close\[N\]、na 传播、序列对齐边界               series 偏移不错位

  var_state/      var 累加器、var 在函数内的行为、varip                    var 持久化正确

  control_flow/   if/elif/else 分支、for 范围循环、while、break/continue   控制流 series 对齐

  stdlib/ta/      sma/ema/rsi/macd/bbands 各一个参数组合                   数值与 TV 误差 \< 1e-8

  stdlib/math/    边界值：0、负数、Inf、NaN 输入                           na 处理正确

  udt/            type 定义、字段访问、method 调用、继承（若支持）         v6 UDT 语义

  strategy/       多次 strategy.entry/exit、pyramiding、commission         信号序列精确

  regression/     历史 bug 复现脚本，文件名含 issue 编号                   不再 panic 或错误
  -----------------------------------------------------------------------------------------------------------

**7.4 CI 流水线配置**

  -------------------------------------------------------------------------------------------------------------
  **Job 名称**   **触发条件**          **步骤**                                             **失败策略**
  -------------- --------------------- ---------------------------------------------------- -------------------
  lint           所有 push 和 PR       cargo fmt \--check → cargo clippy -D warnings        阻断 merge

  test           所有 push 和 PR       cargo test \--workspace → cargo insta test           阻断 merge

  golden         PR to main            运行全部黄金测试脚本，diff expected CSV              阻断 merge

  fuzz-lexer     每日 cron 02:00 UTC   cargo fuzz run fuzz_lexer \-- -max_total_time=300    飞书/邮件告警

  fuzz-parser    每日 cron 02:30 UTC   cargo fuzz run fuzz_parser \-- -max_total_time=300   飞书/邮件告警

  bench          每周一 cron           cargo bench \--workspace → 存储 criterion 报告       回归 \>10% 告警
  -------------------------------------------------------------------------------------------------------------

**7.5 错误诊断质量标准**

所有用户可见的运行时和解析错误必须满足：

14. 包含源码位置：文件名 + 行号 + 列号，精确到字符

15. 用 miette / ariadne 渲染带颜色的源码片段，下划线标注问题位置

16. 给出可操作的 help: 建议（参考 rustc 风格）

17. 有唯一错误码（E001 \~ Exxx），可查文档

示例输出：

> error\[E042\]: type mismatch in binary expression
>
> \--\> strategy.pine:12:15
>
> \|
>
> 12 \| my_var := close + \"hello\"
>
> \| \^\^\^\^\^ \-\-\-\-\-\-\-\-- string value here
>
> \| \|
>
> \| expected float, found string
>
> \|
>
> = help: use str.tostring(close) to convert float to string

**08**

**分阶段交付计划**

*Delivery Phases*

**8.1 Phase 概览**

  --------------------------------------------------------------------------------------------
  **Phase**   **名称**         **周期**       **关键里程碑**
  ----------- ---------------- -------------- ------------------------------------------------
  Phase 1     Lexer + Parser   第 1--3 周     任意 v6 脚本可解析为 AST，snapshot 覆盖率 100%

  Phase 2     核心执行引擎     第 4--7 周     SMA 手算脚本结果与 TV 误差 \< 1e-10

  Phase 3     内置标准库 P1    第 8--10 周    RSI / MACD / Bollinger 黄金测试全部通过

  Phase 4     完整语言特性     第 11--14 周   90% 公开 v6 社区脚本无 panic

  Phase 5     输出层与策略     第 15--17 周   策略信号序列与 TV 策略报告一致

  Phase 6     性能优化 + VM    第 18--22 周   10 万 bar + 复杂指标执行 \< 100ms
  --------------------------------------------------------------------------------------------

**8.2 Phase 1：Lexer + Parser（第 1--3 周）**

  ----------------------------------------------------------------------------------------
  **任务**                                  **负责 crate**   **完成标准**
  ----------------------------------------- ---------------- -----------------------------
  logos Lexer：所有 Token 类型 + 缩进处理   pine-lexer       token snapshot 测试 100%

  chumsky Parser：语句层（Stmt 全覆盖）     pine-parser      任意 v6 stmt snapshot 通过

  Pratt 表达式解析：17 级优先级             pine-parser      运算符优先级测试全通过

  v6 新语法：type / method / switch / ??    pine-parser      专项 snapshot 测试

  错误恢复：多错误报告                      pine-parser      单文件多错误可全部报出

  AST Display impl（调试打印）              pine-parser      cargo doc 可渲染
  ----------------------------------------------------------------------------------------

**8.3 Phase 2：核心执行引擎（第 4--7 周）**

  ------------------------------------------------------------------------------------------------
  **任务**                                    **负责 crate**     **完成标准**
  ------------------------------------------- ------------------ ---------------------------------
  PineType 枚举 + series 传染规则             pine-sema          类型推断 snapshot

  Series 标注 Pass + var 提升 Pass            pine-sema          series 标注正确率 100%

  Value 枚举 + na 传播规则                    pine-runtime       proptest 1000 次随机通过

  SeriesBuf\<T\> 实现 + 边界测试              pine-runtime       越界返回 na，max_len 截断正确

  ExecutionContext 骨架                       pine-runtime       bar 注入 + series push 协调

  树遍历解释器骨架（eval_stmt + eval_expr）   pine-eval          基础赋值/运算/条件执行正确

  bar-by-bar 主循环                           pine-eval/runner   SMA 手算脚本误差 \< 1e-10

  var / varip 持久化                          pine-eval          var_state 黄金测试通过

  if/else series 对齐强制执行                 pine-eval          series alignment 专项测试
  ------------------------------------------------------------------------------------------------

**8.4 Phase 3：内置标准库 P1（第 8--10 周）**

  -----------------------------------------------------------------------------------------------------------------------
  **任务**                **函数范围**                                             **完成标准**
  ----------------------- -------------------------------------------------------- --------------------------------------
  ta.\* 全部实现          sma/ema/rma/wma/rsi/macd/bbands/stoch/atr/crossover 等   黄金测试误差 \< 1e-8

  math.\* 全部实现        abs/ceil/floor/sqrt/pow/log/exp/trig 等                  标准库精度，na 传播正确

  CSV 数据加载            pine-cli                                                 OHLCV CSV → DataFeed trait 实现

  pine-cli run 子命令     pine-cli                                                 能从命令行运行 .pine 脚本并输出 JSON

  pine-cli check 子命令   pine-cli                                                 语法 + 语义检查，输出 miette 错误
  -----------------------------------------------------------------------------------------------------------------------

**8.5 Phase 4：完整语言特性（第 11--14 周）**

  -----------------------------------------------------------------------------------------
  **任务**                    **说明**
  --------------------------- -------------------------------------------------------------
  UDT（type/method）          字段定义、构造、访问、方法绑定、方法调用

  array.\* 完整实现           所有 array 方法，含 array.sort / array.slice / array.concat

  matrix.\* 实现              new/get/set/rows/cols/add/mult/transpose

  map\<K,V\> 实现             类型推断 + runtime 表示 + map.\* 函数

  str.\* 完整实现             格式化、正则匹配（str.match）、分割

  import / export / library   模块系统：library 声明、export 函数、import 导入

  switch 语句                 switch/case/default，含 series switch

  color.\* + 绘图对象 API     color.from_gradient / label.\* / box.\* / line.\*
  -----------------------------------------------------------------------------------------

**8.6 Phase 5：输出层与策略（第 15--17 周）**

  -------------------------------------------------------------------------------------------------------
  **任务**                                  **说明**
  ----------------------------------------- -------------------------------------------------------------
  plot / plotshape / plotchar / plotarrow   完整 plot 系列，含 display / offset 参数

  hline / bgcolor / fill                    水平线、背景色、区间填充

  label.new + label.set\_\*                 标签生命周期管理，max_labels 限制

  box.new + box.set\_\*                     矩形对象

  table.new + table.cell                    表格对象，跨 bar 持久

  strategy.\* 完整实现                      entry/exit/close/order，pyramiding/commission/slippage 配置

  alertcondition                            布尔序列输出

  JSON 输出格式定稿                         输出结构 schema 版本锁定，写入 CHANGELOG
  -------------------------------------------------------------------------------------------------------

**8.7 Phase 6：性能优化与 VM（第 18--22 周）**

  --------------------------------------------------------------------------------------------------------
  **任务**                  **说明**                                             **预期收益**
  ------------------------- ---------------------------------------------------- -------------------------
  字节码编译器（pine-vm）   Typed HIR → 线性字节码，消除树遍历递归开销           2-5x 加速

  栈式 VM                   紧凑指令集，Value 栈，直接 u8 dispatch               ---

  Series 内存优化           SeriesBuf\<f64\> 特化为 f64 Vec（避免 Value 装箱）   内存降 50%+

  变量访问优化              slot index 替换 HashMap lookup（借鉴 Rhai）          10-30% 加速

  函数 hash dispatch        Bloom filter + 预计算 hash（借鉴 Rhai）              内置函数调用加速

  Rayon 并行                多脚本/多股票并行执行（harness 层）                  线性扩展核数

  可选 Cranelift JIT        高频策略的 JIT 编译路径（可选特性）                  近原生速度
  --------------------------------------------------------------------------------------------------------

**09**

**已知难点与风险**

*Known Challenges & Risks*

**9.1 技术难点清单**

  -------------------------------------------------------------------------------------------------------------------------------------
  **难点**                          **难度**   **影响**   **应对策略**
  --------------------------------- ---------- ---------- -----------------------------------------------------------------------------
  Series 对齐（if/else/for 分支）   高         高         在 sema 阶段强制标记；eval 层面在分支退出前显式 push；专项测试套件

  函数调用的 series 隔离            高         高         每个调用点维护独立 SeriesSlotMap；call-site key = (fn_name, call_expr_span)

  类型推断不动点迭代                中         高         递归类型推断加循环依赖检测；series 传染规则最多迭代 N 次（N=脚本行数）

  na 语义边界                       中         高         集中在 na_ops.rs，不分散处理；proptest 全覆盖；对照 TV 逐一验证

  request.security 多周期           高         低         Phase 5 前返回 na 占位；Phase 5 后实现基于预加载的多周期数据

  Pine v6 文档不完整                中         中         以社区脚本实测为补充；建立 UNDOCUMENTED_BEHAVIOR.md 记录逆向发现

  strategy 撮合精度                 中         中         只输出信号，不做撮合；由宿主应用负责 fill 逻辑
  -------------------------------------------------------------------------------------------------------------------------------------

**9.2 pine-lang 借鉴的注意事项**

> **注意 从 pine-lang（xewkf）借鉴代码时，必须注意：**
>
> 1\. 该项目停留在 v4/v5，UDT/method/map/switch 均未实现，AST 结构不可直接复制
>
> 2\. series 对齐可能存在已知 bug（未修复），借鉴 context.rs 后必须运行黄金测试验证
>
> 3\. ta.\* 函数的初始化行为（前 N 根 bar 为 na 的处理）需对照 TV 实测，不可盲信
>
> 4\. nom parser 结构因工具不同无法复用，但可参考其处理的语法结构列表

**9.3 风险登记册**

  -------------------------------------------------------------------------------------------------------------------------------------
  **风险**                            **概率**   **影响**   **触发信号**                     **缓解措施**
  ----------------------------------- ---------- ---------- -------------------------------- ------------------------------------------
  Series 语义 bug 导致指标值偏差      高         高         黄金测试 CSV diff \> 1e-8        从 Phase 2 起持续对比 TV，不等 Phase 5

  chumsky 编译时间过长（大型语法）    中         低         cargo build \> 60s               对大型 parser 模块拆分，启用 incremental

  Phase 6 VM 性能目标未达到           中         低         bench 显示 \> 100ms @ 10万 bar   树遍历已足够多数场景；VM 为可选优化

  Pine v6 新语法无文档化行为          中         中         社区脚本解析/执行错误            建立 fuzz 语料库，社区脚本作为测试语料

  开发者对 Pine series 模型理解错误   中         高         序列对齐 bug 反复出现            先读完 docs/SERIES_SEMANTICS.md 再写代码
  -------------------------------------------------------------------------------------------------------------------------------------

**10**

**快速上手指南**

*Getting Started*

**10.1 环境准备**

> \# 安装 Rust 1.75+
>
> curl \--proto \"=https\" \--tlsv1.2 -sSf https://sh.rustup.rs \| sh
>
> rustup update stable
>
> \# 安装开发工具
>
> cargo install cargo-insta \# 快照测试审阅
>
> cargo install cargo-fuzz \# fuzz 测试
>
> cargo install cargo-watch \# 文件变更自动测试
>
> \# 克隆并构建
>
> git clone https://github.com/your-org/pine-rs.git
>
> cd pine-rs
>
> cargo build \--workspace

**10.2 日常开发命令**

> \# 构建整个 workspace
>
> cargo build \--workspace
>
> \# 运行所有测试
>
> cargo test \--workspace
>
> \# 运行测试 + 自动审阅快照
>
> cargo test \--workspace && cargo insta review
>
> \# 文件变更时自动运行测试（TDD 模式）
>
> cargo watch -x \"test \--workspace\"
>
> \# 运行单个 .pine 脚本
>
> cargo run -p pine-cli \-- run examples/sma_crossover.pine \--data data/BTCUSDT_1h.csv
>
> \# 语法检查（不执行）
>
> cargo run -p pine-cli \-- check my_script.pine
>
> \# 运行性能基准
>
> cargo bench -p pine-eval
>
> \# 格式化 + lint
>
> cargo fmt && cargo clippy \--workspace \-- -D warnings
>
> \# Fuzz 测试（需 nightly）
>
> cargo +nightly fuzz run fuzz_lexer \-- -max_total_time=60

**10.3 新功能开发流程**

18. 从 dev 检出功能分支：git checkout -b feat/XX-feature-name dev

19. 在 tests/scripts/ 下先写 .pine 测试脚本（TDD）

20. 实现功能代码，确保 cargo test 通过

21. 运行 cargo insta review 审阅新增快照

22. 对需要黄金测试的功能：在 tests/golden/ 添加 expected CSV（对照 TV 获取）

23. 确认 PR checklist 全部满足后提交 PR 到 dev

**10.4 调试技巧**

- 使用 PINE_LOG=trace cargo run \... 开启详细日志（eval 每步打印）

- 在 runner.rs 中设置 debug_bar: Option\<usize\>，只对指定 bar 打印详细状态

- 用 insta::assert_debug_snapshot! 快速捕获中间 AST 状态

- 对 series 偏移 bug：在脚本里加 plot(close\[0\])、plot(close\[1\]) 逐一验证

- 对 na bug：用 plot(na(close)) 验证 na 检测，用 plot(nz(close, 0)) 验证 na 替换

**附录 A：Pine Script v6 语法覆盖检查清单**

  ------------------------------------------------------------------------------------
  **语法特性**                              **所在 Phase**   **状态**
  ----------------------------------------- ---------------- -------------------------
  变量声明（int/float/bool/string/color）   Phase 2          待实现

  var / varip 声明                          Phase 2          待实现

  Series 历史访问 close\[N\]                Phase 2          待实现

  算术运算符（+ - \* / % \^）               Phase 2          待实现

  比较运算符（== != \< \<= \> \>=）         Phase 2          待实现

  逻辑运算符（and or not）                  Phase 2          待实现

  三元运算符 condition ? a : b              Phase 2          待实现

  Na 合并运算符 x ?? y                      Phase 2          待实现

  if / elif / else 块                       Phase 2          待实现

  for\...to\...by 循环                      Phase 2          待实现

  while 循环                                Phase 2          待实现

  switch / case / default（v6）             Phase 4          待实现

  用户自定义函数（fn 定义）                 Phase 2          待实现

  type 定义（UDT）                          Phase 4          待实现

  method 方法绑定（v6）                     Phase 4          待实现

  array\<T\>（v6）                          Phase 4          待实现

  matrix\<T\>（v6）                         Phase 4          待实现

  map\<K,V\>（v6）                          Phase 4          待实现

  import / export / library（v6）           Phase 4          待实现

  ta.\* 内置函数                            Phase 3          待实现

  math.\* 内置函数                          Phase 3          待实现

  str.\* 内置函数                           Phase 4          待实现

  array.\* 方法                             Phase 4          待实现

  plot / plotshape / plotchar               Phase 5          待实现

  label.new + label.set\_\*                 Phase 5          待实现

  box.new + box.set\_\*                     Phase 5          待实现

  table.new + table.cell                    Phase 5          待实现

  strategy.\*                               Phase 5          待实现

  alertcondition                            Phase 5          待实现

  request.security                          Phase 5          待实现
  ------------------------------------------------------------------------------------

*--- 文档结束 ---*
