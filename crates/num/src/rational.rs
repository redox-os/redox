// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rational numbers

use Integer;

use redox::cmp;
use redox::error::Error;
use redox::fmt;
use redox::ops::{Add, Div, Mul, Neg, Rem, Sub};
use redox::str::FromStr;

#[cfg(feature = "bigint")]
use bigint::{BigInt, BigUint, Sign};
use traits::{FromPrimitive, Float, PrimInt};
use {Num, Signed, Zero, One};

/// Represents the ratio between 2 numbers.
#[derive(Copy, Clone, Hash, Debug)]
#[allow(missing_docs)]
pub struct Ratio<T> {
    numer: T,
    denom: T,
}

/// Alias for a `Ratio` of machine-sized integers.
pub type Rational = Ratio<isize>;
pub type Rational32 = Ratio<i32>;
pub type Rational64 = Ratio<i64>;

#[cfg(feature = "bigint")]
/// Alias for arbitrary precision rationals.
pub type BigRational = Ratio<BigInt>;

impl<T: Clone + Integer + PartialOrd> Ratio<T> {
    /// Creates a ratio representing the integer `t`.
    #[inline]
    pub fn from_integer(t: T) -> Ratio<T> {
        Ratio::new_raw(t, One::one())
    }

    /// Creates a ratio without checking for `denom == 0` or reducing.
    #[inline]
    pub fn new_raw(numer: T, denom: T) -> Ratio<T> {
        Ratio {
            numer: numer,
            denom: denom,
        }
    }

    /// Create a new Ratio. Fails if `denom == 0`.
    #[inline]
    pub fn new(numer: T, denom: T) -> Ratio<T> {
        if denom == Zero::zero() {
            panic!("denominator == 0");
        }
        let mut ret = Ratio::new_raw(numer, denom);
        ret.reduce();
        ret
    }

    /// Converts to an integer.
    #[inline]
    pub fn to_integer(&self) -> T {
        self.trunc().numer
    }

    /// Gets an immutable reference to the numerator.
    #[inline]
    pub fn numer<'a>(&'a self) -> &'a T {
        &self.numer
    }

    /// Gets an immutable reference to the denominator.
    #[inline]
    pub fn denom<'a>(&'a self) -> &'a T {
        &self.denom
    }

    /// Returns true if the rational number is an integer (denominator is 1).
    #[inline]
    pub fn is_integer(&self) -> bool {
        self.denom == One::one()
    }

    /// Put self into lowest terms, with denom > 0.
    fn reduce(&mut self) {
        let g: T = self.numer.gcd(&self.denom);

        // FIXME(#5992): assignment operator overloads
        // self.numer /= g;
        self.numer = self.numer.clone() / g.clone();
        // FIXME(#5992): assignment operator overloads
        // self.denom /= g;
        self.denom = self.denom.clone() / g;

        // keep denom positive!
        if self.denom < T::zero() {
            self.numer = T::zero() - self.numer.clone();
            self.denom = T::zero() - self.denom.clone();
        }
    }

    /// Returns a `reduce`d copy of self.
    pub fn reduced(&self) -> Ratio<T> {
        let mut ret = self.clone();
        ret.reduce();
        ret
    }

    /// Returns the reciprocal.
    #[inline]
    pub fn recip(&self) -> Ratio<T> {
        Ratio::new_raw(self.denom.clone(), self.numer.clone())
    }

    /// Rounds towards minus infinity.
    #[inline]
    pub fn floor(&self) -> Ratio<T> {
        if *self < Zero::zero() {
            let one: T = One::one();
            Ratio::from_integer((self.numer.clone() - self.denom.clone() + one) /
                                self.denom.clone())
        } else {
            Ratio::from_integer(self.numer.clone() / self.denom.clone())
        }
    }

    /// Rounds towards plus infinity.
    #[inline]
    pub fn ceil(&self) -> Ratio<T> {
        if *self < Zero::zero() {
            Ratio::from_integer(self.numer.clone() / self.denom.clone())
        } else {
            let one: T = One::one();
            Ratio::from_integer((self.numer.clone() + self.denom.clone() - one) /
                                self.denom.clone())
        }
    }

    /// Rounds to the nearest integer. Rounds half-way cases away from zero.
    #[inline]
    pub fn round(&self) -> Ratio<T> {
        let zero: Ratio<T> = Zero::zero();
        let one: T = One::one();
        let two: T = one.clone() + one.clone();

        // Find unsigned fractional part of rational number
        let mut fractional = self.fract();
        if fractional < zero {
            fractional = zero - fractional
        };

        // The algorithm compares the unsigned fractional part with 1/2, that
        // is, a/b >= 1/2, or a >= b/2. For odd denominators, we use
        // a >= (b/2)+1. This avoids overflow issues.
        let half_or_larger = if fractional.denom().is_even() {
            *fractional.numer() >= fractional.denom().clone() / two.clone()
        } else {
            *fractional.numer() >= (fractional.denom().clone() / two.clone()) + one.clone()
        };

        if half_or_larger {
            let one: Ratio<T> = One::one();
            if *self >= Zero::zero() {
                self.trunc() + one
            } else {
                self.trunc() - one
            }
        } else {
            self.trunc()
        }
    }

    /// Rounds towards zero.
    #[inline]
    pub fn trunc(&self) -> Ratio<T> {
        Ratio::from_integer(self.numer.clone() / self.denom.clone())
    }

    /// Returns the fractional part of a number.
    #[inline]
    pub fn fract(&self) -> Ratio<T> {
        Ratio::new_raw(self.numer.clone() % self.denom.clone(), self.denom.clone())
    }
}

