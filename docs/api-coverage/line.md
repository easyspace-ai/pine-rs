# Line

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Styles

| Function | Status | pine-rs | Description |
| ------------------------ | ------ | --- | ----------------- |
| `line.style_arrow_both` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Arrow both style |
| `line.style_arrow_left` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Arrow left style |
| `line.style_arrow_right` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Arrow right style |
| `line.style_dashed` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Dashed style |
| `line.style_dotted` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Dotted style |
| `line.style_solid` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Solid style |

### Management

| Function | Status | pine-rs | Description |
| --------------- | ------ | --- | -------------------- |
| `line.all` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | All lines collection |
| `line()` | ✅ | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Casts na to line |
| `line.copy()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Copy line |
| `line.delete()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Delete line |
| `line.new()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Create new line (supports both `x, y` and `point` signatures, with `force_overlay`) |

### Getters

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | ----------------- |
| `line.get_price()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get line price |
| `line.get_x1()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get x1 coordinate |
| `line.get_x2()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get x2 coordinate |
| `line.get_y1()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get y1 coordinate |
| `line.get_y2()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get y2 coordinate |

### Setters

| Function | Status | pine-rs | Description |
| ------------------------- | ------ | --- | ------------------- |
| `line.set_color()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set line color |
| `line.set_extend()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set extend mode |
| `line.set_first_point()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set first point |
| `line.set_second_point()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set second point |
| `line.set_style()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set line style |
| `line.set_width()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set line width |
| `line.set_x1()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x1 coordinate |
| `line.set_x2()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x2 coordinate |
| `line.set_xloc()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x-location |
| `line.set_xy1()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set xy1 coordinates |
| `line.set_xy2()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set xy2 coordinates |
| `line.set_y1()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set y1 coordinate |
| `line.set_y2()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set y2 coordinate |
