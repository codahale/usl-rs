use criterion::{criterion_group, criterion_main, Criterion};

use usl::{Measurement, Model};

fn build(c: &mut Criterion) {
    let measurements: Vec<Measurement> = MEASUREMENTS.iter().map(|&v| v.into()).collect();
    c.bench_function("build", |b| b.iter(|| Model::build(&measurements)));
}

const MEASUREMENTS: [(u32, f64); 32] = [
    (1, 955.16),
    (2, 1878.91),
    (3, 2688.01),
    (4, 3548.68),
    (5, 4315.54),
    (6, 5130.43),
    (7, 5931.37),
    (8, 6531.08),
    (9, 7219.8),
    (10, 7867.61),
    (11, 8278.71),
    (12, 8646.7),
    (13, 9047.84),
    (14, 9426.55),
    (15, 9645.37),
    (16, 9897.24),
    (17, 10097.6),
    (18, 10240.5),
    (19, 10532.39),
    (20, 10798.52),
    (21, 11151.43),
    (22, 11518.63),
    (23, 11806.0),
    (24, 12089.37),
    (25, 12075.41),
    (26, 12177.29),
    (27, 12211.41),
    (28, 12158.93),
    (29, 12155.27),
    (30, 12118.04),
    (31, 12140.4),
    (32, 12074.39),
];

criterion_group!(benches, build);
criterion_main!(benches);
