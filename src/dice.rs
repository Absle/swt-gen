use rand::distributions::uniform::SampleRange;
use rand::distributions::uniform::SampleUniform;
use rand::Rng;
use std::cmp::PartialOrd;
use std::ops::Add;

pub(crate) fn roll<T, U>(range: U) -> T
where
    T: SampleUniform,
    U: SampleRange<T>,
{
    let mut rng = rand::thread_rng();
    rng.gen_range(range)
}

pub(crate) fn roll_1d<T>(sides: T) -> T
where
    T: Copy + From<u8> + PartialOrd + SampleUniform,
{
    let mut rng = rand::thread_rng();
    let one = T::from(1);
    rng.gen_range(one..=sides)
}

pub(crate) fn roll_2d<T>(sides: T) -> T
where
    T: Add<Output = T> + Copy + From<u8> + PartialOrd + SampleUniform,
{
    roll_1d(sides) + roll_1d(sides)
}

#[allow(dead_code)]
pub(crate) fn roll_d66() -> isize {
    10 * roll_1d(6) + roll_1d(6)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ROLL_ATTEMPTS: usize = 10_000;

    #[test]
    fn test_roll_1d() {
        let mut rng = rand::thread_rng();
        for _ in 0..ROLL_ATTEMPTS {
            let sides = rng.gen_range(3..=20);
            let range = 1..=sides;
            let roll = roll_1d(sides);
            assert!(range.contains(&roll));
        }
    }

    #[test]
    fn test_roll_2d() {
        let mut rng = rand::thread_rng();
        for _ in 0..ROLL_ATTEMPTS {
            let sides = rng.gen_range(3..=20);
            let range = 2..=(2 * sides);
            let roll = roll_2d(sides);
            assert!(range.contains(&roll));
        }
    }

    #[test]
    fn test_roll_d66() {
        let mut possible_outcomes = HashSet::new();
        for i in 1..=6 {
            for j in 1..=6 {
                possible_outcomes.insert(10 * i + j);
            }
        }

        for _ in 0..ROLL_ATTEMPTS {
            let roll = roll_d66();
            assert!(possible_outcomes.contains(&roll));
        }
    }
}
