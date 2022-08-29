use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::Rng;
use std::cmp::PartialOrd;
use std::ops::{Add, AddAssign, Div, Mul, Rem, Sub};

/** Stand-in for "any integer"; any signed or unsigned primitive integer will satisfy this.

If it walks like an integer and quacks like an integer, it's probably an integer.
*/
pub trait DuckInteger:
    Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + TryFrom<u8>
    + TryInto<usize>
    + SampleUniform
    + Copy
{
}

impl<T> DuckInteger for T where
    T: Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + TryFrom<u8>
        + TryInto<usize>
        + SampleUniform
        + Copy
{
}

/** Roll a number within `range` with a uniform distribution.

# Panics
Panics of `range` is empty.
*/
pub fn roll_range<T: DuckInteger, U: SampleRange<T>>(range: U) -> T {
    assert!(!range.is_empty(), "Cannot roll within an empty range");
    let mut rng = rand::thread_rng();
    rng.gen_range(range)
}

/** Roll a `sides`-sided die `rolls` times and return the sum of all rolls.

# Panics
Panics if `rolls` or `sides` is less than 1.
*/
pub fn roll<T: DuckInteger>(rolls: T, sides: T) -> T {
    let one = T::try_from(1).unwrap_or_else(|_| unreachable!());
    assert!(rolls >= one, "Cannot roll zero or fewer dice");
    assert!(sides >= one, "Dice must have at least one side");

    let mut rng = rand::thread_rng();
    let mut roll = T::try_from(0).unwrap_or_else(|_| unreachable!());

    let rolls = rolls.try_into().unwrap_or_else(|_| unreachable!());
    for _ in 1..=rolls {
        roll += rng.gen_range(one..=sides);
    }
    roll
}

/** Wrapper for `dice::roll(1, sides)`. */
pub fn roll_1d<T: DuckInteger>(sides: T) -> T {
    let one = T::try_from(1).unwrap_or_else(|_| unreachable!());
    roll(one, sides)
}

/** Wrapper for `dice::roll(2, sides)`. */
pub fn roll_2d<T: DuckInteger>(sides: T) -> T {
    let two = T::try_from(2).unwrap_or_else(|_| unreachable!());
    roll(two, sides)
}

/** Roll two six-sided dice, treating one as the 10's place in a two-digit number.

For example,

- `roll: 5 & 4 -> 54`,
- `roll: 1 & 6 -> 16`,
- `roll: 6 & 1 -> 61`,
- etc...
*/
#[allow(dead_code)]
pub fn roll_d66() -> isize {
    10 * roll_1d(6) + roll_1d(6)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ROLL_ATTEMPTS: usize = 10_000;

    #[test]
    fn test_roll() {
        let mut rng = rand::thread_rng();
        for _ in 0..ROLL_ATTEMPTS {
            let dice = rng.gen_range(1..=50);
            let sides = rng.gen_range(1..=100);

            let low = dice;
            let high = dice * sides;
            let range = low..=high;

            let roll = roll(dice, sides);
            assert!(range.contains(&roll));
        }
    }

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
