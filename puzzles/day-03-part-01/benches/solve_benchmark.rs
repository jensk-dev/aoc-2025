use std::fs::read_to_string;

use criterion::{Criterion, criterion_group, criterion_main};
use day_03_part_01::solve;

fn bench_solve(c: &mut Criterion) {
    let input = read_to_string("./input.txt").unwrap();
    c.bench_function("solve", |b| b.iter(|| solve(std::hint::black_box(&input))));
}

criterion_group!(benches, bench_solve);
criterion_main!(benches);
