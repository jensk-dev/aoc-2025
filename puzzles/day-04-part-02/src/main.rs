use day_04_part_02::solve;

pub fn main() {
    let input = std::fs::read_to_string("puzzles/day-04-part-02/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = solve(&input);
    let elapsed = timer.elapsed();
    println!("Elapsed time: {:?}", elapsed);
    assert_eq!(8317, result);
}
