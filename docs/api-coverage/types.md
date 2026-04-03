# Types

> **pine-rs 图例**：✅ 已在 `pine-stdlib` 注册或由 runner 注入 / eval 特殊处理，并有测试或黄金或可运行验证  
> · **✔️** 部分实现、仅底层模块、或语义与 TV 仍有差距（脚注见单元格）  
> · **❌** 尚未实现  
> · **⏳** 按 [`AGENTS.md`](../../AGENTS.md) 刻意延后（如 `request.*` 占位）  
>
> **Status** 列保留自 **PineTS**  upstream，不代表 pine-rs。  
>



### Currency

| Function | Status | pine-rs | Description |
| --------------- | ------ | --- | ------------------ |
| `currency.AED` | ✅ | ❌ | UAE Dirham |
| `currency.ARS` | ✅ | ❌ | Argentine Peso |
| `currency.AUD` | ✅ | ❌ | Australian Dollar |
| `currency.BDT` | ✅ | ❌ | Bangladeshi Taka |
| `currency.BHD` | ✅ | ❌ | Bahraini Dinar |
| `currency.BRL` | ✅ | ❌ | Brazilian Real |
| `currency.BTC` | ✅ | ❌ | Bitcoin |
| `currency.CAD` | ✅ | ❌ | Canadian Dollar |
| `currency.CHF` | ✅ | ❌ | Swiss Franc |
| `currency.CLP` | ✅ | ❌ | Chilean Peso |
| `currency.CNY` | ✅ | ❌ | Chinese Yuan |
| `currency.COP` | ✅ | ❌ | Colombian Peso |
| `currency.CZK` | ✅ | ❌ | Czech Koruna |
| `currency.DKK` | ✅ | ❌ | Danish Krone |
| `currency.EGP` | ✅ | ❌ | Egyptian Pound |
| `currency.ETH` | ✅ | ❌ | Ethereum |
| `currency.EUR` | ✅ | ❌ | Euro |
| `currency.GBP` | ✅ | ❌ | British Pound |
| `currency.HKD` | ✅ | ❌ | Hong Kong Dollar |
| `currency.HUF` | ✅ | ❌ | Hungarian Forint |
| `currency.IDR` | ✅ | ❌ | Indonesian Rupiah |
| `currency.ILS` | ✅ | ❌ | Israeli Shekel |
| `currency.INR` | ✅ | ❌ | Indian Rupee |
| `currency.ISK` | ✅ | ❌ | Icelandic Króna |
| `currency.JPY` | ✅ | ❌ | Japanese Yen |
| `currency.KES` | ✅ | ❌ | Kenyan Shilling |
| `currency.KRW` | ✅ | ❌ | South Korean Won |
| `currency.KWD` | ✅ | ❌ | Kuwaiti Dinar |
| `currency.LKR` | ✅ | ❌ | Sri Lankan Rupee |
| `currency.MAD` | ✅ | ❌ | Moroccan Dirham |
| `currency.MXN` | ✅ | ❌ | Mexican Peso |
| `currency.MYR` | ✅ | ❌ | Malaysian Ringgit |
| `currency.NGN` | ✅ | ❌ | Nigerian Naira |
| `currency.NOK` | ✅ | ❌ | Norwegian Krone |
| `currency.NONE` | ✅ | ❌ | No currency |
| `currency.NZD` | ✅ | ❌ | New Zealand Dollar |
| `currency.PEN` | ✅ | ❌ | Peruvian Sol |
| `currency.PHP` | ✅ | ❌ | Philippine Peso |
| `currency.PKR` | ✅ | ❌ | Pakistani Rupee |
| `currency.PLN` | ✅ | ❌ | Polish Złoty |
| `currency.QAR` | ✅ | ❌ | Qatari Riyal |
| `currency.RON` | ✅ | ❌ | Romanian Leu |
| `currency.RSD` | ✅ | ❌ | Serbian Dinar |
| `currency.RUB` | ✅ | ❌ | Russian Ruble |
| `currency.SAR` | ✅ | ❌ | Saudi Riyal |
| `currency.SEK` | ✅ | ❌ | Swedish Krona |
| `currency.SGD` | ✅ | ❌ | Singapore Dollar |
| `currency.THB` | ✅ | ❌ | Thai Baht |
| `currency.TND` | ✅ | ❌ | Tunisian Dinar |
| `currency.TRY` | ✅ | ❌ | Turkish Lira |
| `currency.TWD` | ✅ | ❌ | New Taiwan Dollar |
| `currency.USD` | ✅ | ❌ | US Dollar |
| `currency.USDT` | ✅ | ❌ | Tether |
| `currency.VES` | ✅ | ❌ | Venezuelan Bolívar |
| `currency.VND` | ✅ | ❌ | Vietnamese Đồng |
| `currency.ZAR` | ✅ | ❌ | South African Rand |

