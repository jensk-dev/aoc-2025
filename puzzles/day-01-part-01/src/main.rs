use std::fs::File;
use std::io::BufRead;

fn main() {
    let working_dir = std::env::current_dir().unwrap();
    let path = format!("{}/puzzles/day-01-part-01/input.txt", working_dir.display());
    let f = File::open(path).unwrap();
    let f = std::io::BufReader::new(f);

    let nr_of_turns_to_zero = solve(f);

    println!("nr_of_left_turn_zeros: {}", nr_of_turns_to_zero);
}

fn solve(reader: impl BufRead) -> usize {
    let mut dial = Dial::new();

    let mut nr_of_turns_to_zero = 0;

    for line in reader.lines() {
        let line = line.unwrap();

        if line.is_empty() {
            continue;
        }

        let turn = line.try_into().unwrap();
        dial = dial.turn(&turn);

        if dial.current_position() == 0 {
            nr_of_turns_to_zero += 1;
        }
    }

    nr_of_turns_to_zero
}

impl TryFrom<String> for Turn {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let direction = match &value[0..1] {
            "R" => Direction::Clockwise,
            "L" => Direction::CounterClockwise,
            _ => return Err(format!("Invalid direction: {}", &value[0..1])),
        };

        let steps: usize = value[1..]
            .parse()
            .map_err(|e| format!("Invalid steps: {}", e))?;
        Ok(Turn::new(direction, steps))
    }
}

#[derive(Debug)]
pub struct Turn {
    direction: Direction,
    steps: usize,
}

impl Turn {
    pub fn new(direction: Direction, steps: usize) -> Self {
        Self { direction, steps }
    }
}

pub struct Dial {
    position: u8,
}

impl Default for Dial {
    fn default() -> Self {
        Self::new()
    }
}

impl Dial {
    pub fn new() -> Self {
        Self { position: 50 }
    }

    pub fn turn(self, turn: &Turn) -> Self {
        match turn.direction {
            Direction::Clockwise => Self {
                position: ((self.position as usize + turn.steps) % 100) as u8,
            },
            Direction::CounterClockwise => Self {
                position: ((self.position as usize + 100 - turn.steps % 100) % 100) as u8,
            },
        }
    }

    pub fn current_position(&self) -> u8 {
        self.position
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

#[cfg(test)]
mod tests {
    #[test]
    fn dial_turns_left() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::CounterClockwise,
            steps: 10,
        });
        assert_eq!(dial.current_position(), 40);
    }

    #[test]
    fn dial_turns_right() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::Clockwise,
            steps: 10,
        });
        assert_eq!(dial.current_position(), 60);
    }

    #[test]
    fn dial_turns_left_with_overflow() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::CounterClockwise,
            steps: 60,
        });
        assert_eq!(dial.current_position(), 90);
    }

    #[test]
    fn dial_turns_right_with_overflow() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::Clockwise,
            steps: 60,
        });
        assert_eq!(dial.current_position(), 10);
    }

    #[test]
    fn dial_cannot_reach_100_through_clockwise_turn() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::Clockwise,
            steps: 50,
        });
        assert_eq!(dial.current_position(), 0);
    }

    #[test]
    fn dial_cannot_reach_100_through_counter_clockwise_turn() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::CounterClockwise,
            steps: 50,
        });
        assert_eq!(dial.current_position(), 0);
    }

    #[test]
    fn dial_can_reach_0_through_clockwise_turn() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::Clockwise,
            steps: 50,
        });
        assert_eq!(dial.current_position(), 0);
    }

    #[test]
    fn dial_can_reach_0_through_counter_clockwise_turn() {
        let dial = super::Dial::new();
        let dial = dial.turn(&super::Turn {
            direction: super::Direction::CounterClockwise,
            steps: 50,
        });
        assert_eq!(dial.current_position(), 0);
    }

    #[test]
    fn try_from_string_turn_right() {
        let turn: super::Turn = "R25".to_string().try_into().unwrap();
        assert_eq!(turn.direction, super::Direction::Clockwise);
        assert_eq!(turn.steps, 25);
    }

    #[test]
    fn try_from_string_turn_left() {
        let turn: super::Turn = "L30".to_string().try_into().unwrap();
        assert_eq!(turn.direction, super::Direction::CounterClockwise);
        assert_eq!(turn.steps, 30);
    }

    #[test]
    fn solve_example_input() {
        let input = "L68\nL30\nR48\nL5\nR60\nL55\nL1\nL99\nR14\nL82";
        let reader = std::io::BufReader::new(input.as_bytes());
        let result = super::solve(reader);
        assert_eq!(result, 3);
    }
}