impl<T: Clone + Integer + PartialOrd + PrimInt> Ratio<T> {
    /// Raises the ratio to the power of an exponent
    #[inline]
    pub fn pow(&self, expon: i32) -> Ratio<T> {
        match expon.cmp(&0) {
            cmp::Ordering::Equal => One::one(),
            cmp::Ordering::Less => self.recip().pow(-expon),
            cmp::Ordering::Greater =>
                Ratio::new_raw(self.numer.pow(expon as u32), self.denom.pow(expon as u32)),
        }
    }
}

#[cfg(feature = "bigint")]
impl Ratio<BigInt> {
    /// Converts a float into a rational number.
    pub fn from_float<T: Float>(f: T) -> Option<BigRational> {
        if !f.is_finite() {
            return None;
        }
        let (mantissa, exponent, sign) = f.integer_decode();
        let bigint_sign = if sign == 1 {
            Sign::Plus
        } else {
            Sign::Minus
        };
        if exponent < 0 {
            let one: BigInt = One::one();
            let denom: BigInt = one << ((-exponent) as usize);
            let numer: BigUint = FromPrimitive::from_u64(mantissa).unwrap();
            Some(Ratio::new(BigInt::from_biguint(bigint_sign, numer), denom))
        } else {
            let mut numer: BigUint = FromPrimitive::from_u64(mantissa).unwrap();
            numer = numer << (exponent as usize);
            Some(Ratio::from_integer(BigInt::from_biguint(bigint_sign, numer)))
        }
    }
}

// Comparisons

// comparing a/b and c/d is the same as comparing a*d and b*c, so we
// abstract that pattern. The following macro takes a trait and either
// a comma-separated list of "method name -> return value" or just
// "method name" (return value is bool in that case)
macro_rules! cmp_impl {
    (impl $imp:ident, $($method:ident),+) => {
        cmp_impl!(impl $imp, $($method -> bool),+);
    };
    // return something other than a Ratio<T>
    (impl $imp:ident, $($method:ident -> $res:ty),*) => {
        impl<T> $imp for Ratio<T> where
            T: Clone + Mul<T, Output = T> + $imp
        {
            $(
                #[inline]
                fn $method(&self, other: &Ratio<T>) -> $res {
                    (self.numer.clone() * other.denom.clone()). $method (&(self.denom.clone()*other.numer.clone()))
                }
            )*
        }
    };
}
cmp_impl!(impl PartialEq, eq, ne);
cmp_impl!(impl PartialOrd, lt -> bool, gt -> bool, le -> bool, ge -> bool,
          partial_cmp -> Option<cmp::Ordering>);
