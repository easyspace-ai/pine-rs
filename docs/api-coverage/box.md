# Box

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Management

| Function | Status | pine-rs | Description |
| -------------- | ------ | --- | -------------------- |
| `box.all` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | All boxes collection |
| `box.copy()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Copy box |
| `box.delete()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Delete box |
| `box.new()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Create new box |

### Getters

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | --------------------- |
| `box.get_bottom()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get bottom coordinate |
| `box.get_left()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get left coordinate |
| `box.get_right()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get right coordinate |
| `box.get_top()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get top coordinate |

### Setters

| Function | Status | pine-rs | Description |
| ------------------------------ | ------ | --- | ----------------------------- |
| `box.set_bgcolor()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set background color |
| `box.set_border_color()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set border color |
| `box.set_border_style()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set border style |
| `box.set_border_width()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set border width |
| `box.set_bottom()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set bottom coordinate |
| `box.set_bottom_right_point()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set bottom-right point |
| `box.set_extend()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set extend mode |
| `box.set_left()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set left coordinate |
| `box.set_lefttop()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set left-top point |
| `box.set_right()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set right coordinate |
| `box.set_rightbottom()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set right-bottom point |
| `box.set_text()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text |
| `box.set_text_color()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text color |
| `box.set_text_font_family()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text font family |
| `box.set_text_formatting()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text formatting |
| `box.set_text_halign()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text horizontal alignment |
| `box.set_text_size()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text size |
| `box.set_text_valign()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text vertical alignment |
| `box.set_text_wrap()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text wrap |
| `box.set_top()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set top coordinate |
| `box.set_top_left_point()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set top-left point |
| `box.set_xloc()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x-location |
