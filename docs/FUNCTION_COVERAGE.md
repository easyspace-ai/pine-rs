# pine-stdlib Function Coverage

This document tracks the implementation status of Pine Script v6 built-in functions in pine-rs.

> Generated: 2026-04-03
> Total functions: ~100+

## Status Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully implemented with tests |
| 🟨 | Partially implemented or needs verification |
| ⛔ | Not implemented |
| 🔜 | Explicitly delayed (see AGENTS.md) |

---

## array.* (23 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `array.avg` | ✅ | Tested with golden script |
| `array.clear` | ✅ | Unit tested |
| `array.concat` | ✅ | Unit tested |
| `array.copy` | ✅ | Unit tested |
| `array.fill` | ✅ | Unit tested |
| `array.first` | ✅ | Tested with golden script |
| `array.from` | ✅ | Unit tested |
| `array.get` | ✅ | Via method, tested |
| `array.insert` | ✅ | Unit tested |
| `array.last` | ✅ | Tested with golden script |
| `array.max` | ✅ | Tested with golden script |
| `array.min` | ✅ | Tested with golden script |
| `array.new_bool` | ✅ | Unit tested |
| `array.new_color` | ✅ | Unit tested |
| `array.new_float` | ✅ | Tested with golden script |
| `array.new_int` | ✅ | Unit tested |
| `array.new_string` | ✅ | Unit tested |
| `array.pop` | ✅ | Unit tested |
| `array.push` | ✅ | Tested with golden script |
| `array.remove` | ✅ | Unit tested |
| `array.reverse` | ✅ | Unit tested |
| `array.set` | ✅ | Via method, tested |
| `array.size` | ✅ | Via method, tested |
| `array.sort` | ✅ | Unit tested |
| `array.sum` | ✅ | Tested with golden script |

## color.* (13 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `color.a` | ✅ | Unit tested |
| `color.b` | ✅ | Unit tested |
| `color.darken` | ✅ | Unit tested |
| `color.from_hex` | ✅ | Unit tested |
| `color.g` | ✅ | Unit tested |
| `color.lighten` | ✅ | Unit tested |
| `color.mix` | ✅ | Unit tested |
| `color.new` | ✅ | Alias for color.new_transparency |
| `color.new_transparency` | ✅ | Unit tested |
| `color.r` | ✅ | Unit tested |
| `color.rgb` | ✅ | Unit tested |
| `color.rgba` | ✅ | Unit tested |
| `color.transparency` | ✅ | Unit tested |

## input.* (8 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `input.bool` | ✅ | Unit tested |
| `input.color` | ✅ | Returns color value |
| `input.float` | ✅ | Unit tested with min/max |
| `input.int` | ✅ | Unit tested with min/max |
| `input.source` | ✅ | Returns series value |
| `input.string` | ✅ | Unit tested |
| `input.symbol` | ✅ | Returns string |
| `input.timeframe` | ✅ | Returns string |

## map.* (11 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `map.clear` | ✅ | Tested with script |
| `map.contains` | ✅ | Tested with script |
| `map.get` | ✅ | Tested with script |
| `map.is_empty` | ✅ | Tested with script |
| `map.keys` | ✅ | Unit tested |
| `map.new` | ✅ | Tested with script |
| `map.new_from_pair` | ✅ | Unit tested |
| `map.put` | ✅ | Tested with script |
| `map.remove` | ✅ | Tested with script |
| `map.size` | ✅ | Tested with script |
| `map.values` | ✅ | Unit tested |

## math.* (29 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `math.abs` | ✅ | |
| `math.acos` | ✅ | |
| `math.asin` | ✅ | |
| `math.atan` | ✅ | |
| `math.avg` | ✅ | Rolling average of series, unit tested |
| `math.ceil` | ✅ | |
| `math.cos` | ✅ | |
| `math.cosh` | ✅ | |
| `math.copysign` | ✅ | Unit tested |
| `math.exp` | ✅ | |
| `math.floor` | ✅ | |
| `math.isna` | ✅ | |
| `math.log` | ✅ | |
| `math.log10` | ✅ | |
| `math.max` | ✅ | |
| `math.min` | ✅ | |
| `math.nz` | ✅ | Unit tested |
| `math.pow` | ✅ | Unit tested |
| `math.round` | ✅ | |
| `math.round_to_nearest` | ✅ | Unit tested |
| `math.sign` | ✅ | |
| `math.sin` | ✅ | |
| `math.sinh` | ✅ | |
| `math.sqrt` | ✅ | |
| `math.sum` | ✅ | Rolling sum of series, unit tested |
| `math.tan` | ✅ | |
| `math.tanh` | ✅ | |
| `math.tostring` | ✅ | |
| `math.trunc` | ✅ | |

