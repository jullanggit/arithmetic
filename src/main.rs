#![feature(bigint_helper_methods)]

use std::{cmp::Ordering, ops::Add};

/// A whole number with base 2^64.
/// Biggest digit is stored last.
#[derive(PartialEq, Eq, Debug)]
struct Number {
    positive: bool,
    digits: Vec<u64>,
}
impl Number {
    fn new(positive: bool, digits: Vec<u64>) -> Self {
        Self { positive, digits }
    }
    fn abs(&mut self) {
        self.positive = !self.positive;
    }
    /// computes the equivalent of self.abs().cmp(other.abs())
    fn abs_cmp(&self, other: &Self) -> Ordering {
        self.digits.iter().rev().cmp(other.digits.iter().rev())
    }
    /// removes leading zeroes
    fn cleanup(mut self) -> Self {
        // remove leading zeroes
        let num_leading_zeroes = self
            .digits
            .iter()
            .rev()
            .take_while(|digit| **digit == 0)
            .count();

        self.digits.truncate(self.digits.len() - num_leading_zeroes);

        self
    }
}
impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.positive, other.positive) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            // self and other have the same sign
            _ => match self.digits.len().cmp(&other.digits.len()) {
                Ordering::Equal => {
                    let cmp = self.abs_cmp(other);
                    if !self.positive { cmp.reverse() } else { cmp }
                }
                other => other,
            },
        }
    }
}
impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Add for Number {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.abs_cmp(&rhs) == Ordering::Greater {
            (self, rhs)
        } else {
            (rhs, self)
        };

        type Fn = fn(u64, u64, bool) -> (u64, bool);
        let op = match (bigger.positive, smaller.positive) {
            (true, true) | (false, false) => u64::carrying_add as Fn,
            (true, false) | (false, true) => u64::borrowing_sub as Fn,
        };

        let mut carry = false;
        let mut i = 0;
        while i < smaller.digits.len() || carry {
            let digit_bigger = {
                if i == bigger.digits.len() {
                    bigger.digits.push(0);
                };
                &mut bigger.digits[i]
            };
            let digit_smaller = smaller.digits.get(i).map_or(0, Clone::clone);

            let (new_digit, new_carry) = op(*digit_bigger, digit_smaller, carry);
            *digit_bigger = new_digit;
            carry = new_carry;

            i += 1;
        }

        bigger.cleanup()
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::Number;

    #[test]
    fn add() {
        let tests = [
            [(true, vec![5]), (true, vec![7]), (true, vec![12])],
            [
                (true, vec![u64::MAX]),
                (true, vec![u64::MAX]),
                (true, vec![18446744073709551614, 1]),
            ],
            [
                (true, vec![u64::MAX]),
                (false, vec![u64::MAX]),
                (false, vec![]),
            ],
        ];
        for nums in tests {
            let [a, b, c] = nums.map(|(positive, digits)| Number::new(positive, digits));
            assert_eq!(a + b, c);
        }
    }
}