cmp_impl!(impl Eq, );
cmp_impl!(impl Ord, cmp -> cmp::Ordering);

macro_rules! forward_val_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl<T: Clone + Integer + PartialOrd> $imp<Ratio<T>> for Ratio<T> {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: Ratio<T>) -> Ratio<T> {
                (&self).$method(&other)
            }
        }
    }
}

macro_rules! forward_ref_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, T> $imp<Ratio<T>> for &'a Ratio<T> where
            T: Clone + Integer + PartialOrd
        {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: Ratio<T>) -> Ratio<T> {
                self.$method(&other)
            }
        }
    }
}

macro_rules! forward_val_ref_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, T> $imp<&'a Ratio<T>> for Ratio<T> where
            T: Clone + Integer + PartialOrd
        {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: &Ratio<T>) -> Ratio<T> {
                (&self).$method(other)
            }
        }
    }
}

macro_rules! forward_all_binop {
    (impl $imp:ident, $method:ident) => {
        forward_val_val_binop!(impl $imp, $method);
        forward_ref_val_binop!(impl $imp, $method);
        forward_val_ref_binop!(impl $imp, $method);
    };
}

// Arithmetic
forward_all_binop!(impl Mul, mul);
// a/b * c/d = (a*c)/(b*d)
impl<'a, 'b, T> Mul<&'b Ratio<T>> for &'a Ratio<T>
    where T: Clone + Integer + PartialOrd
{

        type Output = Ratio<T>;
    #[inline]
    fn mul(self, rhs: &Ratio<T>) -> Ratio<T> {
        Ratio::new(self.numer.clone() * rhs.numer.clone(),
                   self.denom.clone() * rhs.denom.clone())
    }
}

forward_all_binop!(impl Div, div);
// (a/b) / (c/d) = (a*d)/(b*c)
impl<'a, 'b, T> Div<&'b Ratio<T>> for &'a Ratio<T>
    where T: Clone + Integer + PartialOrd
{
    type Output = Ratio<T>;

    #[inline]
    fn div(self, rhs: &Ratio<T>) -> Ratio<T> {
        Ratio::new(self.numer.clone() * rhs.denom.clone(),
                   self.denom.clone() * rhs.numer.clone())
    }
}

// Abstracts the a/b `op` c/d = (a*d `op` b*d) / (b*d) pattern
macro_rules! arith_impl {
    (impl $imp:ident, $method:ident) => {
        forward_all_binop!(impl $imp, $method);
        impl<'a, 'b, T: Clone + Integer + PartialOrd>
            $imp<&'b Ratio<T>> for &'a Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn $method(self, rhs: &Ratio<T>) -> Ratio<T> {
                Ratio::new((self.numer.clone() * rhs.denom.clone()).$method(self.denom.clone() * rhs.numer.clone()),
                           self.denom.clone() * rhs.denom.clone())
            }
        }
    }
}

// a/b + c/d = (a*d + b*c)/(b*d)
arith_impl!(impl Add, add);

// a/b - c/d = (a*d - b*c)/(b*d)
arith_impl!(impl Sub, sub);

// a/b % c/d = (a*d % b*c)/(b*d)
arith_impl!(impl Rem, rem);

impl<T> Neg for Ratio<T>
    where T: Clone + Integer + PartialOrd + Neg<Output = T>
{
    type Output = Ratio<T>;

    #[inline]
    fn neg(self) -> Ratio<T> {
        -&self
    }
}

