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

fn scan_bank_rtl(bank: &[u8]) -> u8 {
    let last_idx = bank.len() - 1;
    let mut max_total = 0;
    let mut max_suffix = ascii_char_to_u8(bank[last_idx]);

    for i in (0..last_idx).rev() {
        let curr = ascii_char_to_u8(bank[i]);
        let pair = concat(curr, max_suffix);
        max_total = max_total.max(pair);
        max_suffix = max_suffix.max(curr);
    }

    max_total
}

// in de ascii tabel liggen getallen tussen 48 en 57 inclusief.
// om dus een ascii u8 om te zetten naar de daadwerkelijke u8, kunnen we gewoon
// de offset van het 0de getal eraf halen
#[inline]
fn ascii_char_to_u8(ascii_char: u8) -> u8 {
    ascii_char - b'0'
}

#[inline]
fn concat(a: u8, b: u8) -> u8 {
    10 * a + b
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
    fn concat(#[case] a: u8, #[case] b: u8, #[case] expected: u8) {
        let result = super::concat(a, b);

        assert_eq!(result, expected)
    }

    #[rstest]
    #[case("987654321111111", 98)]
    #[case("811111111111119", 89)]
    #[case("234234234234278", 78)]
    #[case("818181911112111", 92)]
    fn scan_bank_rtl(#[case] input: &str, #[case] expected: u8) {
        let result = super::scan_bank_rtl(input.as_bytes());

        assert_eq!(result, expected)
    }

    #[rstest]
    fn solve_example_test_case() {
        let input = "987654321111111\n811111111111119\n234234234234278\n818181911112111";
        let output = super::solve(input);

        assert_eq!(output, 357)
    }
}
