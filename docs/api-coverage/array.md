# Array

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Creation & Initialization

| Function | Status | pine-rs | Description |
| ---------------------- | ------ | --- | ---------------------------- |
| `array.copy()` | ✅ | ✅ | Create copy of array |
| `array.from()` | ✅ | ✅ | Create array from arguments |
| `array.new_bool()` | ✅ | ✅ | Create boolean array |
| `array.new_box()` |  | ❌ | Create box array |
| `array.new_color()` |  | ✅ | Create color array |
| `array.new_float()` | ✅ | ✅ | Create float array |
| `array.new_int()` | ✅ | ✅ | Create int array |
| `array.new_label()` |  | ❌ | Create label array |
| `array.new_line()` |  | ❌ | Create line array |
| `array.new_linefill()` |  | ❌ | Create linefill array |
| `array.new_string()` | ✅ | ✅ | Create string array |
| `array.new_table()` |  | ❌ | Create table array |
| `array.new<type>()` | ✅ | ❌ | Create typed array (generic) |

### Element Access

| Function | Status | pine-rs | Description |
| --------------- | ------ | --- | ------------------ |
| `array.first()` | ✅ | ✅ | Get first element |
| `array.get()` | ✅ | ✅ | Get value at index |
| `array.last()` | ✅ | ✅ | Get last element |
| `array.set()` | ✅ | ✅ | Set value at index |

### Modification

| Function | Status | pine-rs | Description |
| ----------------- | ------ | --- | ---------------------------- |
| `array.clear()` | ✅ | ✅ | Remove all elements |
| `array.fill()` | ✅ | ✅ | Fill array with value |
| `array.insert()` | ✅ | ✅ | Insert element at index |
| `array.pop()` | ✅ | ✅ | Remove last element |
| `array.push()` | ✅ | ✅ | Append element to end |
| `array.remove()` | ✅ | ✅ | Remove element at index |
| `array.reverse()` | ✅ | ✅ | Reverse order |
| `array.shift()` | ✅ | ❌ | Remove first element |
| `array.unshift()` | ✅ | ❌ | Prepend element to beginning |

### Size & Shape

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | ------------------ |
| `array.concat()` | ✅ | ✅ | Concatenate arrays |
| `array.size()` | ✅ | ✅ | Get array size |
| `array.slice()` | ✅ | ❌ | Extract subarray |

### Search & Query

| Function | Status | pine-rs | Description |
| --------------------------------- | ------ | --- | ------------------------- |
| `array.binary_search()` | ✅ | ❌ | Binary search |
| `array.binary_search_leftmost()` | ✅ | ❌ | Binary search (leftmost) |
| `array.binary_search_rightmost()` | ✅ | ❌ | Binary search (rightmost) |
| `array.includes()` | ✅ | ❌ | Check if value exists |
| `array.indexof()` | ✅ | ❌ | Find first index of value |
| `array.lastindexof()` | ✅ | ❌ | Find last index of value |

### Statistical

| Function | Status | pine-rs | Description |
| -------------------- | ------ | --- | ------------------- |
| `array.avg()` | ✅ | ✅ | Average of elements |
| `array.covariance()` | ✅ | ❌ | Covariance |
| `array.max()` | ✅ | ✅ | Maximum value |
| `array.median()` | ✅ | ❌ | Median value |
| `array.min()` | ✅ | ✅ | Minimum value |
| `array.mode()` | ✅ | ❌ | Mode value |
| `array.range()` | ✅ | ❌ | Range of values |
| `array.stdev()` | ✅ | ❌ | Standard deviation |
| `array.sum()` | ✅ | ✅ | Sum of elements |
| `array.variance()` | ✅ | ❌ | Variance |

### Percentiles

| Function | Status | pine-rs | Description |
| ----------------------------------------- | ------ | --- | ------------------------- |
| `array.percentile_linear_interpolation()` | ✅ | ❌ | Percentile (Linear) |
| `array.percentile_nearest_rank()` | ✅ | ❌ | Percentile (Nearest Rank) |
| `array.percentrank()` | ✅ | ❌ | Percentile rank |

### Transformation

| Function | Status | pine-rs | Description |
| ---------------------- | ------ | --- | -------------------- |
| `array.abs()` | ✅ | ❌ | Absolute values |
| `array.join()` | ✅ | ❌ | Join to string |
| `array.sort()` | ✅ | ✅ | Sort array |
| `array.sort_indices()` | ✅ | ❌ | Get sorted indices |
| `array.standardize()` | ✅ | ❌ | Standardize elements |

### Logical

| Function | Status | pine-rs | Description |
| --------------- | ------ | --- | ---------------------------- |
| `array.every()` | ✅ | ❌ | Check if all elements match |
| `array.some()` | ✅ | ❌ | Check if any element matches |
