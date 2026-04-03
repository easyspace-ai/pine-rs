# Chart

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Chart Properties

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | ---------------- |
| `chart.bg_color` | ✅ | ❌ | Background color |
| `chart.fg_color` | ✅ | ❌ | Foreground color |

### Chart Type Detection

| Function | Status | pine-rs | Description |
| --------------------- | ------ | --- | ----------------------------- |
| `chart.is_heikinashi` | ✅ | ❌ | Check if Heikin Ashi chart |
| `chart.is_kagi` | ✅ | ❌ | Check if Kagi chart |
| `chart.is_linebreak` | ✅ | ❌ | Check if Line Break chart |
| `chart.is_pnf` | ✅ | ❌ | Check if Point & Figure chart |
| `chart.is_range` | ✅ | ❌ | Check if Range chart |
| `chart.is_renko` | ✅ | ❌ | Check if Renko chart |
| `chart.is_standard` | ✅ | ❌ | Check if standard chart |

### Visible Range

| Function | Status | pine-rs | Description |
| ------------------------------ | ------ | --- | ---------------------- |
| `chart.left_visible_bar_time` |  | ❌ | Left visible bar time |
| `chart.right_visible_bar_time` |  | ❌ | Right visible bar time |

### Chart Point

| Function | Status | pine-rs | Description |
| -------------------------- | ------ | --- | ----------------------- |
| `chart.point.copy()` | ✅ | ❌ | Copy chart point |
| `chart.point.from_index()` | ✅ | ❌ | Create point from index |
| `chart.point.from_time()` | ✅ | ❌ | Create point from time |
| `chart.point.new()` | ✅ | ❌ | Create new chart point |
| `chart.point.now()` | ✅ | ❌ | Get current chart point |

### Chart Point Fields

| Field                | Status | Description         |
| -------------------- | ------ | ------------------- |
| `chart.point.index`  | ✅     | Bar index of point  |
| `chart.point.price`  | ✅     | Price of point      |
| `chart.point.time`   | ✅     | Timestamp of point  |
