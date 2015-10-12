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

use std::cmp;
use std::error::Error;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;

#[cfg(feature = "bigint")]
use bigint::{BigInt, BigUint, Sign};
use traits::{FromPrimitive, Float, PrimInt};
use {Num, Signed, Zero, One};

/// Represents the ratio between 2 numbers.
#[derive(Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "rustc-serialize", derive(RustcEncodable, RustcDecodable))]
#[allow(missing_docs)]
pub struct Ratio<T> {
    numer: T,
    denom: T
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
        Ratio { numer: numer, denom: denom }
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
        let g : T = self.numer.gcd(&self.denom);

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
            Ratio::from_integer((self.numer.clone() - self.denom.clone() + one) / self.denom.clone())
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
            Ratio::from_integer((self.numer.clone() + self.denom.clone() - one) / self.denom.clone())
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
        if fractional < zero { fractional = zero - fractional };

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
            cmp::Ordering::Greater => Ratio::new_raw(self.numer.pow(expon as u32),
                                                     self.denom.pow(expon as u32)),
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
        let bigint_sign = if sign == 1 { Sign::Plus } else { Sign::Minus };
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

/* Comparisons */

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

/* Arithmetic */
forward_all_binop!(impl Mul, mul);
// a/b * c/d = (a*c)/(b*d)
impl<'a, 'b, T> Mul<&'b Ratio<T>> for &'a Ratio<T>
    where T: Clone + Integer + PartialOrd
{

        type Output = Ratio<T>;
    #[inline]
    fn mul(self, rhs: &Ratio<T>) -> Ratio<T> {
        Ratio::new(self.numer.clone() * rhs.numer.clone(), self.denom.clone() * rhs.denom.clone())
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
        Ratio::new(self.numer.clone() * rhs.denom.clone(), self.denom.clone() * rhs.numer.clone())
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
    fn neg(self) -> Ratio<T> { -&self }
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

/* Constants */
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
            let a_result: Result<T, _> = T::from_str_radix(
                split[0],
                radix).map_err(|_| ParseRatioError);
            a_result.and_then(|a| {
                let b_result: Result<T, _>  =
                    T::from_str_radix(split[1], radix).map_err(|_| ParseRatioError);
                b_result.and_then(|b| {
                    Ok(Ratio::new(a.clone(), b.clone()))
                })
            })
        }
    }
}

impl<T: Clone + Integer + PartialOrd + Signed> Signed for Ratio<T> {
    #[inline]
    fn abs(&self) -> Ratio<T> {
        if self.is_negative() { -self.clone() } else { self.clone() }
    }

    #[inline]
    fn abs_sub(&self, other: &Ratio<T>) -> Ratio<T> {
        if *self <= *other { Zero::zero() } else { self - other }
    }

    #[inline]
    fn signum(&self) -> Ratio<T> {
        if *self > Zero::zero() {
            One::one()
        } else if self.is_zero() {
            Zero::zero()
        } else {
            - ::one::<Ratio<T>>()
        }
    }

    #[inline]
    fn is_positive(&self) -> bool { *self > Zero::zero() }

    #[inline]
    fn is_negative(&self) -> bool { *self < Zero::zero() }
}

/* String conversions */
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
    fn description(&self) -> &str { "failed to parse bigint/biguint" }
}

#[cfg(test)]
mod test {

    use super::{Ratio, Rational};
    #[cfg(feature = "bigint")]
    use super::BigRational;
    use std::str::FromStr;
    use std::i32;
    use {Zero, One, Signed, FromPrimitive, Float};

    pub const _0 : Rational = Ratio { numer: 0, denom: 1};
    pub const _1 : Rational = Ratio { numer: 1, denom: 1};
    pub const _2: Rational = Ratio { numer: 2, denom: 1};
    pub const _1_2: Rational = Ratio { numer: 1, denom: 2};
    pub const _3_2: Rational = Ratio { numer: 3, denom: 2};
    pub const _NEG1_2: Rational = Ratio { numer: -1, denom: 2};
    pub const _1_3: Rational = Ratio { numer: 1, denom: 3};
    pub const _NEG1_3: Rational = Ratio { numer: -1, denom: 3};
    pub const _2_3: Rational = Ratio { numer: 2, denom: 3};
    pub const _NEG2_3: Rational = Ratio { numer: -2, denom: 3};

