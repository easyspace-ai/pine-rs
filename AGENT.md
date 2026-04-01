# AGENT.md — pine-rs + pine-tv

> **Read this file completely before touching any code.**
> This is the single source of truth for all AI agent behaviour in this repository.
> When in doubt, re-read the relevant section rather than guessing.

---

## 0. Repository at a Glance

This monorepo contains **two products** that share a single Cargo workspace:

| Product | What it is | Entry point |
|---------|-----------|------------|
| **pine-rs** | Pine Script v6 interpreter library + CLI | `crates/pine-*/`, `pine-cli` |
| **pine-tv** | Local TradingView-like web playground | `pine-tv/` (Axum server + static frontend) |

pine-tv depends on pine-rs as a library crate. pine-rs has no knowledge of pine-tv.
**Never add pine-tv imports into any pine-rs crate.**

---

## 1. Full Workspace Layout

```
pine-rs/                          <- repo root
├── AGENT.md                      <- this file (always read first)
├── Cargo.toml                    <- workspace manifest
│
├── crates/                       <- pine-rs: interpreter engine
│   ├── pine-lexer/               <- logos-driven tokenizer, INDENT/DEDENT
│   ├── pine-parser/              <- chumsky recursive-descent, full v6 AST
│   ├── pine-sema/                <- type inference, series annotation, var hoisting
│   ├── pine-eval/                <- tree-walk interpreter, bar-by-bar runner
│   ├── pine-vm/                  <- (Phase 6) bytecode compiler + stack VM
│   ├── pine-runtime/             <- ExecutionContext, SeriesBuf<T>, RuntimeConfig
│   ├── pine-stdlib/              <- ta.* / math.* / str.* / array.* / matrix.*
│   ├── pine-output/              <- plot / label / box / table / strategy output model
│   └── pine-cli/                 <- CLI: pine run, pine check, pine bench
│
├── pine-tv/                      <- pine-tv: web playground
│   ├── Cargo.toml                <- bin crate, depends on pine-runtime + pine-eval
│   ├── src/
│   │   ├── main.rs               <- Axum server entry, port 7070
│   │   ├── routes/
│   │   │   ├── run.rs            <- POST /api/run
│   │   │   ├── check.rs          <- POST /api/check
│   │   │   ├── ai.rs             <- POST /api/ai  (SSE streaming)
│   │   │   ├── data.rs           <- GET  /api/data/{symbol}/{tf}
│   │   │   └── ws.rs             <- WS   /ws/realtime
│   │   ├── engine/
│   │   │   ├── runner.rs         <- wraps pine-eval, manages ExecutionContext lifecycle
│   │   │   └── output.rs         <- serialise PineOutput -> JSON for frontend
│   │   └── data/
│   │       ├── loader.rs         <- CSV / SQLite -> OhlcvBar vec
│   │       └── feed.rs           <- WebSocket proxy for live market data
│   └── static/                   <- frontend (pure HTML + ES modules, no bundler)
│       ├── index.html            <- three-panel shell
│       ├── app.js                <- layout, resize handles, panel coordination
│       ├── chart.js              <- lightweight-charts wrapper + indicator overlay
│       ├── editor.js             <- Monaco editor + Pine syntax definition
│       ├── ai.js                 <- AI panel, SSE streaming, code extraction
│       ├── pine-syntax.js        <- Pine Script v6 Monarch token rules for Monaco
│       └── examples/             <- built-in example scripts (*.pine)
│
├── tests/                        <- pine-rs integration tests
│   ├── snapshots/                <- insta snapshot files (committed to git)
│   ├── scripts/                  <- *.pine test scripts by category
│   │   ├── basic/
│   │   ├── series/
│   │   ├── var_state/
│   │   ├── stdlib/ta/
│   │   ├── udt/
│   │   ├── strategy/
│   │   └── regression/           <- one file per bug: issue-NNN-*.pine
│   └── golden/                   <- expected CSV outputs for golden tests
│
└── docs/
    ├── SERIES_SEMANTICS.md       <- update whenever SeriesBuf is touched
    ├── NA_RULES.md               <- na propagation reference
    ├── API_OUTPUT_SCHEMA.md      <- pine-tv JSON output format (locked after Phase 5)
    └── BUILTIN_COVERAGE.md       <- built-in function implementation status
```

