# Input

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Input Types

| Function | Status | pine-rs | Description |
| ------------------- | ------ | --- | ----------------- |
| `input.bool()` | ✅ | ✅ | Boolean input |
| `input.color()` | ✅ | ✅ | Color input |
| `input.enum()` | ✅ | ❌ | Enumeration input |
| `input.float()` | ✅ | ✅ | Float input |
| `input.int()` | ✅ | ✅ | Integer input |
| `input.price()` | ✅ | ❌ | Price input |
| `input.session()` | ✅ | ❌ | Session input |
| `input.source()` | ✅ | ✅ | Source input |
| `input.string()` | ✅ | ✅ | String input |
| `input.symbol()` | ✅ | ✅ | Symbol input |
| `input.text_area()` | ✅ | ❌ | Text area input |
| `input.time()` | ✅ | ❌ | Time input |
| `input.timeframe()` | ✅ | ✅ | Timeframe input |
