// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Numeric traits for generic mathematics

use redox::ops::{Add, Sub, Mul, Div, Rem, Neg};
use redox::ops::{Not, BitAnd, BitOr, BitXor, Shl, Shr};
use redox::{usize, u8, u16, u32, u64};
use redox::{isize, i8, i16, i32, i64};
use redox::{f32, f64};
use redox::mem::{self, size_of};
use redox::num::FpCategory;

/// The base trait for numeric types
pub trait Num: PartialEq + Zero + One
    + Add<Output = Self> + Sub<Output = Self>
    + Mul<Output = Self> + Div<Output = Self> + Rem<Output = Self>
{
    /// Parse error for `from_str_radix`
    type FromStrRadixErr;

    /// Convert from a string and radix <= 36.
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr>;
}

macro_rules! int_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
        impl $name for $t {
            type FromStrRadixErr = ::redox::num::ParseIntError;
            fn from_str_radix(s: &str, radix: u32)
                              -> Result<Self, ::redox::num::ParseIntError>
            {
                <$t>::from_str_radix(s, radix)
            }
        }
    )*)
}

// FIXME: Temporary replacements for unstable ::redox::num::ParseFloatError and
// ::redox::num::FloatErrorKind. These can be removed once the std float implementation of
// from_str_radix stabilises.
pub enum FloatErrorKind {
    Empty,
    Invalid,
}
pub struct ParseFloatError {
    pub kind: FloatErrorKind,
}

// FIXME: This should be removed and replaced with the std implementation of from_str_radix once
// it is stabilised.
macro_rules! float_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
        impl $name for $t {
            type FromStrRadixErr = ParseFloatError;
            fn from_str_radix(src: &str, radix: u32)
                              -> Result<Self, ParseFloatError>
            {
                use self::FloatErrorKind::*;
                use self::ParseFloatError as PFE;

                // Special values
                match src {
                    "inf"   => return Ok(Float::infinity()),
                    "-inf"  => return Ok(Float::neg_infinity()),
                    "NaN"   => return Ok(Float::nan()),
                    _       => {},
                }

                fn slice_shift_char(src: &str) -> Option<(char, &str)> {
                    src.chars().nth(0).map(|ch| (ch, &src[1..]))
                }

                let (is_positive, src) =  match slice_shift_char(src) {
                    None             => return Err(PFE { kind: Empty }),
                    Some(('-', ""))  => return Err(PFE { kind: Empty }),
                    Some(('-', src)) => (false, src),
                    Some((_, _))     => (true,  src),
                };

                // The significand to accumulate
                let mut sig = if is_positive { 0.0 } else { -0.0 };
                // Necessary to detect overflow
                let mut prev_sig = sig;
                let mut cs = src.chars().enumerate();
                // Exponent prefix and exponent index offset
                let mut exp_info = None::<(char, usize)>;

                // Parse the integer part of the significand
                for (i, c) in cs.by_ref() {
                    match c.to_digit(radix) {
                        Some(digit) => {
                            // shift significand one digit left
                            sig = sig * (radix as $t);

                            // add/subtract current digit depending on sign
                            if is_positive {
                                sig = sig + ((digit as isize) as $t);
                            } else {
                                sig = sig - ((digit as isize) as $t);
                            }

                            // Detect overflow by comparing to last value, except
                            // if we've not seen any non-zero digits.
                            if prev_sig != 0.0 {
                                if is_positive && sig <= prev_sig
                                    { return Ok(Float::infinity()); }
                                if !is_positive && sig >= prev_sig
                                    { return Ok(Float::neg_infinity()); }

                                // Detect overflow by reversing the shift-and-add process
                                if is_positive && (prev_sig != (sig - digit as $t) / radix as $t)
                                    { return Ok(Float::infinity()); }
                                if !is_positive && (prev_sig != (sig + digit as $t) / radix as $t)
                                    { return Ok(Float::neg_infinity()); }
                            }
                            prev_sig = sig;
                        },
                        None => match c {
                            'e' | 'E' | 'p' | 'P' => {
                                exp_info = Some((c, i + 1));
                                break;  // start of exponent
                            },
                            '.' => {
                                break;  // start of fractional part
                            },
                            _ => {
                                return Err(PFE { kind: Invalid });
                            },
                        },
                    }
                }

                // If we are not yet at the exponent parse the fractional
                // part of the significand
                if exp_info.is_none() {
                    let mut power = 1.0;
                    for (i, c) in cs.by_ref() {
                        match c.to_digit(radix) {
                            Some(digit) => {
                                // Decrease power one order of magnitude
                                power = power / (radix as $t);
                                // add/subtract current digit depending on sign
                                sig = if is_positive {
                                    sig + (digit as $t) * power
                                } else {
                                    sig - (digit as $t) * power
                                };
                                // Detect overflow by comparing to last value
                                if is_positive && sig < prev_sig
                                    { return Ok(Float::infinity()); }
                                if !is_positive && sig > prev_sig
                                    { return Ok(Float::neg_infinity()); }
                                prev_sig = sig;
                            },
                            None => match c {
                                'e' | 'E' | 'p' | 'P' => {
                                    exp_info = Some((c, i + 1));
                                    break; // start of exponent
                                },
                                _ => {
                                    return Err(PFE { kind: Invalid });
                                },
                            },
                        }
                    }
                }

                // Parse and calculate the exponent
                let exp = match exp_info {
                    Some((c, offset)) => {
                        let base = match c {
                            'E' | 'e' if radix == 10 => 10.0,
                            'P' | 'p' if radix == 16 => 2.0,
                            _ => return Err(PFE { kind: Invalid }),
                        };

                        // Parse the exponent as decimal integer
                        let src = &src[offset..];
                        let (is_positive, exp) = match slice_shift_char(src) {
                            Some(('-', src)) => (false, src.parse::<usize>()),
                            Some(('+', src)) => (true,  src.parse::<usize>()),
                            Some((_, _))     => (true,  src.parse::<usize>()),
                            None             => return Err(PFE { kind: Invalid }),
                        };

                        match (is_positive, exp) {
                            (true,  Ok(exp)) => base.powi(exp as i32),
                            (false, Ok(exp)) => 1.0 / base.powi(exp as i32),
                            (_, Err(_))      => return Err(PFE { kind: Invalid }),
                        }
                    },
                    None => 1.0, // no exponent
                };

                Ok(sig * exp)

            }
        }
    )*)
}

int_trait_impl!(Num for usize u8 u16 u32 u64 isize i8 i16 i32 i64);
float_trait_impl!(Num for f32 f64);

/// Defines an additive identity element for `Self`.
///
/// # Deriving
///
/// This trait can be automatically be derived using `#[deriving(Zero)]`
/// attribute. If you choose to use this, make sure that the laws outlined in
/// the documentation for `Zero::zero` still hold.
pub trait Zero: Sized + Add<Self, Output = Self> {
    /// Returns the additive identity element of `Self`, `0`.
    ///
    /// # Laws
    ///
    /// ```{.text}
    /// a + 0 = a       ∀ a ∈ Self
    /// 0 + a = a       ∀ a ∈ Self
    /// ```
    ///
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // FIXME (#5527): This should be an associated constant
    fn zero() -> Self;

    /// Returns `true` if `self` is equal to the additive identity.
    #[inline]
    fn is_zero(&self) -> bool;
}