### Dayofweek

| Function | Status | pine-rs | Description |
| --------------------- | ------ | --- | ----------- |
| `dayofweek.friday` | ✅ | ❌ | Friday |
| `dayofweek.monday` | ✅ | ❌ | Monday |
| `dayofweek.saturday` | ✅ | ❌ | Saturday |
| `dayofweek.sunday` | ✅ | ❌ | Sunday |
| `dayofweek.thursday` | ✅ | ❌ | Thursday |
| `dayofweek.tuesday` | ✅ | ❌ | Tuesday |
| `dayofweek.wednesday` | ✅ | ❌ | Wednesday |

### Display

| Function | Status | pine-rs | Description |
| ----------------------- | ------ | --- | ------------------------ |
| `display.all` | ✅ | ❌ | Display all |
| `display.data_window` | ✅ | ❌ | Display in data window |
| `display.none` | ✅ | ❌ | Display none |
| `display.pane` | ✅ | ❌ | Display in pane |
| `display.pine_screener` | ✅ | ❌ | Display in Pine Screener |
| `display.price_scale` | ✅ | ❌ | Display in price scale |
| `display.status_line` | ✅ | ❌ | Display in status line |

### Extend

| Function | Status | pine-rs | Description |
| -------------- | ------ | --- | ------------ |
| `extend.both` | ✅ | ❌ | Extend both |
| `extend.left` | ✅ | ❌ | Extend left |
| `extend.none` | ✅ | ❌ | Extend none |
| `extend.right` | ✅ | ❌ | Extend right |

### Font

| Function | Status | pine-rs | Description |
| ----------------------- | ------ | --- | --------------------- |
| `font.family_default` | ✅ | ❌ | Default font family |
| `font.family_monospace` | ✅ | ❌ | Monospace font family |

### Format

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | -------------- |
| `format.inherit` | ✅ | ❌ | Inherit format |
| `format.mintick` | ✅ | ❌ | Mintick format |
| `format.percent` | ✅ | ❌ | Percent format |
| `format.price` | ✅ | ❌ | Price format |
| `format.volume` | ✅ | ❌ | Volume format |

### Hline

| Function | Status | pine-rs | Description |
| -------------------- | ------ | --- | ---------------------------- |
| `hline.style_dashed` | ✅ | ❌ | Dashed horizontal line style |
| `hline.style_dotted` | ✅ | ❌ | Dotted horizontal line style |
| `hline.style_solid` | ✅ | ❌ | Solid horizontal line style |

### Location

| Function | Status | pine-rs | Description |
| ------------------- | ------ | --- | ------------------ |
| `location.abovebar` | ✅ | ❌ | Above bar location |
| `location.absolute` | ✅ | ❌ | Absolute location |
| `location.belowbar` | ✅ | ❌ | Below bar location |
| `location.bottom` | ✅ | ❌ | Bottom location |
| `location.top` | ✅ | ❌ | Top location |

### Order

| Function | Status | pine-rs | Description |
| ------------------ | ------ | --- | ---------------- |
| `order.ascending` | ✅ | ❌ | Ascending order |
| `order.descending` | ✅ | ❌ | Descending order |