---

## 2. Product 1: pine-rs Interpreter

### 2.1 Architecture Pipeline

```
Pine Script source text (UTF-8)
        |
        v  pine-lexer
   Token stream  (logos + virtual INDENT/DEDENT/NEWLINE tokens)
        |
        v  pine-parser
   AST with Spans  (chumsky + Pratt for expressions, full v6 grammar)
        |
        v  pine-sema
   Typed HIR  (type inference, series annotation, var/varip hoisting)
        |
     +--+------------------+
     v                     v (Phase 6)
pine-eval               pine-vm
tree-walk               bytecode compiler + stack VM
     |
     v  pine-runtime
ExecutionContext
  +-- SeriesBuf<T> registry     (VecDeque per live series)
  +-- var slot table            (persistent across bars)
  +-- call-site series map      (per-callsite isolation for UDF)
  +-- RuntimeConfig             (max_bars_back, max_labels, ...)
        |
        v  pine-output
PineOutput { plots, labels, boxes, tables, signals, errors }
```

### 2.2 Critical Domain Rules (read before every eval change)

**Series alignment — most dangerous invariant**
Every branch of every if/else/for/while must push onto ALL live series buffers on
every bar, even the branch not taken. The sema pass annotates live series at each
branch point; eval enforces the push before branch exit.
Violations cause close[1] to read the wrong bar silently — extremely hard to debug.

**na semantics**
na is a typed missing-value that propagates: na + 1 == na, na == na is false.
All arithmetic/comparison goes through na_ops.rs. Never scatter na-checks inline.

**var / varip persistence**
var initialises once at bar_index 0 and persists. varip additionally updates on
every intra-bar realtime tick. Both stored in ExecutionContext::var_slots.

**UDF call-site series isolation**
Each call site (identified by (fn_name, call_expr_span)) gets its own series slot map.
Two calls f(close) and f(high) are completely independent contexts.

**max_bars_back**
Default 500, configurable via RuntimeConfig. Official TV limit is 5000.
Never hardcode 500 — always read from RuntimeConfig::max_bars_back.

### 2.3 Coding Standards — pine-rs crates

- Rust 2021 edition, MSRV 1.75
- cargo fmt enforced; cargo clippy -- -D warnings must be clean
- No unwrap() / expect() in any library crate (pine-lexer through pine-output)
- unwrap() allowed only in pine-cli and #[test] / #[bench] code
- unsafe requires // SAFETY: comment and PR review annotation
- Errors: thiserror inside libraries; miette for user-visible diagnostics
- All pub items require /// doc comments

### 2.4 Naming Conventions

| Item                  | Style           | Example                   |
|-----------------------|-----------------|---------------------------|
| AST nodes             | PascalCase      | BinExpr, IfStmt           |
| IR opcodes            | SCREAMING_SNAKE | PUSH_SERIES, CALL_BUILTIN |
| Built-in Rust fns     | ns_name         | ta_sma, math_abs          |
| Series types          | SeriesBuf<T>    | SeriesId                  |
| Error types           | PascalCase+Error| ParseError, RuntimeError  |
| Test functions        | test_ prefix    | test_ema_initial_na       |

### 2.5 Agent Workflows — pine-rs

#### Adding a new built-in function
1. Add signature + return type to crates/pine-stdlib/src/registry.rs
2. Implement in the correct module (ta.rs / math.rs / str.rs / ...)
3. Route na through na_ops.rs — never handle na inline in the fn body
4. Add tests/scripts/stdlib/<ns>/fn_name.pine test script
5. cargo test -p pine-stdlib -> cargo insta review -> commit snapshot
6. Add golden test against TradingView output for numeric precision

