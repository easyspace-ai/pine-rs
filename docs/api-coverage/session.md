# Session

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Session Flags

| Function | Status | pine-rs | Description |
| ---------------------------- | ------ | --- | ---------------------------- |
| `session.isfirstbar` |  | ❌ | First bar of session |
| `session.isfirstbar_regular` |  | ❌ | First bar of regular session |
| `session.islastbar` |  | ❌ | Last bar of session |
| `session.islastbar_regular` |  | ❌ | Last bar of regular session |
| `session.ismarket` |  | ❌ | Market session |
| `session.ispostmarket` |  | ❌ | Post-market session |
| `session.ispremarket` |  | ❌ | Pre-market session |

### Session Constants

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | ------------------------- |
| `session.extended` |  | ❌ | Extended session constant |
| `session.regular` |  | ❌ | Regular session constant |
