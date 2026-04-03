# String

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Query

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | ---------------------------------- |
| `str.contains()` | ✅ | ✅ | Check if string contains substring |
| `str.endswith()` | ✅ | ❌ | Check if string ends with suffix |
| `str.length()` | ✅ | ✅ | Get string length |
| `str.match()` | ✅ | ❌ | Match string against regex |
| `str.pos()` | ✅ | ❌ | Find position of substring |
| `str.startswith()` | ✅ | ❌ | Check if string starts with prefix |

### Formatting

| Function | Status | pine-rs | Description |
| ------------------- | ------ | --- | ---------------------------- |
| `str.format()` | ✅ | ❌ | Format string with arguments |
| `str.format_time()` | ❌ | ❌ | Format time value |

### Transformation

| Function | Status | pine-rs | Description |
| ------------------- | ------ | --- | ------------------------ |
| `str.lower()` | ✅ | ✅ | Convert to lowercase |
| `str.repeat()` | ✅ | ❌ | Repeat string |
| `str.replace()` | ✅ | ✅ | Replace first occurrence |
| `str.replace_all()` | ✅ | ❌ | Replace all occurrences |
| `str.trim()` | ✅ | ✅ | Remove whitespace |
| `str.upper()` | ✅ | ✅ | Convert to uppercase |

### Parsing

| Function | Status | pine-rs | Description |
| ----------------- | ------ | --- | ----------------------- |
| `str.split()` |  | ✅ | Split string into array |
| `str.substring()` |  | ✅ | Extract substring |

### Conversion

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | ----------------------- |
| `str.tonumber()` | ✅ | ✅ | Parse string to number |
| `str.tostring()` | ✅ | ✅ | Convert value to string |
