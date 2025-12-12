use std::fs::read_to_string;

use criterion::{Criterion, criterion_group, criterion_main};
use day_05_part_02::solve;

fn bench_solve(c: &mut Criterion) {
    let input = read_to_string("input.txt").unwrap();
    c.bench_function("solve", |b| b.iter(|| solve(std::hint::black_box(&input))));
    let answer = solve(&input);
    assert_eq!(answer, 343143696885053);
}

criterion_group!(benches, bench_solve);
criterion_main!(benches);
