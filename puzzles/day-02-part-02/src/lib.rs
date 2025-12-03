use rayon::prelude::*;

const POWERS_OF_10: [u64; 20] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
    10_000_000_000_000_000_000,
];

pub fn solve<S>(input: &S) -> u64
where
    S: AsRef<str> + ?Sized,
{
    parse_string_of_ranges(input)
        .par_bridge()
        .flat_map(|r| r.into_inner())
        .filter(|idx| {
            let digits = calculate_digits(*idx);
            find_pattern(*idx, digits)
        })
        .sum::<u64>()
}

fn parse_string_of_ranges<'a, S>(input: &'a S) -> impl Iterator<Item = Range> + 'a
where
    S: AsRef<str> + ?Sized,
{
    input
        .as_ref()
        .split(',')
        .map(|part| part.try_into().unwrap())
}

#[inline]
fn calculate_digits(n: u64) -> u32 {
    if n == 0 {
        return 1;
    }
    n.ilog10() + 1
}

// uses the geometric series formula in combination with proper divisors
// to attempt and extract a pattern for every divisor of the length of digits of the number.
// it then uses the geometric series multiplier to check if the number matches the pattern
// for the given repetitions
#[inline]
fn find_pattern(number: u64, digits: u32) -> bool {
    // digits = 6
    proper_divisors(digits)
        // pattern_length = 3
        .any(move |pattern_length| {
            // repetitions = 6 / 3 = 2
            let repetitions = digits / pattern_length;
            // pattern_base = 10 ^ 3 = 1000
            let pattern_base = POWERS_OF_10[pattern_length as usize];
            // pattern = 123123 % 1000 = 123
            let pattern = number % pattern_base;
            // multiplier = (1000 ^ 2 - 1) / (1000 - 1) = 999999 / 999 = 1001
            let multiplier =
                (POWERS_OF_10[(pattern_length * repetitions) as usize] - 1) / (pattern_base - 1);
            // 123 * 1001 = 123123
            pattern * multiplier == number
        })
}

fn proper_divisors(n: u32) -> impl Iterator<Item = u32> {
    let limit = n / 2;
    (1..=limit).filter(move |&d| n.is_multiple_of(d))
}

#[derive(Debug, Eq, PartialEq)]
struct Range {
    inner: std::ops::RangeInclusive<u64>,
}

impl Range {
    pub fn into_inner(self) -> std::ops::RangeInclusive<u64> {
        self.inner
    }
}

impl TryFrom<&str> for Range {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (start_str, end_str) = value
            .split_once('-')
            .ok_or_else(|| format!("Invalid range format: {}", value))?;

        let start: u64 = start_str
            .parse()
            .map_err(|e| format!("Invalid start: {}", e))?;
        let end: u64 = end_str.parse().map_err(|e| format!("Invalid end: {}", e))?;

        if start > end {
            return Err(format!("Start of range greater than end: {}", value));
        }

        Ok(Range {
            inner: std::ops::RangeInclusive::new(start, end),
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{Range, calculate_digits, find_pattern, parse_string_of_ranges, proper_divisors};

    #[rstest]
    #[case(6, vec![1, 2, 3])]
    #[case(100, vec![1, 2, 4, 5, 10, 20, 25, 50])]
    fn proper_divisors_works(#[case] input: u32, #[case] expected: Vec<u32>) {
        let divisors = proper_divisors(input).collect::<Vec<u32>>();
        assert_eq!(divisors, expected)
    }

    #[rstest]
    #[case(11, 2, true)]
    #[case(22, 2, true)]
    #[case(1010, 4, true)]
    #[case(1188511885, 10, true)]
    #[case(222222, 6, true)]
    #[case(446446, 6, true)]
    #[case(38593859, 8, true)]
    #[case(38593859, 8, true)]
    #[case(998, 3, false)]
    #[case(1012, 4, false)]
    #[case(1188511880, 10, false)]
    #[case(1188511890, 10, false)]
    #[case(222220, 6, false)]
    #[case(222224, 6, false)]
    #[case(1698522, 7, false)]
    #[case(1698528, 7, false)]
    #[case(446443, 6, false)]
    #[case(446449, 6, false)]
    #[case(38593856, 8, false)]
    #[case(38593862, 8, false)]
    fn find_pattern_works(#[case] input: u64, #[case] input_len: u32, #[case] expected: bool) {
        let p = find_pattern(input, input_len);

        assert_eq!(p, expected);
    }

    #[test]
    fn calculate_digits_1188511880_gives_10() {
        let n = 1188511880;
        let d = calculate_digits(n);

        assert_eq!(d, 10);
    }

    #[test]
    fn calculate_digits_0_gives_1() {
        let n = 0;
        let d = calculate_digits(n);

        assert_eq!(d, 1);
    }

    #[test]
    fn calculate_digits_123_gives_3() {
        let n = 123;
        let d = calculate_digits(n);

        assert_eq!(d, 3);
    }

    #[test]
    fn parse_range_works() {
        let range = "11-22";
        let range: Result<Range, String> = range.try_into();

        assert!(range.is_ok());
        let range = range.unwrap();
        assert_eq!(
            range,
            Range {
                inner: std::ops::RangeInclusive::new(11, 22)
            }
        )
    }

    #[test]
    fn parse_ranges_works() {
        let ranges = "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124";
        let ranges: Vec<Range> = parse_string_of_ranges(ranges).collect();

        assert_eq!(ranges.len(), 11);
    }

    #[test]
    fn example_test_case() {
        let ranges = "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124";
        let expected_sum_of_invalid_ranges = 4_174_379_265;

        let result = super::solve(ranges);
        assert_eq!(result, expected_sum_of_invalid_ranges);
    }
}