macro_rules! zero_impl {
    ($t:ty, $v:expr) => {
        impl Zero for $t {
            #[inline]
            fn zero() -> $t { $v }
            #[inline]
            fn is_zero(&self) -> bool { *self == $v }
        }
    }
}

zero_impl!(usize, 0usize);
zero_impl!(u8,   0u8);
zero_impl!(u16,  0u16);
zero_impl!(u32,  0u32);
zero_impl!(u64,  0u64);

zero_impl!(isize, 0isize);
zero_impl!(i8,  0i8);
zero_impl!(i16, 0i16);
zero_impl!(i32, 0i32);
zero_impl!(i64, 0i64);

zero_impl!(f32, 0.0f32);
zero_impl!(f64, 0.0f64);

/// Defines a multiplicative identity element for `Self`.
pub trait One: Sized + Mul<Self, Output = Self> {
    /// Returns the multiplicative identity element of `Self`, `1`.
    ///
    /// # Laws
    ///
    /// ```{.text}
    /// a * 1 = a       ∀ a ∈ Self
    /// 1 * a = a       ∀ a ∈ Self
    /// ```
    ///
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // FIXME (#5527): This should be an associated constant
    fn one() -> Self;
}

macro_rules! one_impl {
    ($t:ty, $v:expr) => {
        impl One for $t {
            #[inline]
            fn one() -> $t { $v }
        }
    }
}

one_impl!(usize, 1usize);
one_impl!(u8,  1u8);
one_impl!(u16, 1u16);
one_impl!(u32, 1u32);
one_impl!(u64, 1u64);

one_impl!(isize, 1isize);
one_impl!(i8,  1i8);
one_impl!(i16, 1i16);
one_impl!(i32, 1i32);
one_impl!(i64, 1i64);

one_impl!(f32, 1.0f32);
one_impl!(f64, 1.0f64);

/// Useful functions for signed numbers (i.e. numbers that can be negative).
pub trait Signed: Sized + Num + Neg<Output = Self> {
    /// Computes the absolute value.
    ///
    /// For `f32` and `f64`, `NaN` will be returned if the number is `NaN`.
    ///
    /// For signed integers, `::MIN` will be returned if the number is `::MIN`.
    fn abs(&self) -> Self;

    /// The positive difference of two numbers.
    ///
    /// Returns `zero` if the number is less than or equal to `other`, otherwise the difference
    /// between `self` and `other` is returned.
    fn abs_sub(&self, other: &Self) -> Self;

    /// Returns the sign of the number.
    ///
    /// For `f32` and `f64`:
    ///
    /// * `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// * `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// * `NaN` if the number is `NaN`
    ///
    /// For signed integers:
    ///
    /// * `0` if the number is zero
    /// * `1` if the number is positive
    /// * `-1` if the number is negative
    fn signum(&self) -> Self;

    /// Returns true if the number is positive and false if the number is zero or negative.
    fn is_positive(&self) -> bool;

    /// Returns true if the number is negative and false if the number is zero or positive.
    fn is_negative(&self) -> bool;
}

macro_rules! signed_impl {
    ($($t:ty)*) => ($(
        impl Signed for $t {
            #[inline]
            fn abs(&self) -> $t {
                if self.is_negative() { -*self } else { *self }
            }

            #[inline]
            fn abs_sub(&self, other: &$t) -> $t {
                if *self <= *other { 0 } else { *self - *other }
            }

            #[inline]
            fn signum(&self) -> $t {
                match *self {
                    n if n > 0 => 1,
                    0 => 0,
                    _ => -1,
                }
            }

            #[inline]
            fn is_positive(&self) -> bool { *self > 0 }

            #[inline]
            fn is_negative(&self) -> bool { *self < 0 }
        }
    )*)
}

signed_impl!(isize i8 i16 i32 i64);

macro_rules! signed_float_impl {
    ($t:ty, $nan:expr, $inf:expr, $neg_inf:expr) => {
        impl Signed for $t {
            /// Computes the absolute value. Returns `NAN` if the number is `NAN`.
            #[inline]
            fn abs(&self) -> $t {
                <$t>::abs(*self)
            }

            /// The positive difference of two numbers. Returns `0.0` if the number is
            /// less than or equal to `other`, otherwise the difference between`self`
            /// and `other` is returned.
            #[inline]
            fn abs_sub(&self, other: &$t) -> $t {
                <$t>::abs_sub(*self, *other)
            }

            /// # Returns
            ///
            /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
            /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
            /// - `NAN` if the number is NaN
            #[inline]
            fn signum(&self) -> $t {
                <$t>::signum(*self)
            }

            /// Returns `true` if the number is positive, including `+0.0` and `INFINITY`
            #[inline]
            fn is_positive(&self) -> bool { *self > 0.0 || (1.0 / *self) == $inf }

            /// Returns `true` if the number is negative, including `-0.0` and `NEG_INFINITY`
            #[inline]
            fn is_negative(&self) -> bool { *self < 0.0 || (1.0 / *self) == $neg_inf }
        }
    }
}

signed_float_impl!(f32, f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
signed_float_impl!(f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY);

/// A trait for values which cannot be negative
pub trait Unsigned: Num {}

macro_rules! empty_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
        impl $name for $t {}
    )*)
}

empty_trait_impl!(Unsigned for usize u8 u16 u32 u64);

/// Numbers which have upper and lower bounds
pub trait Bounded {
    // FIXME (#5527): These should be associated constants
    /// returns the smallest finite number this type can represent
    fn min_value() -> Self;
    /// returns the largest finite number this type can represent
    fn max_value() -> Self;
}

macro_rules! bounded_impl {
    ($t:ty, $min:expr, $max:expr) => {
        impl Bounded for $t {
            #[inline]
            fn min_value() -> $t { $min }

            #[inline]
            fn max_value() -> $t { $max }
        }
    }
}

bounded_impl!(usize, usize::MIN, usize::MAX);
bounded_impl!(u8, u8::MIN, u8::MAX);
bounded_impl!(u16, u16::MIN, u16::MAX);
bounded_impl!(u32, u32::MIN, u32::MAX);
bounded_impl!(u64, u64::MIN, u64::MAX);

bounded_impl!(isize, isize::MIN, isize::MAX);
bounded_impl!(i8, i8::MIN, i8::MAX);
bounded_impl!(i16, i16::MIN, i16::MAX);
bounded_impl!(i32, i32::MIN, i32::MAX);
bounded_impl!(i64, i64::MIN, i64::MAX);

bounded_impl!(f32, f32::MIN, f32::MAX);
bounded_impl!(f64, f64::MIN, f64::MAX);

macro_rules! for_each_tuple_ {
    ( $m:ident !! ) => (
        $m! { }
    );
    ( $m:ident !! $h:ident, $($t:ident,)* ) => (
        $m! { $h $($t)* }
        for_each_tuple_! { $m !! $($t,)* }
    );
}
macro_rules! for_each_tuple {
    ( $m:ident ) => (
        for_each_tuple_! { $m !! A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, }
    );
}

macro_rules! bounded_tuple {
    ( $($name:ident)* ) => (
        impl<$($name: Bounded,)*> Bounded for ($($name,)*) {
            fn min_value() -> Self {
                ($($name::min_value(),)*)
            }
            fn max_value() -> Self {
                ($($name::max_value(),)*)
            }
        }
    );
}

for_each_tuple!(bounded_tuple);

