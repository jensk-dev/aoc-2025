use day_04_part_01::solve;

pub fn main() {
    let input = std::fs::read_to_string("puzzles/day-04-part-01/input.txt").unwrap();
    assert_eq!(1445, solve(&input));
}
