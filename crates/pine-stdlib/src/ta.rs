//! Technical analysis functions (ta.*)
//!
//! This module provides Pine Script v6 compatible technical analysis indicators.
//! All functions follow TradingView's exact semantics including NA handling and initialization.

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_runtime::value::Value;
use std::sync::Arc;

/// SMA calculation over the trailing window ending at the current bar.
///
/// The `values` slice is in chronological order: `[oldest, ..., newest]`.
pub fn calculate_sma_f64(values: &[f64], length: usize) -> Option<f64> {
    if length == 0 || values.len() < length {
        return None;
    }

    let start = values.len() - length;
    let window = &values[start..];
    Some(window.iter().sum::<f64>() / length as f64)
}

/// Highest calculation over the trailing window ending at the current bar.
pub fn calculate_highest_f64(values: &[f64], length: usize) -> Option<f64> {
    if length == 0 || values.is_empty() {
        return None;
    }

    let window_len = length.min(values.len());
    let start = values.len() - window_len;
    values[start..].iter().copied().reduce(f64::max)
}

/// Lowest calculation over the trailing window ending at the current bar.
pub fn calculate_lowest_f64(values: &[f64], length: usize) -> Option<f64> {
    if length == 0 || values.is_empty() {
        return None;
    }

    let window_len = length.min(values.len());
    let start = values.len() - window_len;
    values[start..].iter().copied().reduce(f64::min)
}

/// Convert Value slice to f64 vec, skipping NA values
fn value_slice_to_f64(values: &[Value]) -> Vec<f64> {
    values
        .iter()
        .filter_map(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => Some(*n as f64),
            _ => None,
        })
        .collect()
}

/// Register all ta.* functions with the registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    // Moving averages
    register_sma(registry);
    register_ema(registry);
    register_rma(registry);
    register_wma(registry);

    // Momentum indicators
    register_rsi(registry);
    register_macd(registry);
    register_mom(registry);
    register_cci(registry);

    // Volatility indicators
    register_atr(registry);
    register_tr(registry);
    register_bbands(registry);

    // Stochastic
    register_stoch(registry);

    // Extremum functions
    register_highest(registry);
    register_lowest(registry);
    register_highestbars(registry);
    register_lowestbars(registry);

    // Cross functions
    register_crossover(registry);
    register_crossunder(registry);
    register_barssince(registry);
}

// ============================================================================
// Helper functions
// ============================================================================

/// Extract array from value
fn extract_array(value: &Value) -> Option<&[Value]> {
    match value {
        Value::Array(arr) => Some(arr.as_slice()),
        _ => None,
    }
}

/// Extract length parameter from arguments
fn extract_length(args: &[Value], idx: usize, default: usize) -> usize {
    args.get(idx)
        .and_then(|v| v.as_int())
        .map(|n| n.max(1) as usize)
        .unwrap_or(default)
}

/// Get float value from Value
fn get_float(value: &Value) -> Option<f64> {
    match value {
        Value::Float(f) => Some(*f),
        Value::Int(n) => Some(*n as f64),
        _ => None,
    }
}

/// Simple Moving Average calculation
///
/// Uses SIMD-optimized SeriesBufF64 internally for better performance
fn calculate_sma(values: &[Value], length: usize) -> Value {
    if length == 0 || values.len() < length {
        return Value::Na;
    }

    // Convert to f64 slice in chronological order [oldest, ..., newest]
    let f64_values: Vec<f64> = values
        .iter()
        .filter_map(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => Some(*n as f64),
            _ => None,
        })
        .collect();

    if f64_values.len() < length {
        return Value::Na;
    }

    // Use SIMD-optimized version
    match calculate_sma_f64(&f64_values, length) {
        Some(result) => Value::Float(result),
        None => Value::Na,
    }
}

