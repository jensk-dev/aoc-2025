pub fn solve(input: &str) -> usize {
    // create vec of lines
    let lines = input
        .as_bytes()
        .split(|&c| c == b'\n')
        .collect::<Vec<&[u8]>>();

    // create 2 vectors: one for the ranges and one for the ids
    let mut sections = lines.split(|line| line.is_empty());
    let ranges = sections
        .next()
        .unwrap()
        .iter() // todo: parallelize
        .map(|line| {
            let mut parts = line.split(|&c| c == b'-');
            let start = unsafe { std::str::from_utf8_unchecked(parts.next().unwrap()) }
                .parse()
                .unwrap();
            let end = unsafe { std::str::from_utf8_unchecked(parts.next().unwrap()) }
                .parse()
                .unwrap();
            (start, end)
        })
        .collect::<Vec<(usize, usize)>>();
    let ids = sections
        .next()
        .unwrap()
        .iter() // todo parallelize
        .map(|line| {
            unsafe { std::str::from_utf8_unchecked(line) }
                .parse::<usize>()
                .unwrap()
        })
        .collect::<Vec<usize>>();

    ids.iter()
        .filter(|&&id| ranges.iter().any(|&(start, end)| id >= start && id <= end))
        .count()
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    #[test]
    fn example() {
        let input = read_to_string("example.txt").unwrap();
        let answer = super::solve(&input);
        assert_eq!(answer, 3);
    }
}
