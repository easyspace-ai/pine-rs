# Strategy

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Account Info

| Function | Status | pine-rs | Description |
| ----------------------------------- | ------ | --- | ------------------------ |
| `strategy.account_currency` |  | ❌ | Account currency |
| `strategy.equity` |  | ❌ | Account equity |
| `strategy.grossloss` |  | ❌ | Gross loss |
| `strategy.grossloss_percent` |  | ❌ | Gross loss percentage |
| `strategy.grossprofit` |  | ❌ | Gross profit |
| `strategy.grossprofit_percent` |  | ❌ | Gross profit percentage |
| `strategy.initial_capital` |  | ❌ | Initial capital |
| `strategy.margin_liquidation_price` |  | ❌ | Margin liquidation price |
| `strategy.netprofit` |  | ❌ | Net profit |
| `strategy.netprofit_percent` |  | ❌ | Net profit percentage |

### Trade Statistics

| Function | Status | pine-rs | Description |
| ------------------------------------ | ------ | --- | ----------------------------- |
| `strategy.avg_losing_trade` |  | ❌ | Average losing trade |
| `strategy.avg_losing_trade_percent` |  | ❌ | Average losing trade percent |
| `strategy.avg_trade` |  | ❌ | Average trade |
| `strategy.avg_trade_percent` |  | ❌ | Average trade percent |
| `strategy.avg_winning_trade` |  | ❌ | Average winning trade |
| `strategy.avg_winning_trade_percent` |  | ❌ | Average winning trade percent |
| `strategy.closedtrades` |  | ❌ | Number of closed trades |
| `strategy.eventrades` |  | ❌ | Number of even trades |
| `strategy.losstrades` |  | ❌ | Number of losing trades |
| `strategy.opentrades` |  | ❌ | Number of open trades |
| `strategy.wintrades` |  | ❌ | Number of winning trades |

### Closed Trades

| Function | Status | pine-rs | Description |
| ---------------------------------------------- | ------ | --- | ------------------------ |
| `strategy.closedtrades.first_index` |  | ❌ | First closed trade index |
| `strategy.closedtrades.commission()` |  | ❌ | Get commission |
| `strategy.closedtrades.entry_bar_index()` |  | ❌ | Get entry bar index |
| `strategy.closedtrades.entry_comment()` |  | ❌ | Get entry comment |
| `strategy.closedtrades.entry_id()` |  | ❌ | Get entry ID |
| `strategy.closedtrades.entry_price()` |  | ❌ | Get entry price |
| `strategy.closedtrades.entry_time()` |  | ❌ | Get entry time |
| `strategy.closedtrades.exit_bar_index()` |  | ❌ | Get exit bar index |
| `strategy.closedtrades.exit_comment()` |  | ❌ | Get exit comment |
| `strategy.closedtrades.exit_id()` |  | ❌ | Get exit ID |
| `strategy.closedtrades.exit_price()` |  | ❌ | Get exit price |
| `strategy.closedtrades.exit_time()` |  | ❌ | Get exit time |
| `strategy.closedtrades.max_drawdown()` |  | ❌ | Get max drawdown |
| `strategy.closedtrades.max_drawdown_percent()` |  | ❌ | Get max drawdown percent |
| `strategy.closedtrades.max_runup()` |  | ❌ | Get max runup |
| `strategy.closedtrades.max_runup_percent()` |  | ❌ | Get max runup percent |
| `strategy.closedtrades.profit()` |  | ❌ | Get profit |
| `strategy.closedtrades.profit_percent()` |  | ❌ | Get profit percent |
| `strategy.closedtrades.size()` |  | ❌ | Get size |

### Drawdown & Runup

| Function | Status | pine-rs | Description |
| ----------------------------------- | ------ | --- | -------------------------- |
| `strategy.max_contracts_held_all` |  | ❌ | Max contracts held (all) |
| `strategy.max_contracts_held_long` |  | ❌ | Max contracts held (long) |
| `strategy.max_contracts_held_short` |  | ❌ | Max contracts held (short) |
| `strategy.max_drawdown` |  | ❌ | Maximum drawdown |
| `strategy.max_drawdown_percent` |  | ❌ | Maximum drawdown percent |
| `strategy.max_runup` |  | ❌ | Maximum runup |
| `strategy.max_runup_percent` |  | ❌ | Maximum runup percent |

### Position Info

| Function | Status | pine-rs | Description |
| ------------------------------ | ------ | --- | ---------------------- |
| `strategy.openprofit` |  | ❌ | Open profit |
| `strategy.openprofit_percent` |  | ❌ | Open profit percent |
| `strategy.position_avg_price` |  | ❌ | Position average price |
| `strategy.position_entry_name` |  | ❌ | Position entry name |
| `strategy.position_size` |  | ❌ | Position size |