/// Saturating math operations
pub trait Saturating {
    /// Saturating addition operator.
    /// Returns a+b, saturating at the numeric bounds instead of overflowing.
    fn saturating_add(self, v: Self) -> Self;

    /// Saturating subtraction operator.
    /// Returns a-b, saturating at the numeric bounds instead of overflowing.
    fn saturating_sub(self, v: Self) -> Self;
}

impl<T: CheckedAdd + CheckedSub + Zero + PartialOrd + Bounded> Saturating for T {
    #[inline]
    fn saturating_add(self, v: T) -> T {
        match self.checked_add(&v) {
            Some(x) => x,
            None => if v >= Zero::zero() {
                Bounded::max_value()
            } else {
                Bounded::min_value()
            },
        }
    }

    #[inline]
    fn saturating_sub(self, v: T) -> T {
        match self.checked_sub(&v) {
            Some(x) => x,
            None => if v >= Zero::zero() {
                Bounded::min_value()
            } else {
                Bounded::max_value()
            },
        }
    }
}

/// Performs addition that returns `None` instead of wrapping around on
/// overflow.
pub trait CheckedAdd: Sized + Add<Self, Output = Self> {
    /// Adds two numbers, checking for overflow. If overflow happens, `None` is
    /// returned.
    fn checked_add(&self, v: &Self) -> Option<Self>;
}

macro_rules! checked_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        impl $trait_name for $t {
            #[inline]
            fn $method(&self, v: &$t) -> Option<$t> {
                <$t>::$method(*self, *v)
            }
        }
    }
}

checked_impl!(CheckedAdd, checked_add, u8);
checked_impl!(CheckedAdd, checked_add, u16);
checked_impl!(CheckedAdd, checked_add, u32);
checked_impl!(CheckedAdd, checked_add, u64);
checked_impl!(CheckedAdd, checked_add, usize);

checked_impl!(CheckedAdd, checked_add, i8);
checked_impl!(CheckedAdd, checked_add, i16);
checked_impl!(CheckedAdd, checked_add, i32);
checked_impl!(CheckedAdd, checked_add, i64);
checked_impl!(CheckedAdd, checked_add, isize);

/// Performs subtraction that returns `None` instead of wrapping around on underflow.
pub trait CheckedSub: Sized + Sub<Self, Output = Self> {
    /// Subtracts two numbers, checking for underflow. If underflow happens,
    /// `None` is returned.
    fn checked_sub(&self, v: &Self) -> Option<Self>;
}

checked_impl!(CheckedSub, checked_sub, u8);
checked_impl!(CheckedSub, checked_sub, u16);
checked_impl!(CheckedSub, checked_sub, u32);
checked_impl!(CheckedSub, checked_sub, u64);
checked_impl!(CheckedSub, checked_sub, usize);

checked_impl!(CheckedSub, checked_sub, i8);
checked_impl!(CheckedSub, checked_sub, i16);
checked_impl!(CheckedSub, checked_sub, i32);
checked_impl!(CheckedSub, checked_sub, i64);
checked_impl!(CheckedSub, checked_sub, isize);

/// Performs multiplication that returns `None` instead of wrapping around on underflow or
/// overflow.
pub trait CheckedMul: Sized + Mul<Self, Output = Self> {
    /// Multiplies two numbers, checking for underflow or overflow. If underflow
    /// or overflow happens, `None` is returned.
    fn checked_mul(&self, v: &Self) -> Option<Self>;
}

checked_impl!(CheckedMul, checked_mul, u8);
checked_impl!(CheckedMul, checked_mul, u16);
checked_impl!(CheckedMul, checked_mul, u32);
checked_impl!(CheckedMul, checked_mul, u64);
checked_impl!(CheckedMul, checked_mul, usize);

checked_impl!(CheckedMul, checked_mul, i8);
checked_impl!(CheckedMul, checked_mul, i16);
checked_impl!(CheckedMul, checked_mul, i32);
checked_impl!(CheckedMul, checked_mul, i64);
checked_impl!(CheckedMul, checked_mul, isize);

/// Performs division that returns `None` instead of panicking on division by zero and instead of
/// wrapping around on underflow and overflow.
pub trait CheckedDiv: Sized + Div<Self, Output = Self> {
    /// Divides two numbers, checking for underflow, overflow and division by
    /// zero. If any of that happens, `None` is returned.
    fn checked_div(&self, v: &Self) -> Option<Self>;
}

macro_rules! checkeddiv_int_impl {
    ($t:ty, $min:expr) => {
        impl CheckedDiv for $t {
            #[inline]
            fn checked_div(&self, v: &$t) -> Option<$t> {
                if *v == 0 || (*self == $min && *v == -1) {
                    None
                } else {
                    Some(*self / *v)
                }
            }
        }
    }
}

checkeddiv_int_impl!(isize, isize::MIN);
checkeddiv_int_impl!(i8, i8::MIN);
checkeddiv_int_impl!(i16, i16::MIN);
checkeddiv_int_impl!(i32, i32::MIN);
checkeddiv_int_impl!(i64, i64::MIN);

macro_rules! checkeddiv_uint_impl {
    ($($t:ty)*) => ($(
        impl CheckedDiv for $t {
            #[inline]
            fn checked_div(&self, v: &$t) -> Option<$t> {
                if *v == 0 {
                    None
                } else {
                    Some(*self / *v)
                }
            }
        }
    )*)
}

checkeddiv_uint_impl!(usize u8 u16 u32 u64);

pub trait PrimInt
    : Sized
    + Copy
    + Num + NumCast
    + Bounded
    + PartialOrd + Ord + Eq
    + Not<Output=Self>
    + BitAnd<Output=Self>
    + BitOr<Output=Self>
    + BitXor<Output=Self>
    + Shl<usize, Output=Self>
    + Shr<usize, Output=Self>
    + CheckedAdd<Output=Self>
    + CheckedSub<Output=Self>
    + CheckedMul<Output=Self>
    + CheckedDiv<Output=Self>
    + Saturating
{
    /// Returns the number of ones in the binary representation of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0b01001100u8;
    ///
    /// assert_eq!(n.count_ones(), 3);
    /// ```
    fn count_ones(self) -> u32;

    /// Returns the number of zeros in the binary representation of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0b01001100u8;
    ///
    /// assert_eq!(n.count_zeros(), 5);
    /// ```
    fn count_zeros(self) -> u32;

    /// Returns the number of leading zeros in the binary representation
    /// of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0b0101000u16;
    ///
    /// assert_eq!(n.leading_zeros(), 10);
    /// ```
    fn leading_zeros(self) -> u32;

    /// Returns the number of trailing zeros in the binary representation
    /// of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0b0101000u16;
    ///
    /// assert_eq!(n.trailing_zeros(), 3);
    /// ```
    fn trailing_zeros(self) -> u32;

    /// Shifts the bits to the left by a specified amount amount, `n`, wrapping
    /// the truncated bits to the end of the resulting integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    /// let m = 0x3456789ABCDEF012u64;
    ///
    /// assert_eq!(n.rotate_left(12), m);
    /// ```
    fn rotate_left(self, n: u32) -> Self;

    /// Shifts the bits to the right by a specified amount amount, `n`, wrapping
    /// the truncated bits to the beginning of the resulting integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    /// let m = 0xDEF0123456789ABCu64;
    ///
    /// assert_eq!(n.rotate_right(12), m);
    /// ```
    fn rotate_right(self, n: u32) -> Self;

    /// Reverses the byte order of the integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    /// let m = 0xEFCDAB8967452301u64;
    ///
    /// assert_eq!(n.swap_bytes(), m);
    /// ```
    fn swap_bytes(self) -> Self;

    /// Convert an integer from big endian to the target's endianness.
    ///
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    ///
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(u64::from_be(n), n)
    /// } else {
    ///     assert_eq!(u64::from_be(n), n.swap_bytes())
    /// }
    /// ```
    fn from_be(x: Self) -> Self;

    /// Convert an integer from little endian to the target's endianness.
    ///
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    ///
    /// if cfg!(target_endian = "little") {
    ///     assert_eq!(u64::from_le(n), n)
    /// } else {
    ///     assert_eq!(u64::from_le(n), n.swap_bytes())
    /// }
    /// ```
    fn from_le(x: Self) -> Self;

    /// Convert `self` to big endian from the target's endianness.
    ///
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    ///
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(n.to_be(), n)
    /// } else {
    ///     assert_eq!(n.to_be(), n.swap_bytes())
    /// }
    /// ```
    fn to_be(self) -> Self;

    /// Convert `self` to little endian from the target's endianness.
    ///
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// let n = 0x0123456789ABCDEFu64;
    ///
    /// if cfg!(target_endian = "little") {
    ///     assert_eq!(n.to_le(), n)
    /// } else {
    ///     assert_eq!(n.to_le(), n.swap_bytes())
    /// }
    /// ```
    fn to_le(self) -> Self;

    /// Raises self to the power of `exp`, using exponentiation by squaring.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::traits::PrimInt;
    ///
    /// assert_eq!(2i32.pow(4), 16);
    /// ```
    fn pow(self, mut exp: u32) -> Self;
}