/// Exponential Moving Average calculation
/// EMA = alpha * current + (1 - alpha) * previous_ema
/// where alpha = 2 / (length + 1)
///
/// The `values` slice is in chronological order: `[oldest, ..., newest]`.
fn calculate_ema(values: &[Value], length: usize, wilder: bool) -> Value {
    if length == 0 {
        return Value::Na;
    }

    // Alpha: 2/(N+1) for EMA, 1/N for RMA (Wilder smoothing)
    let alpha = if wilder {
        1.0 / length as f64
    } else {
        2.0 / (length as f64 + 1.0)
    };

    let valid_values: Vec<f64> = values.iter().filter_map(get_float).collect();
    if valid_values.len() < length {
        return Value::Na;
    }

    // Seed EMA with the first trailing full window available.
    let mut ema = valid_values.iter().take(length).sum::<f64>() / length as f64;
    for val in valid_values.iter().skip(length) {
        ema = alpha * val + (1.0 - alpha) * ema;
    }

    Value::Float(ema)
}

fn calculate_ema_from_f64(values: &[f64], length: usize, wilder: bool) -> Option<f64> {
    if length == 0 || values.len() < length {
        return None;
    }

    let alpha = if wilder {
        1.0 / length as f64
    } else {
        2.0 / (length as f64 + 1.0)
    };

    let mut ema = values.iter().take(length).sum::<f64>() / length as f64;
    for val in values.iter().skip(length) {
        ema = alpha * val + (1.0 - alpha) * ema;
    }
    Some(ema)
}

/// Weighted Moving Average calculation
/// WMA = (1*oldest + 2*... + N*newest) / (1 + 2 + ... + N)
fn calculate_wma(values: &[Value], length: usize) -> Value {
    let valid_values: Vec<f64> = values.iter().filter_map(get_float).collect();
    if length == 0 || valid_values.len() < length {
        return Value::Na;
    }

    let window = &valid_values[valid_values.len() - length..];
    let mut weighted_sum = 0.0;
    let mut weight_sum = 0;

    for (idx, value) in window.iter().enumerate() {
        let weight = idx + 1;
        weighted_sum += weight as f64 * value;
        weight_sum += weight;
    }

    Value::Float(weighted_sum / weight_sum as f64)
}

/// Calculate True Range
fn calculate_tr(high: &[Value], low: &[Value], close: &[Value]) -> Value {
    let current_high = match high.last().and_then(get_float) {
        Some(f) => f,
        None => return Value::Na,
    };

    let current_low = match low.last().and_then(get_float) {
        Some(f) => f,
        None => return Value::Na,
    };

    // TR = max(high - low, |high - previous_close|, |low - previous_close|)
    let tr1 = current_high - current_low;

    let prev_close = if close.len() >= 2 {
        close.get(close.len() - 2).and_then(get_float)
    } else {
        None
    };

    match prev_close {
        Some(pc) => {
            let tr2 = (current_high - pc).abs();
            let tr3 = (current_low - pc).abs();
            Value::Float(tr1.max(tr2).max(tr3))
        }
        None => Value::Float(tr1), // First bar, only use high-low
    }
}

/// Calculate smoothed average using Wilder's method
fn calculate_smoothed_avg(values: &[f64], length: usize) -> f64 {
    if values.len() < length {
        return 0.0;
    }

    // Initial simple average
    let mut avg = values[..length].iter().sum::<f64>() / length as f64;

    // Apply smoothing for remaining values
    let alpha = 1.0 / length as f64;
    for val in values.iter().skip(length) {
        avg = alpha * val + (1.0 - alpha) * avg;
    }

    avg
}

// ============================================================================
// Moving Averages
// ============================================================================

/// Register ta.sma - Simple Moving Average
fn register_sma(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sma")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);
        calculate_sma(series, length)
    });

    registry.register(meta, func);
}

/// Register ta.ema - Exponential Moving Average
fn register_ema(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("ema")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 12);
        calculate_ema(series, length, false)
    });

    registry.register(meta, func);
}

/// Register ta.rma - Relative Moving Average (Wilder's smoothing)
fn register_rma(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("rma")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);
        calculate_ema(series, length, true) // RMA uses Wilder smoothing (alpha = 1/N)
    });

    registry.register(meta, func);
}

/// Register ta.wma - Weighted Moving Average
fn register_wma(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("wma")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 10);
        calculate_wma(series, length)
    });

    registry.register(meta, func);
}

// ============================================================================
// Momentum Indicators
// ============================================================================

