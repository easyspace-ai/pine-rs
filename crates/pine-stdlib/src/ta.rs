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
    register_vwma(registry);
    register_roc(registry);
    register_obv(registry);
    register_change(registry);
    register_cum(registry);
    register_pvt(registry);

    // Momentum indicators
    register_rsi(registry);
    register_macd(registry);
    register_mom(registry);
    register_cci(registry);
    register_mfi(registry);

    // Volatility indicators
    register_atr(registry);
    register_tr(registry);
    register_bbands(registry);
    register_dmi(registry);
    register_supertrend(registry);

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

/// Extract length parameter from arguments with a default value.
///
/// # Arguments
/// * `args` - The argument slice
/// * `idx` - The index of the length argument (after series arguments)
/// * `default` - The default value if argument is not provided
///
/// # TV Compatibility Note
/// Some ta.* functions in TV have default parameters (e.g., ta.macd, ta.bb, ta.atr),
/// while others require explicit parameters (e.g., ta.sma, ta.ema).
/// This function supports both cases via the `default` parameter.
fn extract_length(args: &[Value], idx: usize, default: usize) -> usize {
    args.get(idx)
        .and_then(|v| v.as_int())
        .map(|n| n.max(1) as usize)
        .unwrap_or(default)
}

/// Extract required length parameter from arguments (no default).
/// Returns None if the parameter is not provided, which should result in NA.
fn extract_length_required(args: &[Value], idx: usize) -> Option<usize> {
    args.get(idx)
        .and_then(|v| v.as_int())
        .map(|n| n.max(1) as usize)
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
/// Calculate Weighted Moving Average (WMA)
///
/// WMA gives more weight to recent data points.
/// Formula: WMA = (P1×n + P2×(n-1) + ... + Pn×1) / (n + (n-1) + ... + 1)
/// where P1 is the most recent (latest) price, Pn is the oldest.
fn calculate_wma(values: &[Value], length: usize) -> Value {
    let valid_values: Vec<f64> = values.iter().filter_map(get_float).collect();
    if length == 0 || valid_values.len() < length {
        return Value::Na;
    }

    // Get the trailing window of 'length' values
    let window = &valid_values[valid_values.len() - length..];

    // Calculate weighted sum: most recent (last element) gets weight = length
    // oldest (first element) gets weight = 1
    let mut weighted_sum = 0.0;
    let mut weight_sum = 0;

    for (idx, value) in window.iter().enumerate() {
        // idx = 0 is oldest, idx = length-1 is most recent
        // weight should be: oldest=1, ..., most recent=length
        let weight = idx + 1;
        weighted_sum += weight as f64 * value;
        weight_sum += weight;
    }

    Value::Float(weighted_sum / weight_sum as f64)
}

fn calculate_vwma(source: &[Value], volume: &[Value], length: usize) -> Value {
    let source_values = value_slice_to_f64(source);
    let volume_values = value_slice_to_f64(volume);
    let window_len = length.min(source_values.len()).min(volume_values.len());
    if length == 0 || window_len < length {
        return Value::Na;
    }

    let source_window = &source_values[source_values.len() - window_len..];
    let volume_window = &volume_values[volume_values.len() - window_len..];

    let weighted_sum: f64 = source_window
        .iter()
        .zip(volume_window.iter())
        .map(|(price, volume)| price * volume)
        .sum();
    let volume_sum: f64 = volume_window.iter().sum();

    if volume_sum == 0.0 {
        return Value::Na;
    }

    Value::Float(weighted_sum / volume_sum)
}

fn calculate_roc(source: &[Value], length: usize) -> Value {
    let values = value_slice_to_f64(source);
    if length == 0 || values.len() <= length {
        return Value::Na;
    }

    let current = values[values.len() - 1];
    let previous = values[values.len() - 1 - length];
    if previous == 0.0 {
        return Value::Na;
    }

    Value::Float(((current - previous) / previous) * 100.0)
}

fn calculate_obv(source: &[Value], volume: &[Value]) -> Value {
    let price_values = value_slice_to_f64(source);
    let volume_values = value_slice_to_f64(volume);
    let window_len = price_values.len().min(volume_values.len());

    if window_len == 0 {
        return Value::Na;
    }

    let price_window = &price_values[price_values.len() - window_len..];
    let volume_window = &volume_values[volume_values.len() - window_len..];

    let mut obv = 0.0;
    for idx in 1..window_len {
        if price_window[idx] > price_window[idx - 1] {
            obv += volume_window[idx];
        } else if price_window[idx] < price_window[idx - 1] {
            obv -= volume_window[idx];
        }
    }

    Value::Float(obv)
}

fn calculate_change(source: &[Value], length: usize) -> Value {
    if length == 0 || source.len() <= length {
        return Value::Na;
    }

    let current = &source[source.len() - 1];
    let previous = &source[source.len() - 1 - length];

    match (current, previous) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a != b),
        _ => match (get_float(current), get_float(previous)) {
            (Some(a), Some(b)) => Value::Float(a - b),
            _ => Value::Na,
        },
    }
}

