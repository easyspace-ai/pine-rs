# WebSocket Schema

## 当前版本

- `schema_version`: `1`

## 外层约定

所有通过 `pine-tv /api/ws` 推送的消息，都使用统一外壳：

```json
{
  "schema_version": 1,
  "channel": "market | script | control",
  "seq": 1,
  "...": "具体消息体"
}
```

字段说明：

- `schema_version`: 当前消息 schema 版本
- `channel`: 消息通道
  - `market`: 行情事件
  - `script`: 脚本执行结果
  - `control`: 会话控制与错误
- `seq`: 当前连接内单调递增序号，用于排序、去重、乱序保护

## 行情消息

### `snapshot`

```json
{
  "type": "snapshot",
  "bars": []
}
```

### `bar_opened`

```json
{
  "type": "bar_opened",
  "bar": {}
}
```

含义：新 bar 的首个 forming 更新。

### `forming_update`

```json
{
  "type": "forming_update",
  "bar": {}
}
```

含义：当前 forming bar 的盘中更新。

### `bar_closed`

```json
{
  "type": "bar_closed",
  "bar": {}
}
```

含义：当前 bar 最终收盘值。

### `new_bar`

```json
{
  "type": "new_bar",
  "closed_bar": {},
  "new_bar": {}
}
```

含义：语义事件，表示上一根已关闭且新 bar 已开始。

## 脚本结果消息

### `result`

```json
{
  "type": "result",
  "session_id": "session_xxx",
  "is_full": true,
  "update_kind": "snapshot | bar_open | bar_update | bar_close | order_fill",
  "script_kind": "indicator | strategy | unknown",
  "trigger": "snapshot | tick | bar_close | order_fill",
  "bar_time": 1710000000,
  "timestamp": 1710000000123,
  "result": {}
}
```

字段说明：

- `session_id`: 脚本会话 ID
- `is_full`: 是否为全量快照
- `update_kind`: 前端消费层使用的更新类型
- `script_kind`: 脚本声明类型
- `trigger`: 后端执行原因
- `bar_time`: 当前次执行对应的 bar 时间；无 bar 上下文时可为空
- `timestamp`: 服务端发送时间
- `result`: 原 `ApiResponse`

补充约定：

- `update_kind=bar_close` 表示常规收盘执行
- `update_kind=order_fill` 表示同一 bar 上因为成交事件触发的追加执行
- 当 strategy 声明 `calc_on_order_fills=true` 时，成交后允许出现 `order_fill`
- 当 strategy 声明 `process_orders_on_close=true` 时，`bar_close` 执行如果产生本 bar 成交，也允许紧跟一次 `order_fill`
- 前端应按 `session_id + seq` 处理顺序，不应假设一次 `bar_close` 之后不会再收到同 bar 的追加结果

## 控制消息

### `session`

```json
{
  "type": "session",
  "session_id": "session_xxx",
  "status": "active | stopped"
}
```

字段说明：

- `session_id`: 脚本会话 ID
- `status`: 当前会话状态
- 脚本声明信息由后续 `result` 消息中的 `script_kind` 等字段给出

### `error`

```json
{
  "type": "error",
  "errors": []
}
```

## 兼容原则

- 保留现有 `type` 字段，不改前端已有分支逻辑
- 新增字段优先追加，不在 `schema_version = 1` 内改名或删除已有字段
- 如果后续必须改顶层结构，才提升 `schema_version`
- 前端消费层应优先依赖 `channel + type + session_id + seq`，不要把临时 UI 状态当协议字段
