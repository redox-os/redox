// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! External iterators for generic mathematics

use {Integer, Zero, One, CheckedAdd, ToPrimitive};
use redox::ops::{Add, Sub};

/// An iterator over the range [start, stop)
#[derive(Clone)]
pub struct Range<A> {
    state: A,
    stop: A,
    one: A,
}

/// Returns an iterator over the given range [start, stop) (that is, starting
/// at start (inclusive), and ending at stop (exclusive)).
///
/// # Example
///
/// ```rust
/// use num::iter;
///
/// let array = [0, 1, 2, 3, 4];
///
/// for i in iter::range(0, 5) {
///     println!("{}", i);
///     assert_eq!(i,  array[i]);
/// }
/// ```
#[inline]
pub fn range<A>(start: A, stop: A) -> Range<A>
    where A: Add<A, Output = A> + PartialOrd + Clone + One
{
    Range {
        state: start,
        stop: stop,
        one: One::one(),
    }
}

// FIXME: rust-lang/rust#10414: Unfortunate type bound
impl<A> Iterator for Range<A>
    where A: Add<A, Output = A> + PartialOrd + Clone + ToPrimitive
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        if self.state < self.stop {
            let result = self.state.clone();
            self.state = self.state.clone() + self.one.clone();
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // This first checks if the elements are representable as i64. If they aren't, try u64 (to
        // handle cases like range(huge, huger)). We don't use usize/int because the difference of
        // the i64/u64 might lie within their range.
        let bound = match self.state.to_i64() {
            Some(a) => {
                let sz = self.stop.to_i64().map(|b| b.checked_sub(a));
                match sz {
                    Some(Some(bound)) => bound.to_usize(),
                    _ => None,
                }
            }
            None => match self.state.to_u64() {
                Some(a) => {
                    let sz = self.stop.to_u64().map(|b| b.checked_sub(a));
                    match sz {
                        Some(Some(bound)) => bound.to_usize(),
                        _ => None,
                    }
                }
                None => None,
            },
        };

        match bound {
            Some(b) => (b, Some(b)),
            // Standard fallback for unbounded/unrepresentable bounds
            None => (0, None),
        }
    }
}

/// `Integer` is required to ensure the range will be the same regardless of
/// the direction it is consumed.
impl<A> DoubleEndedIterator for Range<A>
    where A: Integer + PartialOrd + Clone + ToPrimitive
{
    #[inline]
    fn next_back(&mut self) -> Option<A> {
        if self.stop > self.state {
            self.stop = self.stop.clone() - self.one.clone();
            Some(self.stop.clone())
        } else {
            None
        }
    }
}

/// An iterator over the range [start, stop]
#[derive(Clone)]
pub struct RangeInclusive<A> {
    range: Range<A>,
    done: bool,
}

/// Return an iterator over the range [start, stop]
#[inline]
pub fn range_inclusive<A>(start: A, stop: A) -> RangeInclusive<A>
    where A: Add<A, Output = A> + PartialOrd + Clone + One
{
    RangeInclusive {
        range: range(start, stop),
        done: false,
    }
}

impl<A> Iterator for RangeInclusive<A>
    where A: Add<A, Output = A> + PartialOrd + Clone + ToPrimitive
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        match self.range.next() {
            Some(x) => Some(x),
            None => {
                if !self.done && self.range.state == self.range.stop {
                    self.done = true;
                    Some(self.range.stop.clone())
                } else {
                    None
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.range.size_hint();
        if self.done {
            (lo, hi)
        } else {
            let lo = lo.saturating_add(1);
            let hi = match hi {
                Some(x) => x.checked_add(1),
                None => None,
            };
            (lo, hi)
        }
    }
}

impl<A> DoubleEndedIterator for RangeInclusive<A>
    where A: Sub<A, Output = A> + Integer + PartialOrd + Clone + ToPrimitive
{
    #[inline]
    fn next_back(&mut self) -> Option<A> {
        if self.range.stop > self.range.state {
            let result = self.range.stop.clone();
            self.range.stop = self.range.stop.clone() - self.range.one.clone();
            Some(result)
        } else if !self.done && self.range.state == self.range.stop {
            self.done = true;
            Some(self.range.stop.clone())
        } else {
            None
        }
    }
}

/// An iterator over the range [start, stop) by `step`. It handles overflow by stopping.
#[derive(Clone)]
pub struct RangeStep<A> {
    state: A,
    stop: A,
    step: A,
    rev: bool,
}

/// Return an iterator over the range [start, stop) by `step`. It handles overflow by stopping.
#[inline]
pub fn range_step<A>(start: A, stop: A, step: A) -> RangeStep<A>
    where A: CheckedAdd + PartialOrd + Clone + Zero
{
    let rev = step < Zero::zero();
    RangeStep {
        state: start,
        stop: stop,
        step: step,
        rev: rev,
    }
}

impl<A> Iterator for RangeStep<A>
    where A: CheckedAdd + PartialOrd + Clone
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        if (self.rev && self.state > self.stop) || (!self.rev && self.state < self.stop) {
            let result = self.state.clone();
            match self.state.checked_add(&self.step) {
                Some(x) => self.state = x,
                None => self.state = self.stop.clone(),
            }
            Some(result)
        } else {
            None
        }
    }
}

/// An iterator over the range [start, stop] by `step`. It handles overflow by stopping.
#[derive(Clone)]
pub struct RangeStepInclusive<A> {
    state: A,
    stop: A,
    step: A,
    rev: bool,
    done: bool,
}

/// Return an iterator over the range [start, stop] by `step`. It handles overflow by stopping.
#[inline]
pub fn range_step_inclusive<A>(start: A, stop: A, step: A) -> RangeStepInclusive<A>
    where A: CheckedAdd + PartialOrd + Clone + Zero
{
    let rev = step < Zero::zero();
    RangeStepInclusive {
        state: start,
        stop: stop,
        step: step,
        rev: rev,
        done: false,
    }
}

impl<A> Iterator for RangeStepInclusive<A>
    where A: CheckedAdd + PartialOrd + Clone + PartialEq
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        if !self.done &&
           ((self.rev && self.state >= self.stop) || (!self.rev && self.state <= self.stop)) {
            let result = self.state.clone();
            match self.state.checked_add(&self.step) {
                Some(x) => self.state = x,
                None => self.done = true,
            }
            Some(result)
        } else {
            None
        }
    }
}