## plot.* (1 function)

| Function | Status | Notes |
|----------|--------|-------|
| `plot` | ✅ | Core plotting function |

## strategy.* (6 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `strategy` | ✅ | Strategy declaration with title, overlay, capital |
| `strategy.close` | ✅ | Close position by id |
| `strategy.entry` | ✅ | Entry order with id, direction, qty |
| `strategy.exit` | ✅ | Exit order with id, from_entry |
| `strategy.long` | ✅ | Constant for long direction |
| `strategy.short` | ✅ | Constant for short direction |

## str.* (16 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `str.concat` | ✅ | Unit tested |
| `str.contains` | ✅ | Unit tested |
| `str.ends_with` | ✅ | Unit tested |
| `str.join` | ✅ | Unit tested |
| `str.length` | ✅ | Unit tested |
| `str.lower` | ✅ | Unit tested |
| `str.replace` | ✅ | Unit tested |
| `str.split` | ✅ | Unit tested |
| `str.starts_with` | ✅ | Unit tested |
| `str.substring` | ✅ | Unit tested |
| `str.tonumber` | ✅ | Unit tested |
| `str.tostring` | ✅ | Unit tested |
| `str.trim` | ✅ | Unit tested |
| `str.trim_end` | ✅ | Unit tested |
| `str.trim_start` | ✅ | Unit tested |
| `str.upper` | ✅ | Unit tested |

## ta.* (19 functions)

| Function | Status | Notes |
|----------|--------|-------|
| `ta.atr` | ✅ | Golden test: atr_14 |
| `ta.barssince` | ✅ | |
| `ta.bb` | ✅ | Golden test: bbands_20_2 |
| `ta.cci` | ✅ | Golden test: cci_20 |
| `ta.crossover` | ✅ | |
| `ta.crossunder` | ✅ | |
| `ta.ema` | ✅ | Golden test: ema_12 |
| `ta.highest` | ✅ | Golden test: highest_10 |
| `ta.highestbars` | ✅ | Golden test: highestbars_10 |
| `ta.lowest` | ✅ | Golden test: lowest_10 |
| `ta.lowestbars` | ✅ | Golden test: lowestbars_10 |
| `ta.macd` | ✅ | Golden test: macd_12_26_9 |
| `ta.mom` | ✅ | Golden test: mom_10 |
| `ta.rma` | ✅ | Unit tested, uses Wilder smoothing (alpha=1/N) |
| `ta.rsi` | ✅ | Golden test: rsi_14 |
| `ta.sma` | ✅ | Golden test: sma_14, sma_manual |
| `ta.stoch` | ✅ | Golden test: stoch_14_3_3 |
| `ta.tr` | ✅ | Golden test: tr_basic |
| `ta.wma` | ✅ | Unit tested |

---

## Summary by Namespace

| Namespace | Functions | ✅ Complete | 🟨 Partial | ⛔ Missing |
|-----------|-----------|-------------|------------|------------|
| array | 23 | 23 | 0 | 0 |
| color | 13 | 13 | 0 | 0 |
| input | 8 | 8 | 0 | 0 |
| map | 11 | 11 | 0 | 0 |
| math | 29 | 29 | 0 | 0 |
| plot | 1 | 1 | 0 | 0 |
| strategy | 6 | 6 | 0 | 0 |
| str | 16 | 16 | 0 | 0 |
| ta | 19 | 19 | 0 | 0 |
| **Total** | **~126** | **~126** | **0** | **0** |

---

## Next Steps (Phase 5)

Per AGENTS.md Phase 5 goals:

1. **P0: array.*** - Core data operations, high priority
2. **P1: color.*, input.*** - Common functionality
3. **P2: map.*, str.*** - Already have large subsets
4. **Ongoing: math.*, ta.*** - Maintain and expand golden test coverage

Each new function should include:
- Implementation in `crates/pine-stdlib/src/`
- Unit tests in the same file
- Golden test script in `tests/scripts/stdlib/`
- Entry in this coverage table
