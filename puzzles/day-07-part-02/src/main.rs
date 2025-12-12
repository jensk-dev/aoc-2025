use day_07_part_02::solve;
use mimalloc::MiMalloc;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

fn main() {
    let input = std::fs::read_to_string("puzzles/day-07-part-02/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = solve(&input);
    let elapsed = timer.elapsed();

    println!("Elapsed time: {:?}", elapsed);
    assert_eq!(result, 23607984027985);
}
