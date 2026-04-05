# pine-stdlib Function Coverage

This document tracks the current built-in surface of pine-rs in a way that matches the codebase as it exists today.

> Updated: 2026-04-05
> Scope: functions registered in `crates/pine-stdlib` plus their current verification depth

## How to read this file

- This is not the official Pine Script total-function matrix.
- This file answers a narrower question: what pine-rs currently registers, and how well it is verified.
- For official-coverage comparison by namespace, use [API_COVERAGE.md](./API_COVERAGE.md) and `docs/api-coverage/*.md`.
- For syntax and runtime alignment status, use [V6_ALIGNMENT.md](./V6_ALIGNMENT.md).

## Verification levels

| Level | Meaning |
|-------|---------|
| `golden` | Covered by golden script or equivalent end-to-end output check |
| `script` | Covered by runnable script or integration test |
| `unit` | Covered by unit test only |
| `mixed` | Namespace contains a mix of golden, script, and unit coverage |

## Registry snapshot

| Namespace | Registered in pine-rs | Verification | Notes |
|-----------|------------------------|--------------|-------|
| `array.*` | 42 | mixed | Large implemented subset; object arrays such as `array.new_box` remain out |
| `color.*` | 13 | unit | Stable helper namespace |
| `input.*` | 8 | unit | Common subset present |
| `map.*` | 11 | mixed | Core CRUD present |
| `math.*` | 32 | mixed | Broad subset; most covered by unit tests |
| `plot*` | 7 | mixed | `plot` is end-to-end verified; shape and arrow outputs are lighter-weight than full TV behavior |
| `str.*` | 22 | mixed | Larger than earlier docs claimed; includes alias forms such as `startswith` and `endswith` |
| `strategy.*` | 6 | script | Signal-level subset only, not full TV strategy model |
| `ta.*` | 46 | mixed | Core indicator subset with broad golden coverage |
| **Total** | **187** | **mixed** | Registry count only, not official Pine total |

## Namespace notes

### `ta.*`

- Strongest end-to-end coverage today.
- Golden coverage already includes representative core functions such as:
  - `sma`, `ema`, `rsi`, `atr`, `bb`, `macd`, `stoch`
  - `highest`, `lowest`, `highestbars`, `lowestbars`
  - `linreg`, `mfi`, `supertrend`, `wpr`, `tsi`
- This is still a subset of the full official TradingView `ta.*` matrix.

### `array.*`

- Actual registered surface is much larger than older summaries suggested.
- Implemented areas already include:
  - creation and core CRUD
  - search helpers
  - statistical helpers such as `median`, `stdev`, `variance`, `range`, `percentrank`
- Missing area is mainly object-oriented array families and deeper TV parity validation.

### `str.*`

- Current registry includes both canonical and compatibility-style names:
  - `starts_with` and `ends_with`
  - `startswith` and `endswith`
  - `replace_all`, `format`, `match`, `pos`
- This namespace is no longer "unknown" or "very small".

### `plot*`

- Registered functions are:
  - `plot`
  - `hline`
  - `bgcolor`
  - `fill`
  - `plotshape`
  - `plotchar`
  - `plotarrow`
- Registration does not mean full TradingView-equivalent drawing behavior is complete.

### `strategy.*`

- Current subset is:
  - `strategy`
  - `strategy.entry`
  - `strategy.close`
  - `strategy.exit`
  - `strategy.long`
  - `strategy.short`
- This remains a signal-level subset, not a claim of full strategy parity.

## What this file fixes

This file replaces the earlier misleading summary that implied:

- listed functions == full namespace coverage
- `~132 complete / 0 missing`

That framing was inaccurate because several namespaces had already grown beyond the rows listed there.

## Next work

1. Add a generated export so this registry snapshot can be refreshed automatically.
2. Split verification further into:
   - registered only
   - unit-covered
   - script-covered
   - golden-covered
3. Keep this file aligned with:
   - [API_COVERAGE.md](./API_COVERAGE.md)
   - [V6_ALIGNMENT.md](./V6_ALIGNMENT.md)
   - `docs/api-coverage/*.md`