fn calculate_cum(source: &[Value]) -> Value {
    let mut sum = 0.0;
    let mut found = false;

    for value in source {
        if let Some(v) = get_float(value) {
            sum += v;
            found = true;
        }
    }

    if found {
        Value::Float(sum)
    } else {
        Value::Na
    }
}

fn calculate_pvt(source: &[Value], volume: &[Value]) -> Value {
    let price_values = value_slice_to_f64(source);
    let volume_values = value_slice_to_f64(volume);
    let window_len = price_values.len().min(volume_values.len());

    if window_len == 0 {
        return Value::Na;
    }

    let price_window = &price_values[price_values.len() - window_len..];
    let volume_window = &volume_values[volume_values.len() - window_len..];

    let mut pvt = 0.0;
    for idx in 1..window_len {
        let previous = price_window[idx - 1];
        if previous == 0.0 {
            continue;
        }
        pvt += ((price_window[idx] - previous) / previous) * volume_window[idx];
    }

    Value::Float(pvt)
}

fn calculate_mfi(source: &[Value], volume: &[Value], length: usize) -> Value {
    let window_len = source.len().min(volume.len());
    if length == 0 || window_len <= length {
        return Value::Na;
    }

    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;

    let source_window = &source[source.len() - (length + 1)..];
    let volume_window = &volume[volume.len() - (length + 1)..];

    for idx in 1..source_window.len() {
        let current_src = match get_float(&source_window[idx]) {
            Some(value) => value,
            None => continue,
        };
        let previous_src = match get_float(&source_window[idx - 1]) {
            Some(value) => value,
            None => continue,
        };
        let current_volume = match get_float(&volume_window[idx]) {
            Some(value) => value,
            None => continue,
        };

        if current_src > previous_src {
            positive_flow += current_volume * current_src;
        } else if current_src < previous_src {
            negative_flow += current_volume * current_src;
        }
    }

    if negative_flow == 0.0 {
        return Value::Float(100.0);
    }

    let money_ratio = positive_flow / negative_flow;
    Value::Float(100.0 - (100.0 / (1.0 + money_ratio)))
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

fn calculate_rma_series(values: &[f64], length: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; values.len()];
    if length == 0 || values.len() < length {
        return result;
    }

    let mut avg = values[..length].iter().sum::<f64>() / length as f64;
    result[length - 1] = Some(avg);

    let alpha = 1.0 / length as f64;
    for (idx, value) in values.iter().enumerate().skip(length) {
        avg = alpha * value + (1.0 - alpha) * avg;
        result[idx] = Some(avg);
    }

    result
}

fn calculate_atr_series_from_hlc(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    length: usize,
) -> Vec<Option<f64>> {
    let source_len = high.len().min(low.len()).min(close.len());
    let mut result = vec![None; source_len];
    if length == 0 || source_len == 0 {
        return result;
    }

    let mut tr_values = Vec::with_capacity(source_len);
    for idx in 0..source_len {
        let tr1 = high[idx] - low[idx];
        let tr = if idx == 0 {
            tr1
        } else {
            tr1.max((high[idx] - close[idx - 1]).abs())
                .max((low[idx] - close[idx - 1]).abs())
        };
        tr_values.push(tr);
    }

    let smoothed = calculate_rma_series(&tr_values, length);
    for (idx, value) in smoothed.into_iter().enumerate() {
        result[idx] = value;
    }
    result
}

