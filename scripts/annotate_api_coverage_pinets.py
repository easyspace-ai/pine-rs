#!/usr/bin/env python3
"""
Post-process docs/api-coverage/*.md copied from PineTS:
- Strip Jekyll front matter
- Insert a pine-rs status column (✅ / ✔️ / ❌ / ⏳) next to PineTS Status

Re-run after refreshing the copy from PineTS.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
STDLIB_SRC = REPO / "crates/pine-stdlib/src"


def extract_registry_keys() -> set[str]:
    names: set[str] = set()
    for path in STDLIB_SRC.glob("*.rs"):
        text = path.read_text(encoding="utf-8")
        for m in re.finditer(
            r'#\[pine_builtin\(name\s*=\s*"([^"]+)"\s*,\s*namespace\s*=\s*"([^"]+)"',
            text,
        ):
            names.add(f"{m.group(2)}.{m.group(1)}")
        lines = text.splitlines()
        for i, line in enumerate(lines):
            m = re.search(r'FunctionMeta::new\("([^"]+)"\)', line)
            if not m:
                continue
            base = m.group(1)
            ns = None
            for j in range(i + 1, min(i + 12, len(lines))):
                m2 = re.search(r'\.with_namespace\("([^"]+)"\)', lines[j])
                if m2:
                    ns = m2.group(1)
                    break
            if ns:
                names.add(f"{ns}.{base}")
            else:
                names.add(base)
    return names


# Injected by pine-eval runner (see crates/pine-eval/src/runner.rs)
RUNNER_SERIES_VARS: frozenset[str] = frozenset(
    {
        "open",
        "high",
        "low",
        "close",
        "volume",
        "time",
        "hl2",
        "hlc3",
        "ohlc4",
        "bar_index",
    }
)

# Eval/global builtins not always visible as registry keys
SPECIAL_OK: frozenset[str] = frozenset(
    {
        "na",
        "nz",
        "plot",
        "true",
        "false",
    }
)

# parse-only or stub; show as partial
PARTIAL_NOTE: dict[str, str] = {
    "indicator": "✔️ 多为普通调用，无完整 TV 声明语义",
    "library": "✔️ 解析/AST 向官方靠拢；执行层 stub",
    "strategy": "✔️ 信号级子集；非完整撮合",
    "box": "✔️ pine-output 有对象模型；脚本 API 未贯通",
    "label": "✔️ pine-output 有对象模型；脚本 API 未贯通",
    "line": "✔️ pine-output 有对象模型；脚本 API 未贯通",
    "linefill": "❌",
    "table": "✔️ pine-output 有对象模型；脚本 API 未贯通",
    "hline": "✔️ pine-output 有 helper；eval 未挂接",
    "bgcolor": "✔️ pine-output 有 helper；eval 未挂接",
    "plotshape": "✔️ pine-output 有 helper；eval 未挂接",
    "plotchar": "✔️ pine-output 有 helper；eval 未挂接",
    "plotarrow": "✔️ pine-output 有 helper；eval 未挂接",
    "fill": "❌",
    "barcolor": "❌",
    "plotbar": "❌",
    "plotcandle": "❌",
}

LEGEND = """\
> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>
"""


def normalize_api_key(raw: str) -> str:
    s = raw.strip().strip("`").strip()
    if s.endswith("()"):
        s = s[:-2]
    return s


def is_call_form(fn_cell: str) -> bool:
    raw = fn_cell.strip().strip("`").strip()
    return raw.endswith("()")


def classify_pine_rs(key: str, reg: set[str]) -> str:
    nk = normalize_api_key(key)
    # Bar time series `time` is injected; callable `time(...)` overloads are not.
    if nk == "time" and is_call_form(key):
        return "❌"
    if nk.startswith("request."):
        return "⏳"

    if nk in PARTIAL_NOTE:
        return PARTIAL_NOTE[nk]

    # strategy.* methods
    if nk.startswith("strategy.") and nk in reg:
        return "✅"

    if nk in SPECIAL_OK:
        return "✅"
    if nk in RUNNER_SERIES_VARS:
        return "✅"
    if nk in reg:
        return "✅"

    # Common alternate forms
    if nk == "na()" or nk == "na":
        return "✅"

    # Type-looking builtins in PineTS tables (int, float, string, bool, color)
    if nk in ("int", "float", "string", "bool", "color"):
        return "✔️ 类型/转换语义未与 TV 完全对齐"

    if nk.startswith("syminfo.") or nk.startswith("ticker.") or nk.startswith("session."):
        return "❌"

    if nk.startswith("chart."):
        return "❌"

    if nk.startswith("matrix.") or nk.startswith("polyline."):
        return "❌"

    if nk.startswith("log.") and nk != "math.log":
        return "❌"

    # input.* not in registry
    if nk.startswith("input.") and nk not in reg:
        return "❌"

    if nk.startswith("runtime."):
        return "❌"

    if nk.startswith("barstate."):
        return "❌"

    # drawing namespaces
    if nk.startswith("label.") or nk.startswith("line.") or nk.startswith("box."):
        if nk in reg:
            return "✅"
        return "✔️ 多数仅在 pine-output / 常量解析"

    if nk.startswith("color.") and nk in reg:
        return "✅"

    if nk.startswith("array.") and nk in reg:
        return "✅"

    if nk.startswith("map.") and nk in reg:
        return "✅"

    if nk.startswith("str.") and nk in reg:
        return "✅"

    if nk.startswith("math."):
        if nk in reg:
            return "✅"
        # constants math.pi etc.
        return "❌"

    if nk.startswith("ta.") and nk in reg:
        return "✅"

    if nk.startswith("ta."):
        return "❌"

    if nk.startswith("timeframe."):
        return "❌"

    if nk in ("timenow", "last_bar_index", "last_bar_time", "hlcc4", "time_close", "time_tradingday"):
        return "❌"

    if nk.startswith("dayof") or nk in ("hour", "minute", "second", "month", "year", "weekofyear"):
        # Pine uses dayofmonth vs dayofmonth() — we have neither in runner
        return "❌"

    if nk in ("ask", "bid", "fixnan", "alert", "alertcondition", "max_bars_back", "timestamp"):
        return "❌"

    if nk.startswith("table."):
        return "✔️ 多数未贯通"

    if nk.startswith("plot"):
        if nk == "plot":
            return "✅"
        return "✔️ 见 pine-output；eval 未全线挂钩"

    return "❌"


def strip_front_matter(lines: list[str]) -> list[str]:
    if not lines or lines[0].strip() != "---":
        return lines
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            return lines[i + 1 :]
    return lines


def first_md_heading(raw: list[str]) -> str | None:
    for ln in raw:
        m = re.match(r"^#+\s+(.+)$", ln)
        if m:
            return m.group(1).strip()
    return None


def process_markdown(text: str, reg: set[str]) -> str:
    lines = text.splitlines(keepends=True)
    raw = [ln.rstrip("\n") for ln in lines]
    raw = strip_front_matter(raw)

    title = first_md_heading(raw)
    out: list[str] = []
    skip_first_heading = False
    if title:
        out.append(f"# {title}\n\n")
        out.append(LEGEND)
        out.append("\n")
        skip_first_heading = True

    i = 0
    while i < len(raw):
        ln = raw[i]
        if skip_first_heading and re.match(r"^#+\s+", ln):
            skip_first_heading = False
            i += 1
            continue

        parts = [p.strip() for p in ln.split("|")]
        if parts and parts[0] == "":
            parts = parts[1:]
        if parts and parts[-1] == "":
            parts = parts[:-1]

        is_table_row = ln.strip().startswith("|") and len(parts) >= 3
        already = (
            len(parts) >= 4 and parts[0] == "Function" and parts[2] == "pine-rs"
        )
        if already:
            while i < len(raw) and raw[i].strip().startswith("|"):
                out.append(raw[i] + "\n")
                i += 1
            continue

        if (
            is_table_row
            and parts[0] == "Function"
            and parts[1] == "Status"
            and i + 1 < len(raw)
        ):
            sep = [p.strip() for p in raw[i + 1].split("|")]
            if sep and sep[0] == "":
                sep = sep[1:]
            if sep and sep[-1] == "":
                sep = sep[:-1]
            if len(sep) >= 3 and set(sep[1]) <= set("- :"):
                out.append(f"| {parts[0]} | {parts[1]} | pine-rs | {parts[2]} |\n")
                out.append(f"| {sep[0]} | {sep[1]} | --- | {sep[2]} |\n")
                i += 2
                while i < len(raw):
                    row_ln = raw[i]
                    if not row_ln.strip().startswith("|"):
                        break
                    rp = [p.strip() for p in row_ln.split("|")]
                    if rp and rp[0] == "":
                        rp = rp[1:]
                    if rp and rp[-1] == "":
                        rp = rp[:-1]
                    if len(rp) < 3:
                        out.append(row_ln + "\n")
                        i += 1
                        break
                    psts = classify_pine_rs(rp[0], reg)
                    rest = " | ".join(rp[2:])
                    out.append(f"| {rp[0]} | {rp[1]} | {psts} | {rest} |\n")
                    i += 1
                continue

        out.append(ln + "\n")
        i += 1

    return "".join(out)


def write_index(repo: Path) -> None:
    """README.md for docs/api-coverage (mirrors PineTS nav list)."""
    index = repo / "docs" / "api-coverage" / "README.md"
    body = """# API Coverage（PineTS 结构 + pine-rs 核对）