macro_rules! prim_int_impl {
    ($($T:ty)*) => ($(
        impl PrimInt for $T {
            fn count_ones(self) -> u32 {
                <$T>::count_ones(self)
            }

            fn count_zeros(self) -> u32 {
                <$T>::count_zeros(self)
            }

            fn leading_zeros(self) -> u32 {
                <$T>::leading_zeros(self)
            }

            fn trailing_zeros(self) -> u32 {
                <$T>::trailing_zeros(self)
            }

            fn rotate_left(self, n: u32) -> Self {
                <$T>::rotate_left(self, n)
            }

            fn rotate_right(self, n: u32) -> Self {
                <$T>::rotate_right(self, n)
            }

            fn swap_bytes(self) -> Self {
                <$T>::swap_bytes(self)
            }

            fn from_be(x: Self) -> Self {
                <$T>::from_be(x)
            }

            fn from_le(x: Self) -> Self {
                <$T>::from_le(x)
            }

            fn to_be(self) -> Self {
                <$T>::to_be(self)
            }

            fn to_le(self) -> Self {
                <$T>::to_le(self)
            }

            fn pow(self, exp: u32) -> Self {
                <$T>::pow(self, exp)
            }
        }
    )*)
}

prim_int_impl!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);

/// A generic trait for converting a value to a number.
pub trait ToPrimitive {
    /// Converts the value of `self` to an `isize`.
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        self.to_i64().and_then(|x| x.to_isize())
    }

    /// Converts the value of `self` to an `i8`.
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        self.to_i64().and_then(|x| x.to_i8())
    }

    /// Converts the value of `self` to an `i16`.
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        self.to_i64().and_then(|x| x.to_i16())
    }

    /// Converts the value of `self` to an `i32`.
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        self.to_i64().and_then(|x| x.to_i32())
    }

    /// Converts the value of `self` to an `i64`.
    fn to_i64(&self) -> Option<i64>;

    /// Converts the value of `self` to a `usize`.
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        self.to_u64().and_then(|x| x.to_usize())
    }

    /// Converts the value of `self` to an `u8`.
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        self.to_u64().and_then(|x| x.to_u8())
    }

    /// Converts the value of `self` to an `u16`.
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        self.to_u64().and_then(|x| x.to_u16())
    }

    /// Converts the value of `self` to an `u32`.
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        self.to_u64().and_then(|x| x.to_u32())
    }

    /// Converts the value of `self` to an `u64`.
    #[inline]
    fn to_u64(&self) -> Option<u64>;

    /// Converts the value of `self` to an `f32`.
    #[inline]
    fn to_f32(&self) -> Option<f32> {
        self.to_f64().and_then(|x| x.to_f32())
    }

    /// Converts the value of `self` to an `f64`.
    #[inline]
    fn to_f64(&self) -> Option<f64> {
        self.to_i64().and_then(|x| x.to_f64())
    }
}

macro_rules! impl_to_primitive_int_to_int {
    ($SrcT:ty, $DstT:ty, $slf:expr) => (
        {
            if size_of::<$SrcT>() <= size_of::<$DstT>() {
                Some($slf as $DstT)
            } else {
                let n = $slf as i64;
                let min_value: $DstT = Bounded::min_value();
                let max_value: $DstT = Bounded::max_value();
                if min_value as i64 <= n && n <= max_value as i64 {
                    Some($slf as $DstT)
                } else {
                    None
                }
            }
        }
    )
}

macro_rules! impl_to_primitive_int_to_uint {
    ($SrcT:ty, $DstT:ty, $slf:expr) => (
        {
            let zero: $SrcT = Zero::zero();
            let max_value: $DstT = Bounded::max_value();
            if zero <= $slf && $slf as u64 <= max_value as u64 {
                Some($slf as $DstT)
            } else {
                None
            }
        }
    )
}

macro_rules! impl_to_primitive_int {
    ($T:ty) => (
        impl ToPrimitive for $T {
            #[inline]
            fn to_isize(&self) -> Option<isize> { impl_to_primitive_int_to_int!($T, isize, *self) }
            #[inline]
            fn to_i8(&self) -> Option<i8> { impl_to_primitive_int_to_int!($T, i8, *self) }
            #[inline]
            fn to_i16(&self) -> Option<i16> { impl_to_primitive_int_to_int!($T, i16, *self) }
            #[inline]
            fn to_i32(&self) -> Option<i32> { impl_to_primitive_int_to_int!($T, i32, *self) }
            #[inline]
            fn to_i64(&self) -> Option<i64> { impl_to_primitive_int_to_int!($T, i64, *self) }

            #[inline]
            fn to_usize(&self) -> Option<usize> { impl_to_primitive_int_to_uint!($T, usize, *self) }
            #[inline]
            fn to_u8(&self) -> Option<u8> { impl_to_primitive_int_to_uint!($T, u8, *self) }
            #[inline]
            fn to_u16(&self) -> Option<u16> { impl_to_primitive_int_to_uint!($T, u16, *self) }
            #[inline]
            fn to_u32(&self) -> Option<u32> { impl_to_primitive_int_to_uint!($T, u32, *self) }
            #[inline]
            fn to_u64(&self) -> Option<u64> { impl_to_primitive_int_to_uint!($T, u64, *self) }

            #[inline]
            fn to_f32(&self) -> Option<f32> { Some(*self as f32) }
            #[inline]
            fn to_f64(&self) -> Option<f64> { Some(*self as f64) }
        }
    )
}

impl_to_primitive_int! { isize }
impl_to_primitive_int! { i8 }
impl_to_primitive_int! { i16 }
impl_to_primitive_int! { i32 }
impl_to_primitive_int! { i64 }

