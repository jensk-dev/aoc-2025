pub fn solve(input: &str) -> usize {
    let factory = parse_input(input.as_bytes());
    factory
        .machines
        .iter()
        .map(|machine| solve_machine(machine))
        .sum()
}

fn solve_machine(machine: &Machine) -> usize {
    let mut system = LinearSystem::from_machine(machine);
    system.reduce_to_echelon_form();
    system.minimum_weight_solution()
}

struct LinearSystem {
    rows: Vec<Row>,
    num_variables: usize,
    pivot_columns: BitSet,
}

impl LinearSystem {
    fn from_machine(machine: &Machine) -> Self {
        let num_variables = machine.button_masks.len();
        let num_equations = Self::count_equations(machine);

        let rows = (0..num_equations)
            .map(|eq| Row::for_equation(eq, machine, num_variables))
            .collect();

        Self {
            rows,
            num_variables,
            pivot_columns: BitSet::empty(),
        }
    }

    fn count_equations(machine: &Machine) -> usize {
        let all_bits = machine
            .button_masks
            .iter()
            .fold(machine.diagram_mask, |acc, &btn| acc | btn);

        if all_bits == 0 {
            0
        } else {
            16 - all_bits.leading_zeros() as usize
        }
    }

    fn reduce_to_echelon_form(&mut self) {
        let mut current_row = 0;

        for col in 0..self.num_variables {
            if let Some(pivot_row) = self.find_pivot_in_column(col, current_row) {
                self.rows.swap(current_row, pivot_row);
                self.eliminate_column(col, current_row);
                self.pivot_columns.insert(col);
                current_row += 1;
            }
        }

        self.assert_system_is_consistent(current_row);
    }

    fn find_pivot_in_column(&self, col: usize, start_row: usize) -> Option<usize> {
        (start_row..self.rows.len()).find(|&r| self.rows[r].has_variable(col))
    }

    fn eliminate_column(&mut self, col: usize, pivot_row: usize) {
        let pivot = self.rows[pivot_row];
        for r in 0..self.rows.len() {
            if r != pivot_row && self.rows[r].has_variable(col) {
                self.rows[r].xor_with(pivot);
            }
        }
    }

    fn assert_system_is_consistent(&self, num_pivots: usize) {
        for row in &self.rows[num_pivots..] {
            if row.is_inconsistent(self.num_variables) {
                panic!("No solution exists for machine");
            }
        }
    }

    fn minimum_weight_solution(&self) -> usize {
        let free_variables = self.pivot_columns.complement(self.num_variables);
        let num_pivots = self.pivot_columns.len();

        let mut min_presses = u32::MAX;

        for assignment in free_variables.all_assignments() {
            let solution = self.back_substitute(assignment, num_pivots);
            min_presses = min_presses.min(solution.count());
        }

        min_presses as usize
    }

    fn back_substitute(&self, free_assignment: BitSet, num_pivots: usize) -> BitSet {
        let mut solution = free_assignment;

        for row_idx in (0..num_pivots).rev() {
            let row = self.rows[row_idx];
            let pivot_col = row.leading_variable();
            let value = row.evaluate_rhs(solution, self.num_variables);

            if value {
                solution.insert(pivot_col);
            }
        }

        solution
    }
}

#[derive(Clone, Copy)]
struct Row(u32);

impl Row {
    fn for_equation(equation_index: usize, machine: &Machine, num_variables: usize) -> Self {
        let mut bits = 0u32;

        for (var_idx, &button_mask) in machine.button_masks.iter().enumerate() {
            if (button_mask >> equation_index) & 1 == 1 {
                bits |= 1 << var_idx;
            }
        }

        if (machine.diagram_mask >> equation_index) & 1 == 1 {
            bits |= 1 << num_variables;
        }

        Self(bits)
    }

    fn has_variable(self, col: usize) -> bool {
        (self.0 >> col) & 1 == 1
    }

    fn xor_with(&mut self, other: Row) {
        self.0 ^= other.0;
    }

    fn is_inconsistent(self, num_variables: usize) -> bool {
        self.0 == (1 << num_variables)
    }

    fn leading_variable(self) -> usize {
        self.0.trailing_zeros() as usize
    }

    fn evaluate_rhs(self, solution: BitSet, num_variables: usize) -> bool {
        let target_bit = (self.0 >> num_variables) & 1;
        let dot_product_parity = (self.0 & solution.0).count_ones() & 1;
        (target_bit ^ dot_product_parity) == 1
    }
}

#[derive(Clone, Copy)]
struct BitSet(u32);

impl BitSet {
    fn empty() -> Self {
        Self(0)
    }

    fn insert(&mut self, index: usize) {
        self.0 |= 1 << index;
    }

    fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    fn count(self) -> u32 {
        self.0.count_ones()
    }

    fn complement(self, universe_size: usize) -> Self {
        let universe = (1u32 << universe_size) - 1;
        Self(universe & !self.0)
    }

    fn all_assignments(self) -> impl Iterator<Item = BitSet> {
        let mask = self.0;
        let num_bits = mask.count_ones();
        (0u32..(1 << num_bits)).map(move |i| Self::from_dense(i, mask))
    }

