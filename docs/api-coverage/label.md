# Label

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Styles

| Function | Status | pine-rs | Description |
| ------------------------------- | ------ | --- | ----------------------- |
| `label.style_arrowdown` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Arrow down style |
| `label.style_arrowup` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Arrow up style |
| `label.style_circle` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Circle style |
| `label.style_cross` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Cross style |
| `label.style_diamond` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Diamond style |
| `label.style_flag` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Flag style |
| `label.style_label_center` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label center style |
| `label.style_label_down` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label down style |
| `label.style_label_left` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label left style |
| `label.style_label_lower_left` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label lower left style |
| `label.style_label_lower_right` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label lower right style |
| `label.style_label_right` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label right style |
| `label.style_label_up` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label up style |
| `label.style_label_upper_left` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label upper left style |
| `label.style_label_upper_right` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Label upper right style |
| `label.style_none` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | No style |
| `label.style_square` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Square style |
| `label.style_text_outline` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Text outline style |
| `label.style_triangledown` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Triangle down style |
| `label.style_triangleup` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Triangle up style |
| `label.style_xcross` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | X-cross style |

### Management

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | --------------------- |
| `label.all` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | All labels collection |
| `label` | ✅ | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Casts na to label |
| `label.copy()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Copy label |
| `label.delete()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Delete label |
| `label.new()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Create new label (supports both `x, y` and `point` signatures, with `force_overlay`) |

### Getters

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | ---------------- |
| `label.get_text()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get label text |
| `label.get_x()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get x coordinate |
| `label.get_y()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Get y coordinate |

### Setters

| Function | Status | pine-rs | Description |
| ------------------------------ | ------ | --- | ----------------------- |
| `label.set_color()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set label color |
| `label.set_point()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set label point |
| `label.set_size()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set label size |
| `label.set_style()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set label style |
| `label.set_text()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set label text |
| `label.set_text_font_family()` |  | ✔️ 多数仅在 pine-output / 常量解析 | Set text font family |
| `label.set_text_formatting()` |  | ✔️ 多数仅在 pine-output / 常量解析 | Set text formatting |
| `label.set_textalign()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text alignment |
| `label.set_textcolor()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set text color |
| `label.set_tooltip()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set tooltip |
| `label.set_x()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x coordinate |
| `label.set_xloc()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x-location |
| `label.set_xy()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set x and y coordinates |
| `label.set_y()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set y coordinate |
| `label.set_yloc()` | ✅ | ✔️ 多数仅在 pine-output / 常量解析 | Set y-location |