### Open Trades

| Function | Status | pine-rs | Description |
| -------------------------------------------- | ------ | --- | ------------------------ |
| `strategy.opentrades.capital_held` |  | ❌ | Capital held |
| `strategy.opentrades.commission()` |  | ❌ | Get commission |
| `strategy.opentrades.entry_bar_index()` |  | ❌ | Get entry bar index |
| `strategy.opentrades.entry_comment()` |  | ❌ | Get entry comment |
| `strategy.opentrades.entry_id()` |  | ❌ | Get entry ID |
| `strategy.opentrades.entry_price()` |  | ❌ | Get entry price |
| `strategy.opentrades.entry_time()` |  | ❌ | Get entry time |
| `strategy.opentrades.max_drawdown()` |  | ❌ | Get max drawdown |
| `strategy.opentrades.max_drawdown_percent()` |  | ❌ | Get max drawdown percent |
| `strategy.opentrades.max_runup()` |  | ❌ | Get max runup |
| `strategy.opentrades.max_runup_percent()` |  | ❌ | Get max runup percent |
| `strategy.opentrades.profit()` |  | ❌ | Get profit |
| `strategy.opentrades.profit_percent()` |  | ❌ | Get profit percent |
| `strategy.opentrades.size()` |  | ❌ | Get size |

### Constants

| Function | Status | pine-rs | Description |
| ---------------------------- | ------ | --- | -------------------------- |
| `strategy.cash` |  | ❌ | Cash constant |
| `strategy.fixed` |  | ❌ | Fixed constant |
| `strategy.long` |  | ✅ | Long constant |
| `strategy.percent_of_equity` |  | ❌ | Percent of equity constant |
| `strategy.short` |  | ✅ | Short constant |

### Commission

| Function | Status | pine-rs | Description |
| --------------------------------------- | ------ | --- | ------------------ |
| `strategy.commission.cash_per_contract` |  | ❌ | Cash per contract |
| `strategy.commission.cash_per_order` |  | ❌ | Cash per order |
| `strategy.commission.percent` |  | ❌ | Commission percent |

### Direction

| Function | Status | pine-rs | Description |
| -------------------------- | ------ | --- | --------------- |
| `strategy.direction.all` |  | ❌ | All directions |
| `strategy.direction.long` |  | ❌ | Long direction |
| `strategy.direction.short` |  | ❌ | Short direction |

### OCA

| Function | Status | pine-rs | Description |
| --------------------- | ------ | --- | ----------- |
| `strategy.oca.cancel` |  | ❌ | OCA cancel |
| `strategy.oca.none` |  | ❌ | OCA none |
| `strategy.oca.reduce` |  | ❌ | OCA reduce |

### Order Management

| Function | Status | pine-rs | Description |
| ----------------------- | ------ | --- | ----------------- |
| `strategy.cancel()` |  | ❌ | Cancel order |
| `strategy.cancel_all()` |  | ❌ | Cancel all orders |

### Position Management

| Function | Status | pine-rs | Description |
| ---------------------- | ------ | --- | ------------------- |
| `strategy.close()` |  | ✅ | Close position |
| `strategy.close_all()` |  | ❌ | Close all positions |
| `strategy.entry()` |  | ✅ | Enter position |
| `strategy.exit()` |  | ✅ | Exit position |
| `strategy.order()` |  | ❌ | Place order |

### Conversion

| Function | Status | pine-rs | Description |
| ------------------------------- | ------ | --- | ---------------------- |
| `strategy.convert_to_account()` |  | ❌ | Convert to account |
| `strategy.convert_to_symbol()` |  | ❌ | Convert to symbol |
| `strategy.default_entry_qty()` |  | ❌ | Default entry quantity |

### Risk Management

| Function | Status | pine-rs | Description |
| -------------------------------------------- | ------ | --- | -------------------------- |
| `strategy.risk.allow_entry_in()` |  | ❌ | Allow entry in |
| `strategy.risk.max_cons_loss_days()` |  | ❌ | Max consecutive loss days |
| `strategy.risk.max_drawdown()` |  | ❌ | Max drawdown |
| `strategy.risk.max_intraday_filled_orders()` |  | ❌ | Max intraday filled orders |
| `strategy.risk.max_intraday_loss()` |  | ❌ | Max intraday loss |
| `strategy.risk.max_position_size()` |  | ❌ | Max position size |
