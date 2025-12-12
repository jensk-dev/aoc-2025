pub fn solve(reader: &str) -> usize {
    let mut dial = TrackingDial::new();

    for line in reader.lines() {
        if line.is_empty() {
            continue;
        }

        let turn = line.try_into().unwrap();
        dial.turn(&turn);
    }

    dial.revolutions
}

impl TryFrom<&str> for Direction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let steps: usize = value[1..]
            .parse()
            .map_err(|e| format!("Invalid steps: {}", e))?;
        let direction = match &value[0..1] {
            "R" => Direction::Clockwise(steps),
            "L" => Direction::CounterClockwise(steps),
            _ => return Err(format!("Invalid direction: {}", &value[0..1])),
        };

        Ok(direction)
    }
}

#[derive(Debug)]
pub struct TrackingDial {
    position: usize,
    revolutions: usize,
}

impl Default for TrackingDial {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingDial {
    #[inline]
    const fn n() -> usize {
        100
    }

    pub fn new() -> Self {
        Self {
            position: 50,
            revolutions: 0,
        }
    }

    pub fn turn(&mut self, direction: &Direction) {
        match direction {
            Direction::Clockwise(steps) => self.turn_clockwise(steps),
            Direction::CounterClockwise(steps) => self.turn_counter_clockwise(steps),
        }
    }

    #[inline]
    fn turn_clockwise(&mut self, steps: &usize) {
        let n = Self::n();
        let actual = self.position;
        let total = actual + steps;
        let actual = total % n;
        let revolutions = total / n;

        self.position = actual;
        self.revolutions += revolutions;
    }

    #[inline]
    fn turn_counter_clockwise(&mut self, steps: &usize) {
        let n = Self::n();
        let actual = (n - self.position) % n;
        let total = actual + steps;
        let actual = total % n;
        let revolutions = total / n;

        self.position = (n - actual) % n;
        self.revolutions += revolutions;
    }

    pub fn current_position(&self) -> usize {
        self.position
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Clockwise(usize),
    CounterClockwise(usize),
}

#[cfg(test)]
mod tests {
    use crate::Direction;

    #[test]
    fn dial_from_zero_counter_clockwise() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(50);

        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 1);

        dial.turn(&Direction::CounterClockwise(5));
        assert_eq!(dial.current_position(), 95);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn dial_from_zero_clockwise() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(50);

        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 1);

        dial.turn(&Direction::Clockwise(105));
        assert_eq!(dial.current_position(), 5);
        assert_eq!(dial.revolutions, 2);
    }

    #[test]
    fn dial_with_multiple_clockwise_revolutions() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::Clockwise(249);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 99);
        assert_eq!(dial.revolutions, 2);
    }

    #[test]
    fn dial_with_exact_clockwise_revolutions() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::Clockwise(250);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 3);
    }

    #[test]
    fn dial_with_multiple_counter_clockwise_revolutions() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(249);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 1);
        assert_eq!(dial.revolutions, 2);
    }

    #[test]
    fn dial_with_exact_counter_clockwise_revolutions() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(250);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 3);
    }

    #[test]
    fn dial_turns_clockwise() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::Clockwise(40);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 90);
        assert_eq!(dial.revolutions, 0);
    }

    #[test]
    fn dial_turns_counter_clockwise() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(10);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 40);
    }

    #[test]
    fn dial_turns_counter_clockwise_with_overflow() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(60);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 90);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn dial_turns_clockwise_with_overflow() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::Clockwise(60);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 10);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn dial_cannot_reach_100_through_clockwise_turn() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::Clockwise(51);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 1);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn dial_cannot_reach_100_through_counter_clockwise_turn() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(51);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 99);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn dial_can_reach_0_through_clockwise_turn() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::Clockwise(50);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn dial_can_reach_0_through_counter_clockwise_turn() {
        let mut dial = super::TrackingDial::new();
        let direction = Direction::CounterClockwise(50);
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 1);
    }

    #[test]
    fn try_from_string_turn_right() {
        let direction: super::Direction = "R25".try_into().unwrap();
        assert_eq!(direction, super::Direction::Clockwise(25));
    }

    #[test]
    fn try_from_string_turn_left() {
        let direction: super::Direction = "L30".try_into().unwrap();
        assert_eq!(direction, super::Direction::CounterClockwise(30));
    }

    #[test]
    fn solve_example_input() {
        let input = "L68\nL30\nR48\nL5\nR60\nL55\nL1\nL99\nR14\nL82";
        let result = super::solve(input);
        assert_eq!(result, 6);
    }

    #[test]
    fn solve_r1000() {
        let input = "R1000";
        let result = super::solve(input);
        assert_eq!(result, 10);
    }

    #[test]
    fn solve_l1000() {
        let input = "L1000";
        let result = super::solve(input);
        assert_eq!(result, 10);
    }

    #[test]
    fn try_from_r250() {
        let direction: super::Direction = "R250".try_into().unwrap();
        let mut dial = super::TrackingDial::new();
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 3);
    }

    #[test]
    fn try_from_l250() {
        let direction: super::Direction = "L250".try_into().unwrap();
        let mut dial = super::TrackingDial::new();
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 0);
        assert_eq!(dial.revolutions, 3);
    }

    #[test]
    fn r1000() {
        let direction = Direction::Clockwise(1000);
        let mut dial = super::TrackingDial::new();
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 50);
        assert_eq!(dial.revolutions, 10);
    }

    #[test]
    fn l1000() {
        let direction = Direction::CounterClockwise(1000);
        let mut dial = super::TrackingDial::new();
        dial.turn(&direction);
        assert_eq!(dial.current_position(), 50);
        assert_eq!(dial.revolutions, 10);
    }
}
