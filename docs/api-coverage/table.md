# Table

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Cell Operations

| Function | Status | pine-rs | Description |
| ----------------------------------- | ------ | --- | ------------------------------ |
| `table.cell()` | ✅ | ✔️ 多数未贯通 | Get cell |
| `table.cell_set_bgcolor()` | ✅ | ✔️ 多数未贯通 | Set cell background color |
| `table.cell_set_height()` | ✅ | ✔️ 多数未贯通 | Set cell height |
| `table.cell_set_text()` | ✅ | ✔️ 多数未贯通 | Set cell text |
| `table.cell_set_text_color()` | ✅ | ✔️ 多数未贯通 | Set cell text color |
| `table.cell_set_text_font_family()` | ✅ | ✔️ 多数未贯通 | Set cell text font family |
| `table.cell_set_text_formatting()` |  | ✔️ 多数未贯通 | Set cell text formatting |
| `table.cell_set_text_halign()` | ✅ | ✔️ 多数未贯通 | Set cell text horizontal align |
| `table.cell_set_text_size()` | ✅ | ✔️ 多数未贯通 | Set cell text size |
| `table.cell_set_text_valign()` | ✅ | ✔️ 多数未贯通 | Set cell text vertical align |
| `table.cell_set_tooltip()` | ✅ | ✔️ 多数未贯通 | Set cell tooltip |
| `table.cell_set_width()` | ✅ | ✔️ 多数未贯通 | Set cell width |
| `table.merge_cells()` | ✅ | ✔️ 多数未贯通 | Merge cells |

### Management

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | --------------------- |
| `table()` |  | ✔️ pine-output 有对象模型；脚本 API 未贯通 | Casts na to table |
| `table.all` | ✅ | ✔️ 多数未贯通 | All tables collection |
| `table.clear()` | ✅ | ✔️ 多数未贯通 | Clear table |
| `table.delete()` | ✅ | ✔️ 多数未贯通 | Delete table |
| `table.new()` | ✅ | ✔️ 多数未贯通 | Create new table |

### Table Settings

| Function | Status | pine-rs | Description |
| -------------------------- | ------ | --- | -------------------- |
| `table.set_bgcolor()` | ✅ | ✔️ 多数未贯通 | Set background color |
| `table.set_border_color()` | ✅ | ✔️ 多数未贯通 | Set border color |
| `table.set_border_width()` | ✅ | ✔️ 多数未贯通 | Set border width |
| `table.set_frame_color()` | ✅ | ✔️ 多数未贯通 | Set frame color |
| `table.set_frame_width()` | ✅ | ✔️ 多数未贯通 | Set frame width |
| `table.set_position()` | ✅ | ✔️ 多数未贯通 | Set table position |
