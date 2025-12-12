use day_06_part_01::solve;

fn main() {
    let input = std::fs::read_to_string("puzzles/day-06-part-01/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = solve(input.as_bytes());
    let elapsed = timer.elapsed();
    println!("Elapsed time: {:?}", elapsed);
    assert_eq!(result, 6605396225322);
}
