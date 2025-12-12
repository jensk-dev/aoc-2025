use day_05_part_01::solve;

fn main() {
    let input = std::fs::read_to_string("puzzles/day-05-part-01/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = solve(&input);
    let elapsed = timer.elapsed();
    println!("Elapsed time: {:?}", elapsed);
    assert_eq!(758, result);
}
