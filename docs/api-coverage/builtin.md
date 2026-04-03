# Builtin

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Variables

| Function | Status | pine-rs | Description |
| ----------------- | ------ | --- | -------------------------------------- |
| `bar_index` | ✅ | ✅ | Current bar index |
| `close` | ✅ | ✅ | Close price |
| `high` | ✅ | ✅ | High price |
| `hl2` | ✅ | ✅ | Average of high and low |
| `hlc3` | ✅ | ✅ | Average of high, low, and close |
| `hlcc4` | ✅ | ❌ | Average of high, low, close, and close |
| `last_bar_index` | ✅ | ❌ | Index of last bar |
| `last_bar_time` | ✅ | ❌ | Time of last bar |
| `low` | ✅ | ✅ | Low price |
| `na` | ✅ | ✅ | Not a number (NaN) |
| `ohlc4` | ✅ | ✅ | Average of open, high, low, and close |
| `open` | ✅ | ✅ | Open price |
| `timenow` | ✅ | ❌ | Current time |
| `volume` | ✅ | ✅ | Volume |
| `ask` |  | ❌ | Ask price |
| `bid` |  | ❌ | Bid price |
| `dayofmonth` | ✅ | ❌ | Day of month |
| `dayofweek` | ✅ | ❌ | Day of week |
| `hour` | ✅ | ❌ | Hour |
| `minute` | ✅ | ❌ | Minute |
| `month` | ✅ | ❌ | Month |
| `second` | ✅ | ❌ | Second |
| `time` | ✅ | ✅ | Bar time |
| `time_close` | ✅ | ❌ | Bar close time |
| `time_tradingday` | ✅ | ❌ | Trading day time |
| `weekofyear` | ✅ | ❌ | Week of year |
| `year` | ✅ | ❌ | Year |

### Constants

| Function | Status | pine-rs | Description |
| -------- | ------ | --- | ------------- |
| `false` | ✅ | ✅ | Boolean false |
| `true` | ✅ | ✅ | Boolean true |

### Functions

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | --------------------- |
| `indicator()` | ✅ | ✔️ 多为普通调用，无完整 TV 声明语义 | Indicator declaration |
| `input()` | ✅ | ❌ | Input parameter |
| `na()` | ✅ | ✅ | Check if value is NaN |
| `nz()` | ✅ | ✅ | Replace NaN with zero |
| `alert()` | ✅ | ❌ | Alert function |
| `alertcondition()` | ✅ | ❌ | Alert condition |
| `bool()` | ✅ | ✔️ 类型/转换语义未与 TV 完全对齐 | Boolean conversion |
| `box()` | ✅ | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Box object |
| `color()` | ✅ | ✔️ 类型/转换语义未与 TV 完全对齐 | Color object |
| `dayofmonth()` | ✅ | ❌ | Day of month function |
| `dayofweek()` | ✅ | ❌ | Day of week function |
| `fill()` | ✅ | ❌ | Fill function |
| `fixnan()` | ✅ | ❌ | Fix NaN values |
| `float()` | ✅ | ✔️ 类型/转换语义未与 TV 完全对齐 | Float conversion |
| `hline()` | ✅ | ✔️ pine-output 有 helper；eval 未挂接 | Horizontal line |
| `hour()` | ✅ | ❌ | Hour function |
| `int()` | ✅ | ✔️ 类型/转换语义未与 TV 完全对齐 | Integer conversion |
| `label()` | ✅ | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Label object |
| `library()` |  | ✔️ 解析/AST 向官方靠拢；执行层 stub | Library declaration |
| `line()` | ✅ | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Line object |
| `linefill()` | ✅ | ❌ | Linefill object |
| `max_bars_back()` |  | ❌ | Maximum bars back |
| `minute()` | ✅ | ❌ | Minute function |
| `month()` | ✅ | ❌ | Month function |
| `second()` | ✅ | ❌ | Second function |
| `strategy()` |  | ✔️ 信号级子集；非完整撮合 | Strategy declaration |
| `string()` | ✅ | ✔️ 类型/转换语义未与 TV 完全对齐 | String conversion |
| `table()` | ✅ | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Table object |
| `time()` | ✅ | ❌ | Time function |
| `time_close()` | ✅ | ❌ | Time close function |
| `timestamp()` | ✅ | ❌ | Timestamp function |
| `weekofyear()` | ✅ | ❌ | Week of year function |
| `year()` | ✅ | ❌ | Year function |
| `runtime.error()` |  | ❌ | Runtime error |