#### Adding a new AST node
1. Add variant to crates/pine-parser/src/ast.rs
2. Add parsing rule in the relevant parse_* combinator
3. Add Display impl (used by snapshot tests)
4. Add sema analysis pass in pine-sema/src/infer.rs
5. Add eval branch in pine-eval/src/eval_expr.rs or eval_stmt.rs
6. Add snapshot test

#### Fixing a series alignment bug
1. Write minimal repro .pine in tests/scripts/regression/issue-NNN-*.pine
2. cargo test to confirm wrong output
3. Trace SeriesBuf::push call sites in the relevant eval branch
4. Fix, confirm snapshot matches expected TV output
5. Update docs/SERIES_SEMANTICS.md

#### Never — pine-rs
- No unwrap() / expect() in library crates
- No unsafe without // SAFETY: comment
- No hardcoded 500 — use RuntimeConfig::max_bars_back
- No pine-tv imports in any pine-rs crate
- Never skip: cargo fmt --check && cargo clippy --workspace -- -D warnings

---

## 3. Product 2: pine-tv Playground

### 3.1 What pine-tv Is

pine-tv is a local, single-user web application that provides:
- A K-line chart (left panel) powered by lightweight-charts
- A Pine Script editor (middle panel) powered by Monaco with Pine v6 syntax
- An AI assistant (right panel) that generates and refines Pine code

It runs as a local Axum HTTP server on localhost:7070. The frontend is pure static
HTML + ES modules served directly — no bundler, no npm, no Node runtime.
The backend executes Pine scripts by calling pine-rs and returns JSON results.

### 3.2 Three-Panel Layout Contract

```
+----------------------------------------------------------+
|  pine-tv  [BTCUSDT] [1h] [Run] [Stop]           [...]   |  <- toolbar
+------------------+---------------+---------------------+
|                  |               |                     |
|   K-line chart   |  Pine editor  |   AI assistant      |
|  lightweight-    |   Monaco      |  +--------------+   |
|  charts          |               |  | chat history |   |
|                  |               |  +--------------+   |
|  plot overlays   |  [Run] button |  | input + send |   |
|  labels / boxes  |  error panel  |  +--------------+   |
|                  |               |  [Insert code]      |
+------------------+---------------+---------------------+
     ~45%               ~30%              ~25%
     (panels resizable with drag handles)
```

Cross-panel communication uses CustomEvent on window only.
Panels never call each other's functions directly.

| Event             | Fired by      | Consumed by | Payload            |
|-------------------|---------------|-------------|--------------------|
| pine:run          | editor Run btn| app.js      | {code, symbol, tf} |
| pine:result       | app.js        | chart.js    | PineOutput JSON    |
| pine:error        | app.js        | editor.js   | {errors[]}         |
| pine:insert-code  | ai.js         | editor.js   | {code: string}     |
| pine:symbol-change| toolbar       | chart+app   | {symbol, tf}       |

### 3.3 Backend API Contract

#### POST /api/run

Request:
```json
{ "code": "string", "symbol": "BTCUSDT", "timeframe": "1h", "bars": 500 }
```

Success response:
```json
{
  "ok": true,
  "exec_ms": 42,
  "plots": [
    {
      "id": "unique-id",
      "title": "SMA 20",
      "type": "line",
      "color": "#2196F3",
      "linewidth": 1,
      "pane": 0,
      "data": [{ "time": 1700000000, "value": 42000.5 }]
    }
  ],
  "labels":  [{ "time": 1700100000, "y": 43000.0, "text": "Buy", "color": "#4CAF50", "style": "label_up" }],
  "boxes":   [{ "left": 1700000000, "top": 43500.0, "right": 1700100000, "bottom": 42500.0, "border_color": "#FF5722", "bg_color": "#FF572233" }],
  "signals": [{ "time": 1700100000, "dir": "long", "qty": 1.0, "comment": "" }],
  "hlines":  [{ "y": 70.0, "color": "#FF0000", "title": "Overbought" }]
}
```

Error response:
```json
{
  "ok": false,
  "errors": [{ "line": 5, "col": 12, "end_col": 20, "msg": "type mismatch: expected float, got string" }]
}
```

#### POST /api/check
Request: `{ "code": "string" }`
Response: same error format, no execution performed.