    fn from_dense(dense: u32, positions: u32) -> Self {
        let mut result = 0u32;
        let mut src_bit = 0;
        let mut remaining_positions = positions;

        while remaining_positions != 0 {
            let target_position = remaining_positions.trailing_zeros();

            if (dense >> src_bit) & 1 == 1 {
                result |= 1 << target_position;
            }

            remaining_positions &= remaining_positions - 1;
            src_bit += 1;
        }

        Self(result)
    }
}

fn parse_input(input: &[u8]) -> Factory {
    let machines = input
        .split(|&c| c == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| parse_machine(line))
        .collect();

    Factory { machines }
}

fn parse_machine(line: &[u8]) -> Machine {
    let (mask, mut current_index) = parse_diagram(&line, 1);
    let mut buttons = vec![];

    loop {
        let byte = line[current_index];
        
        if byte == b'(' {
            let (button_mask, new_index) = parse_button(&line, current_index + 1);
            buttons.push(button_mask);
            current_index = new_index;
        }

        if byte == b'{' {
            return Machine {
                diagram_mask: mask,
                button_masks: buttons,
            }
        }

        current_index += 1;
    }
}

fn parse_button(input: &[u8], index: usize) -> (u16, usize) {
    let mut mask = 0u16;

    for i in index..input.len() {
        let b = input[i];
        if b == b')' {
            return (mask, i + 1);
        }

        if b == b',' {
            continue;
        }

        mask |= 1 << (b - b'0');
    }
    
    panic!("Unterminated button");
}

fn parse_diagram(input: &[u8], index: usize) -> (u16, usize) {
    let mut mask = 0u16;

    for i in index..input.len() {
        let b = input[i];
        if b == b']' {
            return (mask, i + 1);
        }

        if b == b'.' {
            continue;
        }

        mask |= 1 << (i - index);
    }
    
    panic!("Unterminated diagram");
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Machine {
    diagram_mask: u16,
    button_masks: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Factory {
    machines: Vec<Machine>,
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use crate::Machine;

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
        assert_eq!(crate::solve(EXAMPLE_INPUT), 7);
    }

    #[rstest]
    #[case(EXAMPLE_LINE_1, 2)]
    #[case(EXAMPLE_LINE_2, 3)]
    #[case(EXAMPLE_LINE_3, 2)]
    fn test_solve_machine(#[case] line: &str, #[case] expected: usize) {
        let machine = crate::parse_machine(line.as_bytes());
        assert_eq!(crate::solve_machine(&machine), expected);
    }

    #[test]
    fn test_parse_input() {
        let factory = crate::parse_input(EXAMPLE_INPUT.as_bytes());
        assert_eq!(factory.machines.len(), 3);
    }

    #[rstest]
    #[case(
        EXAMPLE_LINE_1,
        Machine {
            diagram_mask: 1 << 1 | 1 << 2,
            button_masks: vec![
                1 << 3,
                1 << 1 | 1 << 3,
                1 << 2,
                1 << 2 | 1 << 3,
                1 << 0 | 1 << 2,
                1 << 0 | 1 << 1
            ]
        }
    )]
    #[case(
        EXAMPLE_LINE_2,
        Machine {
            diagram_mask: 1 << 3,
            button_masks: vec![
                1 << 0 | 1 << 2 | 1 << 3 | 1 << 4,
                1 << 2 | 1 << 3,
                1 << 0 | 1 << 4,
                1 << 0 | 1 << 1 | 1 << 2,
                1 << 1 | 1 << 2 | 1 << 3 | 1 << 4,
            ]
        }
    )]
    #[case(
        EXAMPLE_LINE_3,
        Machine {
            diagram_mask: 1 << 1 | 1 << 2 | 1 << 3 | 1 << 5,
            button_masks: vec![
                1 << 0 | 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4,
                1 << 0 | 1 << 3 | 1 << 4,
                1 << 0 | 1 << 1 | 1 << 2 | 1 << 4 | 1 << 5,
                1 << 1 | 1 << 2,
            ]
        }
    )]
    fn parse_machine(#[case] line: &str, #[case] expected: Machine) {
        let parsed = crate::parse_machine(line.as_bytes());
        assert_eq!(parsed, expected);
    }

    #[rstest]
    #[case(b"[.##.]", 1 << 1 | 1 << 2)]
    #[case(b"[...#.]", 1 << 3)]
    #[case(b"[.###.#]", 1 << 1 | 1 << 2 | 1 << 3 | 1 << 5)]
    fn parse_diagram(#[case] input: &[u8], #[case] expected_mask: u16) {
        let (parsed_mask, idx) = crate::parse_diagram(&input, 1);
        assert_eq!(parsed_mask, expected_mask);
        assert_eq!(idx, input.len());
    }

    #[rstest]
    #[case(b"(3)", 1 << 3)]
    #[case(b"(1,3)", 1 << 1 | 1 << 3)]
    #[case(b"(0,1,2,3,4,5)", 1 << 0 | 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4 | 1 << 5)]
    fn parse_button(#[case] input: &[u8], #[case] expected_mask: u16) {
        let (parsed_mask, idx) = crate::parse_button(&input, 1);
        assert_eq!(parsed_mask, expected_mask);
        assert_eq!(idx, input.len());
    }
}