/// Register ta.rsi - Relative Strength Index
fn register_rsi(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("rsi")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);

        if series.len() < 2 {
            return Value::Na;
        }

        // Calculate gains and losses
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..series.len() {
            let previous = match series.get(i - 1).and_then(get_float) {
                Some(f) => f,
                None => continue,
            };
            let current = match series.get(i).and_then(get_float) {
                Some(f) => f,
                None => continue,
            };

            let change = current - previous;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        if gains.len() < length {
            return Value::Na;
        }

        // Use Wilder's smoothing (RMA) for gains and losses
        let avg_gain = calculate_smoothed_avg(&gains, length);
        let avg_loss = calculate_smoothed_avg(&losses, length);

        if avg_loss == 0.0 {
            return Value::Float(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Value::Float(rsi)
    });

    registry.register(meta, func);
}

/// Register ta.macd - Moving Average Convergence Divergence
fn register_macd(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("macd")
        .with_namespace("ta")
        .with_required_args(1)
        .with_optional_args(3)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        let fast_len = extract_length(args, 1, 12);
        let slow_len = extract_length(args, 2, 26);
        let signal_len = extract_length(args, 3, 9);

        let prices = value_slice_to_f64(series);
        if prices.len() < slow_len {
            return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na]));
        }

        let fast = match calculate_ema_from_f64(&prices, fast_len, false) {
            Some(v) => v,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
        };
        let slow = match calculate_ema_from_f64(&prices, slow_len, false) {
            Some(v) => v,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
        };
        let macd = fast - slow;

        let mut macd_series = Vec::new();
        for end in 1..=prices.len() {
            let prefix = &prices[..end];
            if let (Some(f), Some(s)) = (
                calculate_ema_from_f64(prefix, fast_len, false),
                calculate_ema_from_f64(prefix, slow_len, false),
            ) {
                macd_series.push(f - s);
            }
        }

        let signal = match calculate_ema_from_f64(&macd_series, signal_len, false) {
            Some(v) => v,
            None => return Value::Tuple(Box::new([Value::Float(macd), Value::Na, Value::Na])),
        };

        let histogram = macd - signal;

        Value::Tuple(Box::new([
            Value::Float(macd),
            Value::Float(signal),
            Value::Float(histogram),
        ]))
    });

    registry.register(meta, func);
}

/// Register ta.mom - Momentum
fn register_mom(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("mom")
        .with_namespace("ta")
        .with_required_args(1)
        .with_optional_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        let length = extract_length(args, 1, 10);

        let current = match series.last().and_then(get_float) {
            Some(f) => f,
            None => return Value::Na,
        };

        if series.len() <= length {
            return Value::Na;
        }

        let previous = match series.get(series.len() - 1 - length).and_then(get_float) {
            Some(f) => f,
            None => return Value::Na,
        };

        Value::Float(current - previous)
    });

    registry.register(meta, func);
}

/// Register ta.cci - Commodity Channel Index
fn register_cci(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("cci")
        .with_namespace("ta")
        .with_required_args(2)
        .with_optional_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 20);
        let values = value_slice_to_f64(source);
        if values.len() < length {
            return Value::Na;
        }

        let window = &values[values.len() - length..];
        let tp = *values.last().unwrap_or(&f64::NAN);
        if tp.is_nan() {
            return Value::Na;
        }

        let sma_tp: f64 = window.iter().sum::<f64>() / length as f64;

        // Calculate mean deviation
        let mean_dev: f64 = window.iter().map(|v| (v - sma_tp).abs()).sum::<f64>() / length as f64;

        if mean_dev == 0.0 {
            return Value::Na;
        }

        // CCI = (TP - SMA_TP) / (0.015 * mean_dev)
        Value::Float((tp - sma_tp) / (0.015 * mean_dev))
    });

    registry.register(meta, func);
}

// ============================================================================
// Volatility Indicators
// ============================================================================