#### POST /api/ai — Server-Sent Events

Request:
```json
{
  "message": "Write an RSI indicator with alerts",
  "context_code": "// current editor content",
  "context_errors": []
}
```

SSE stream (each line: `data: <json>\n\n`):
```
data: {"type": "text",       "delta": "Here is an RSI indicator:\n"}
data: {"type": "code_start", "lang": "pine"}
data: {"type": "code_delta", "delta": "//@version=6\n"}
data: {"type": "code_delta", "delta": "indicator(\"RSI\")\n"}
data: {"type": "code_end"}
data: {"type": "text",       "delta": "This fires alerts at 70/30."}
data: {"type": "done",       "full_code": "//@version=6\nindicator(...)..."}
```

Frontend shows "Insert code" button only after receiving `type: done`.

#### GET /api/data/{symbol}/{tf}
Returns OHLCV bars array. tf values: 1m 5m 15m 1h 4h 1d 1w

#### WS /ws/realtime
```
Client -> server:  {"action": "subscribe", "symbol": "BTCUSDT", "tf": "1m"}
Server -> client:  {"type": "tick",        "time": ..., "open": ..., ...}
Server -> client:  {"type": "bar_close",   "time": ..., "plots_delta": [...]}
```

### 3.4 Frontend Rules

No framework, no bundler. All JS is plain ES modules loaded directly by the browser.
Never introduce npm, webpack, vite, React, or Vue.

File responsibilities:

**app.js** — panel sizing, resize handles, toolbar events, calls /api/run, fires pine:result/pine:error

**chart.js** — lightweight-charts init; on pine:result adds/updates ISeriesApi for each
plot; manages pane 0 (overlay) vs pane 1+ (sub-pane); on symbol change clears all series

**editor.js** — Monaco init; registers pine language via pine-syntax.js; debounces
/api/check on keystrokes (300ms); on pine:error calls setModelMarkers; on pine:insert-code
replaces full editor content

**ai.js** — opens SSE to /api/ai; injects current editor code + errors as context in
every request; parses stream; shows Insert button only after full_code arrives

**pine-syntax.js** — Monarch tokenizer rules for Pine Script v6 keywords, operators,
built-in functions, colour literals

JavaScript standards:
- ES2022, no transpilation
- const and let only — never var
- All fetch calls wrapped in try/catch with user-visible error in status bar
- No global mutable state — use module-level WeakMap or closure where needed

### 3.5 Agent Workflows — pine-tv

#### Adding a new API endpoint
1. Create handler in pine-tv/src/routes/<name>.rs
2. Register route in main.rs router
3. Document request/response in section 3.3 of this file
4. Add integration test in pine-tv/tests/api_<name>.rs

#### Adding a new plot type (chart.js)
1. Add variant to pine-output/src/plot.rs
2. Serialise in pine-tv/src/engine/output.rs
3. Update docs/API_OUTPUT_SCHEMA.md version if shape changes
4. Handle new type field in chart.js applyPlots() function
5. Test visually with a .pine script that emits the new output type

#### Adding a new example script
1. Write script in pine-tv/static/examples/<name>.pine
2. Verify: cargo run -p pine-cli -- run pine-tv/static/examples/<name>.pine
3. Add filename to examples array in editor.js

#### Updating the AI system prompt
1. Edit prompt template in pine-tv/src/routes/ai.rs
2. Always inject: current editor code, last run errors, Pine v6 version header
3. Test with representative user messages before committing

#### Never — pine-tv
- No npm / bundler — keep frontend as plain ES modules
- No hardcoded API keys — read from ANTHROPIC_API_KEY env or ~/.pine-tv/config.toml
- No pine-rs internals in static/ JS files
- Do not call /api/run on every keystroke — only on explicit Run button click
- Do not auto-insert AI code without user clicking Insert

---

## 4. Development Phases

