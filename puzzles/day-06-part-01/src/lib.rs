pub fn solve(input: &[u8]) -> u64 {
    let matrix = CephalopodMathWorksheet::from_slice(input);

    (0..matrix.columns)
        .map(|c| matrix.apply_operator_to_column(c as usize))
        .sum()
}

#[derive(Debug)]
struct CephalopodMathWorksheet {
    flattened: Vec<u64>,
    columns: u64,
    rows: u64,
    operators: Vec<Operator>,
}

#[derive(Debug)]
enum Operator {
    Add,
    Multiply,
}

impl CephalopodMathWorksheet {
    fn from_slice(input: &[u8]) -> Self {
        let width = input.iter().position(|&c| c == b'\n').unwrap();
        let stride = width + 1; // including newline
        let cols = unsafe { std::str::from_utf8_unchecked(&input[..width]) }
            .split_ascii_whitespace()
            .count();
        let rows = input.len() / stride;

        let mut flattened: Vec<u64> = Vec::with_capacity(rows * cols);
        unsafe {
            flattened.set_len(rows * cols);
        }

        // iterate over every byte, and incrementally parse numbers
        let mut row = 0;
        let mut col = 0;
        let mut current_number: u64 = 0;

        // pak alles tot de laatste kolom, de laatste kolom bevat operators, dus die moeten
        // anders opgeslagen worden
        for (i, &byte) in input.iter().enumerate().take(input.len() - stride) {
            match byte {
                b'0'..=b'9' => {
                    current_number = current_number * 10 + (byte - b'0') as u64;
                }
                b' ' | b'\n' => {
                    if i > 0 && input[i - 1] != b' ' && input[i - 1] != b'\n' {
                        let index = col * rows + row;
                        unsafe {
                            *flattened.get_unchecked_mut(index) = current_number;
                        }
                        current_number = 0;
                        col += 1;
                    }

                    if byte == b'\n' {
                        row += 1;
                        col = 0;
                    }
                }
                b => panic!("Ongeldige input: {:?}", b),
            }
        }

        // the last row should be its own vector parsed to operators
        let mut operators_per_column: Vec<Operator> = Vec::with_capacity(cols);
        unsafe {
            operators_per_column.set_len(cols);
        }
        let start_of_last_row = (rows - 1) * stride;
        let mut current_col = 0;
        for &byte in input[start_of_last_row..].iter() {
            match byte {
                b'+' => {
                    unsafe {
                        *operators_per_column.get_unchecked_mut(current_col) = Operator::Add;
                    }
                    current_col += 1;
                }
                b'*' => {
                    unsafe {
                        *operators_per_column.get_unchecked_mut(current_col) = Operator::Multiply;
                    }
                    current_col += 1;
                }
                b' ' | b'\n' => {}
                b => panic!("Ongeldige operator: {:?}", b),
            }
        }

        CephalopodMathWorksheet {
            flattened,
            columns: cols as u64,
            rows: rows as u64,
            operators: operators_per_column,
        }
    }

    #[inline]
    pub fn get_column(&self, col: usize) -> &[u64] {
        let start = col * self.rows as usize;
        let end = start + self.rows as usize - 1;
        unsafe { self.flattened.get_unchecked(start..end) }
    }

    #[inline]
    pub fn get_column_operator(&self, col: usize) -> &Operator {
        unsafe { self.operators.get_unchecked(col) }
    }

    #[inline]
    pub fn apply_operator_to_column(&self, col: usize) -> u64 {
        let column = self.get_column(col);
        let operator = self.get_column_operator(col);

        match operator {
            Operator::Add => column.iter().sum(),
            Operator::Multiply => column.iter().product(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    #[test]
    fn solve_example() {
        let input = read_to_string("example.txt").unwrap();
        let result = super::solve(input.as_bytes());

        assert_eq!(result, 4277556);
    }

    #[test]
    fn get_columns() {
        let input = read_to_string("example.txt").unwrap();
        let worksheet = super::CephalopodMathWorksheet::from_slice(input.as_bytes());

        assert_eq!(worksheet.get_column(0), &[123, 45, 6]);
        assert_eq!(worksheet.get_column(1), &[328, 64, 98]);
        assert_eq!(worksheet.get_column(2), &[51, 387, 215]);
        assert_eq!(worksheet.get_column(3), &[64, 23, 314]);
    }

    #[test]
    fn get_column_operator() {
        let input = read_to_string("example.txt").unwrap();
        let worksheet = super::CephalopodMathWorksheet::from_slice(input.as_bytes());

        use super::Operator::*;

        assert!(matches!(worksheet.get_column_operator(0), Multiply));
        assert!(matches!(worksheet.get_column_operator(1), Add));
        assert!(matches!(worksheet.get_column_operator(2), Multiply));
        assert!(matches!(worksheet.get_column_operator(3), Add));
    }
}
