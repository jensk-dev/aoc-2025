use good_lp::{constraint, variable, Expression, ProblemVariables, Solution, SolverModel};
use rayon::prelude::*;

pub fn solve(input: &str) -> usize {
    parse_input(input.as_bytes())
        .par_iter()
        .map(|machine| solve_machine(machine))
        .sum()
}

fn solve_machine(machine: &Machine) -> usize {
    let num_buttons = machine.buttons.len();
    let num_counters = machine.targets.len();

    if num_counters == 0 {
        return 0;
    }

    let mut vars = ProblemVariables::new();

    let buttons: Vec<_> = (0..num_buttons)
        .map(|_| vars.add(variable().integer().min(0)))
        .collect();

    let objective: Expression = buttons.iter().copied().sum();

    let mut problem = vars.minimise(objective).using(good_lp::microlp);

    for (counter_idx, &target) in machine.targets.iter().enumerate() {
        let mut lhs = Expression::default();
        for (button_idx, button_counters) in machine.buttons.iter().enumerate() {
            if button_counters.contains(&counter_idx) {
                lhs += buttons[button_idx];
            }
        }
        problem = problem.with(constraint!(lhs == target as f64));
    }

    match problem.solve() {
        Ok(solution) => {
            let total: f64 = buttons.iter().map(|&b| solution.value(b)).sum();
            total.round() as usize
        }
        Err(_) => usize::MAX,
    }
}

struct Machine {
    buttons: Vec<Vec<usize>>,
    targets: Vec<i64>,
}

fn parse_input(input: &[u8]) -> Vec<Machine> {
    input
        .split(|&c| c == b'\n')
        .filter(|line| !line.is_empty())
        .map(parse_machine)
        .collect()
}

fn parse_machine(line: &[u8]) -> Machine {
    let mut buttons = Vec::new();
    let mut targets = Vec::new();
    let mut i = 0;

    // Skip diagram [...]
    while line[i] != b']' {
        i += 1;
    }
    i += 1;

    while i < line.len() {
        match line[i] {
            b'(' => {
                i += 1;
                let mut counters = Vec::new();
                while line[i] != b')' {
                    if line[i].is_ascii_digit() {
                        counters.push((line[i] - b'0') as usize);
                    }
                    i += 1;
                }
                buttons.push(counters);
                i += 1;
            }
            b'{' => {
                i += 1;
                let mut num = 0i64;
                let mut in_num = false;
                while line[i] != b'}' {
                    if line[i].is_ascii_digit() {
                        num = num * 10 + (line[i] - b'0') as i64;
                        in_num = true;
                    } else if line[i] == b',' && in_num {
                        targets.push(num);
                        num = 0;
                        in_num = false;
                    }
                    i += 1;
                }
                if in_num {
                    targets.push(num);
                }
                break;
            }
            _ => i += 1,
        }
    }

    Machine { buttons, targets }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    const EXAMPLE_LINE_1: &str = r#"[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}"#;
    const EXAMPLE_LINE_2: &str = r#"[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}"#;
    const EXAMPLE_LINE_3: &str = r#"[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}"#;

    const EXAMPLE_INPUT: &str = indoc! {r#"
        [.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
        [...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
        [.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}
    "#};

    #[test]
    fn test_solve() {
        assert_eq!(crate::solve(EXAMPLE_INPUT), 33);
    }

    #[rstest]
    #[case(EXAMPLE_LINE_1, 10)]
    #[case(EXAMPLE_LINE_2, 12)]
    #[case(EXAMPLE_LINE_3, 11)]
    fn test_solve_machine(#[case] line: &str, #[case] expected: usize) {
        let machine = crate::parse_machine(line.as_bytes());
        assert_eq!(crate::solve_machine(&machine), expected);
    }

    #[test]
    fn test_parse_input() {
        let machines = crate::parse_input(EXAMPLE_INPUT.as_bytes());
        assert_eq!(machines.len(), 3);
    }
}