macro_rules! impl_to_primitive_uint_to_int {
    ($DstT:ty, $slf:expr) => (
        {
            let max_value: $DstT = Bounded::max_value();
            if $slf as u64 <= max_value as u64 {
                Some($slf as $DstT)
            } else {
                None
            }
        }
    )
}

macro_rules! impl_to_primitive_uint_to_uint {
    ($SrcT:ty, $DstT:ty, $slf:expr) => (
        {
            if size_of::<$SrcT>() <= size_of::<$DstT>() {
                Some($slf as $DstT)
            } else {
                let zero: $SrcT = Zero::zero();
                let max_value: $DstT = Bounded::max_value();
                if zero <= $slf && $slf as u64 <= max_value as u64 {
                    Some($slf as $DstT)
                } else {
                    None
                }
            }
        }
    )
}

macro_rules! impl_to_primitive_uint {
    ($T:ty) => (
        impl ToPrimitive for $T {
            #[inline]
            fn to_isize(&self) -> Option<isize> { impl_to_primitive_uint_to_int!(isize, *self) }
            #[inline]
            fn to_i8(&self) -> Option<i8> { impl_to_primitive_uint_to_int!(i8, *self) }
            #[inline]
            fn to_i16(&self) -> Option<i16> { impl_to_primitive_uint_to_int!(i16, *self) }
            #[inline]
            fn to_i32(&self) -> Option<i32> { impl_to_primitive_uint_to_int!(i32, *self) }
            #[inline]
            fn to_i64(&self) -> Option<i64> { impl_to_primitive_uint_to_int!(i64, *self) }

            #[inline]
            fn to_usize(&self) -> Option<usize> {
                impl_to_primitive_uint_to_uint!($T, usize, *self)
            }
            #[inline]
            fn to_u8(&self) -> Option<u8> { impl_to_primitive_uint_to_uint!($T, u8, *self) }
            #[inline]
            fn to_u16(&self) -> Option<u16> { impl_to_primitive_uint_to_uint!($T, u16, *self) }
            #[inline]
            fn to_u32(&self) -> Option<u32> { impl_to_primitive_uint_to_uint!($T, u32, *self) }
            #[inline]
            fn to_u64(&self) -> Option<u64> { impl_to_primitive_uint_to_uint!($T, u64, *self) }

            #[inline]
            fn to_f32(&self) -> Option<f32> { Some(*self as f32) }
            #[inline]
            fn to_f64(&self) -> Option<f64> { Some(*self as f64) }
        }
    )
}

impl_to_primitive_uint! { usize }
impl_to_primitive_uint! { u8 }
impl_to_primitive_uint! { u16 }
impl_to_primitive_uint! { u32 }
impl_to_primitive_uint! { u64 }

macro_rules! impl_to_primitive_float_to_float {
    ($SrcT:ident, $DstT:ident, $slf:expr) => (
        if size_of::<$SrcT>() <= size_of::<$DstT>() {
            Some($slf as $DstT)
        } else {
            let n = $slf as f64;
            let max_value: $SrcT = ::redox::$SrcT::MAX;
            if -max_value as f64 <= n && n <= max_value as f64 {
                Some($slf as $DstT)
            } else {
                None
            }
        }
    )
}

macro_rules! impl_to_primitive_float {
    ($T:ident) => (
        impl ToPrimitive for $T {
            #[inline]
            fn to_isize(&self) -> Option<isize> { Some(*self as isize) }
            #[inline]
            fn to_i8(&self) -> Option<i8> { Some(*self as i8) }
            #[inline]
            fn to_i16(&self) -> Option<i16> { Some(*self as i16) }
            #[inline]
            fn to_i32(&self) -> Option<i32> { Some(*self as i32) }
            #[inline]
            fn to_i64(&self) -> Option<i64> { Some(*self as i64) }

            #[inline]
            fn to_usize(&self) -> Option<usize> { Some(*self as usize) }
            #[inline]
            fn to_u8(&self) -> Option<u8> { Some(*self as u8) }
            #[inline]
            fn to_u16(&self) -> Option<u16> { Some(*self as u16) }
            #[inline]
            fn to_u32(&self) -> Option<u32> { Some(*self as u32) }
            #[inline]
            fn to_u64(&self) -> Option<u64> { Some(*self as u64) }

            #[inline]
            fn to_f32(&self) -> Option<f32> { impl_to_primitive_float_to_float!($T, f32, *self) }
            #[inline]
            fn to_f64(&self) -> Option<f64> { impl_to_primitive_float_to_float!($T, f64, *self) }
        }
    )
}

impl_to_primitive_float! { f32 }
impl_to_primitive_float! { f64 }

/// A generic trait for converting a number to a value.
pub trait FromPrimitive: Sized {
    /// Convert an `isize` to return an optional value of this type. If the
    /// value cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        FromPrimitive::from_i64(n as i64)
    }

    /// Convert an `i8` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        FromPrimitive::from_i64(n as i64)
    }

    /// Convert an `i16` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        FromPrimitive::from_i64(n as i64)
    }

    /// Convert an `i32` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        FromPrimitive::from_i64(n as i64)
    }

    /// Convert an `i64` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    fn from_i64(n: i64) -> Option<Self>;

    /// Convert a `usize` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        FromPrimitive::from_u64(n as u64)
    }

    /// Convert an `u8` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        FromPrimitive::from_u64(n as u64)
    }

    /// Convert an `u16` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        FromPrimitive::from_u64(n as u64)
    }

    /// Convert an `u32` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        FromPrimitive::from_u64(n as u64)
    }

    /// Convert an `u64` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    fn from_u64(n: u64) -> Option<Self>;

    /// Convert a `f32` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        FromPrimitive::from_f64(n as f64)
    }

    /// Convert a `f64` to return an optional value of this type. If the
    /// type cannot be represented by this value, the `None` is returned.
    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        FromPrimitive::from_i64(n as i64)
    }
}

macro_rules! impl_from_primitive {
    ($T:ty, $to_ty:ident) => (
        #[allow(deprecated)]
        impl FromPrimitive for $T {
            #[inline] fn from_i8(n: i8) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i16(n: i16) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i32(n: i32) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i64(n: i64) -> Option<$T> { n.$to_ty() }

            #[inline] fn from_u8(n: u8) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u16(n: u16) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u32(n: u32) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u64(n: u64) -> Option<$T> { n.$to_ty() }

            #[inline] fn from_f32(n: f32) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_f64(n: f64) -> Option<$T> { n.$to_ty() }
        }
    )
}

impl_from_primitive! { isize, to_isize }
impl_from_primitive! { i8, to_i8 }
impl_from_primitive! { i16, to_i16 }
impl_from_primitive! { i32, to_i32 }
impl_from_primitive! { i64, to_i64 }
impl_from_primitive! { usize, to_usize }
impl_from_primitive! { u8, to_u8 }
impl_from_primitive! { u16, to_u16 }
impl_from_primitive! { u32, to_u32 }
impl_from_primitive! { u64, to_u64 }
impl_from_primitive! { f32, to_f32 }
impl_from_primitive! { f64, to_f64 }

/// Cast from one machine scalar to another.
///
/// # Examples
///
/// ```
/// use num;
///
/// let twenty: f32 = num::cast(0x14).unwrap();
/// assert_eq!(twenty, 20f32);
/// ```
///
#[inline]
pub fn cast<T: NumCast, U: NumCast>(n: T) -> Option<U> {
    NumCast::from(n)
}

