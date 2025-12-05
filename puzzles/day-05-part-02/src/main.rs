use day_05_part_02::solve;

fn main() {
    let input = std::fs::read_to_string("puzzles/day-05-part-02/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = solve(&input);
    let elapsed = timer.elapsed();
    println!("Elapsed time: {:?}", elapsed);
    assert_eq!(343143696885053, result);
}
