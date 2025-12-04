use day_01_part_02::solve;

fn main() {
    let input = std::fs::read_to_string("puzzles/day-01-part-02/input.txt").unwrap();

    let instant = std::time::Instant::now();
    let nr_of_revolutions_over_zero = solve(&input);
    let elapsed = instant.elapsed();

    println!(
        "nr_of_revolutions_over_zero: {}",
        nr_of_revolutions_over_zero
    );

    println!("Elapsed: {:.2?}", elapsed);
}