// ============================================================================
// Moving Averages
// ============================================================================

/// Register ta.sma - Simple Moving Average
///
/// TV Reference: `ta.sma(source, length)` - length has no default value
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
        // SMA requires explicit length parameter (no default in TV)
        let length = match extract_length_required(args, 1) {
            Some(len) => len,
            None => return Value::Na,
        };
        calculate_sma(series, length)
    });

    registry.register(meta, func);
}

/// Register ta.vwma - Volume Weighted Moving Average
fn register_vwma(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("vwma")
        .with_namespace("ta")
        .with_required_args(2)
        .with_optional_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let volume = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 2, 1);

        calculate_vwma(source, volume, length)
    });

    registry.register(meta, func);
}

/// Register ta.roc - Rate of Change
fn register_roc(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("roc")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 1);
        calculate_roc(source, length)
    });

    registry.register(meta, func);
}

/// Register ta.obv - On-Balance Volume
fn register_obv(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("obv")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let volume = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        calculate_obv(source, volume)
    });

    registry.register(meta, func);
}

/// Register ta.change - Difference from N bars ago, or bool change flag
fn register_change(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("change")
        .with_namespace("ta")
        .with_required_args(1)
        .with_optional_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 1, 1);

        calculate_change(source, length)
    });

    registry.register(meta, func);
}

/// Register ta.cum - Cumulative sum over the full source history
fn register_cum(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("cum")
        .with_namespace("ta")
        .with_required_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        calculate_cum(source)
    });

    registry.register(meta, func);
}

/// Register ta.pvt - Price-Volume Trend
fn register_pvt(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("pvt")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let volume = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        calculate_pvt(source, volume)
    });

    registry.register(meta, func);
}

/// Register ta.ema - Exponential Moving Average
///
/// TV Reference: `ta.ema(source, length)` - length has no default value
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
        // EMA requires explicit length parameter (no default in TV)
        let length = match extract_length_required(args, 1) {
            Some(len) => len,
            None => return Value::Na,
        };
        calculate_ema(series, length, false)
    });

    registry.register(meta, func);
}

/// Register ta.rma - Relative Moving Average (Wilder's smoothing)
///
/// TV Reference: `ta.rma(source, length)` - length has no default value
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
        // RMA requires explicit length parameter (no default in TV)
        let length = match extract_length_required(args, 1) {
            Some(len) => len,
            None => return Value::Na,
        };
        calculate_ema(series, length, true) // RMA uses Wilder smoothing (alpha = 1/N)
    });

    registry.register(meta, func);
}

/// Register ta.wma - Weighted Moving Average
///
/// TV Reference: `ta.wma(source, length)` - length has no default value
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
        // WMA requires explicit length parameter (no default in TV)
        let length = match extract_length_required(args, 1) {
            Some(len) => len,
            None => return Value::Na,
        };
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
///
/// TV Reference: `ta.mom(source, length)` - length has no default value
fn register_mom(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("mom")
        .with_namespace("ta")
        .with_required_args(2)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let series = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };

        // Momentum requires explicit length parameter (no default in TV)
        let length = match extract_length_required(args, 1) {
            Some(len) => len,
            None => return Value::Na,
        };

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