    #[cfg(feature = "bigint")]
    pub fn to_big(n: Rational) -> BigRational {
        Ratio::new(
            FromPrimitive::from_isize(n.numer).unwrap(),
            FromPrimitive::from_isize(n.denom).unwrap()
        )
    }
    #[cfg(not(feature = "bigint"))]
    pub fn to_big(n: Rational) -> Rational {
        Ratio::new(
            FromPrimitive::from_isize(n.numer).unwrap(),
            FromPrimitive::from_isize(n.denom).unwrap()
        )
    }

    #[test]
    fn test_test_constants() {
        // check our constants are what Ratio::new etc. would make.
        assert_eq!(_0, Zero::zero());
        assert_eq!(_1, One::one());
        assert_eq!(_2, Ratio::from_integer(2));
        assert_eq!(_1_2, Ratio::new(1,2));
        assert_eq!(_3_2, Ratio::new(3,2));
        assert_eq!(_NEG1_2, Ratio::new(-1,2));
    }

    #[test]
    fn test_new_reduce() {
        let one22 = Ratio::new(2,2);

        assert_eq!(one22, One::one());
    }
    #[test]
    #[should_panic]
    fn test_new_zero() {
        let _a = Ratio::new(1,0);
    }


    #[test]
    fn test_cmp() {
        assert!(_0 == _0 && _1 == _1);
        assert!(_0 != _1 && _1 != _0);
        assert!(_0 < _1 && !(_1 < _0));
        assert!(_1 > _0 && !(_0 > _1));

        assert!(_0 <= _0 && _1 <= _1);
        assert!(_0 <= _1 && !(_1 <= _0));

        assert!(_0 >= _0 && _1 >= _1);
        assert!(_1 >= _0 && !(_0 >= _1));
    }


    #[test]
    fn test_to_integer() {
        assert_eq!(_0.to_integer(), 0);
        assert_eq!(_1.to_integer(), 1);
        assert_eq!(_2.to_integer(), 2);
        assert_eq!(_1_2.to_integer(), 0);
        assert_eq!(_3_2.to_integer(), 1);
        assert_eq!(_NEG1_2.to_integer(), 0);
    }


    #[test]
    fn test_numer() {
        assert_eq!(_0.numer(), &0);
        assert_eq!(_1.numer(), &1);
        assert_eq!(_2.numer(), &2);
        assert_eq!(_1_2.numer(), &1);
        assert_eq!(_3_2.numer(), &3);
        assert_eq!(_NEG1_2.numer(), &(-1));
    }
    #[test]
    fn test_denom() {
        assert_eq!(_0.denom(), &1);
        assert_eq!(_1.denom(), &1);
        assert_eq!(_2.denom(), &1);
        assert_eq!(_1_2.denom(), &2);
        assert_eq!(_3_2.denom(), &2);
        assert_eq!(_NEG1_2.denom(), &2);
    }


    #[test]
    fn test_is_integer() {
        assert!(_0.is_integer());
        assert!(_1.is_integer());
        assert!(_2.is_integer());
        assert!(!_1_2.is_integer());
        assert!(!_3_2.is_integer());
        assert!(!_NEG1_2.is_integer());
    }

    #[test]
    fn test_show() {
        assert_eq!(format!("{}", _2), "2".to_string());
        assert_eq!(format!("{}", _1_2), "1/2".to_string());
        assert_eq!(format!("{}", _0), "0".to_string());
        assert_eq!(format!("{}", Ratio::from_integer(-2)), "-2".to_string());
    }

    mod arith {
        use super::{_0, _1, _2, _1_2, _3_2, _NEG1_2, to_big};
        use super::super::{Ratio, Rational};