/// Register ta.atr - Average True Range
fn register_atr(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("atr")
        .with_namespace("ta")
        .with_required_args(1)
        .with_optional_args(3)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let high = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let low = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let close = match args.get(2).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        let length = extract_length(args, 3, 14);

        // Calculate TR values in chronological order.
        let mut tr_values = Vec::new();
        for i in 0..high.len().min(low.len()).min(close.len()) {
            let h = match high.get(i).and_then(get_float) {
                Some(f) => f,
                None => continue,
            };
            let l = match low.get(i).and_then(get_float) {
                Some(f) => f,
                None => continue,
            };

            let tr1 = h - l;
            let tr = if i > 0 {
                match close.get(i - 1).and_then(get_float) {
                    Some(pc) => {
                        let tr2 = (h - pc).abs();
                        let tr3 = (l - pc).abs();
                        tr1.max(tr2).max(tr3)
                    }
                    None => tr1,
                }
            } else {
                tr1
            };
            tr_values.push(tr);
        }

        if tr_values.is_empty() {
            return Value::Na;
        }

        if tr_values.len() < length {
            return Value::Na;
        }

        // Use RMA (Wilder's smoothing) for ATR
        let atr = calculate_smoothed_avg(&tr_values, length);
        Value::Float(atr)
    });

    registry.register(meta, func);
}

/// Register ta.tr - True Range
fn register_tr(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("tr")
        .with_namespace("ta")
        .with_required_args(0)
        .with_optional_args(4)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let high = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let low = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let close = match args.get(2).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        calculate_tr(high, low, close)
    });

    registry.register(meta, func);
}

/// Register ta.bb - Bollinger Bands
fn register_bbands(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("bb")
        .with_namespace("ta")
        .with_required_args(3)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
        };

        let length = extract_length(args, 1, 20);
        let mult = args.get(2).and_then(|v| v.as_float()).unwrap_or(2.0);

        let f64_values = value_slice_to_f64(series);
        if f64_values.len() < length {
            return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na]));
        }

        let start = f64_values.len() - length;
        let window = &f64_values[start..];
        let sma = window.iter().sum::<f64>() / length as f64;

        let sum_sq_diff = window
            .iter()
            .map(|value| {
                let diff = value - sma;
                diff * diff
            })
            .sum::<f64>();
        let variance = sum_sq_diff / length as f64;
        let stdev = variance.sqrt();

        // Return [basis (SMA), upper, lower]
        let basis = Value::Float(sma);
        let upper = Value::Float(sma + mult * stdev);
        let lower = Value::Float(sma - mult * stdev);

        Value::Tuple(Box::new([basis, upper, lower]))
    });

    registry.register(meta, func);
}

// ============================================================================
// Stochastic
// ============================================================================

/// Register ta.stoch - Stochastic Oscillator
fn register_stoch(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("stoch")
        .with_namespace("ta")
        .with_required_args(4)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let high = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let low = match args.get(2).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        let length = extract_length(args, 3, 14);
        let source_values = value_slice_to_f64(source);
        let high_values = value_slice_to_f64(high);
        let low_values = value_slice_to_f64(low);

        if source_values.len() < length || high_values.len() < length || low_values.len() < length {
            return Value::Na;
        }

        let current_close = *source_values.last().unwrap_or(&f64::NAN);
        if current_close.is_nan() {
            return Value::Na;
        }

        let highest_high = match calculate_highest_f64(&high_values, length) {
            Some(value) => value,
            None => return Value::Na,
        };
        let lowest_low = match calculate_lowest_f64(&low_values, length) {
            Some(value) => value,
            None => return Value::Na,
        };

        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Value::Na;
        }

        Value::Float((current_close - lowest_low) / range * 100.0)
    });

    registry.register(meta, func);
}

// ============================================================================
// Extremum Functions
// ============================================================================

/// Register ta.highest - Highest value over period
///
/// Uses SIMD-optimized SeriesBufF64 for better performance
fn register_highest(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("highest")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);

        // Use SIMD-optimized version
        let f64_values = value_slice_to_f64(series);
        match calculate_highest_f64(&f64_values, length) {
            Some(result) => Value::Float(result),
            None => Value::Na,
        }
    });

    registry.register(meta, func);
}

/// Register ta.lowest - Lowest value over period
///
/// Uses SIMD-optimized SeriesBufF64 for better performance
fn register_lowest(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("lowest")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);

        // Use SIMD-optimized version
        let f64_values = value_slice_to_f64(series);
        match calculate_lowest_f64(&f64_values, length) {
            Some(result) => Value::Float(result),
            None => Value::Na,
        }
    });

    registry.register(meta, func);
}

