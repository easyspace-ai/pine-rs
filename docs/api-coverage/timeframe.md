# Timeframe

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Timeframe Type Checks

| Function | Status | pine-rs | Description |
| ---------------------- | ------ | --- | ----------------------------- |
| `timeframe.isdaily` | ✅ | ❌ | Check if daily timeframe |
| `timeframe.isdwm` | ✅ | ❌ | Check if daily/weekly/monthly |
| `timeframe.isintraday` | ✅ | ❌ | Check if intraday timeframe |
| `timeframe.isminutes` | ✅ | ❌ | Check if minutes timeframe |
| `timeframe.ismonthly` | ✅ | ❌ | Check if monthly timeframe |
| `timeframe.isseconds` | ✅ | ❌ | Check if seconds timeframe |
| `timeframe.isticks` | ✅ | ❌ | Check if ticks timeframe |
| `timeframe.isweekly` | ✅ | ❌ | Check if weekly timeframe |

### Timeframe Properties

| Function | Status | pine-rs | Description |
| ----------------------- | ------ | --- | ------------------------ |
| `timeframe.main_period` | ✅ | ❌ | Main period of timeframe |
| `timeframe.multiplier` | ✅ | ❌ | Timeframe multiplier |
| `timeframe.period` | ✅ | ❌ | Timeframe period |

### Timeframe Functions

| Function | Status | pine-rs | Description |
| -------------------------- | ------ | --- | ----------------------------- |
| `timeframe.change()` | ✅ | ❌ | Change timeframe |
| `timeframe.from_seconds()` | ✅ | ❌ | Create timeframe from seconds |
| `timeframe.in_seconds()` | ✅ | ❌ | Convert timeframe to seconds |