/// Register ta.dmi - Directional Movement Index
fn register_dmi(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("dmi")
        .with_namespace("ta")
        .with_required_args(2)
        .with_optional_args(3)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let high = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
        };
        let low = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
        };
        let close = match args.get(2).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
        };

        let di_length = extract_length(args, 3, 14);
        let adx_smoothing = extract_length(args, 4, 14);
        let source_len = high.len().min(low.len()).min(close.len());
        if source_len < 2 {
            return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na]));
        }

        let mut tr_values = Vec::with_capacity(source_len - 1);
        let mut plus_dm_values = Vec::with_capacity(source_len - 1);
        let mut minus_dm_values = Vec::with_capacity(source_len - 1);

        for index in 1..source_len {
            let current_high = match high.get(index).and_then(get_float) {
                Some(value) => value,
                None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
            };
            let current_low = match low.get(index).and_then(get_float) {
                Some(value) => value,
                None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
            };
            let previous_high = match high.get(index - 1).and_then(get_float) {
                Some(value) => value,
                None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
            };
            let previous_low = match low.get(index - 1).and_then(get_float) {
                Some(value) => value,
                None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
            };
            let previous_close = match close.get(index - 1).and_then(get_float) {
                Some(value) => value,
                None => return Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])),
            };

            let up_move = current_high - previous_high;
            let down_move = previous_low - current_low;
            let plus_dm = if up_move > down_move && up_move > 0.0 {
                up_move
            } else {
                0.0
            };
            let minus_dm = if down_move > up_move && down_move > 0.0 {
                down_move
            } else {
                0.0
            };

            let tr = (current_high - current_low)
                .max((current_high - previous_close).abs())
                .max((current_low - previous_close).abs());

            plus_dm_values.push(plus_dm);
            minus_dm_values.push(minus_dm);
            tr_values.push(tr);
        }

        let smoothed_tr = calculate_rma_series(&tr_values, di_length);
        let smoothed_plus_dm = calculate_rma_series(&plus_dm_values, di_length);
        let smoothed_minus_dm = calculate_rma_series(&minus_dm_values, di_length);

        let last_di_plus = match (
            smoothed_tr.last().copied().flatten(),
            smoothed_plus_dm.last().copied().flatten(),
        ) {
            (Some(tr), Some(plus_dm)) if tr != 0.0 => Some(100.0 * plus_dm / tr),
            _ => None,
        };
        let last_di_minus = match (
            smoothed_tr.last().copied().flatten(),
            smoothed_minus_dm.last().copied().flatten(),
        ) {
            (Some(tr), Some(minus_dm)) if tr != 0.0 => Some(100.0 * minus_dm / tr),
            _ => None,
        };

        let dx_values: Vec<f64> = smoothed_tr
            .iter()
            .zip(smoothed_plus_dm.iter())
            .zip(smoothed_minus_dm.iter())
            .filter_map(|((tr, plus_dm), minus_dm)| {
                let tr = tr.as_ref().copied()?;
                let plus_dm = plus_dm.as_ref().copied()?;
                let minus_dm = minus_dm.as_ref().copied()?;
                if tr == 0.0 {
                    return None;
                }
                let di_plus = 100.0 * plus_dm / tr;
                let di_minus = 100.0 * minus_dm / tr;
                let sum = di_plus + di_minus;
                if sum == 0.0 {
                    None
                } else {
                    Some(100.0 * (di_plus - di_minus).abs() / sum)
                }
            })
            .collect();

        let adx = if dx_values.len() < adx_smoothing {
            None
        } else {
            Some(calculate_smoothed_avg(&dx_values, adx_smoothing))
        };

        Value::Tuple(Box::new([
            last_di_plus.map_or(Value::Na, Value::Float),
            last_di_minus.map_or(Value::Na, Value::Float),
            adx.map_or(Value::Na, Value::Float),
        ]))
    });

    registry.register(meta, func);
}

/// Register ta.mfi - Money Flow Index
fn register_mfi(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("mfi")
        .with_namespace("ta")
        .with_required_args(2)
        .with_optional_args(1)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let source = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let volume = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Na,
        };
        let length = extract_length(args, 2, 14);

        calculate_mfi(source, volume, length)
    });

    registry.register(meta, func);
}