本目录从 [`three/PineTS/docs/api-coverage`](../../three/PineTS/docs/api-coverage) **原样复制** PineTS 的逐函数表格，并经脚本插入 **pine-rs** 列。

- **更新(copy)**：`rm -rf docs/api-coverage && cp -R three/PineTS/docs/api-coverage docs/api-coverage`
- **更新(标注)**：`python3 scripts/annotate_api_coverage_pinets.py`

维度总览（非函数级）：[`docs/API_COVERAGE.md`](../API_COVERAGE.md)

---

## 目录（与 PineTS 一致）

| 主题 | 文档 |
|------|------|
| Builtin | [builtin.md](./builtin.md) |
| Input | [input.md](./input.md) |
| Math | [math.md](./math.md) |
| Technical Analysis | [ta.md](./ta.md) |
| Array | [array.md](./array.md) |
| Box | [box.md](./box.md) |
| Chart | [chart.md](./chart.md) |
| Color | [color.md](./color.md) |
| Label | [label.md](./label.md) |
| Line | [line.md](./line.md) |
| Linefill | [linefill.md](./linefill.md) |
| Log | [log.md](./log.md) |
| Map | [map.md](./map.md) |
| Matrix | [matrix.md](./matrix.md) |
| Plots | [plots.md](./plots.md) |
| Request | [request.md](./request.md) |
| String | [str.md](./str.md) |
| Strategy | [strategy.md](./strategy.md) |
| Table | [table.md](./table.md) |
| Syminfo | [syminfo.md](./syminfo.md) |
| Runtime | [runtime.md](./runtime.md) |
| Polyline | [polyline.md](./polyline.md) |
| Others | [others.md](./others.md) |
| Barstate | [barstate.md](./barstate.md) |
| Session | [session.md](./session.md) |
| Ticker | [ticker.md](./ticker.md) |
| Timeframe | [timeframe.md](./timeframe.md) |
| Types | [types.md](./types.md) |
| Constants audit | [constants-audit.md](./constants-audit.md) |

子目录 `pinescript-v6/*.json` 为 PineTS 元数据，可供脚本或外部工具使用。
"""
    index.write_text(body, encoding="utf-8")


def main() -> int:
    reg = extract_registry_keys()
    cov = REPO / "docs" / "api-coverage"
    if not cov.is_dir():
        print("missing docs/api-coverage", file=sys.stderr)
        return 1

    for md in sorted(cov.glob("*.md")):
        if md.name == "README.md":
            continue
        text = md.read_text(encoding="utf-8")
        new_text = process_markdown(text, reg)
        md.write_text(new_text, encoding="utf-8")
        print("annotated", md.relative_to(REPO))

    write_index(REPO)
    print("wrote", cov / "README.md")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
