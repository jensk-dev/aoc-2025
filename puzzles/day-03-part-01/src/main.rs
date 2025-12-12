use std::{fs::read_to_string, time::Instant};

use day_03_part_01::solve;

fn main() {
    let working_dir = std::env::current_dir().unwrap();
    let path = format!("{}/puzzles/day-03-part-01/input.txt", working_dir.display());
    let input = read_to_string(path).unwrap();

    let start = Instant::now();
    let sum = solve(&input);
    let duration = start.elapsed();

    println!(
        "Sum of optimal batteries: {}. Elapsed: {}",
        sum,
        duration.as_micros()
    );
}
