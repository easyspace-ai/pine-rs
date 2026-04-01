//! Performance benchmarks for parallel execution
//!
//! These benchmarks measure the performance of:
//! - Parallel script execution
//! - Parallel data processing
//! - Series operations (SMA, sum, max, min)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use pine_eval::parallel::{execute_scripts_parallel, scan_symbols_parallel, parallel_series_map, parallel_series_reduce, ParallelConfig, ScriptTask};
use pine_runtime::series::SeriesBufF64;

fn generate_price_data(size: usize) -> Vec<f64> {
    // Generate synthetic price data (random walk)
    let mut prices = Vec::with_capacity(size);
    let mut price = 100.0;
    for i in 0..size {
        let change = ((i as f64 * 0.1).sin() * 2.0 - 1.0) * 0.02;
        price *= 1.0 + change;
        prices.push(price);
    }
    prices
}

fn bench_parallel_script_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_scripts");

    for num_scripts in [10, 50, 100].iter() {
        let tasks: Vec<ScriptTask> = (0..*num_scripts)
            .map(|i| ScriptTask {
                id: format!("script_{}", i),
                source: "plot(close)".to_string(),
                symbol: format!("SYM{}", i),
                timeframe: "1h".to_string(),
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("sequential", num_scripts),
            num_scripts,
            |b, _| {
                b.iter(|| {
                    black_box(execute_scripts_parallel(
                        tasks.clone(),
                        &ParallelConfig::disabled(),
                    ))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", num_scripts),
            num_scripts,
            |b, _| {
                b.iter(|| {
                    black_box(execute_scripts_parallel(
                        tasks.clone(),
                        &ParallelConfig::default(),
                    ))
                });
            },
        );
    }

    group.finish();
}

fn bench_symbol_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbol_scan");

    for num_symbols in [10, 50, 100].iter() {
        let symbols: Vec<String> = (0..*num_symbols)
            .map(|i| format!("SYM{}", i))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("sequential", num_symbols),
            num_symbols,
            |b, _| {
                b.iter(|| {
                    black_box(scan_symbols_parallel(
                        "plot(close)".to_string(),
                        symbols.clone(),
                        "1h".to_string(),
                        &ParallelConfig::disabled(),
                    ))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", num_symbols),
            num_symbols,
            |b, _| {
                b.iter(|| {
                    black_box(scan_symbols_parallel(
                        "plot(close)".to_string(),
                        symbols.clone(),
                        "1h".to_string(),
                        &ParallelConfig::default(),
                    ))
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_series_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_series_map");

    for data_size in [10000, 50000, 100000].iter() {
        let data: Vec<f64> = generate_price_data(*data_size);

        group.throughput(Throughput::Elements(*data_size as u64));

        group.bench_with_input(
            BenchmarkId::new("sequential", data_size),
            data_size,
            |b, _| {
                b.iter(|| {
                    let result: Vec<f64> = data.iter().map(|x| x * 2.0).collect();
                    black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", data_size),
            data_size,
            |b, _| {
                b.iter(|| {
                    let result: Vec<f64> = parallel_series_map(&data, |x| x * 2.0);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_series_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_series_reduce");

    for data_size in [10000, 50000, 100000].iter() {
        let data: Vec<f64> = generate_price_data(*data_size);

        group.throughput(Throughput::Elements(*data_size as u64));

        group.bench_with_input(
            BenchmarkId::new("sequential", data_size),
            data_size,
            |b, _| {
                b.iter(|| {
                    let sum: f64 = data.iter().sum();
                    black_box(sum);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", data_size),
            data_size,
            |b, _| {
                b.iter(|| {
                    let sum = parallel_series_reduce(&data, 0.0, |a, b| a + b);
                    black_box(sum);
                });
            },
        );
    }

    group.finish();
}

fn bench_series_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_ops");

    for data_size in [10000, 50000, 100000].iter() {
        let data = generate_price_data(*data_size);
        let mut series = SeriesBufF64::new(*data_size);
        series.extend(&data);

        group.throughput(Throughput::Elements(*data_size as u64));

        group.bench_with_input(BenchmarkId::new("sma_14", data_size), data_size, |b, _| {
            b.iter(|| {
                black_box(series.sma(black_box(14)));
            });
        });

        group.bench_with_input(BenchmarkId::new("sma_50", data_size), data_size, |b, _| {
            b.iter(|| {
                black_box(series.sma(black_box(50)));
            });
        });

        group.bench_with_input(BenchmarkId::new("sum", data_size), data_size, |b, _| {
            b.iter(|| {
                black_box(series.sum(black_box(*data_size)));
            });
        });

        group.bench_with_input(BenchmarkId::new("max", data_size), data_size, |b, _| {
            b.iter(|| {
                black_box(series.max(black_box(*data_size)));
            });
        });

        group.bench_with_input(BenchmarkId::new("min", data_size), data_size, |b, _| {
            b.iter(|| {
                black_box(series.min(black_box(*data_size)));
            });
        });
    }

    group.finish();
}

fn bench_combined_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_indicators");

    for data_size in [10000, 50000, 100000].iter() {
        let data = generate_price_data(*data_size);
        let mut close = SeriesBufF64::new(*data_size);
        close.extend(&data);

        group.throughput(Throughput::Elements(*data_size as u64));

        group.bench_with_input(
            BenchmarkId::new("sma_rsi_like", data_size),
            data_size,
            |b, _| {
                b.iter(|| {
                    // Multiple SMA calculations (simulating complex indicator)
                    let sma_fast = close.sma(12);
                    let sma_slow = close.sma(26);
                    let sma_signal = close.sma(9);

                    // Simulated RSI-like calculation using price changes
                    let mut gains = 0.0;
                    let mut losses = 0.0;
                    for i in 0..14.min(close.len()) {
                        let current = close.get(i).unwrap_or(0.0);
                        let prev = close.get(i + 1).unwrap_or(current);
                        let change = current - prev;
                        if change > 0.0 {
                            gains += change;
                        } else {
                            losses += change.abs();
                        }
                    }
                    let avg_gain = gains / 14.0;
                    let avg_loss = losses / 14.0;
                    let rs = if avg_loss == 0.0 { 0.0 } else { avg_gain / avg_loss };
                    let _rsi = 100.0 - (100.0 / (1.0 + rs));

                    black_box((sma_fast, sma_slow, sma_signal, _rsi));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parallel_script_execution,
    bench_symbol_scanning,
    bench_parallel_series_map,
    bench_parallel_series_reduce,
    bench_series_operations,
    bench_combined_indicators
);
criterion_main!(benches);
