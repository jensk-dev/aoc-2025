mod bytes {
    pub const NEWLINE: u8 = b'\n';
    pub const SPACE: u8 = b' ';
    pub const PLUS: u8 = b'+';
    pub const STAR: u8 = b'*';
    pub const ZERO: u8 = b'0';
}

pub fn solve(input: &[u8]) -> u64 {
    let worksheet = CephalopodMathWorksheet::from_slice(input);
    worksheet.sum()
}

#[derive(Debug, Clone, Copy)]
enum OpKind {
    Add,
    Multiply,
}

#[derive(Debug, Clone, Copy)]
struct Operator {
    kind: OpKind,
    width: u32,
    offset: u32,
}

impl Operator {
    #[inline]
    const fn new(kind: OpKind, width: u32, offset: u32) -> Self {
        Self {
            kind,
            width,
            offset,
        }
    }

    #[inline]
    const fn identity(&self) -> u64 {
        match self.kind {
            OpKind::Add => 0,
            OpKind::Multiply => 1,
        }
    }

    #[inline]
    fn apply(&self, acc: u64, val: u64) -> u64 {
        match self.kind {
            OpKind::Add => acc + val,
            OpKind::Multiply => acc * val,
        }
    }
}

#[derive(Debug)]
struct CephalopodMathWorksheet<'a> {
    data: &'a [u8],
    height_in_bytes: usize,
    stride_in_bytes: usize,
    operators: Vec<Operator>,
}

impl<'a> CephalopodMathWorksheet<'a> {
    fn from_slice(input: &'a [u8]) -> Self {
        let width_in_bytes = input.iter().position(|&c| c == bytes::NEWLINE).unwrap();
        let stride_in_bytes = width_in_bytes + 1;
        let height_in_bytes = input.len() / stride_in_bytes;
        let last_line = &input[input.len() - stride_in_bytes..];
        let operator_columns_count = unsafe { std::str::from_utf8_unchecked(last_line) }
            .split_ascii_whitespace()
            .count();

        let mut operators: Vec<Operator> = unsafe {
            let mut vec = Vec::with_capacity(operator_columns_count);
            vec.set_len(operator_columns_count);
            vec
        };

        let mut operator_index = 0;
        let mut cumulative_offset = 1;
        let mut column_width = 0;

        for &byte in last_line.iter().rev() {
            column_width += 1;

            let kind = match byte {
                bytes::PLUS => OpKind::Add,
                bytes::STAR => OpKind::Multiply,
                _ => continue,
            };

            let op = Operator::new(kind, column_width - 1, cumulative_offset);
            operators[operator_columns_count - 1 - operator_index] = op;
            operator_index += 1;
            cumulative_offset += column_width;
            column_width = 0;
        }

        Self {
            data: input,
            height_in_bytes,
            stride_in_bytes,
            operators,
        }
    }

    #[inline]
    fn byte_at(&self, row: usize, offset: u32, byte_pos: u32) -> u8 {
        let idx = (row + 1) * self.stride_in_bytes - 1 - offset as usize - byte_pos as usize;
        self.data[idx]
    }

    #[inline]
    fn read_transposed_number(&self, offset: u32, byte_pos: u32) -> u64 {
        let mut num = 0u64;
        for row in 0..(self.height_in_bytes - 1) {
            let byte = self.byte_at(row, offset, byte_pos);
            if byte != bytes::SPACE {
                num = num * 10 + (byte - bytes::ZERO) as u64;
            }
        }
        num
    }