### Plot

| Function | Status | pine-rs | Description |
| ----------------------------- | ------ | --- | --------------------------- |
| `plot.linestyle_dashed` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Dashed line style |
| `plot.linestyle_dotted` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Dotted line style |
| `plot.linestyle_solid` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Solid line style |
| `plot.style_area` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Area plot style |
| `plot.style_areabr` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Area break plot style |
| `plot.style_circles` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Circles plot style |
| `plot.style_columns` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Columns plot style |
| `plot.style_cross` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Cross plot style |
| `plot.style_histogram` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Histogram plot style |
| `plot.style_line` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Line plot style |
| `plot.style_linebr` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Line break plot style |
| `plot.style_stepline` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Stepline plot style |
| `plot.style_stepline_diamond` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Stepline diamond plot style |
| `plot.style_steplinebr` | ✅ | ✔️ 见 pine-output；eval 未全线挂钩 | Stepline break plot style |

### Position

| Function | Status | pine-rs | Description |
| ------------------------ | ------ | --- | ---------------------- |
| `position.bottom_center` | ✅ | ❌ | Bottom center position |
| `position.bottom_left` | ✅ | ❌ | Bottom left position |
| `position.bottom_right` | ✅ | ❌ | Bottom right position |
| `position.middle_center` | ✅ | ❌ | Middle center position |
| `position.middle_left` | ✅ | ❌ | Middle left position |
| `position.middle_right` | ✅ | ❌ | Middle right position |
| `position.top_center` | ✅ | ❌ | Top center position |
| `position.top_left` | ✅ | ❌ | Top left position |
| `position.top_right` | ✅ | ❌ | Top right position |

### Scale

| Function | Status | pine-rs | Description |
| ------------- | ------ | --- | ----------- |
| `scale.left` | ✅ | ❌ | Left scale |
| `scale.none` | ✅ | ❌ | No scale |
| `scale.right` | ✅ | ❌ | Right scale |

### Settlement_as_close

| Function | Status | pine-rs | Description |
| ----------------------------- | ------ | --- | --------------------------- |
| `settlement_as_close.inherit` | ✅ | ❌ | Inherit settlement as close |
| `settlement_as_close.off` | ✅ | ❌ | Settlement as close off |
| `settlement_as_close.on` | ✅ | ❌ | Settlement as close on |

### Shape

| Function | Status | pine-rs | Description |
| -------------------- | ------ | --- | ------------------- |
| `shape.arrowdown` | ✅ | ❌ | Arrow down shape |
| `shape.arrowup` | ✅ | ❌ | Arrow up shape |
| `shape.circle` | ✅ | ❌ | Circle shape |
| `shape.cross` | ✅ | ❌ | Cross shape |
| `shape.diamond` | ✅ | ❌ | Diamond shape |
| `shape.flag` | ✅ | ❌ | Flag shape |
| `shape.labeldown` | ✅ | ❌ | Label down shape |
| `shape.labelup` | ✅ | ❌ | Label up shape |
| `shape.square` | ✅ | ❌ | Square shape |
| `shape.triangledown` | ✅ | ❌ | Triangle down shape |
| `shape.triangleup` | ✅ | ❌ | Triangle up shape |
| `shape.xcross` | ✅ | ❌ | X-cross shape |

### Size

| Function | Status | pine-rs | Description |
| ------------- | ------ | --- | ----------- |
| `size.auto` | ✅ | ❌ | Auto size |
| `size.huge` | ✅ | ❌ | Huge size |
| `size.large` | ✅ | ❌ | Large size |
| `size.normal` | ✅ | ❌ | Normal size |
| `size.small` | ✅ | ❌ | Small size |
| `size.tiny` | ✅ | ❌ | Tiny size |

### Splits

| Function | Status | pine-rs | Description |
| -------------------- | ------ | --- | ----------------- |
| `splits.denominator` | ✅ | ❌ | Split denominator |
| `splits.numerator` | ✅ | ❌ | Split numerator |

### Text

