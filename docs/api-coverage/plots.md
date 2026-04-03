# Plots

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



| Function | Status | pine-rs | Description |
| -------------- | ------ | --- | ---------------------- |
| `plot()` | ✅ | ✅ | Plot a series |
| `plotchar()` | ✅ | ✔️ pine-output 有 helper；eval 未挂接 | Plot character markers |
| `plotarrow()` | ✅ | ✔️ pine-output 有 helper；eval 未挂接 | Plot arrow markers |
| `plotbar()` | ✅ | ❌ | Plot bar chart |
| `plotcandle()` | ✅ | ❌ | Plot candlestick chart |
| `plotshape()` | ✅ | ✔️ pine-output 有 helper；eval 未挂接 | Plot shape markers |
| `barcolor()` | ✅ | ❌ | Set bar color |
| `bgcolor()` | ✅ | ✔️ pine-output 有 helper；eval 未挂接 | Set background color |
| `hline()` | ✅ | ✔️ pine-output 有 helper；eval 未挂接 | Plot horizontal line |
| `fill()` | ✅ | ❌ | Fill between two plots or hlines |

---

### Plot Title Collisions

When multiple `plot()` (or `hline()`) calls share the same `title`, PineTS disambiguates them by appending a `#N` suffix to the plot key. The first plot keeps the plain title, and subsequent collisions are numbered sequentially:

- First `plot(close, "SMA")` &rarr; plot key `"SMA"`
- Second `plot(open, "SMA")` &rarr; plot key `"SMA#1"`
- Third `plot(high, "SMA")` &rarr; plot key `"SMA#2"`

{: .warning }
Using duplicate plot titles is **not recommended**. The `#N` suffix ordering depends on execution order and may lead to fragile references. Always prefer unique titles for each plot. A more elegant solution for this case will be provided in a future version of PineTS.