| Phase | Scope | Weeks | Milestone |
|-------|-------|-------|-----------|
| 1 | pine-rs: Lexer + Parser | 1–3 | Any v6 script parses; 100% snapshot coverage |
| 2 | pine-rs: Core Execution | 4–7 | SMA output matches TV ±1e-10 |
| 3 | pine-rs: Stdlib P1 | 8–10 | RSI/MACD/BB golden tests pass ±1e-8 |
| 4 | pine-rs: Full Language | 11–14 | 90% of public v6 scripts run without panic |
| 5A | pine-tv: Shell + Chart + Editor | 15–16 | Three panels running; /api/run works end-to-end |
| 5B | pine-tv: AI Panel | 16–17 | SSE streaming; Insert code button; error context |
| 5C | pine-rs: Full Output Layer | 17 | label/box/table/strategy.* complete |
| 6A | pine-rs: VM | 18–20 | Bytecode compiler + stack VM; 100k bar < 100ms |
| 6B | pine-tv: Sub-panes + RT | 20–22 | Sub-pane indicators; WS realtime; strategy tester |

---

## 5. Build & Run Reference

```bash
# pine-rs
cargo build --workspace
cargo test --workspace
cargo test --workspace && cargo insta review
cargo bench -p pine-eval
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo run -p pine-cli -- run examples/sma.pine --data data/BTCUSDT_1h.csv
cargo run -p pine-cli -- check my_script.pine

# pine-tv
export ANTHROPIC_API_KEY=sk-ant-...
cargo run -p pine-tv                           # http://localhost:7070
PINE_TV_STATIC=pine-tv/static cargo run -p pine-tv   # hot-reload static files
cargo test -p pine-tv

# fuzz (nightly)
cargo +nightly fuzz run fuzz_lexer  -- -max_total_time=300
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

---

## 6. CI Pipeline

| Job | Trigger | Steps | Blocks merge? |
|-----|---------|-------|--------------|
| lint | all push/PR | cargo fmt --check + clippy -D warnings | Yes |
| test | all push/PR | cargo test --workspace + insta test | Yes |
| golden | PR to main | run golden scripts, diff expected CSVs | Yes |
| tv-api | PR to main | cargo test -p pine-tv | Yes |
| fuzz-lexer | daily 02:00 UTC | 300s fuzz | alert on crash |
| fuzz-parser | daily 02:30 UTC | 300s fuzz | alert on crash |
| bench | weekly | cargo bench, store criterion output | alert if >10% regression |

---

## 7. Key Dependencies

```toml
# pine-rs
logos        = "0.14"
chumsky      = "0.9"
miette       = "5"
thiserror    = "1"
indexmap     = "2"
smallvec     = "1"
smartstring  = "1"
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
insta        = "1"
proptest     = "1"
criterion    = "0.5"

# pine-tv (additional)
axum              = "0.7"
tokio             = { version = "1", features = ["full"] }
tower-http        = { version = "0.5", features = ["fs", "cors", "compression-gzip"] }
sqlx              = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
tokio-tungstenite = "0.21"
reqwest           = { version = "0.11", features = ["stream"] }
```

---

## 8. Open Design Decisions

| # | Question | Current decision | Revisit when |
|---|----------|-----------------|-------------|
| 1 | Tree-walk vs VM Phase 1-5 | Tree-walk first | Phase 6 start |
| 2 | Series<T> generic or erased | Generic; erase only at stdlib boundary | If compile times blow up |
| 3 | max_bars_back global or per-var | Global RuntimeConfig + per-var fn override | Phase 4 |
| 4 | Strategy engine built-in or external | Output signals only; host app does fills | Phase 5 |
| 5 | pine-tv multiple scripts | Single script per session | Phase 6 |
| 6 | pine-tv realtime tick execution | Bar-close only Phase 5; tick-level Phase 6 | Phase 6 |
| 7 | AI provider | Claude primary; OpenAI fallback via same interface | Phase 5B |
| 8 | pine-tv sub-pane indicators | Phase 6 — needs pane layout system in chart.js | Phase 6 |

---

*Last updated: covers pine-rs Phases 1-6 + pine-tv Phases A-C (5A/5B/5C/6A/6B)*
*Update this file whenever: workspace layout changes, API contract changes, new agent workflow needed.*
