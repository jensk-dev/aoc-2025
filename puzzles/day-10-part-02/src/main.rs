fn main() {
    let input = std::fs::read_to_string("puzzles/day-10-part-02/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = day_10_part_02::solve(&input);
    let elapsed = timer.elapsed();

    println!("Elapsed time: {:?}", elapsed);
    println!("Result: {}", result);
}
