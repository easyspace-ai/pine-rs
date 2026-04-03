# Math Functions

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Constants

| Function | Status | pine-rs | Description |
| ----------- | ------ | --- | ------------------------------------- |
| `math.e` | ✅ | ❌ | Euler's number (≈2.71828) |
| `math.phi` | ✅ | ❌ | Golden ratio (≈1.61803) |
| `math.pi` | ✅ | ❌ | Pi constant (≈3.14159) |
| `math.rphi` | ✅ | ❌ | Reciprocal of golden ratio (≈0.61803) |

### Basic Operations

| Function | Status | pine-rs | Description |
| ------------------------- | ------ | --- | ------------------------------ |
| `math.abs()` | ✅ | ✅ | Absolute value |
| `math.ceil()` | ✅ | ✅ | Round up to nearest integer |
| `math.floor()` | ✅ | ✅ | Round down to nearest integer |
| `math.round()` | ✅ | ✅ | Round to nearest integer |
| `math.round_to_mintick()` | ✅ | ❌ | Round to nearest mintick |
| `math.sign()` | ✅ | ✅ | Sign of a number (-1, 0, or 1) |

### Trigonometric

| Function | Status | pine-rs | Description |
| ------------- | ------ | --- | ----------------------------- |
| `math.acos()` | ✔️ | ✅ | Arc cosine (inverse cosine) |
| `math.asin()` | ✔️ | ✅ | Arc sine (inverse sine) |
| `math.atan()` | ✔️ | ✅ | Arc tangent (inverse tangent) |
| `math.cos()` | ✔️ | ✅ | Cosine |
| `math.sin()` | ✔️ | ✅ | Sine |
| `math.tan()` | ✔️ | ✅ | Tangent |

### Statistical

| Function | Status | pine-rs | Description |
| ------------ | ------ | --- | ------------------------------ |
| `math.avg()` | ✔️ | ✅ | Average of all arguments |
| `math.max()` | ✅ | ✅ | Maximum value of all arguments |
| `math.min()` | ✅ | ✅ | Minimum value of all arguments |
| `math.sum()` | ✔️ | ✅ | Sum of all arguments |

### Exponential & Logarithmic

| Function | Status | pine-rs | Description |
| -------------- | ------ | --- | ------------------------------ |
| `math.exp()` | ✔️ | ✅ | Exponential function (e^x) |
| `math.log()` | ✔️ | ✅ | Natural logarithm (base e) |
| `math.log10()` | ✅ | ✅ | Base-10 logarithm |
| `math.pow()` | ✔️ | ✅ | Power function (base^exponent) |
| `math.sqrt()` | ✔️ | ✅ | Square root |

### Utilities

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | -------------------------- |
| `math.random()` | ✔️ | ❌ | Random number generator |
| `math.todegrees()` |  | ❌ | Convert radians to degrees |
| `math.toradians()` |  | ❌ | Convert degrees to radians |