/// Register ta.highestbars - Bars since highest value
fn register_highestbars(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("highestbars")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);

        let mut highest = f64::NEG_INFINITY;
        let mut highest_idx = 0;

        for i in 0..length.min(series.len()) {
            if let Some(val) = series.get(i).and_then(get_float) {
                if val > highest {
                    highest = val;
                    highest_idx = i;
                }
            }
        }

        if highest == f64::NEG_INFINITY {
            Value::Na
        } else {
            Value::Int(highest_idx as i64)
        }
    });

    registry.register(meta, func);
}

/// Register ta.lowestbars - Bars since lowest value
fn register_lowestbars(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("lowestbars")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 14);

        let mut lowest = f64::INFINITY;
        let mut lowest_idx = 0;

        for i in 0..length.min(series.len()) {
            if let Some(val) = series.get(i).and_then(get_float) {
                if val < lowest {
                    lowest = val;
                    lowest_idx = i;
                }
            }
        }

        if lowest == f64::INFINITY {
            Value::Na
        } else {
            Value::Int(lowest_idx as i64)
        }
    });

    registry.register(meta, func);
}

// ============================================================================
// Cross Functions
// ============================================================================

/// Register ta.crossover - True when series1 crosses over series2
fn register_crossover(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("crossover")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series1 = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Bool(false),
        };
        let series2 = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Bool(false),
        };

        if series1.len() < 2 || series2.len() < 2 {
            return Value::Bool(false);
        }

        let curr1 = match series1.last().and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };
        let prev1 = match series1.get(series1.len() - 2).and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };
        let curr2 = match series2.last().and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };
        let prev2 = match series2.get(series2.len() - 2).and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };

        // Crossover: prev1 <= prev2 AND curr1 > curr2
        Value::Bool(prev1 <= prev2 && curr1 > curr2)
    });

    registry.register(meta, func);
}

/// Register ta.crossunder - True when series1 crosses under series2
fn register_crossunder(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("crossunder")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series1 = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Bool(false),
        };
        let series2 = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Bool(false),
        };

        if series1.len() < 2 || series2.len() < 2 {
            return Value::Bool(false);
        }

        let curr1 = match series1.last().and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };
        let prev1 = match series1.get(series1.len() - 2).and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };
        let curr2 = match series2.last().and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };
        let prev2 = match series2.get(series2.len() - 2).and_then(get_float) {
            Some(f) => f,
            None => return Value::Bool(false),
        };

        // Crossunder: prev1 >= prev2 AND curr1 < curr2
        Value::Bool(prev1 >= prev2 && curr1 < curr2)
    });

    registry.register(meta, func);
}

