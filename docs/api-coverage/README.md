# API Coverage（PineTS 结构 + pine-rs 核对）

本目录从 [`three/PineTS/docs/api-coverage`](../../three/PineTS/docs/api-coverage) **原样复制** PineTS 的逐函数表格，并经脚本插入 **pine-rs** 列。

注意：

- 这些页面的强项是“逐函数对照 PineTS 结构”。
- 这些页面不保证始终等于 pine-rs 当前 registry 的实时计数。
- 若要看 pine-rs 当前已注册函数数量与验证深度，请优先看 [`docs/FUNCTION_COVERAGE.md`](../FUNCTION_COVERAGE.md)。

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