/// An interface for casting between machine scalars.
pub trait NumCast: Sized + ToPrimitive {
    /// Creates a number from another value that can be converted into
    /// a primitive via the `ToPrimitive` trait.
    fn from<T: ToPrimitive>(n: T) -> Option<Self>;
}

macro_rules! impl_num_cast {
    ($T:ty, $conv:ident) => (
        impl NumCast for $T {
            #[inline]
            #[allow(deprecated)]
            fn from<N: ToPrimitive>(n: N) -> Option<$T> {
                // `$conv` could be generated using `concat_idents!`, but that
                // macro seems to be broken at the moment
                n.$conv()
            }
        }
    )
}

impl_num_cast! { u8,    to_u8 }
impl_num_cast! { u16,   to_u16 }
impl_num_cast! { u32,   to_u32 }
impl_num_cast! { u64,   to_u64 }
impl_num_cast! { usize,  to_usize }
impl_num_cast! { i8,    to_i8 }
impl_num_cast! { i16,   to_i16 }
impl_num_cast! { i32,   to_i32 }
impl_num_cast! { i64,   to_i64 }
impl_num_cast! { isize,   to_isize }
impl_num_cast! { f32,   to_f32 }
impl_num_cast! { f64,   to_f64 }

pub trait Float
    : Num
    + Copy
    + NumCast
    + PartialOrd
    + Neg<Output = Self>
{
    /// Returns the `NaN` value.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let nan: f32 = Float::nan();
    ///
    /// assert!(nan.is_nan());
    /// ```
    fn nan() -> Self;
    /// Returns the infinite value.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f32;
    ///
    /// let infinity: f32 = Float::infinity();
    ///
    /// assert!(infinity.is_infinite());
    /// assert!(!infinity.is_finite());
    /// assert!(infinity > f32::MAX);
    /// ```
    fn infinity() -> Self;
    /// Returns the negative infinite value.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f32;
    ///
    /// let neg_infinity: f32 = Float::neg_infinity();
    ///
    /// assert!(neg_infinity.is_infinite());
    /// assert!(!neg_infinity.is_finite());
    /// assert!(neg_infinity < f32::MIN);
    /// ```
    fn neg_infinity() -> Self;
    /// Returns `-0.0`.
    ///
    /// ```
    /// use num::traits::{Zero, Float};
    ///
    /// let inf: f32 = Float::infinity();
    /// let zero: f32 = Zero::zero();
    /// let neg_zero: f32 = Float::neg_zero();
    ///
    /// assert_eq!(zero, neg_zero);
    /// assert_eq!(7.0f32/inf, zero);
    /// assert_eq!(zero * 10.0, zero);
    /// ```
    fn neg_zero() -> Self;

    /// Returns the smallest finite value that this type can represent.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x: f64 = Float::min_value();
    ///
    /// assert_eq!(x, f64::MIN);
    /// ```
    fn min_value() -> Self;

    /// Returns the smallest positive, normalized value that this type can represent.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x: f64 = Float::min_positive_value();
    ///
    /// assert_eq!(x, f64::MIN_POSITIVE);
    /// ```
    fn min_positive_value() -> Self;

    /// Returns the largest finite value that this type can represent.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x: f64 = Float::max_value();
    /// assert_eq!(x, f64::MAX);
    /// ```
    fn max_value() -> Self;

    /// Returns `true` if this value is `NaN` and false otherwise.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let nan = f64::NAN;
    /// let f = 7.0;
    ///
    /// assert!(nan.is_nan());
    /// assert!(!f.is_nan());
    /// ```
    fn is_nan(self) -> bool;

    /// Returns `true` if this value is positive infinity or negative infinity and
    /// false otherwise.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f32;
    ///
    /// let f = 7.0f32;
    /// let inf: f32 = Float::infinity();
    /// let neg_inf: f32 = Float::neg_infinity();
    /// let nan: f32 = f32::NAN;
    ///
    /// assert!(!f.is_infinite());
    /// assert!(!nan.is_infinite());
    ///
    /// assert!(inf.is_infinite());
    /// assert!(neg_inf.is_infinite());
    /// ```
    fn is_infinite(self) -> bool;

    /// Returns `true` if this number is neither infinite nor `NaN`.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f32;
    ///
    /// let f = 7.0f32;
    /// let inf: f32 = Float::infinity();
    /// let neg_inf: f32 = Float::neg_infinity();
    /// let nan: f32 = f32::NAN;
    ///
    /// assert!(f.is_finite());
    ///
    /// assert!(!nan.is_finite());
    /// assert!(!inf.is_finite());
    /// assert!(!neg_inf.is_finite());
    /// ```
    fn is_finite(self) -> bool;

    /// Returns `true` if the number is neither zero, infinite,
    /// [subnormal][subnormal], or `NaN`.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f32;
    ///
    /// let min = f32::MIN_POSITIVE; // 1.17549435e-38f32
    /// let max = f32::MAX;
    /// let lower_than_min = 1.0e-40_f32;
    /// let zero = 0.0f32;
    ///
    /// assert!(min.is_normal());
    /// assert!(max.is_normal());
    ///
    /// assert!(!zero.is_normal());
    /// assert!(!f32::NAN.is_normal());
    /// assert!(!f32::INFINITY.is_normal());
    /// // Values between `0` and `min` are Subnormal.
    /// assert!(!lower_than_min.is_normal());
    /// ```
    /// [subnormal]: http://en.wikipedia.org/wiki/Denormal_number
    fn is_normal(self) -> bool;

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::num::FpCategory;
    /// use redox::f32;
    ///
    /// let num = 12.4f32;
    /// let inf = f32::INFINITY;
    ///
    /// assert_eq!(num.classify(), FpCategory::Normal);
    /// assert_eq!(inf.classify(), FpCategory::Infinite);
    /// ```
    fn classify(self) -> FpCategory;

    /// Returns the largest integer less than or equal to a number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let f = 3.99;
    /// let g = 3.0;
    ///
    /// assert_eq!(f.floor(), 3.0);
    /// assert_eq!(g.floor(), 3.0);
    /// ```
    fn floor(self) -> Self;

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let f = 3.01;
    /// let g = 4.0;
    ///
    /// assert_eq!(f.ceil(), 4.0);
    /// assert_eq!(g.ceil(), 4.0);
    /// ```
    fn ceil(self) -> Self;

    /// Returns the nearest integer to a number. Round half-way cases away from
    /// `0.0`.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let f = 3.3;
    /// let g = -3.3;
    ///
    /// assert_eq!(f.round(), 3.0);
    /// assert_eq!(g.round(), -3.0);
    /// ```
    fn round(self) -> Self;

    /// Return the integer part of a number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let f = 3.3;
    /// let g = -3.7;
    ///
    /// assert_eq!(f.trunc(), 3.0);
    /// assert_eq!(g.trunc(), -3.0);
    /// ```
    fn trunc(self) -> Self;

    /// Returns the fractional part of a number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 3.5;
    /// let y = -3.5;
    /// let abs_difference_x = (x.fract() - 0.5).abs();
    /// let abs_difference_y = (y.fract() - (-0.5)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    fn fract(self) -> Self;

    /// Computes the absolute value of `self`. Returns `Float::nan()` if the
    /// number is `Float::nan()`.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x = 3.5;
    /// let y = -3.5;
    ///
    /// let abs_difference_x = (x.abs() - x).abs();
    /// let abs_difference_y = (y.abs() - (-y)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    ///
    /// assert!(f64::NAN.abs().is_nan());
    /// ```
    fn abs(self) -> Self;

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `Float::infinity()`
    /// - `-1.0` if the number is negative, `-0.0` or `Float::neg_infinity()`
    /// - `Float::nan()` if the number is `Float::nan()`
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let f = 3.5;
    ///
    /// assert_eq!(f.signum(), 1.0);
    /// assert_eq!(f64::NEG_INFINITY.signum(), -1.0);
    ///
    /// assert!(f64::NAN.signum().is_nan());
    /// ```
    fn signum(self) -> Self;

    /// Returns `true` if `self` is positive, including `+0.0` and
    /// `Float::infinity()`.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let nan: f64 = f64::NAN;
    ///
    /// let f = 7.0;
    /// let g = -7.0;
    ///
    /// assert!(f.is_sign_positive());
    /// assert!(!g.is_sign_positive());
    /// // Requires both tests to determine if is `NaN`
    /// assert!(!nan.is_sign_positive() && !nan.is_sign_negative());
    /// ```
    fn is_sign_positive(self) -> bool;

    /// Returns `true` if `self` is negative, including `-0.0` and
    /// `Float::neg_infinity()`.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let nan = f64::NAN;
    ///
    /// let f = 7.0;
    /// let g = -7.0;
    ///
    /// assert!(!f.is_sign_negative());
    /// assert!(g.is_sign_negative());
    /// // Requires both tests to determine if is `NaN`.
    /// assert!(!nan.is_sign_positive() && !nan.is_sign_negative());
    /// ```
    fn is_sign_negative(self) -> bool;

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error. This produces a more accurate result with better performance than
    /// a separate multiplication operation followed by an add.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let m = 10.0;
    /// let x = 4.0;
    /// let b = 60.0;
    ///
    /// // 100.0
    /// let abs_difference = (m.mul_add(x, b) - (m*x + b)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn mul_add(self, a: Self, b: Self) -> Self;
    /// Take the reciprocal (inverse) of a number, `1/x`.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 2.0;
    /// let abs_difference = (x.recip() - (1.0/x)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn recip(self) -> Self;

    /// Raise a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 2.0;
    /// let abs_difference = (x.powi(2) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn powi(self, n: i32) -> Self;

    /// Raise a number to a floating point power.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 2.0;
    /// let abs_difference = (x.powf(2.0) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn powf(self, n: Self) -> Self;

    /// Take the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let positive = 4.0;
    /// let negative = -4.0;
    ///
    /// let abs_difference = (positive.sqrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// assert!(negative.sqrt().is_nan());
    /// ```
    fn sqrt(self) -> Self;

    /// Returns `e^(self)`, (the exponential function).
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let one = 1.0;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn exp(self) -> Self;

    /// Returns `2^(self)`.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let f = 2.0;
    ///
    /// // 2^2 - 4 == 0
    /// let abs_difference = (f.exp2() - 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn exp2(self) -> Self;

    /// Returns the natural logarithm of the number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let one = 1.0;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn ln(self) -> Self;

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let ten = 10.0;
    /// let two = 2.0;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference_10 = (ten.log(10.0) - 1.0).abs();
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference_2 = (two.log(2.0) - 1.0).abs();
    ///
    /// assert!(abs_difference_10 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    fn log(self, base: Self) -> Self;

    /// Returns the base 2 logarithm of the number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let two = 2.0;
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference = (two.log2() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn log2(self) -> Self;

    /// Returns the base 10 logarithm of the number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let ten = 10.0;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference = (ten.log10() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn log10(self) -> Self;

    /// Returns the maximum of the two numbers.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 1.0;
    /// let y = 2.0;
    ///
    /// assert_eq!(x.max(y), y);
    /// ```
    fn max(self, other: Self) -> Self;

    /// Returns the minimum of the two numbers.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 1.0;
    /// let y = 2.0;
    ///
    /// assert_eq!(x.min(y), x);
    /// ```
    fn min(self, other: Self) -> Self;

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0:0`
    /// * Else: `self - other`
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 3.0;
    /// let y = -3.0;
    ///
    /// let abs_difference_x = (x.abs_sub(1.0) - 2.0).abs();
    /// let abs_difference_y = (y.abs_sub(1.0) - 0.0).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    fn abs_sub(self, other: Self) -> Self;

    /// Take the cubic root of a number.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 8.0;
    ///
    /// // x^(1/3) - 2 == 0
    /// let abs_difference = (x.cbrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn cbrt(self) -> Self;

    /// Calculate the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 2.0;
    /// let y = 3.0;
    ///
    /// // sqrt(x^2 + y^2)
    /// let abs_difference = (x.hypot(y) - (x.powi(2) + y.powi(2)).sqrt()).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn hypot(self, other: Self) -> Self;

    /// Computes the sine of a number (in radians).
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x = f64::consts::PI/2.0;
    ///
    /// let abs_difference = (x.sin() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn sin(self) -> Self;

    /// Computes the cosine of a number (in radians).
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x = 2.0*f64::consts::PI;
    ///
    /// let abs_difference = (x.cos() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn cos(self) -> Self;

    /// Computes the tangent of a number (in radians).
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let abs_difference = (x.tan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-14);
    /// ```
    fn tan(self) -> Self;

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let f = f64::consts::PI / 2.0;
    ///
    /// // asin(sin(pi/2))
    /// let abs_difference = (f.sin().asin() - f64::consts::PI / 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn asin(self) -> Self;

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let f = f64::consts::PI / 4.0;
    ///
    /// // acos(cos(pi/4))
    /// let abs_difference = (f.cos().acos() - f64::consts::PI / 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn acos(self) -> Self;

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let f = 1.0;
    ///
    /// // atan(tan(1))
    /// let abs_difference = (f.tan().atan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn atan(self) -> Self;

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`).
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let pi = f64::consts::PI;
    /// // All angles from horizontal right (+x)
    /// // 45 deg counter-clockwise
    /// let x1 = 3.0;
    /// let y1 = -3.0;
    ///
    /// // 135 deg clockwise
    /// let x2 = -3.0;
    /// let y2 = 3.0;
    ///
    /// let abs_difference_1 = (y1.atan2(x1) - (-pi/4.0)).abs();
    /// let abs_difference_2 = (y2.atan2(x2) - 3.0*pi/4.0).abs();
    ///
    /// assert!(abs_difference_1 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    fn atan2(self, other: Self) -> Self;

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let f = x.sin_cos();
    ///
    /// let abs_difference_0 = (f.0 - x.sin()).abs();
    /// let abs_difference_1 = (f.1 - x.cos()).abs();
    ///
    /// assert!(abs_difference_0 < 1e-10);
    /// assert!(abs_difference_0 < 1e-10);
    /// ```
    fn sin_cos(self) -> (Self, Self);

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 7.0;
    ///
    /// // e^(ln(7)) - 1
    /// let abs_difference = (x.ln().exp_m1() - 6.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn exp_m1(self) -> Self;

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let x = f64::consts::E - 1.0;
    ///
    /// // ln(1 + (e - 1)) == ln(e) == 1
    /// let abs_difference = (x.ln_1p() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn ln_1p(self) -> Self;

    /// Hyperbolic sine function.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0;
    ///
    /// let f = x.sinh();
    /// // Solving sinh() at 1 gives `(e^2-1)/(2e)`
    /// let g = (e*e - 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn sinh(self) -> Self;

    /// Hyperbolic cosine function.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0;
    /// let f = x.cosh();
    /// // Solving cosh() at 1 gives this result
    /// let g = (e*e + 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// // Same result
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn cosh(self) -> Self;

    /// Hyperbolic tangent function.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0;
    ///
    /// let f = x.tanh();
    /// // Solving tanh() at 1 gives `(1 - e^(-2))/(1 + e^(-2))`
    /// let g = (1.0 - e.powi(-2))/(1.0 + e.powi(-2));
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn tanh(self) -> Self;

    /// Inverse hyperbolic sine function.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 1.0;
    /// let f = x.sinh().asinh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn asinh(self) -> Self;

    /// Inverse hyperbolic cosine function.
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let x = 1.0;
    /// let f = x.cosh().acosh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn acosh(self) -> Self;

    /// Inverse hyperbolic tangent function.
    ///
    /// ```
    /// use num::traits::Float;
    /// use redox::f64;
    ///
    /// let e = f64::consts::E;
    /// let f = e.tanh().atanh();
    ///
    /// let abs_difference = (f - e).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn atanh(self) -> Self;


    /// Returns the mantissa, base 2 exponent, and sign as integers, respectively.
    /// The original number can be recovered by `sign * mantissa * 2 ^ exponent`.
    /// The floating point encoding is documented in the [Reference][floating-point].
    ///
    /// ```
    /// use num::traits::Float;
    ///
    /// let num = 2.0f32;
    ///
    /// // (8388608, -22, 1)
    /// let (mantissa, exponent, sign) = Float::integer_decode(num);
    /// let sign_f = sign as f32;
    /// let mantissa_f = mantissa as f32;
    /// let exponent_f = num.powf(exponent as f32);
    ///
    /// // 1 * 8388608 * 2^(-22) == 2
    /// let abs_difference = (sign_f * mantissa_f * exponent_f - num).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    /// [floating-point]: ../../../../../reference.html#machine-types
    fn integer_decode(self) -> (u64, i16, i8);
}

macro_rules! float_impl {
    ($T:ident $decode:ident) => (
        impl Float for $T {
            fn nan() -> Self {
                ::redox::$T::NAN
            }

            fn infinity() -> Self {
                ::redox::$T::INFINITY
            }

            fn neg_infinity() -> Self {
                ::redox::$T::NEG_INFINITY
            }

            fn neg_zero() -> Self {
                -0.0
            }

            fn min_value() -> Self {
                ::redox::$T::MIN
            }

            fn min_positive_value() -> Self {
                ::redox::$T::MIN_POSITIVE
            }

            fn max_value() -> Self {
                ::redox::$T::MAX
            }

            fn is_nan(self) -> bool {
                <$T>::is_nan(self)
            }

            fn is_infinite(self) -> bool {
                <$T>::is_infinite(self)
            }

            fn is_finite(self) -> bool {
                <$T>::is_finite(self)
            }

            fn is_normal(self) -> bool {
                <$T>::is_normal(self)
            }

            fn classify(self) -> FpCategory {
                <$T>::classify(self)
            }

            fn floor(self) -> Self {
                <$T>::floor(self)
            }

            fn ceil(self) -> Self {
                <$T>::ceil(self)
            }

            fn round(self) -> Self {
                <$T>::round(self)
            }

            fn trunc(self) -> Self {
                <$T>::trunc(self)
            }

            fn fract(self) -> Self {
                <$T>::fract(self)
            }

            fn abs(self) -> Self {
                <$T>::abs(self)
            }

            fn signum(self) -> Self {
                <$T>::signum(self)
            }

            fn is_sign_positive(self) -> bool {
                <$T>::is_sign_positive(self)
            }

            fn is_sign_negative(self) -> bool {
                <$T>::is_sign_negative(self)
            }

            fn mul_add(self, a: Self, b: Self) -> Self {
                <$T>::mul_add(self, a, b)
            }

            fn recip(self) -> Self {
                <$T>::recip(self)
            }

            fn powi(self, n: i32) -> Self {
                <$T>::powi(self, n)
            }

            fn powf(self, n: Self) -> Self {
                <$T>::powf(self, n)
            }

            fn sqrt(self) -> Self {
                <$T>::sqrt(self)
            }

            fn exp(self) -> Self {
                <$T>::exp(self)
            }

            fn exp2(self) -> Self {
                <$T>::exp2(self)
            }

            fn ln(self) -> Self {
                <$T>::ln(self)
            }

            fn log(self, base: Self) -> Self {
                <$T>::log(self, base)
            }

            fn log2(self) -> Self {
                <$T>::log2(self)
            }

            fn log10(self) -> Self {
                <$T>::log10(self)
            }

            fn max(self, other: Self) -> Self {
                <$T>::max(self, other)
            }

            fn min(self, other: Self) -> Self {
                <$T>::min(self, other)
            }

            fn abs_sub(self, other: Self) -> Self {
                <$T>::abs_sub(self, other)
            }

            fn cbrt(self) -> Self {
                <$T>::cbrt(self)
            }

            fn hypot(self, other: Self) -> Self {
                <$T>::hypot(self, other)
            }

            fn sin(self) -> Self {
                <$T>::sin(self)
            }

            fn cos(self) -> Self {
                <$T>::cos(self)
            }

            fn tan(self) -> Self {
                <$T>::tan(self)
            }

            fn asin(self) -> Self {
                <$T>::asin(self)
            }

            fn acos(self) -> Self {
                <$T>::acos(self)
            }

            fn atan(self) -> Self {
                <$T>::atan(self)
            }

            fn atan2(self, other: Self) -> Self {
                <$T>::atan2(self, other)
            }

            fn sin_cos(self) -> (Self, Self) {
                <$T>::sin_cos(self)
            }

            fn exp_m1(self) -> Self {
                <$T>::exp_m1(self)
            }

            fn ln_1p(self) -> Self {
                <$T>::ln_1p(self)
            }

            fn sinh(self) -> Self {
                <$T>::sinh(self)
            }

            fn cosh(self) -> Self {
                <$T>::cosh(self)
            }

            fn tanh(self) -> Self {
                <$T>::tanh(self)
            }

            fn asinh(self) -> Self {
                <$T>::asinh(self)
            }

            fn acosh(self) -> Self {
                <$T>::acosh(self)
            }

            fn atanh(self) -> Self {
                <$T>::atanh(self)
            }

            fn integer_decode(self) -> (u64, i16, i8) {
                $decode(self)
            }
        }
    )
}

fn integer_decode_f32(f: f32) -> (u64, i16, i8) {
    let bits: u32 = unsafe { mem::transmute(f) };
    let sign: i8 = if bits >> 31 == 0 {
        1
    } else {
        -1
    };
    let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0x7fffff) << 1
    } else {
        (bits & 0x7fffff) | 0x800000
    };
    // Exponent bias + mantissa shift
    exponent -= 127 + 23;
    (mantissa as u64, exponent, sign)
}

fn integer_decode_f64(f: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(f) };
    let sign: i8 = if bits >> 63 == 0 {
        1
    } else {
        -1
    };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    // Exponent bias + mantissa shift
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

float_impl!(f32 integer_decode_f32);
float_impl!(f64 integer_decode_f64);
