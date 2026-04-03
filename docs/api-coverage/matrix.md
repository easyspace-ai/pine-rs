# Matrix

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Modification

| Function                | Status |
| ----------------------- | ------ |
| `matrix.add_col()`      | ✅     |
| `matrix.add_row()`      | ✅     |
| `matrix.fill()`         | ✅     |
| `matrix.remove_col()`   | ✅     |
| `matrix.remove_row()`   | ✅     |
| `matrix.reverse()`      | ✅     |
| `matrix.swap_columns()` | ✅     |
| `matrix.swap_rows()`    | ✅     |

### Statistical

| Function          | Status |
| ----------------- | ------ |
| `matrix.avg()`    | ✅     |
| `matrix.max()`    | ✅     |
| `matrix.median()` | ✅     |
| `matrix.min()`    | ✅     |
| `matrix.mode()`   | ✅     |
| `matrix.sum()`    | ✅     |

### Element Access

| Function       | Status |
| -------------- | ------ |
| `matrix.col()` | ✅     |
| `matrix.get()` | ✅     |
| `matrix.row()` | ✅     |
| `matrix.set()` | ✅     |

### Size & Shape

| Function                  | Status |
| ------------------------- | ------ |
| `matrix.columns()`        | ✅     |
| `matrix.elements_count()` | ✅     |
| `matrix.reshape()`        | ✅     |
| `matrix.rows()`           | ✅     |
| `matrix.submatrix()`      | ✅     |

### Operations

| Function          | Status |
| ----------------- | ------ |
| `matrix.concat()` | ✅     |
| `matrix.diff()`   | ✅     |
| `matrix.kron()`   | ✅     |
| `matrix.mult()`   | ✅     |
| `matrix.pow()`    | ✅     |

### Creation & Initialization

| Function             | Status |
| -------------------- | ------ |
| `matrix.copy()`      | ✅     |
| `matrix.new<type>()` | ✅     |

### Linear Algebra

| Function                | Status |
| ----------------------- | ------ |
| `matrix.det()`          | ✅     |
| `matrix.eigenvalues()`  | ✅     |
| `matrix.eigenvectors()` | ✅     |
| `matrix.inv()`          | ✅     |
| `matrix.pinv()`         | ✅     |
| `matrix.rank()`         | ✅     |
| `matrix.trace()`        | ✅     |
| `matrix.transpose()`    | ✅     |

### Properties

| Function                    | Status |
| --------------------------- | ------ |
| `matrix.is_antidiagonal()`  | ✅     |
| `matrix.is_antisymmetric()` | ✅     |
| `matrix.is_binary()`        | ✅     |
| `matrix.is_diagonal()`      | ✅     |
| `matrix.is_identity()`      | ✅     |
| `matrix.is_square()`        | ✅     |
| `matrix.is_stochastic()`    | ✅     |
| `matrix.is_symmetric()`     | ✅     |
| `matrix.is_triangular()`    | ✅     |
| `matrix.is_zero()`          | ✅     |

### Sorting

| Function        | Status |
| --------------- | ------ |
| `matrix.sort()` | ✅     |
