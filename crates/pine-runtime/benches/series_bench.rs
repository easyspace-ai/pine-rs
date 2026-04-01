use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pine_runtime::series::SeriesBufF64;

fn generate_test_data(size: usize) -> Vec<f64> {
    (0..size).map(|i| (i as f64).sin() * 100.0 + 50.0).collect()
}

fn bench_series_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_push");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("push", size), size, |b, &size| {
            b.iter(|| {
                let mut series = SeriesBufF64::new(size);
                for i in 0..size {
                    series.push(black_box(i as f64));
                }
                series
            });
        });
    }

    group.finish();
}

fn bench_series_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_sma");

    for (data_size, window) in [(1000, 14), (10000, 14), (100000, 14), (100000, 50)].iter() {
        let data = generate_test_data(*data_size);
        let mut series = SeriesBufF64::new(*data_size);
        series.extend(&data);

        group.bench_with_input(
            BenchmarkId::new(format!("sma_{}", window), data_size),
            &(*data_size, *window),
            |b, (_, window)| {
                b.iter(|| {
                    black_box(series.sma(black_box(*window)));
                });
            },
        );
    }

    group.finish();
}

fn bench_series_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_sum");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);
        let mut series = SeriesBufF64::new(*size);
        series.extend(&data);

        group.bench_with_input(BenchmarkId::new("sum", size), size, |b, _| {
            b.iter(|| {
                black_box(series.sum(black_box(*size)));
            });
        });
    }

    group.finish();
}

fn bench_series_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_max");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);
        let mut series = SeriesBufF64::new(*size);
        series.extend(&data);

        group.bench_with_input(BenchmarkId::new("max", size), size, |b, _| {
            b.iter(|| {
                black_box(series.max(black_box(*size)));
            });
        });
    }

    group.finish();
}

fn bench_series_min(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_min");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);
        let mut series = SeriesBufF64::new(*size);
        series.extend(&data);

        group.bench_with_input(BenchmarkId::new("min", size), size, |b, _| {
            b.iter(|| {
                black_box(series.min(black_box(*size)));
            });
        });
    }

    group.finish();
}

fn bench_series_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_get");

    for size in [100, 1000, 10000].iter() {
        let data = generate_test_data(*size);
        let mut series = SeriesBufF64::new(*size);
        series.extend(&data);

        group.bench_with_input(BenchmarkId::new("get_sequential", size), size, |b, _| {
            b.iter(|| {
                for i in 0..100 {
                    black_box(series.get(black_box(i)));
                }
            });
        });
    }

    group.finish();
}

fn bench_series_extend(c: &mut Criterion) {
    let mut group = c.benchmark_group("series_extend");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("extend", size), size, |b, _| {
            b.iter(|| {
                let mut series = SeriesBufF64::new(*size);
                series.extend(black_box(&data));
                series
            });
        });
    }

    group.finish();
}

fn bench_sma_fast_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("sma_fast_path");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);
        let mut series = SeriesBufF64::new(*size);
        // Fill to capacity for fast path
        series.extend(&data);

        group.bench_with_input(BenchmarkId::new("sma_fast", size), size, |b, _| {
            b.iter(|| {
                black_box(series.sma_fast(black_box(14)));
            });
        });

        group.bench_with_input(BenchmarkId::new("sma_regular", size), size, |b, _| {
            b.iter(|| {
                black_box(series.sma(black_box(14)));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_series_push,
    bench_series_sma,
    bench_series_sum,
    bench_series_max,
    bench_series_min,
    bench_series_get,
    bench_series_extend,
    bench_sma_fast_path
);
criterion_main!(benches);
