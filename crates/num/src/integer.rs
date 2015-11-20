// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integer trait and functions.

use {Num, Signed};

pub trait Integer
    : Sized
    + Num
    + PartialOrd + Ord + Eq
{
    /// Floored integer division.
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert!(( 8).div_floor(& 3) ==  2);
    /// assert!(( 8).div_floor(&-3) == -3);
    /// assert!((-8).div_floor(& 3) == -3);
    /// assert!((-8).div_floor(&-3) ==  2);
    ///
    /// assert!(( 1).div_floor(& 2) ==  0);
    /// assert!(( 1).div_floor(&-2) == -1);
    /// assert!((-1).div_floor(& 2) == -1);
    /// assert!((-1).div_floor(&-2) ==  0);
    /// ~~~
    fn div_floor(&self, other: &Self) -> Self;

    /// Floored integer modulo, satisfying:
    ///
    /// ~~~
    /// # use num::Integer;
    /// # let n = 1; let d = 1;
    /// assert!(n.div_floor(&d) * d + n.mod_floor(&d) == n)
    /// ~~~
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert!(( 8).mod_floor(& 3) ==  2);
    /// assert!(( 8).mod_floor(&-3) == -1);
    /// assert!((-8).mod_floor(& 3) ==  1);
    /// assert!((-8).mod_floor(&-3) == -2);
    ///
    /// assert!(( 1).mod_floor(& 2) ==  1);
    /// assert!(( 1).mod_floor(&-2) == -1);
    /// assert!((-1).mod_floor(& 2) ==  1);
    /// assert!((-1).mod_floor(&-2) == -1);
    /// ~~~
    fn mod_floor(&self, other: &Self) -> Self;

    /// Greatest Common Divisor (GCD).
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(6.gcd(&8), 2);
    /// assert_eq!(7.gcd(&3), 1);
    /// ~~~
    fn gcd(&self, other: &Self) -> Self;

    /// Lowest Common Multiple (LCM).
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(7.lcm(&3), 21);
    /// assert_eq!(2.lcm(&4), 4);
    /// ~~~
    fn lcm(&self, other: &Self) -> Self;

    /// Deprecated, use `is_multiple_of` instead.
    fn divides(&self, other: &Self) -> bool;

    /// Returns `true` if `other` is a multiple of `self`.
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(9.is_multiple_of(&3), true);
    /// assert_eq!(3.is_multiple_of(&9), false);
    /// ~~~
    fn is_multiple_of(&self, other: &Self) -> bool;

    /// Returns `true` if the number is even.
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(3.is_even(), false);
    /// assert_eq!(4.is_even(), true);
    /// ~~~
    fn is_even(&self) -> bool;

    /// Returns `true` if the number is odd.
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(3.is_odd(), true);
    /// assert_eq!(4.is_odd(), false);
    /// ~~~
    fn is_odd(&self) -> bool;

    /// Simultaneous truncated integer division and modulus.
    /// Returns `(quotient, remainder)`.
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(( 8).div_rem( &3), ( 2,  2));
    /// assert_eq!(( 8).div_rem(&-3), (-2,  2));
    /// assert_eq!((-8).div_rem( &3), (-2, -2));
    /// assert_eq!((-8).div_rem(&-3), ( 2, -2));
    ///
    /// assert_eq!(( 1).div_rem( &2), ( 0,  1));
    /// assert_eq!(( 1).div_rem(&-2), ( 0,  1));
    /// assert_eq!((-1).div_rem( &2), ( 0, -1));
    /// assert_eq!((-1).div_rem(&-2), ( 0, -1));
    /// ~~~
    #[inline]
    fn div_rem(&self, other: &Self) -> (Self, Self);

    /// Simultaneous floored integer division and modulus.
    /// Returns `(quotient, remainder)`.
    ///
    /// # Examples
    ///
    /// ~~~
    /// # use num::Integer;
    /// assert_eq!(( 8).div_mod_floor( &3), ( 2,  2));
    /// assert_eq!(( 8).div_mod_floor(&-3), (-3, -1));
    /// assert_eq!((-8).div_mod_floor( &3), (-3,  1));
    /// assert_eq!((-8).div_mod_floor(&-3), ( 2, -2));
    ///
    /// assert_eq!(( 1).div_mod_floor( &2), ( 0,  1));
    /// assert_eq!(( 1).div_mod_floor(&-2), (-1, -1));
    /// assert_eq!((-1).div_mod_floor( &2), (-1,  1));
    /// assert_eq!((-1).div_mod_floor(&-2), ( 0, -1));
    /// ~~~
    fn div_mod_floor(&self, other: &Self) -> (Self, Self) {
        (self.div_floor(other), self.mod_floor(other))
    }
}

/// Simultaneous integer division and modulus
#[inline]
pub fn div_rem<T: Integer>(x: T, y: T) -> (T, T) {
    x.div_rem(&y)
}
/// Floored integer division
#[inline]
pub fn div_floor<T: Integer>(x: T, y: T) -> T {
    x.div_floor(&y)
}
/// Floored integer modulus
#[inline]
pub fn mod_floor<T: Integer>(x: T, y: T) -> T {
    x.mod_floor(&y)
}
/// Simultaneous floored integer division and modulus
#[inline]
pub fn div_mod_floor<T: Integer>(x: T, y: T) -> (T, T) {
    x.div_mod_floor(&y)
}

/// Calculates the Greatest Common Divisor (GCD) of the number and `other`. The
/// result is always positive.
#[inline(always)]
pub fn gcd<T: Integer>(x: T, y: T) -> T {
    x.gcd(&y)
}
/// Calculates the Lowest Common Multiple (LCM) of the number and `other`.
#[inline(always)]
pub fn lcm<T: Integer>(x: T, y: T) -> T {
    x.lcm(&y)
}

