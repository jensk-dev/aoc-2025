use criterion::{Criterion, criterion_group, criterion_main};
use day_02_part_02::solve;

fn bench_solve(c: &mut Criterion) {
    let input = include_str!("../input.txt").lines().next().unwrap();

    c.bench_function("solve", |b| b.iter(|| solve(std::hint::black_box(input))));
}

criterion_group!(benches, bench_solve);
criterion_main!(benches);