/// Register ta.supertrend - Supertrend indicator
fn register_supertrend(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("supertrend")
        .with_namespace("ta")
        .with_required_args(2)
        .with_optional_args(5)
        .with_series_return();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let high = match args.first().and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na])),
        };
        let low = match args.get(1).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na])),
        };
        let close = match args.get(2).and_then(extract_array) {
            Some(s) => s,
            None => return Value::Tuple(Box::new([Value::Na, Value::Na])),
        };

        let factor = args.get(3).and_then(get_float).unwrap_or(3.0);
        let atr_period = extract_length(args, 4, 10);

        let high_values = value_slice_to_f64(high);
        let low_values = value_slice_to_f64(low);
        let close_values = value_slice_to_f64(close);
        let source_len = high_values
            .len()
            .min(low_values.len())
            .min(close_values.len());
        if source_len == 0 {
            return Value::Tuple(Box::new([Value::Na, Value::Na]));
        }

        let atr_series = calculate_atr_series_from_hlc(
            &high_values[..source_len],
            &low_values[..source_len],
            &close_values[..source_len],
            atr_period,
        );

        let mut final_upper: Vec<Option<f64>> = vec![None; source_len];
        let mut final_lower: Vec<Option<f64>> = vec![None; source_len];
        let mut trend: Vec<Option<f64>> = vec![None; source_len];
        let mut direction: Vec<Option<f64>> = vec![None; source_len];

        for idx in 0..source_len {
            let atr = match atr_series[idx] {
                Some(value) => value,
                None => continue,
            };
            let hl2 = (high_values[idx] + low_values[idx]) / 2.0;
            let basic_upper = hl2 + factor * atr;
            let basic_lower = hl2 - factor * atr;

            let prev_upper = idx
                .checked_sub(1)
                .and_then(|prev| final_upper[prev])
                .unwrap_or(basic_upper);
            let prev_lower = idx
                .checked_sub(1)
                .and_then(|prev| final_lower[prev])
                .unwrap_or(basic_lower);
            let prev_close = idx
                .checked_sub(1)
                .map(|prev| close_values[prev])
                .unwrap_or(close_values[idx]);

            let current_lower = if basic_lower > prev_lower || prev_close < prev_lower {
                basic_lower
            } else {
                prev_lower
            };
            let current_upper = if basic_upper < prev_upper || prev_close > prev_upper {
                basic_upper
            } else {
                prev_upper
            };

            final_lower[idx] = Some(current_lower);
            final_upper[idx] = Some(current_upper);

            let current_direction = if idx == 0 || atr_series[idx - 1].is_none() {
                1.0
            } else if let Some(prev_trend) = trend[idx - 1] {
                if (prev_trend - prev_upper).abs() < 1e-10 {
                    if close_values[idx] > current_upper {
                        -1.0
                    } else {
                        1.0
                    }
                } else if close_values[idx] < current_lower {
                    1.0
                } else {
                    -1.0
                }
            } else {
                1.0
            };

            let current_trend = if current_direction < 0.0 {
                current_lower
            } else {
                current_upper
            };

            direction[idx] = Some(current_direction);
            trend[idx] = Some(current_trend);
        }

        Value::Tuple(Box::new([
            trend
                .last()
                .and_then(|value| *value)
                .map_or(Value::Na, Value::Float),
            direction
                .last()
                .and_then(|value| *value)
                .map_or(Value::Na, Value::Float),
        ]))
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

        let n = series.len();
        if n == 0 {
            return Value::Na;
        }
        let wl = length.min(n);
        let start = n - wl;
        let mut highest_idx = start;
        for i in start..n {
            if let Some(val) = series.get(i).and_then(get_float) {
                if val >= highest {
                    highest = val;
                    highest_idx = i;
                }
            }
        }

        if highest == f64::NEG_INFINITY {
            Value::Na
        } else {
            Value::Int((n - 1 - highest_idx) as i64)
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

        let n = series.len();
        if n == 0 {
            return Value::Na;
        }
        let wl = length.min(n);
        let start = n - wl;
        let mut lowest_idx = start;
        for i in start..n {
            if let Some(val) = series.get(i).and_then(get_float) {
                if val <= lowest {
                    lowest = val;
                    lowest_idx = i;
                }
            }
        }

        if lowest == f64::INFINITY {
            Value::Na
        } else {
            Value::Int((n - 1 - lowest_idx) as i64)
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
    fn test_highestbars_trailing_most_recent_peak() {
        let registry = test_registry();

        let data = series(vec![100.0, 110.0, 120.0, 130.0, 125.0]);
        let result = registry.dispatch("ta.highestbars", &[data.clone(), Value::Int(3)]);
        assert_eq!(result, Some(Value::Int(1)));

        let data2 = series(vec![10.0, 50.0, 30.0, 40.0, 20.0]);
        let result2 = registry.dispatch("ta.highestbars", &[data2, Value::Int(5)]);
        assert_eq!(result2, Some(Value::Int(3)));
    }

    #[test]
    fn test_lowestbars_trailing_most_recent_trough() {
        let registry = test_registry();

        let data = series(vec![100.0, 110.0, 90.0, 95.0, 105.0]);
        let result = registry.dispatch("ta.lowestbars", &[data, Value::Int(3)]);
        assert_eq!(result, Some(Value::Int(2)));
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
    fn test_dmi_returns_expected_tuple() {
        let registry = test_registry();

        let high = series(vec![
            30.20, 30.60, 31.10, 31.40, 31.80, 31.60, 32.00, 32.50, 32.80, 33.10, 33.40, 33.00,
        ]);
        let low = series(vec![
            29.60, 29.90, 30.20, 30.50, 30.90, 30.80, 31.10, 31.70, 31.90, 32.20, 32.60, 32.30,
        ]);
        let close = series(vec![
            29.90, 30.40, 30.80, 31.10, 31.50, 31.10, 31.80, 32.30, 32.40, 32.90, 33.10, 32.50,
        ]);

        let result = registry.dispatch("ta.dmi", &[high, low, close, Value::Int(5), Value::Int(5)]);

        match result {
            Some(Value::Tuple(values)) => {
                assert!(matches!(values[0], Value::Float(v) if (v - 31.9232316337).abs() < 1e-9));
                assert!(matches!(values[1], Value::Float(v) if (v - 7.7662757957).abs() < 1e-9));
                assert!(matches!(values[2], Value::Float(v) if (v - 86.7180957741).abs() < 1e-9));
            }
            other => panic!("expected DMI tuple, got {other:?}"),
        }
    }

    #[test]
    fn test_dmi_returns_warmup_na_when_not_enough_history() {
        let registry = test_registry();

        let high = series(vec![10.0, 11.0, 12.0]);
        let low = series(vec![9.0, 9.5, 10.5]);
        let close = series(vec![9.5, 10.5, 11.0]);
        let result = registry.dispatch("ta.dmi", &[high, low, close, Value::Int(5), Value::Int(5)]);

        assert_eq!(
            result,
            Some(Value::Tuple(Box::new([Value::Na, Value::Na, Value::Na])))
        );
    }

    #[test]
    fn test_supertrend_returns_line_and_direction() {
        let registry = test_registry();

        let high = series(vec![
            11.1, 11.6, 12.1, 12.6, 13.1, 13.6, 14.1, 14.6, 15.8, 17.0, 18.2, 19.4, 17.4, 15.4,
            13.4, 11.4, 14.4, 16.2, 18.0, 19.8,
        ]);
        let low = series(vec![
            9.9, 10.4, 10.9, 11.4, 11.9, 12.4, 12.9, 13.4, 14.6, 15.8, 17.0, 18.2, 16.2, 14.2,
            12.2, 10.2, 13.2, 15.0, 16.8, 18.6,
        ]);
        let close = series(vec![
            10.5, 11.0, 11.5, 12.0, 12.5, 13.0, 13.5, 14.0, 15.2, 16.4, 17.6, 18.8, 16.8, 14.8,
            12.8, 10.8, 13.8, 15.6, 17.4, 19.2,
        ]);

        let result = registry.dispatch(
            "ta.supertrend",
            &[high, low, close, Value::Float(3.0), Value::Int(5)],
        );

        match result {
            Some(Value::Tuple(values)) => {
                assert!(matches!(values[0], Value::Float(v) if (v - 11.9119482421).abs() < 1e-6));
                assert!(matches!(values[1], Value::Float(v) if (v + 1.0).abs() < 1e-9));
            }
            other => panic!("expected supertrend tuple, got {other:?}"),
        }
    }

    #[test]
    fn test_supertrend_returns_warmup_na_when_not_enough_history() {
        let registry = test_registry();

        let high = series(vec![10.0, 11.0, 12.0]);
        let low = series(vec![9.0, 10.0, 11.0]);
        let close = series(vec![9.5, 10.5, 11.5]);
        let result = registry.dispatch(
            "ta.supertrend",
            &[high, low, close, Value::Float(3.0), Value::Int(5)],
        );

        assert_eq!(result, Some(Value::Tuple(Box::new([Value::Na, Value::Na]))));
    }

    #[test]
    fn test_mom_uses_current_minus_length_bars_ago() {
        let registry = test_registry();

        let data = series(vec![10.0, 12.0, 15.0, 19.0]);
        let result = registry.dispatch("ta.mom", &[data, Value::Int(2)]);

        assert_eq!(result, Some(Value::Float(7.0)));
    }

    #[test]
    fn test_vwma_weights_by_volume() {
        let registry = test_registry();

        let source = series(vec![10.0, 20.0, 30.0, 40.0]);
        let volume = series(vec![1.0, 1.0, 2.0, 6.0]);
        let result = registry.dispatch("ta.vwma", &[source, volume, Value::Int(4)]);

        assert_eq!(result, Some(Value::Float(33.0)));
    }

    #[test]
    fn test_vwma_returns_na_when_volume_sum_zero() {
        let registry = test_registry();

        let source = series(vec![10.0, 20.0, 30.0]);
        let volume = series(vec![0.0, 0.0, 0.0]);
        let result = registry.dispatch("ta.vwma", &[source, volume, Value::Int(3)]);

        assert_eq!(result, Some(Value::Na));
    }

    #[test]
    fn test_roc_returns_percent_change() {
        let registry = test_registry();

        let source = series(vec![100.0, 110.0, 121.0, 133.1]);
        let result = registry.dispatch("ta.roc", &[source, Value::Int(2)]);

        assert!(matches!(result, Some(Value::Float(v)) if (v - 21.0).abs() < 1e-9));
    }

    #[test]
    fn test_obv_accumulates_volume_by_close_direction() {
        let registry = test_registry();

        let source = series(vec![100.0, 103.0, 103.0, 101.0, 104.0]);
        let volume = series(vec![1000.0, 1200.0, 900.0, 1300.0, 1100.0]);
        let result = registry.dispatch("ta.obv", &[source, volume]);

        assert_eq!(result, Some(Value::Float(1000.0)));
    }

    #[test]
    fn test_change_returns_difference_from_length_bars_ago() {
        let registry = test_registry();

        let source = series(vec![100.0, 103.0, 101.0, 105.0]);
        let result = registry.dispatch("ta.change", &[source, Value::Int(2)]);

        assert_eq!(result, Some(Value::Float(2.0)));
    }

    #[test]
    fn test_change_returns_bool_when_bool_series_changes() {
        let registry = test_registry();

        let source = Value::Array(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(false),
        ]);
        let result = registry.dispatch("ta.change", &[source]);

        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_cum_sums_numeric_history() {
        let registry = test_registry();

        let source = series(vec![10.0, 5.0, -2.0, 8.0]);
        let result = registry.dispatch("ta.cum", &[source]);

        assert_eq!(result, Some(Value::Float(21.0)));
    }

    #[test]
    fn test_pvt_accumulates_price_volume_trend() {
        let registry = test_registry();

        let source = series(vec![100.0, 103.0, 103.0, 101.0, 104.0]);
        let volume = series(vec![1000.0, 1200.0, 900.0, 1300.0, 1100.0]);
        let result = registry.dispatch("ta.pvt", &[source, volume]);

        assert!(matches!(result, Some(Value::Float(v)) if (v - 43.43054888013073).abs() < 1e-9));
    }

    #[test]
    fn test_mfi_returns_100_when_no_negative_flow() {
        let registry = test_registry();

        let source = series(vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0]);
        let volume = series(vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0]);
        let result = registry.dispatch("ta.mfi", &[source, volume, Value::Int(5)]);

        assert_eq!(result, Some(Value::Float(100.0)));
    }

    #[test]
    fn test_mfi_oscillates_between_zero_and_hundred() {
        let registry = test_registry();

        let source = series(vec![10.0, 11.0, 10.5, 11.5, 10.8, 11.8]);
        let volume = series(vec![100.0, 120.0, 110.0, 130.0, 115.0, 140.0]);
        let result = registry.dispatch("ta.mfi", &[source, volume, Value::Int(5)]);

        assert!(matches!(result, Some(Value::Float(v)) if v > 0.0 && v < 100.0));
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

    #[test]
    fn test_wma_weights_recent_data_more() {
        let registry = test_registry();

        // WMA with length 3: [10, 20, 30]
        // Weights: 10*1 + 20*2 + 30*3 = 10 + 40 + 90 = 140
        // Weight sum: 1 + 2 + 3 = 6
        // WMA = 140 / 6 = 23.333...
        let data = series(vec![10.0, 20.0, 30.0]);
        let result = registry.dispatch("ta.wma", &[data, Value::Int(3)]);

        assert!(matches!(result, Some(Value::Float(v)) if (v - 23.333333333333332).abs() < 1e-10));
    }

    #[test]
    fn test_wma_returns_na_when_insufficient_data() {
        let registry = test_registry();

        // Only 2 values but requesting WMA of length 3
        let data = series(vec![10.0, 20.0]);
        let result = registry.dispatch("ta.wma", &[data, Value::Int(3)]);

        assert_eq!(result, Some(Value::Na));
    }

    #[test]
    fn test_wma_single_value() {
        let registry = test_registry();

        // WMA of single value is that value itself
        let data = series(vec![42.0]);
        let result = registry.dispatch("ta.wma", &[data, Value::Int(1)]);

        assert_eq!(result, Some(Value::Float(42.0)));
    }

    #[test]
    fn test_rma_uses_wilder_smoothing() {
        let registry = test_registry();

        // RMA with Wilder smoothing: alpha = 1/length
        // For length=3, alpha = 1/3 = 0.333...
        // Seed = (10+20+30)/3 = 20
        // EMA = 0.333*40 + 0.667*20 = 13.33 + 13.33 = 26.67
        let data = series(vec![10.0, 20.0, 30.0, 40.0]);
        let result = registry.dispatch("ta.rma", &[data, Value::Int(3)]);

        // RMA should be different from standard EMA
        // Standard EMA with alpha=2/(3+1)=0.5: 0.5*40 + 0.5*20 = 30
        // RMA with alpha=1/3=0.333: 0.333*40 + 0.667*20 = 26.67
        assert!(matches!(result, Some(Value::Float(v)) if (v - 26.666666666666668).abs() < 1e-10));
    }

    #[test]
    fn test_rma_returns_na_when_insufficient_data() {
        let registry = test_registry();

        // Only 2 values but requesting RMA of length 3
        let data = series(vec![10.0, 20.0]);
        let result = registry.dispatch("ta.rma", &[data, Value::Int(3)]);

        assert_eq!(result, Some(Value::Na));
    }

    #[test]
    fn test_rma_single_value() {
        let registry = test_registry();

        // RMA of single value is that value itself
        let data = series(vec![42.0]);
        let result = registry.dispatch("ta.rma", &[data, Value::Int(1)]);

        assert_eq!(result, Some(Value::Float(42.0)));
    }

    #[test]
    fn test_rma_produces_different_values_than_ema() {
        let registry = test_registry();

        // Same data, same length - RMA and EMA should produce different results
        let data = series(vec![10.0, 20.0, 30.0, 40.0, 50.0]);

        let rma_result = registry.dispatch("ta.rma", &[data.clone(), Value::Int(3)]);
        let ema_result = registry.dispatch("ta.ema", &[data, Value::Int(3)]);

        // Both should return valid floats
        let rma_val = match rma_result {
            Some(Value::Float(v)) => v,
            _ => panic!("RMA should return a float"),
        };
        let ema_val = match ema_result {
            Some(Value::Float(v)) => v,
            _ => panic!("EMA should return a float"),
        };

        // RMA and EMA should produce different values due to different alpha
        assert!(
            (rma_val - ema_val).abs() > 1e-10,
            "RMA ({}) should differ from EMA ({})",
            rma_val,
            ema_val
        );
    }
}
