pub fn solve<S>(input: S) -> usize
where
    S: AsRef<str>,
{
    input
        .as_ref()
        .as_bytes()
        .split(|&b| b == b'\n')
        .map(|line| scan_bank_rtl(line) as usize)
        .sum()
}

fn scan_bank_rtl(bank: &[u8]) -> u64 {
    const KEEP: usize = 12;
    let n = bank.len();

    let mut result = 0u64;
    let mut start = 0;

    for remaining in (1..=KEEP).rev() {
        let end = n - remaining;
        let slice = &bank[start..=end];

        let mut max_idx = 0;
        let mut max_val = slice[0];
        for (i, &v) in slice.iter().enumerate().skip(1) {
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }

        result = append(result, ascii_char_to_u8(max_val));
        start = start + max_idx + 1;
    }

    result
}

// in de ascii tabel liggen getallen tussen 48 en 57 inclusief.
// om dus een ascii u8 om te zetten naar de daadwerkelijke u8, kunnen we gewoon
// de offset van het 0de getal eraf halen
#[inline]
fn ascii_char_to_u8(ascii_char: u8) -> u8 {
    ascii_char - b'0'
}

#[inline]
fn append(acc: u64, digit: u8) -> u64 {
    10 * acc + digit as u64
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case(b'0', 0)]
    #[case(b'1', 1)]
    #[case(b'2', 2)]
    #[case(b'3', 3)]
    #[case(b'4', 4)]
    #[case(b'5', 5)]
    #[case(b'6', 6)]
    #[case(b'7', 7)]
    #[case(b'8', 8)]
    #[case(b'9', 9)]
    fn ascii_char_to_u8(#[case] input: u8, #[case] expected: u8) {
        let result = super::ascii_char_to_u8(input);

        assert_eq!(result, expected)
    }

    #[rstest]
    #[case(9, 1, 91)]
    #[case(20, 1, 201)]
    fn append(#[case] a: u64, #[case] b: u8, #[case] expected: u64) {
        let result = super::append(a, b);

        assert_eq!(result, expected)
    }

    #[rstest]
    #[case("987654321111111", 987_654_321_111)]
    #[case("811111111111119", 811_111_111_119)]
    #[case("234234234234278", 434_234_234_278)]
    #[case("818181911112111", 888_911_112_111)]
    fn scan_bank_rtl(#[case] input: &str, #[case] expected: u64) {
        let result = super::scan_bank_rtl(input.as_bytes());

        assert_eq!(result, expected)
    }

    #[rstest]
    fn solve_example_test_case() {
        let input = "987654321111111\n811111111111119\n234234234234278\n818181911112111";
        let output = super::solve(input);

        assert_eq!(output, 3_121_910_778_619)
    }
}