| Function | Status | pine-rs | Description |
| -------------------- | ------ | --- | --------------------- |
| `text.align_bottom` | ✅ | ❌ | Bottom text alignment |
| `text.align_center` | ✅ | ❌ | Center text alignment |
| `text.align_left` | ✅ | ❌ | Left text alignment |
| `text.align_right` | ✅ | ❌ | Right text alignment |
| `text.align_top` | ✅ | ❌ | Top text alignment |
| `text.format_bold` | ✅ | ❌ | Bold text format |
| `text.format_italic` | ✅ | ❌ | Italic text format |
| `text.format_none` | ✅ | ❌ | No text format |
| `text.wrap_auto` | ✅ | ❌ | Auto text wrap |
| `text.wrap_none` | ✅ | ❌ | No text wrap |

### Xloc

| Function | Status | pine-rs | Description |
| ---------------- | ------ | --- | -------------------- |
| `xloc.bar_index` | ✅ | ❌ | Bar index x-location |
| `xloc.bar_time` | ✅ | ❌ | Bar time x-location |

### Yloc

| Function | Status | pine-rs | Description |
| --------------- | ------ | --- | -------------------- |
| `yloc.abovebar` | ✅ | ❌ | Above bar y-location |
| `yloc.belowbar` | ✅ | ❌ | Below bar y-location |
| `yloc.price` | ✅ | ❌ | Price y-location |

### Dividends

| Function | Status | pine-rs | Description |
| --------------------------- | ------ | --- | ---------------------- |
| `dividends.future_amount` | ✅ | ❌ | Future dividend amount |
| `dividends.future_ex_date` | ✅ | ❌ | Future ex-date |
| `dividends.future_pay_date` | ✅ | ❌ | Future pay date |
| `dividends.gross` | ✅ | ❌ | Gross dividend |
| `dividends.net` | ✅ | ❌ | Net dividend |

### Earnings

| Function | Status | pine-rs | Description |
| --------------------------------- | ------ | --- | ---------------------- |
| `earnings.future_eps` | ✅ | ❌ | Future EPS |
| `earnings.future_period_end_time` | ✅ | ❌ | Future period end time |
| `earnings.future_revenue` | ✅ | ❌ | Future revenue |
| `earnings.future_time` | ✅ | ❌ | Future time |
| `earnings.actual` | ✅ | ❌ | Actual earnings |
| `earnings.estimate` | ✅ | ❌ | Estimated earnings |
| `earnings.standardized` | ✅ | ❌ | Standardized earnings |

### Adjustment

| Function | Status | pine-rs | Description |
| ---------------------- | ------ | --- | -------------------- |
| `adjustment.dividends` | ✅ | ❌ | Dividends adjustment |
| `adjustment.none` | ✅ | ❌ | No adjustment |
| `adjustment.splits` | ✅ | ❌ | Splits adjustment |

### Alert

| Function | Status | pine-rs | Description |
| ------------------------------- | ------ | --- | ---------------------------------- |
| `alert.freq_all` |  | ❌ | Alert frequency all |
| `alert.freq_once_per_bar` |  | ❌ | Alert frequency once per bar |
| `alert.freq_once_per_bar_close` |  | ❌ | Alert frequency once per bar close |

### Backadjustment

| Function | Status | pine-rs | Description |
| ------------------------ | ------ | --- | ---------------------- |
| `backadjustment.inherit` | ✅ | ❌ | Inherit backadjustment |
| `backadjustment.off` | ✅ | ❌ | Backadjustment off |
| `backadjustment.on` | ✅ | ❌ | Backadjustment on |

### Barmerge

| Function | Status | pine-rs | Description |
| ------------------------ | ------ | --- | ------------- |
| `barmerge.gaps_off` | ✅ | ❌ | Gaps off |
| `barmerge.gaps_on` | ✅ | ❌ | Gaps on |
| `barmerge.lookahead_off` | ✅ | ❌ | Lookahead off |
| `barmerge.lookahead_on` | ✅ | ❌ | Lookahead on |
