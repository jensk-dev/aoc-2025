fn main() {
    let input = std::fs::read_to_string("puzzles/day-11-part-01/input.txt").unwrap();
    let timer = std::time::Instant::now();
    let result = day_11_part_01::solve(&input);
    let elapsed = timer.elapsed();

    println!("Elapsed time: {:?}", elapsed);
    assert_eq!(result, 472);
}
