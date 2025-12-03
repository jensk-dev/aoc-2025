use std::{fs::File, io::BufRead, time::Instant};

use day_02_part_02::solve;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    let working_dir = std::env::current_dir().unwrap();
    let path = format!("{}/puzzles/day-02-part-02/input.txt", working_dir.display());
    let f = File::open(path).unwrap();
    let f = std::io::BufReader::new(f);
    let input = f.lines().next().unwrap().unwrap();

    let start = Instant::now();
    let sum = solve(&input);
    let duration = start.elapsed();

    println!("sum of sequences: {}, took {}", sum, duration.as_millis());
}