    #[inline]
    pub fn sum(self) -> u64 {
        let mut final_sum = 0u64;

        for op in &self.operators {
            let mut column_result = op.identity();

            for byte_pos in 0..op.width {
                let transposed_num = self.read_transposed_number(op.offset, byte_pos);
                column_result = op.apply(column_result, transposed_num);
            }

            final_sum += column_result;
        }

        final_sum
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::{CephalopodMathWorksheet, OpKind, Operator, bytes};

    mod operator_tests {
        use super::*;

        #[test]
        fn add_identity_is_zero() {
            let op = Operator::new(OpKind::Add, 1, 1);
            assert_eq!(op.identity(), 0);
        }

        #[test]
        fn multiply_identity_is_one() {
            let op = Operator::new(OpKind::Multiply, 1, 1);
            assert_eq!(op.identity(), 1);
        }

        #[test]
        fn add_apply_sums_values() {
            let op = Operator::new(OpKind::Add, 1, 1);
            assert_eq!(op.apply(10, 5), 15);
            assert_eq!(op.apply(0, 42), 42);
            assert_eq!(op.apply(100, 0), 100);
        }

        #[test]
        fn multiply_apply_multiplies_values() {
            let op = Operator::new(OpKind::Multiply, 1, 1);
            assert_eq!(op.apply(10, 5), 50);
            assert_eq!(op.apply(1, 42), 42);
            assert_eq!(op.apply(100, 0), 0);
        }

        #[test]
        fn new_preserves_fields() {
            let op = Operator::new(OpKind::Add, 3, 5);
            assert_eq!(op.width, 3);
            assert_eq!(op.offset, 5);
            assert!(matches!(op.kind, OpKind::Add));
        }
    }

    mod bytes_tests {
        use super::*;

        #[test]
        fn byte_constants_are_correct() {
            assert_eq!(bytes::NEWLINE, b'\n');
            assert_eq!(bytes::SPACE, b' ');
            assert_eq!(bytes::PLUS, b'+');
            assert_eq!(bytes::STAR, b'*');
            assert_eq!(bytes::ZERO, b'0');
        }
    }

    mod worksheet_parsing_tests {
        use super::*;

        #[test]
        fn parses_single_add_column() {
            let input = b"1\n2\n3\n+\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);

            assert_eq!(worksheet.operators.len(), 1);
            assert!(matches!(worksheet.operators[0].kind, OpKind::Add));
        }

        #[test]
        fn parses_single_multiply_column() {
            let input = b"1\n2\n3\n*\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);

            assert_eq!(worksheet.operators.len(), 1);
            assert!(matches!(worksheet.operators[0].kind, OpKind::Multiply));
        }

        #[test]
        fn parses_multiple_operators() {
            let input = b"1 2\n3 4\n+ *\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);

            assert_eq!(worksheet.operators.len(), 2);
            assert!(matches!(worksheet.operators[0].kind, OpKind::Add));
            assert!(matches!(worksheet.operators[1].kind, OpKind::Multiply));
        }

        #[test]
        fn calculates_correct_dimensions() {
            let input = b"123\n456\n789\n+  \n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);

            assert_eq!(worksheet.stride_in_bytes, 4);
            assert_eq!(worksheet.height_in_bytes, 4);
        }
    }

    mod worksheet_computation_tests {
        use super::*;

        #[test]
        fn single_digit_add_column() {
            let input = b"1\n2\n3\n+\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 123);
        }

        #[test]
        fn single_digit_multiply_column() {
            let input = b"1\n2\n3\n*\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 123);
        }

        #[test]
        fn two_digit_add_column() {
            let input = b"12\n34\n56\n+ \n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 381);
        }

        #[test]
        fn two_digit_multiply_column() {
            let input = b"12\n34\n56\n* \n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 33210);
        }

        #[test]
        fn handles_spaces_in_numbers() {
            let input = b" 1\n 2\n 3\n +\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 123);
        }

        #[test]
        fn multiple_columns_mixed_operators() {
            let input = b"1 2\n3 4\n+ *\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 37);
        }

        #[test]
        fn all_add_operators() {
            let input = b"1 2 3\n4 5 6\n+ + +\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 75);
        }

        #[test]
        fn all_multiply_operators() {
            let input = b"1 2 3\n4 5 6\n* * *\n";
            let worksheet = CephalopodMathWorksheet::from_slice(input);
            assert_eq!(worksheet.sum(), 75);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn solve_example() {
            let input = read_to_string("example.txt").unwrap();
            let result = super::super::solve(input.as_bytes());
            assert_eq!(result, 3263827);
        }
    }
}