macro_rules! impl_integer_for_isize {
    ($T:ty, $test_mod:ident) => (
        impl Integer for $T {
            /// Floored integer division
            #[inline]
            fn div_floor(&self, other: &$T) -> $T {
                // Algorithm from [Daan Leijen. _Division and Modulus for Computer Scientists_,
                // December 2001](http://research.microsoft.com/pubs/151917/divmodnote-letter.pdf)
                match self.div_rem(other) {
                    (d, r) if (r > 0 && *other < 0)
                           || (r < 0 && *other > 0) => d - 1,
                    (d, _)                          => d,
                }
            }

            /// Floored integer modulo
            #[inline]
            fn mod_floor(&self, other: &$T) -> $T {
                // Algorithm from [Daan Leijen. _Division and Modulus for Computer Scientists_,
                // December 2001](http://research.microsoft.com/pubs/151917/divmodnote-letter.pdf)
                match *self % *other {
                    r if (r > 0 && *other < 0)
                      || (r < 0 && *other > 0) => r + *other,
                    r                          => r,
                }
            }

            /// Calculates `div_floor` and `mod_floor` simultaneously
            #[inline]
            fn div_mod_floor(&self, other: &$T) -> ($T,$T) {
                // Algorithm from [Daan Leijen. _Division and Modulus for Computer Scientists_,
                // December 2001](http://research.microsoft.com/pubs/151917/divmodnote-letter.pdf)
                match self.div_rem(other) {
                    (d, r) if (r > 0 && *other < 0)
                           || (r < 0 && *other > 0) => (d - 1, r + *other),
                    (d, r)                          => (d, r),
                }
            }

            /// Calculates the Greatest Common Divisor (GCD) of the number and
            /// `other`. The result is always positive.
            #[inline]
            fn gcd(&self, other: &$T) -> $T {
                // Use Euclid's algorithm
                let mut m = *self;
                let mut n = *other;
                while m != 0 {
                    let temp = m;
                    m = n % temp;
                    n = temp;
                }
                n.abs()
            }

            /// Calculates the Lowest Common Multiple (LCM) of the number and
            /// `other`.
            #[inline]
            fn lcm(&self, other: &$T) -> $T {
                // should not have to recalculate abs
                ((*self * *other) / self.gcd(other)).abs()
            }

            /// Deprecated, use `is_multiple_of` instead.
            #[inline]
            fn divides(&self, other: &$T) -> bool { return self.is_multiple_of(other); }

            /// Returns `true` if the number is a multiple of `other`.
            #[inline]
            fn is_multiple_of(&self, other: &$T) -> bool { *self % *other == 0 }

            /// Returns `true` if the number is divisible by `2`
            #[inline]
            fn is_even(&self) -> bool { (*self) & 1 == 0 }

            /// Returns `true` if the number is not divisible by `2`
            #[inline]
            fn is_odd(&self) -> bool { !self.is_even() }

            /// Simultaneous truncated integer division and modulus.
            #[inline]
            fn div_rem(&self, other: &$T) -> ($T, $T) {
                (*self / *other, *self % *other)
            }
        }

        #[cfg(test)]
        mod $test_mod {}
    )
}

impl_integer_for_isize!(i8,   test_integer_i8);
impl_integer_for_isize!(i16,  test_integer_i16);
impl_integer_for_isize!(i32,  test_integer_i32);
impl_integer_for_isize!(i64,  test_integer_i64);
impl_integer_for_isize!(isize,  test_integer_isize);

macro_rules! impl_integer_for_usize {
    ($T:ty, $test_mod:ident) => (
        impl Integer for $T {
            /// Unsigned integer division. Returns the same result as `div` (`/`).
            #[inline]
            fn div_floor(&self, other: &$T) -> $T { *self / *other }

            /// Unsigned integer modulo operation. Returns the same result as `rem` (`%`).
            #[inline]
            fn mod_floor(&self, other: &$T) -> $T { *self % *other }

            /// Calculates the Greatest Common Divisor (GCD) of the number and `other`
            #[inline]
            fn gcd(&self, other: &$T) -> $T {
                // Use Euclid's algorithm
                let mut m = *self;
                let mut n = *other;
                while m != 0 {
                    let temp = m;
                    m = n % temp;
                    n = temp;
                }
                n
            }

            /// Calculates the Lowest Common Multiple (LCM) of the number and `other`.
            #[inline]
            fn lcm(&self, other: &$T) -> $T {
                (*self * *other) / self.gcd(other)
            }

            /// Deprecated, use `is_multiple_of` instead.
            #[inline]
            fn divides(&self, other: &$T) -> bool { return self.is_multiple_of(other); }

            /// Returns `true` if the number is a multiple of `other`.
            #[inline]
            fn is_multiple_of(&self, other: &$T) -> bool { *self % *other == 0 }

            /// Returns `true` if the number is divisible by `2`.
            #[inline]
            fn is_even(&self) -> bool { (*self) & 1 == 0 }

            /// Returns `true` if the number is not divisible by `2`.
            #[inline]
            fn is_odd(&self) -> bool { !(*self).is_even() }

            /// Simultaneous truncated integer division and modulus.
            #[inline]
            fn div_rem(&self, other: &$T) -> ($T, $T) {
                (*self / *other, *self % *other)
            }
        }

        #[cfg(test)]
        mod $test_mod {}
    )
}

impl_integer_for_usize!(u8,   test_integer_u8);
impl_integer_for_usize!(u16,  test_integer_u16);
impl_integer_for_usize!(u32,  test_integer_u32);
impl_integer_for_usize!(u64,  test_integer_u64);
impl_integer_for_usize!(usize, test_integer_usize);
