use criterion::{criterion_group, criterion_main, Criterion};

use usl::{Measurement, Model};

fn build(c: &mut Criterion) {
    let measurements: Vec<Measurement> =
        MEASUREMENTS.iter().map(|&(n, x)| Measurement::concurrency_and_throughput(n, x)).collect();
    c.bench_function("build", |b| b.iter(|| Model::build(&measurements)));
}

const MEASUREMENTS: [(f64, f64); 32] = [
    (1.0, 955.16),
    (2.0, 1878.91),
    (3.0, 2688.01),
    (4.0, 3548.68),
    (5.0, 4315.54),
    (6.0, 5130.43),
    (7.0, 5931.37),
    (8.0, 6531.08),
    (9.0, 7219.8),
    (10.0, 7867.61),
    (11.0, 8278.71),
    (12.0, 8646.7),
    (13.0, 9047.84),
    (14.0, 9426.55),
    (15.0, 9645.37),
    (16.0, 9897.24),
    (17.0, 10097.6),
    (18.0, 10240.5),
    (19.0, 10532.39),
    (20.0, 10798.52),
    (21.0, 11151.43),
    (22.0, 11518.63),
    (23.0, 11806.0),
    (24.0, 12089.37),
    (25.0, 12075.41),
    (26.0, 12177.29),
    (27.0, 12211.41),
    (28.0, 12158.93),
    (29.0, 12155.27),
    (30.0, 12118.04),
    (31.0, 12140.4),
    (32.0, 12074.39),
];

criterion_group!(benches, build);
criterion_main!(benches);
