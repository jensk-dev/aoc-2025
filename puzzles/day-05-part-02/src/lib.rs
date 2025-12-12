#![feature(slice_split_once)]

pub fn solve(input: &str) -> usize {
    let ranges = parse_input(input);
    let consolidated = consolidate(ranges);
    sum_ranges(&consolidated)
}

#[inline]
fn parse_input(input: &str) -> Vec<(usize, usize)> {
    input
        .as_bytes()
        .split(|&c| c == b'\n')
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let parts = line.split_once(|&c| c == b'-').unwrap();

            let start = unsafe { std::str::from_utf8_unchecked(parts.0) }
                .parse()
                .unwrap();

            let end = unsafe { std::str::from_utf8_unchecked(parts.1) }
                .parse()
                .unwrap();

            (start, end)
        })
        .collect::<Vec<(usize, usize)>>()
}

#[inline]
fn consolidate(mut ranges: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    ranges.sort_by(|&a, &b| a.0.cmp(&b.0));

    let mut consolidated: Vec<(usize, usize)> = Vec::new();
    consolidated.push(unsafe { *ranges.get_unchecked(0) });

    for range in ranges.into_iter().skip(1) {
        let previous_range = consolidated.last_mut().unwrap();

        if range.0 <= previous_range.1 {
            previous_range.1 = previous_range.1.max(range.1);
        } else {
            consolidated.push(range);
        }
    }

    consolidated
}

#[inline]
fn sum_ranges(ranges: &Vec<(usize, usize)>) -> usize {
    ranges.iter().map(|&(start, end)| end - start + 1).sum()
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use rstest::rstest;

    #[test]
    fn example() {
        let input = read_to_string("example.txt").unwrap();
        let answer = super::solve(&input);
        assert_eq!(answer, 14);
    }

    #[rstest]
    #[case(vec![(3, 5),(10, 14),(16, 20),(12, 18)], vec![(3, 5),(10, 20)])]
    fn consolidate(#[case] input: Vec<(usize, usize)>, #[case] expected: Vec<(usize, usize)>) {
        let result = super::consolidate(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(vec![(3, 5),(10, 20)], 14)]
    fn sum_ranges(#[case] input: Vec<(usize, usize)>, #[case] expected: usize) {
        let result = super::sum_ranges(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn adjacent_ranges_sum_equals_merged() {
        let adjacent = vec![(1, 5), (6, 10)];
        let consolidated_adjacent = super::consolidate(adjacent);

        let overlapping = vec![(1, 7), (5, 10)];
        let consolidated_overlapping = super::consolidate(overlapping);

        assert_eq!(consolidated_adjacent, vec![(1, 5), (6, 10)]);
        assert_eq!(consolidated_overlapping, vec![(1, 10)]);

        let sum_adjacent = super::sum_ranges(&consolidated_adjacent);
        let sum_overlapping = super::sum_ranges(&consolidated_overlapping);
        assert_eq!(sum_adjacent, sum_overlapping);
        assert_eq!(sum_adjacent, 10);
    }

    #[test]
    fn parse_input() {
        let input = read_to_string("example.txt").unwrap();
        let expected = vec![(3, 5), (10, 14), (16, 20), (12, 18)];
        let result = super::parse_input(&input);
        assert_eq!(result, expected);
    }
}
