use std::{fs::read_to_string, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use day_10_part_01::solve;

fn bench_solve(c: &mut Criterion) {
    let input = read_to_string("input.txt").unwrap();
    assert_eq!(solve(&input), 452);
    c.bench_function("solve", |b| b.iter(|| solve(std::hint::black_box(&input))));
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(500)
        .measurement_time(Duration::from_secs(60));
    targets = bench_solve
}
criterion_main!(benches);
