use criterion::{Criterion, criterion_group, criterion_main};
use day_01_part_02::solve;

fn bench_solve(c: &mut Criterion) {
    let input = std::fs::read_to_string("input.txt").unwrap();

    c.bench_function("solve", |b| b.iter(|| solve(std::hint::black_box(&input))));

    let answer = solve(&input);
    assert_eq!(answer, 5831);
}

criterion_group!(benches, bench_solve);
criterion_main!(benches);