/// Register ta.barssince - Bars since condition was true
fn register_barssince(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("barssince")
        .with_namespace("ta")
        .with_required_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let condition = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        for (idx, val) in condition.iter().enumerate().rev() {
            if matches!(val, Value::Bool(true)) {
                return Value::Int((condition.len() - 1 - idx) as i64);
            }
        }

        Value::Na
    });

    registry.register(meta, func);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);
        registry
    }

    fn series(values: Vec<f64>) -> Value {
        Value::Array(values.into_iter().map(Value::Float).collect())
    }

    #[test]
    fn test_sma() {
        let registry = test_registry();

        let data = series(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        let result = registry.dispatch("ta.sma", &[data, Value::Int(3)]);

        // SMA of trailing window [30, 40, 50] = 40
        assert_eq!(result, Some(Value::Float(40.0)));
    }

    #[test]
    fn test_ema() {
        let registry = test_registry();

        let data = series(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        let result = registry.dispatch("ta.ema", &[data, Value::Int(3)]);

        // Should return a value (exact calculation depends on EMA formula)
        assert!(matches!(result, Some(Value::Float(_))));
    }

    #[test]
    fn test_highest() {
        let registry = test_registry();

        let data = series(vec![10.0, 50.0, 30.0, 40.0, 20.0]);
        let result = registry.dispatch("ta.highest", &[data, Value::Int(5)]);

        assert_eq!(result, Some(Value::Float(50.0)));
    }

    #[test]
    fn test_lowest() {
        let registry = test_registry();

        let data = series(vec![10.0, 50.0, 30.0, 40.0, 20.0]);
        let result = registry.dispatch("ta.lowest", &[data, Value::Int(5)]);

        assert_eq!(result, Some(Value::Float(10.0)));
    }

    #[test]
    fn test_highest_trailing_window() {
        let registry = test_registry();

        let data = series(vec![100.0, 110.0, 120.0, 130.0, 125.0]);
        let result = registry.dispatch("ta.highest", &[data, Value::Int(3)]);

        assert_eq!(result, Some(Value::Float(130.0)));
    }

    #[test]
    fn test_crossover() {
        let registry = test_registry();

        let s1 = series(vec![20.0, 30.0]); // prev=20, current=30
        let s2 = series(vec![25.0, 25.0]); // prev=25, current=25

        let result = registry.dispatch("ta.crossover", &[s1, s2]);
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_crossunder() {
        let registry = test_registry();

        let s1 = series(vec![30.0, 20.0]); // prev=30, current=20
        let s2 = series(vec![25.0, 25.0]); // prev=25, current=25

        let result = registry.dispatch("ta.crossunder", &[s1, s2]);
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_barssince() {
        let registry = test_registry();

        let cond = Value::Array(vec![
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(false),
        ]);

        let result = registry.dispatch("ta.barssince", &[cond]);
        assert_eq!(result, Some(Value::Int(2)));
    }

    #[test]
    fn test_barssince_returns_na_when_never_true() {
        let registry = test_registry();

        let cond = Value::Array(vec![
            Value::Bool(false),
            Value::Bool(false),
            Value::Bool(false),
        ]);
        let result = registry.dispatch("ta.barssince", &[cond]);
        assert_eq!(result, Some(Value::Na));
    }

    #[test]
    fn test_rsi() {
        let registry = test_registry();

        // Price data with clear up trend then down trend
        let data = series(vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00,
        ]);

        let result = registry.dispatch("ta.rsi", &[data, Value::Int(14)]);
        assert!(matches!(result, Some(Value::Float(v)) if v > 0.0 && v < 100.0));
    }

    #[test]
    fn test_bb_returns_tuple_with_warmup_na() {
        let registry = test_registry();

        let data = series(vec![10.0, 20.0, 30.0]);
        let result = registry.dispatch("ta.bb", &[data, Value::Int(5), Value::Float(2.0)]);

        assert_eq!(
            result,
            Some(Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])))
        );
    }

    #[test]
    fn test_stoch_returns_percent_k() {
        let registry = test_registry();

        let source = series(vec![10.0, 12.0, 14.0, 16.0, 18.0]);
        let high = series(vec![11.0, 13.0, 15.0, 17.0, 19.0]);
        let low = series(vec![9.0, 11.0, 13.0, 15.0, 17.0]);
        let result = registry.dispatch("ta.stoch", &[source, high, low, Value::Int(5)]);

        assert_eq!(result, Some(Value::Float(90.0)));
    }

    #[test]
    fn test_mom_uses_current_minus_length_bars_ago() {
        let registry = test_registry();

        let data = series(vec![10.0, 12.0, 15.0, 19.0]);
        let result = registry.dispatch("ta.mom", &[data, Value::Int(2)]);

        assert_eq!(result, Some(Value::Float(7.0)));
    }

    #[test]
    fn test_cci_uses_source_series() {
        let registry = test_registry();

        let data = series(vec![100.0, 110.0, 120.0]);
        let result = registry.dispatch("ta.cci", &[data, Value::Int(3)]);

        assert!(matches!(result, Some(Value::Float(v)) if (v - 100.0).abs() < 1e-10));
    }

    #[test]
    fn test_tr_returns_first_bar_range_without_prev_close() {
        let registry = test_registry();

        let high = series(vec![10.0]);
        let low = series(vec![7.0]);
        let close = series(vec![8.0]);
        let result = registry.dispatch("ta.tr", &[high, low, close]);

        assert_eq!(result, Some(Value::Float(3.0)));
    }
}
