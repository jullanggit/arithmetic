#![feature(bigint_helper_methods)]

use std::{
    cmp::Ordering,
    ops::{Add, Neg},
};

/// A whole number with base 2^64.
/// Biggest digit is stored last.
/// Zero is represented as a single, positive zero digit.
#[derive(PartialEq, Eq, Debug)]
struct Number {
    positive: bool,
    digits: Vec<u64>,
}
impl Number {
    fn new(positive: bool, digits: Vec<u64>) -> Self {
        Self { positive, digits }.normalize()
    }
    fn abs(&mut self) -> &mut Self {
        self.positive = true;
        self
    }
    /// computes the equivalent of self.abs().cmp(other.abs())
    fn cmp_abs(&self, other: &Self) -> Ordering {
        self.digits.iter().rev().cmp(other.digits.iter().rev())
    }
    /// removes leading zeroes and canonicalizes a zero value
    fn normalize(mut self) -> Self {
        // canonicalize empty vec to positive zero
        if self.digits.is_empty() {
            self.digits.push(0);
            self.positive = true;
        } else {
            // remove leading zeroes
            let num_leading_zeroes = self
                .digits
                .iter()
                .rev()
                .take_while(|digit| **digit == 0)
                .count();
            self.digits
                .truncate(1.max(self.digits.len() - num_leading_zeroes)); // always keep at least one digit

            if self.digits.len() == 1 && self.digits[0] == 0 {
                self.positive = true
            }
        }

        self
    }
}
impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.positive, other.positive) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            // self and other have the same sign
            (true, true) | (false, false) => match self.digits.len().cmp(&other.digits.len()) {
                Ordering::Equal => {
                    let cmp = self.cmp_abs(other);
                    if self.positive { cmp } else { cmp.reverse() }
                }
                other => {
                    if self.positive {
                        other
                    } else {
                        other.reverse()
                    }
                }
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
        let (mut bigger, smaller) = if self.cmp_abs(&rhs) == Ordering::Greater {
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

        bigger.normalize()
    }
}
impl Neg for Number {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        self.positive = !self.positive;
        self.normalize()
    }
}

fn main() {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::Number;

    #[test]
    fn add_positive() {
        let a = Number::new(true, vec![5]);
        let b = Number::new(true, vec![7]);
        let expected = Number::new(true, vec![12]);
        assert_eq!(a + b, expected);
    }

    #[test]
    fn add_carry_across_digit() {
        let a = Number::new(true, vec![u64::MAX]);
        let b = Number::new(true, vec![u64::MAX]);
        let expected = Number::new(true, vec![18446744073709551614, 1]);
        assert_eq!(a + b, expected);
    }

    #[test]
    fn add_opposite_cancel() {
        let a = Number::new(true, vec![u64::MAX]);
        let b = Number::new(false, vec![u64::MAX]);
        let expected = Number::new(true, vec![0]);
        assert_eq!(a + b, expected);
    }

    #[test]
    fn neg_zero_normalizes() {
        let zero = Number::new(true, vec![0]);
        assert_eq!(-zero, Number::new(true, vec![0]));
    }

    #[test]
    fn add_negative_and_positive() {
        let a = Number::new(false, vec![100]);
        let b = Number::new(true, vec![40]);
        let result = a + b; // -100 + 40 = -60
        assert_eq!(result, Number::new(false, vec![60]));
    }

    #[test]
    fn subtraction_bigger_smaller() {
        let a = Number::new(true, vec![0, 1]); // 2^64
        let b = Number::new(true, vec![1]); // 1
        let result = a + Number::new(false, vec![1]); // 2^64 - 1
        assert_eq!(result, Number::new(true, vec![u64::MAX]));
    }

    #[test]
    fn comparison_various() {
        let p = Number::new(true, vec![1]);
        let n = Number::new(false, vec![1]);
        assert!(p > Number::new(true, vec![0]));
        assert!(n < Number::new(true, vec![0]));
        assert!(n < p);
    }
}