        #[test]
        fn test_add() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a + b, c);
                assert_eq!(to_big(a) + to_big(b), to_big(c));
            }

            test(_1, _1_2, _3_2);
            test(_1, _1, _2);
            test(_1_2, _3_2, _2);
            test(_1_2, _NEG1_2, _0);
        }

        #[test]
        fn test_sub() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a - b, c);
                assert_eq!(to_big(a) - to_big(b), to_big(c))
            }

            test(_1, _1_2, _1_2);
            test(_3_2, _1_2, _1);
            test(_1, _NEG1_2, _3_2);
        }

        #[test]
        fn test_mul() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a * b, c);
                assert_eq!(to_big(a) * to_big(b), to_big(c))
            }

            test(_1, _1_2, _1_2);
            test(_1_2, _3_2, Ratio::new(3,4));
            test(_1_2, _NEG1_2, Ratio::new(-1, 4));
        }

        #[test]
        fn test_div() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a / b, c);
                assert_eq!(to_big(a) / to_big(b), to_big(c))
            }

            test(_1, _1_2, _2);
            test(_3_2, _1_2, _1 + _2);
            test(_1, _NEG1_2, _NEG1_2 + _NEG1_2 + _NEG1_2 + _NEG1_2);
        }

        #[test]
        fn test_rem() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a % b, c);
                assert_eq!(to_big(a) % to_big(b), to_big(c))
            }

            test(_3_2, _1, _1_2);
            test(_2, _NEG1_2, _0);
            test(_1_2, _2,  _1_2);
        }

        #[test]
        fn test_neg() {
            fn test(a: Rational, b: Rational) {
                assert_eq!(-a, b);
                assert_eq!(-to_big(a), to_big(b))
            }

            test(_0, _0);
            test(_1_2, _NEG1_2);
            test(-_1, _1);
        }
        #[test]
        fn test_zero() {
            assert_eq!(_0 + _0, _0);
            assert_eq!(_0 * _0, _0);
            assert_eq!(_0 * _1, _0);
            assert_eq!(_0 / _NEG1_2, _0);
            assert_eq!(_0 - _0, _0);
        }
        #[test]
        #[should_panic]
        fn test_div_0() {
            let _a =  _1 / _0;
        }
    }

    #[test]
    fn test_round() {
        assert_eq!(_1_3.ceil(), _1);
        assert_eq!(_1_3.floor(), _0);
        assert_eq!(_1_3.round(), _0);
        assert_eq!(_1_3.trunc(), _0);

        assert_eq!(_NEG1_3.ceil(), _0);
        assert_eq!(_NEG1_3.floor(), -_1);
        assert_eq!(_NEG1_3.round(), _0);
        assert_eq!(_NEG1_3.trunc(), _0);

        assert_eq!(_2_3.ceil(), _1);
        assert_eq!(_2_3.floor(), _0);
        assert_eq!(_2_3.round(), _1);
        assert_eq!(_2_3.trunc(), _0);

        assert_eq!(_NEG2_3.ceil(), _0);
        assert_eq!(_NEG2_3.floor(), -_1);
        assert_eq!(_NEG2_3.round(), -_1);
        assert_eq!(_NEG2_3.trunc(), _0);

        assert_eq!(_1_2.ceil(), _1);
        assert_eq!(_1_2.floor(), _0);
        assert_eq!(_1_2.round(), _1);
        assert_eq!(_1_2.trunc(), _0);

        assert_eq!(_NEG1_2.ceil(), _0);
        assert_eq!(_NEG1_2.floor(), -_1);
        assert_eq!(_NEG1_2.round(), -_1);
        assert_eq!(_NEG1_2.trunc(), _0);

        assert_eq!(_1.ceil(), _1);
        assert_eq!(_1.floor(), _1);
        assert_eq!(_1.round(), _1);
        assert_eq!(_1.trunc(), _1);

        // Overflow checks

        let _neg1 = Ratio::from_integer(-1);
        let _large_rat1 = Ratio::new(i32::MAX, i32::MAX-1);
        let _large_rat2 = Ratio::new(i32::MAX-1, i32::MAX);
        let _large_rat3 = Ratio::new(i32::MIN+2, i32::MIN+1);
        let _large_rat4 = Ratio::new(i32::MIN+1, i32::MIN+2);
        let _large_rat5 = Ratio::new(i32::MIN+2, i32::MAX);
        let _large_rat6 = Ratio::new(i32::MAX, i32::MIN+2);
        let _large_rat7 = Ratio::new(1, i32::MIN+1);
        let _large_rat8 = Ratio::new(1, i32::MAX);

        assert_eq!(_large_rat1.round(), One::one());
        assert_eq!(_large_rat2.round(), One::one());
        assert_eq!(_large_rat3.round(), One::one());
        assert_eq!(_large_rat4.round(), One::one());
        assert_eq!(_large_rat5.round(), _neg1);
        assert_eq!(_large_rat6.round(), _neg1);
        assert_eq!(_large_rat7.round(), Zero::zero());
        assert_eq!(_large_rat8.round(), Zero::zero());
    }

    #[test]
    fn test_fract() {
        assert_eq!(_1.fract(), _0);
        assert_eq!(_NEG1_2.fract(), _NEG1_2);
        assert_eq!(_1_2.fract(), _1_2);
        assert_eq!(_3_2.fract(), _1_2);
    }

    #[test]
    fn test_recip() {
        assert_eq!(_1 * _1.recip(), _1);
        assert_eq!(_2 * _2.recip(), _1);
        assert_eq!(_1_2 * _1_2.recip(), _1);
        assert_eq!(_3_2 * _3_2.recip(), _1);
        assert_eq!(_NEG1_2 * _NEG1_2.recip(), _1);
    }

    #[test]
    fn test_pow() {
        assert_eq!(_1_2.pow(2), Ratio::new(1, 4));
        assert_eq!(_1_2.pow(-2), Ratio::new(4, 1));
        assert_eq!(_1.pow(1), _1);
        assert_eq!(_NEG1_2.pow(2), _1_2.pow(2));
        assert_eq!(_NEG1_2.pow(3), -_1_2.pow(3));
        assert_eq!(_3_2.pow(0), _1);
        assert_eq!(_3_2.pow(-1), _3_2.recip());
        assert_eq!(_3_2.pow(3), Ratio::new(27, 8));
    }

    #[test]
    fn test_to_from_str() {
        fn test(r: Rational, s: String) {
            assert_eq!(FromStr::from_str(&s), Ok(r));
            assert_eq!(r.to_string(), s);
        }
        test(_1, "1".to_string());
        test(_0, "0".to_string());
        test(_1_2, "1/2".to_string());
        test(_3_2, "3/2".to_string());
        test(_2, "2".to_string());
        test(_NEG1_2, "-1/2".to_string());
    }
    #[test]
    fn test_from_str_fail() {
        fn test(s: &str) {
            let rational: Result<Rational, _> = FromStr::from_str(s);
            assert!(rational.is_err());
        }

        let xs = ["0 /1", "abc", "", "1/", "--1/2","3/2/1"];
        for &s in xs.iter() {
            test(s);
        }
    }

    #[cfg(feature = "bigint")]
    #[test]
    fn test_from_float() {
        fn test<T: Float>(given: T, (numer, denom): (&str, &str)) {
            let ratio: BigRational = Ratio::from_float(given).unwrap();
            assert_eq!(ratio, Ratio::new(
                FromStr::from_str(numer).unwrap(),
                FromStr::from_str(denom).unwrap()));
        }

        // f32
        test(3.14159265359f32, ("13176795", "4194304"));
        test(2f32.powf(100.), ("1267650600228229401496703205376", "1"));
        test(-2f32.powf(100.), ("-1267650600228229401496703205376", "1"));
        test(1.0 / 2f32.powf(100.), ("1", "1267650600228229401496703205376"));
        test(684729.48391f32, ("1369459", "2"));
        test(-8573.5918555f32, ("-4389679", "512"));

        // f64
        test(3.14159265359f64, ("3537118876014453", "1125899906842624"));
        test(2f64.powf(100.), ("1267650600228229401496703205376", "1"));
        test(-2f64.powf(100.), ("-1267650600228229401496703205376", "1"));
        test(684729.48391f64, ("367611342500051", "536870912"));
        test(-8573.5918555f64, ("-4713381968463931", "549755813888"));
        test(1.0 / 2f64.powf(100.), ("1", "1267650600228229401496703205376"));
    }

    #[cfg(feature = "bigint")]
    #[test]
    fn test_from_float_fail() {
        use std::{f32, f64};

        assert_eq!(Ratio::from_float(f32::NAN), None);
        assert_eq!(Ratio::from_float(f32::INFINITY), None);
        assert_eq!(Ratio::from_float(f32::NEG_INFINITY), None);
        assert_eq!(Ratio::from_float(f64::NAN), None);
        assert_eq!(Ratio::from_float(f64::INFINITY), None);
        assert_eq!(Ratio::from_float(f64::NEG_INFINITY), None);
    }

    #[test]
    fn test_signed() {
        assert_eq!(_NEG1_2.abs(), _1_2);
        assert_eq!(_3_2.abs_sub(&_1_2), _1);
        assert_eq!(_1_2.abs_sub(&_3_2), Zero::zero());
        assert_eq!(_1_2.signum(), One::one());
        assert_eq!(_NEG1_2.signum(), - ::one::<Ratio<isize>>());
        assert!(_NEG1_2.is_negative());
        assert!(! _NEG1_2.is_positive());
        assert!(! _1_2.is_negative());
    }

    #[test]
    fn test_hash() {
        assert!(::hash(&_0) != ::hash(&_1));
        assert!(::hash(&_0) != ::hash(&_3_2));
    }
}