impl<'a, T> Neg for &'a Ratio<T>
    where T: Clone + Integer + PartialOrd + Neg<Output = T>
{
    type Output = Ratio<T>;

    #[inline]
    fn neg(self) -> Ratio<T> {
        Ratio::new_raw(-self.numer.clone(), self.denom.clone())
    }
}

// Constants
impl<T: Clone + Integer + PartialOrd>
    Zero for Ratio<T> {
    #[inline]
    fn zero() -> Ratio<T> {
        Ratio::new_raw(Zero::zero(), One::one())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == Zero::zero()
    }
}

impl<T: Clone + Integer + PartialOrd>
    One for Ratio<T> {
    #[inline]
    fn one() -> Ratio<T> {
        Ratio::new_raw(One::one(), One::one())
    }
}

impl<T: Clone + Integer + PartialOrd> Num for Ratio<T> {
    type FromStrRadixErr = ParseRatioError;

    /// Parses `numer/denom` where the numbers are in base `radix`.
    fn from_str_radix(s: &str, radix: u32) -> Result<Ratio<T>, ParseRatioError> {
        let split: Vec<&str> = s.splitn(2, '/').collect();
        if split.len() < 2 {
            Err(ParseRatioError)
        } else {
            let a_result: Result<T, _> = T::from_str_radix(split[0], radix)
                                             .map_err(|_| ParseRatioError);
            a_result.and_then(|a| {
                let b_result: Result<T, _> = T::from_str_radix(split[1], radix)
                                                 .map_err(|_| ParseRatioError);
                b_result.and_then(|b| Ok(Ratio::new(a.clone(), b.clone())))
            })
        }
    }
}

impl<T: Clone + Integer + PartialOrd + Signed> Signed for Ratio<T> {
    #[inline]
    fn abs(&self) -> Ratio<T> {
        if self.is_negative() {
            -self.clone()
        } else {
            self.clone()
        }
    }

    #[inline]
    fn abs_sub(&self, other: &Ratio<T>) -> Ratio<T> {
        if *self <= *other {
            Zero::zero()
        } else {
            self - other
        }
    }

    #[inline]
    fn signum(&self) -> Ratio<T> {
        if *self > Zero::zero() {
            One::one()
        } else if self.is_zero() {
            Zero::zero()
        } else {
            -::one::<Ratio<T>>()
        }
    }

    #[inline]
    fn is_positive(&self) -> bool {
        *self > Zero::zero()
    }

    #[inline]
    fn is_negative(&self) -> bool {
        *self < Zero::zero()
    }
}

// String conversions
impl<T> fmt::Display for Ratio<T> where
    T: fmt::Display + Eq + One
{
    /// Renders as `numer/denom`. If denom=1, renders as numer.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.denom == One::one() {
            write!(f, "{}", self.numer)
        } else {
            write!(f, "{}/{}", self.numer, self.denom)
        }
    }
}

impl<T: FromStr + Clone + Integer + PartialOrd> FromStr for Ratio<T> {
    type Err = ParseRatioError;

    /// Parses `numer/denom` or just `numer`.
    fn from_str(s: &str) -> Result<Ratio<T>, ParseRatioError> {
        let mut split = s.splitn(2, '/');

        let n = try!(split.next().ok_or(ParseRatioError));
        let num = try!(FromStr::from_str(n).map_err(|_| ParseRatioError));

        let d = split.next().unwrap_or("1");
        let den = try!(FromStr::from_str(d).map_err(|_| ParseRatioError));

        Ok(Ratio::new(num, den))
    }
}

// FIXME: Bubble up specific errors
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParseRatioError;

impl fmt::Display for ParseRatioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "failed to parse provided string".fmt(f)
    }
}

impl Error for ParseRatioError {
    fn description(&self) -> &str {
        "failed to parse bigint/biguint"
    }
}
