// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Big integer (signed version: `BigInt`, unsigned version: `BigUint`).
//!
//! A `BigUint` is represented as a vector of `BigDigit`s.
//! A `BigInt` is a combination of `BigUint` and `Sign`.
//!
//! Common numerical operations are overloaded, so we can treat them
//! the same way we treat other numbers.
//!
//! ## Example
//!
//! ```rust
//! use num::{BigUint, Zero, One};
//! use redox::mem::replace;
//!
//! // Calculate large fibonacci numbers.
//! fn fib(n: usize) -> BigUint {
//!     let mut f0: BigUint = Zero::zero();
//!     let mut f1: BigUint = One::one();
//!     for _ in (0..n) {
//!         let f2 = f0 + &f1;
//!         // This is a low cost way of swapping f0 with f1 and f1 with f2.
//!         f0 = replace(&mut f1, f2);
//!     }
//!     f0
//! }
//!
//! // This is a very large number.
//! println!("fib(1000) = {}", fib(1000));
//! ```
//!
//! It's easy to generate large random numbers:
//!
//! ```rust
//! extern crate rand;
//! extern crate num;
//! # fn main() {
//! use num::bigint::{ToBigInt, RandBigInt};
//!
//! let mut rng = rand::thread_rng();
//! let a = rng.gen_bigint(1000);
//!
//! let low = -10000.to_bigint().unwrap();
//! let high = 10000.to_bigint().unwrap();
//! let b = rng.gen_bigint_range(&low, &high);
//!
//! // Probably an even larger number.
//! println!("{}", a * b);
//! # }
//! ```

use Integer;

use redox::default::Default;
use redox::error::Error;
use redox::iter::repeat;
use redox::num::ParseIntError;
use redox::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};
use redox::str::{self, FromStr};
use redox::{cmp, fmt, hash, mem};
use redox::cmp::Ordering::{self, Less, Greater, Equal};
use redox::{i64, u64};

use rand::Rng;

use traits::{ToPrimitive, FromPrimitive, cast};

use {Num, Unsigned, CheckedAdd, CheckedSub, CheckedMul, CheckedDiv, Signed, Zero, One};
use self::Sign::{Minus, NoSign, Plus};

/// A `BigDigit` is a `BigUint`'s composing element.
pub type BigDigit = u32;

/// A `DoubleBigDigit` is the internal type used to do the computations.  Its
/// size is the double of the size of `BigDigit`.
pub type DoubleBigDigit = u64;

pub const ZERO_BIG_DIGIT: BigDigit = 0;
static ZERO_VEC: [BigDigit; 1] = [ZERO_BIG_DIGIT];

#[allow(non_snake_case)]
pub mod big_digit {
    use super::BigDigit;
    use super::DoubleBigDigit;

    // `DoubleBigDigit` size dependent
    pub const BITS: usize = 32;

    pub const BASE: DoubleBigDigit = 1 << BITS;
    const LO_MASK: DoubleBigDigit = (-1i32 as DoubleBigDigit) >> BITS;

    #[inline]
    fn get_hi(n: DoubleBigDigit) -> BigDigit {
        (n >> BITS) as BigDigit
    }
    #[inline]
    fn get_lo(n: DoubleBigDigit) -> BigDigit {
        (n & LO_MASK) as BigDigit
    }

    /// Split one `DoubleBigDigit` into two `BigDigit`s.
    #[inline]
    pub fn from_doublebigdigit(n: DoubleBigDigit) -> (BigDigit, BigDigit) {
        (get_hi(n), get_lo(n))
    }

    /// Join two `BigDigit`s into one `DoubleBigDigit`
    #[inline]
    pub fn to_doublebigdigit(hi: BigDigit, lo: BigDigit) -> DoubleBigDigit {
        (lo as DoubleBigDigit) | ((hi as DoubleBigDigit) << BITS)
    }
}

/// A big unsigned integer type.
///
/// A `BigUint`-typed value `BigUint { data: vec!(a, b, c) }` represents a number
/// `(a + b * big_digit::BASE + c * big_digit::BASE^2)`.
#[derive(Clone, Debug)]
pub struct BigUint {
    data: Vec<BigDigit>,
}

impl PartialEq for BigUint {
    #[inline]
    fn eq(&self, other: &BigUint) -> bool {
        match self.cmp(other) {
            Equal => true,
            _ => false,
        }
    }
}
impl Eq for BigUint {}

impl PartialOrd for BigUint {
    #[inline]
    fn partial_cmp(&self, other: &BigUint) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigUint {
    #[inline]
    fn cmp(&self, other: &BigUint) -> Ordering {
        let (s_len, o_len) = (self.data.len(), other.data.len());
        if s_len < o_len {
            return Less;
        }
        if s_len > o_len {
            return Greater;
        }

        for (&self_i, &other_i) in self.data.iter().rev().zip(other.data.iter().rev()) {
            if self_i < other_i {
                return Less;
            }
            if self_i > other_i {
                return Greater;
            }
        }
        return Equal;
    }
}

impl Default for BigUint {
    #[inline]
    fn default() -> BigUint {
        Zero::zero()
    }
}

impl hash::Hash for BigUint {
    fn hash<H>(&self, state: &mut H)
        where H: hash::Hasher
    {
        // hash 0 in case it's all 0's
        0u32.hash(state);

        let mut found_first_value = false;
        for elem in self.data.iter().rev() {
            // don't hash any leading 0's, they shouldn't affect the hash
            if found_first_value || *elem != 0 {
                found_first_value = true;
                elem.hash(state);
            }
        }
    }
}

impl fmt::Display for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_str_radix(self, 10))
    }
}

impl FromStr for BigUint {
    type Err = ParseBigIntError;

    #[inline]
    fn from_str(s: &str) -> Result<BigUint, ParseBigIntError> {
        BigUint::from_str_radix(s, 10)
    }
}

impl Num for BigUint {
    type FromStrRadixErr = ParseBigIntError;

    /// Creates and initializes a `BigUint`.
    #[inline]
    fn from_str_radix(s: &str, radix: u32) -> Result<BigUint, ParseBigIntError> {
        let (base, unit_len) = get_radix_base(radix);
        let base_num = match base.to_biguint() {
            Some(base_num) => base_num,
            None => {
                return Err(ParseBigIntError::Other);
            }
        };

        let mut end = s.len();
        let mut n: BigUint = Zero::zero();
        let mut power: BigUint = One::one();
        loop {
            let start = cmp::max(end, unit_len) - unit_len;
            let d = try!(usize::from_str_radix(&s[start..end], radix));
            let d: Option<BigUint> = FromPrimitive::from_usize(d);
            match d {
                Some(d) => {
                    // FIXME(#5992): assignment operator overloads
                    // n += d * &power;
                    n = n + d * &power;
                }
                None => {
                    return Err(ParseBigIntError::Other);
                }
            }
            if end <= unit_len {
                return Ok(n);
            }
            end -= unit_len;
            // FIXME(#5992): assignment operator overloads
            // power *= &base_num;
            power = power * &base_num;
        }
    }
}

macro_rules! forward_val_val_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self).$method(&other)
            }
        }
    }
}

macro_rules! forward_ref_val_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl<'a> $imp<$res> for &'a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                self.$method(&other)
            }
        }
    }
}

macro_rules! forward_val_ref_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl<'a> $imp<&'a $res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                (&self).$method(other)
            }
        }
    }
}

macro_rules! forward_all_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        forward_val_val_binop!(impl $imp for $res, $method);
        forward_ref_val_binop!(impl $imp for $res, $method);
        forward_val_ref_binop!(impl $imp for $res, $method);
    };
}

forward_all_binop!(impl BitAnd for BigUint, bitand);

impl<'a, 'b> BitAnd<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    #[inline]
    fn bitand(self, other: &BigUint) -> BigUint {
        BigUint::new(self.data.iter().zip(other.data.iter()).map(|(ai, bi)| *ai & *bi).collect())
    }
}

forward_all_binop!(impl BitOr for BigUint, bitor);

impl<'a, 'b> BitOr<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    fn bitor(self, other: &BigUint) -> BigUint {
        let zeros = ZERO_VEC.iter().cycle();
        let (a, b) = if self.data.len() > other.data.len() {
            (self, other)
        } else {
            (other, self)
        };
        let ored = a.data
                    .iter()
                    .zip(b.data.iter().chain(zeros))
                    .map(|(ai, bi)| *ai | *bi)
                    .collect();
        return BigUint::new(ored);
    }
}

forward_all_binop!(impl BitXor for BigUint, bitxor);

impl<'a, 'b> BitXor<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    fn bitxor(self, other: &BigUint) -> BigUint {
        let zeros = ZERO_VEC.iter().cycle();
        let (a, b) = if self.data.len() > other.data.len() {
            (self, other)
        } else {
            (other, self)
        };
        let xored = a.data
                     .iter()
                     .zip(b.data.iter().chain(zeros))
                     .map(|(ai, bi)| *ai ^ *bi)
                     .collect();
        return BigUint::new(xored);
    }
}

impl Shl<usize> for BigUint {
    type Output = BigUint;

    #[inline]
    fn shl(self, rhs: usize) -> BigUint {
        (&self) << rhs
    }
}

impl<'a> Shl<usize> for &'a BigUint {
    type Output = BigUint;

    #[inline]
    fn shl(self, rhs: usize) -> BigUint {
        let n_unit = rhs / big_digit::BITS;
        let n_bits = rhs % big_digit::BITS;
        return self.shl_unit(n_unit).shl_bits(n_bits);
    }
}

impl Shr<usize> for BigUint {
    type Output = BigUint;

    #[inline]
    fn shr(self, rhs: usize) -> BigUint {
        (&self) >> rhs
    }
}

impl<'a> Shr<usize> for &'a BigUint {
    type Output = BigUint;

    #[inline]
    fn shr(self, rhs: usize) -> BigUint {
        let n_unit = rhs / big_digit::BITS;
        let n_bits = rhs % big_digit::BITS;
        return self.shr_unit(n_unit).shr_bits(n_bits);
    }
}

impl Zero for BigUint {
    #[inline]
    fn zero() -> BigUint {
        BigUint::new(Vec::new())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.data.is_empty()
    }
}

impl One for BigUint {
    #[inline]
    fn one() -> BigUint {
        BigUint::new(vec![1])
    }
}

impl Unsigned for BigUint {}

forward_all_binop!(impl Add for BigUint, add);

impl<'a, 'b> Add<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    fn add(self, other: &BigUint) -> BigUint {
        let zeros = ZERO_VEC.iter().cycle();
        let (a, b) = if self.data.len() > other.data.len() {
            (self, other)
        } else {
            (other, self)
        };

        let mut carry = 0;
        let mut sum: Vec<BigDigit> = a.data
                                      .iter()
                                      .zip(b.data.iter().chain(zeros))
                                      .map(|(ai, bi)| {
                                          let (hi, lo) = big_digit::from_doublebigdigit(
                (*ai as DoubleBigDigit) + (*bi as DoubleBigDigit) + (carry as DoubleBigDigit));
                                          carry = hi;
                                          lo
                                      })
                                      .collect();
        if carry != 0 {
            sum.push(carry);
        }
        return BigUint::new(sum);
    }
}

forward_all_binop!(impl Sub for BigUint, sub);

impl<'a, 'b> Sub<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    fn sub(self, other: &BigUint) -> BigUint {
        let new_len = cmp::max(self.data.len(), other.data.len());
        let zeros = ZERO_VEC.iter().cycle();
        let (a, b) = (self.data.iter().chain(zeros.clone()),
                      other.data.iter().chain(zeros));

        let mut borrow = 0isize;
        let diff: Vec<BigDigit> = a.take(new_len)
                                   .zip(b)
                                   .map(|(ai, bi)| {
                                       let (hi, lo) = big_digit::from_doublebigdigit(
                big_digit::BASE
                    + (*ai as DoubleBigDigit)
                    - (*bi as DoubleBigDigit)
                    - (borrow as DoubleBigDigit)
                    );
                                       //
                                       // hi * (base) + lo == 1*(base) + ai - bi - borrow
                                       // => ai - bi - borrow < 0 <=> hi == 0
                                       //
                                       borrow = if hi == 0 {
                                           1
                                       } else {
                                           0
                                       };
                                       lo
                                   })
                                   .collect();

        assert!(borrow == 0,
                "Cannot subtract other from self because other is larger than self.");
        return BigUint::new(diff);
    }
}


forward_all_binop!(impl Mul for BigUint, mul);

impl<'a, 'b> Mul<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    fn mul(self, other: &BigUint) -> BigUint {
        if self.is_zero() || other.is_zero() {
            return Zero::zero();
        }

        let (s_len, o_len) = (self.data.len(), other.data.len());
        if s_len == 1 {
            return mul_digit(other, self.data[0]);
        }
        if o_len == 1 {
            return mul_digit(self, other.data[0]);
        }

        // Using Karatsuba multiplication
        // (a1 * base + a0) * (b1 * base + b0)
        // = a1*b1 * base^2 +
        //   (a1*b1 + a0*b0 - (a1-b0)*(b1-a0)) * base +
        //   a0*b0
        let half_len = cmp::max(s_len, o_len) / 2;
        let (s_hi, s_lo) = cut_at(self, half_len);
        let (o_hi, o_lo) = cut_at(other, half_len);

        let ll = &s_lo * &o_lo;
        let hh = &s_hi * &o_hi;
        let mm = {
            let (s1, n1) = sub_sign(s_hi, s_lo);
            let (s2, n2) = sub_sign(o_hi, o_lo);
            match (s1, s2) {
                (Equal, _) | (_, Equal) => &hh + &ll,
                (Less, Greater) | (Greater, Less) => &hh + &ll + (n1 * n2),
                (Less, Less) | (Greater, Greater) => &hh + &ll - (n1 * n2),
            }
        };

        return ll + mm.shl_unit(half_len) + hh.shl_unit(half_len * 2);


        fn mul_digit(a: &BigUint, n: BigDigit) -> BigUint {
            if n == 0 {
                return Zero::zero();
            }
            if n == 1 {
                return a.clone();
            }

            let mut carry = 0;
            let mut prod: Vec<BigDigit> = a.data
                                           .iter()
                                           .map(|ai| {
                                               let (hi, lo) = big_digit::from_doublebigdigit(
                    (*ai as DoubleBigDigit) * (n as DoubleBigDigit) + (carry as DoubleBigDigit)
                        );
                                               carry = hi;
                                               lo
                                           })
                                           .collect();
            if carry != 0 {
                prod.push(carry);
            }
            return BigUint::new(prod);
        }

        #[inline]
        fn cut_at(a: &BigUint, n: usize) -> (BigUint, BigUint) {
            let mid = cmp::min(a.data.len(), n);
            (BigUint::from_slice(&a.data[mid..]),
             BigUint::from_slice(&a.data[..mid]))
        }

        #[inline]
        fn sub_sign(a: BigUint, b: BigUint) -> (Ordering, BigUint) {
            match a.cmp(&b) {
                Less => (Less, b - a),
                Greater => (Greater, a - b),
                _ => (Equal, Zero::zero()),
            }
        }
    }
}


forward_all_binop!(impl Div for BigUint, div);

impl<'a, 'b> Div<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    #[inline]
    fn div(self, other: &BigUint) -> BigUint {
        let (q, _) = self.div_rem(other);
        return q;
    }
}

forward_all_binop!(impl Rem for BigUint, rem);

impl<'a, 'b> Rem<&'b BigUint> for &'a BigUint {
    type Output = BigUint;

    #[inline]
    fn rem(self, other: &BigUint) -> BigUint {
        let (_, r) = self.div_rem(other);
        return r;
    }
}

impl Neg for BigUint {
    type Output = BigUint;

    #[inline]
    fn neg(self) -> BigUint {
        panic!()
    }
}

impl<'a> Neg for &'a BigUint {
    type Output = BigUint;

    #[inline]
    fn neg(self) -> BigUint {
        panic!()
    }
}

impl CheckedAdd for BigUint {
    #[inline]
    fn checked_add(&self, v: &BigUint) -> Option<BigUint> {
        return Some(self.add(v));
    }
}

impl CheckedSub for BigUint {
    #[inline]
    fn checked_sub(&self, v: &BigUint) -> Option<BigUint> {
        if *self < *v {
            return None;
        }
        return Some(self.sub(v));
    }
}

impl CheckedMul for BigUint {
    #[inline]
    fn checked_mul(&self, v: &BigUint) -> Option<BigUint> {
        return Some(self.mul(v));
    }
}

impl CheckedDiv for BigUint {
    #[inline]
    fn checked_div(&self, v: &BigUint) -> Option<BigUint> {
        if v.is_zero() {
            return None;
        }
        return Some(self.div(v));
    }
}

impl Integer for BigUint {
    #[inline]
    fn div_rem(&self, other: &BigUint) -> (BigUint, BigUint) {
        self.div_mod_floor(other)
    }

    #[inline]
    fn div_floor(&self, other: &BigUint) -> BigUint {
        let (d, _) = self.div_mod_floor(other);
        return d;
    }

    #[inline]
    fn mod_floor(&self, other: &BigUint) -> BigUint {
        let (_, m) = self.div_mod_floor(other);
        return m;
    }

    fn div_mod_floor(&self, other: &BigUint) -> (BigUint, BigUint) {
        if other.is_zero() {
            panic!()
        }
        if self.is_zero() {
            return (Zero::zero(), Zero::zero());
        }
        if *other == One::one() {
            return ((*self).clone(), Zero::zero());
        }

        match self.cmp(other) {
            Less => return (Zero::zero(), (*self).clone()),
            Equal => return (One::one(), Zero::zero()),
            Greater => {} // Do nothing
        }

        let mut shift = 0;
        let mut n = *other.data.last().unwrap();
        while n < (1 << big_digit::BITS - 2) {
            n <<= 1;
            shift += 1;
        }
        assert!(shift < big_digit::BITS);
        let (d, m) = div_mod_floor_inner(self << shift, other << shift);
        return (d, m >> shift);


        fn div_mod_floor_inner(a: BigUint, b: BigUint) -> (BigUint, BigUint) {
            let mut m = a;
            let mut d: BigUint = Zero::zero();
            let mut n = 1;
            while m >= b {
                let (d0, d_unit, b_unit) = div_estimate(&m, &b, n);
                let mut d0 = d0;
                let mut prod = &b * &d0;
                while prod > m {
                    // FIXME(#5992): assignment operator overloads
                    // d0 -= &d_unit
                    d0 = d0 - &d_unit;
                    // FIXME(#5992): assignment operator overloads
                    // prod -= &b_unit;
                    prod = prod - &b_unit
                }
                if d0.is_zero() {
                    n = 2;
                    continue;
                }
                n = 1;
                // FIXME(#5992): assignment operator overloads
                // d += d0;
                d = d + d0;
                // FIXME(#5992): assignment operator overloads
                // m -= prod;
                m = m - prod;
            }
            return (d, m);
        }


        fn div_estimate(a: &BigUint, b: &BigUint, n: usize) -> (BigUint, BigUint, BigUint) {
            if a.data.len() < n {
                return (Zero::zero(), Zero::zero(), (*a).clone());
            }

            let an = &a.data[a.data.len() - n..];
            let bn = *b.data.last().unwrap();
            let mut d = Vec::with_capacity(an.len());
            let mut carry = 0;
            for elt in an.iter().rev() {
                let ai = big_digit::to_doublebigdigit(carry, *elt);
                let di = ai / (bn as DoubleBigDigit);
                assert!(di < big_digit::BASE);
                carry = (ai % (bn as DoubleBigDigit)) as BigDigit;
                d.push(di as BigDigit)
            }
            d.reverse();

            let shift = (a.data.len() - an.len()) - (b.data.len() - 1);
            if shift == 0 {
                return (BigUint::new(d), One::one(), (*b).clone());
            }
            let one: BigUint = One::one();
            return (BigUint::new(d).shl_unit(shift),
                    one.shl_unit(shift),
                    b.shl_unit(shift));
        }
    }

    /// Calculates the Greatest Common Divisor (GCD) of the number and `other`.
    ///
    /// The result is always positive.
    #[inline]
    fn gcd(&self, other: &BigUint) -> BigUint {
        // Use Euclid's algorithm
        let mut m = (*self).clone();
        let mut n = (*other).clone();
        while !m.is_zero() {
            let temp = m;
            m = n % &temp;
            n = temp;
        }
        return n;
    }

    /// Calculates the Lowest Common Multiple (LCM) of the number and `other`.
    #[inline]
    fn lcm(&self, other: &BigUint) -> BigUint {
        ((self * other) / self.gcd(other))
    }

    /// Deprecated, use `is_multiple_of` instead.
    #[inline]
    fn divides(&self, other: &BigUint) -> bool {
        self.is_multiple_of(other)
    }

    /// Returns `true` if the number is a multiple of `other`.
    #[inline]
    fn is_multiple_of(&self, other: &BigUint) -> bool {
        (self % other).is_zero()
    }

    /// Returns `true` if the number is divisible by `2`.
    #[inline]
    fn is_even(&self) -> bool {
        // Considering only the last digit.
        match self.data.first() {
            Some(x) => x.is_even(),
            None => true,
        }
    }

    /// Returns `true` if the number is not divisible by `2`.
    #[inline]
    fn is_odd(&self) -> bool {
        !self.is_even()
    }
}

impl ToPrimitive for BigUint {
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        self.to_u64().and_then(|n| {
            // If top bit of u64 is set, it's too large to convert to i64.
            if n >> 63 == 0 {
                Some(n as i64)
            } else {
                None
            }
        })
    }

    // `DoubleBigDigit` size dependent
    #[inline]
    fn to_u64(&self) -> Option<u64> {
        match self.data.len() {
            0 => Some(0),
            1 => Some(self.data[0] as u64),
            2 => Some(big_digit::to_doublebigdigit(self.data[1], self.data[0]) as u64),
            _ => None,
        }
    }
}

impl FromPrimitive for BigUint {
    #[inline]
    fn from_i64(n: i64) -> Option<BigUint> {
        if n > 0 {
            FromPrimitive::from_u64(n as u64)
        } else if n == 0 {
            Some(Zero::zero())
        } else {
            None
        }
    }

    // `DoubleBigDigit` size dependent
    #[inline]
    fn from_u64(n: u64) -> Option<BigUint> {
        let n = match big_digit::from_doublebigdigit(n) {
            (0, 0) => Zero::zero(),
            (0, n0) => BigUint::new(vec![n0]),
            (n1, n0) => BigUint::new(vec![n0, n1]),
        };
        Some(n)
    }
}

/// A generic trait for converting a value to a `BigUint`.
pub trait ToBigUint {
    /// Converts the value of `self` to a `BigUint`.
    fn to_biguint(&self) -> Option<BigUint>;
}

impl ToBigUint for BigInt {
    #[inline]
    fn to_biguint(&self) -> Option<BigUint> {
        if self.sign == Plus {
            Some(self.data.clone())
        } else if self.sign == NoSign {
            Some(Zero::zero())
        } else {
            None
        }
    }
}

impl ToBigUint for BigUint {
    #[inline]
    fn to_biguint(&self) -> Option<BigUint> {
        Some(self.clone())
    }
}

macro_rules! impl_to_biguint {
    ($T:ty, $from_ty:path) => {
        impl ToBigUint for $T {
            #[inline]
            fn to_biguint(&self) -> Option<BigUint> {
                $from_ty(*self)
            }
        }
    }
}

impl_to_biguint!(isize,  FromPrimitive::from_isize);
impl_to_biguint!(i8,   FromPrimitive::from_i8);
impl_to_biguint!(i16,  FromPrimitive::from_i16);
impl_to_biguint!(i32,  FromPrimitive::from_i32);
impl_to_biguint!(i64,  FromPrimitive::from_i64);
impl_to_biguint!(usize, FromPrimitive::from_usize);
impl_to_biguint!(u8,   FromPrimitive::from_u8);
impl_to_biguint!(u16,  FromPrimitive::from_u16);
impl_to_biguint!(u32,  FromPrimitive::from_u32);
impl_to_biguint!(u64,  FromPrimitive::from_u64);

// Cribbed from core/fmt/num.rs
#[derive(Copy, Clone)]
pub struct RadixFmt {
    data: BigDigit,
    base: u8,
}

impl RadixFmt {
    fn digit(&self, x: u8) -> u8 {
        match x {
            x @  0 ... 9 => b'0' + x,
            x if x < self.base => b'a' + (x - 10),
            x => panic!("number not in the range 0..{}: {}", self.base - 1, x),
        }
    }
}

impl fmt::Display for RadixFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The radix can be as low as 2, so we need a buffer of at least 64
        // characters for a base 2 number.
        let mut x = self.data;
        let zero = 0;
        let is_positive = x >= zero;
        let mut buf = [0u8; 64];
        let mut curr = buf.len();
        let base = self.base as BigDigit;
        if is_positive {
            // Accumulate each digit of the number from the least significant
            // to the most significant figure.
            for byte in buf.iter_mut().rev() {
                let n = x % base;                         // Get the current place value.
                x = x / base;                             // Deaccumulate the number.
                *byte = self.digit(cast(n).unwrap());     // Store the digit in the buffer.
                curr -= 1;
                if x == zero {
                    break;
                };                   // No more digits left to accumulate.
            }
        } else {
            // Do the same as above, but accounting for two's complement.
            for byte in buf.iter_mut().rev() {
                let n = zero - (x % base);                // Get the current place value.
                x = x / base;                             // Deaccumulate the number.
                *byte = self.digit(cast(n).unwrap());     // Store the digit in the buffer.
                curr -= 1;
                if x == zero {
                    break;
                };                   // No more digits left to accumulate.
            }
        }
        let buf = unsafe { str::from_utf8_unchecked(&buf[curr..]) };
        f.pad_integral(is_positive, "", buf)
    }
}

fn to_str_radix(me: &BigUint, radix: u32) -> String {
    assert!(1 < radix && radix <= 16, "The radix must be within (1, 16]");
    let (base, max_len) = get_radix_base(radix);
    if base == big_digit::BASE {
        return fill_concat(&me.data, radix, max_len);
    }
    return fill_concat(&convert_base(me, base), radix, max_len);

    fn convert_base(n: &BigUint, base: DoubleBigDigit) -> Vec<BigDigit> {
        let divider = base.to_biguint().unwrap();
        let mut result = Vec::new();
        let mut m = n.clone();
        while m >= divider {
            let (d, m0) = m.div_mod_floor(&divider);
            result.push(m0.to_usize().unwrap() as BigDigit);
            m = d;
        }
        if !m.is_zero() {
            result.push(m.to_usize().unwrap() as BigDigit);
        }
        return result;
    }

    fn fill_concat(v: &[BigDigit], radix: u32, l: usize) -> String {
        if v.is_empty() {
            return "0".to_string();
        }
        let mut s = String::with_capacity(v.len() * l);
        for n in v.iter().rev() {
            let ss = format!("{}",
                             RadixFmt {
                                 data: *n,
                                 base: radix as u8,
                             });
            s.extend(repeat("0").take(l - ss.len()));
            s.push_str(&ss);
        }
        s.trim_left_matches('0').to_string()
    }
}

fn to_str_radix_signed(me: &BigInt, radix: u32) -> String {
    match me.sign {
        Plus => to_str_radix(&me.data, radix),
        NoSign => "0".to_string(),
        Minus => format!("-{}", to_str_radix(&me.data, radix)),
    }
}

impl BigUint {
    /// Creates and initializes a `BigUint`.
    ///
    /// The digits are in little-endian base 2^32.
    #[inline]
    pub fn new(mut digits: Vec<BigDigit>) -> BigUint {
        // omit trailing zeros
        let new_len = digits.iter().rposition(|n| *n != 0).map_or(0, |p| p + 1);
        digits.truncate(new_len);
        BigUint { data: digits }
    }

    /// Creates and initializes a `BigUint`.
    ///
    /// The digits are in little-endian base 2^32.
    #[inline]
    pub fn from_slice(slice: &[BigDigit]) -> BigUint {
        BigUint::new(slice.to_vec())
    }

    /// Creates and initializes a `BigUint`.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_bytes_be(b"A"),
    ///            BigUint::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(BigUint::from_bytes_be(b"AA"),
    ///            BigUint::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(BigUint::from_bytes_be(b"AB"),
    ///            BigUint::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(BigUint::from_bytes_be(b"Hello world!"),
    ///            BigUint::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(bytes: &[u8]) -> BigUint {
        if bytes.is_empty() {
            Zero::zero()
        } else {
            BigUint::parse_bytes(bytes.to_hex().as_bytes(), 16).unwrap()
        }
    }

    /// Creates and initializes a `BigUint`.
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(bytes: &[u8]) -> BigUint {
        let mut v = bytes.to_vec();
        v.reverse();
        BigUint::from_bytes_be(&*v)
    }

    /// Returns the byte representation of the `BigUint` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::BigUint;
    ///
    /// let i = BigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_le(), vec![101, 4]);
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for word in self.data.iter() {
            let mut w = *word;
            for _ in 0..mem::size_of::<BigDigit>() {
                let b = (w & 0xFF) as u8;
                w = w >> 8;
                result.push(b);
            }
        }

        if let Some(index) = result.iter().rposition(|x| *x != 0) {
            result.truncate(index + 1);
        }

        if result.is_empty() {
            vec![0]
        } else {
            result
        }
    }

    /// Returns the byte representation of the `BigUint` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::BigUint;
    ///
    /// let i = BigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_be(), vec![4, 101]);
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> Vec<u8> {
        let mut v = self.to_bytes_le();
        v.reverse();
        v
    }

    /// Creates and initializes a `BigUint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::{BigUint, ToBigUint};
    ///
    /// assert_eq!(BigUint::parse_bytes(b"1234", 10), ToBigUint::to_biguint(&1234));
    /// assert_eq!(BigUint::parse_bytes(b"ABCD", 16), ToBigUint::to_biguint(&0xABCD));
    /// assert_eq!(BigUint::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<BigUint> {
        str::from_utf8(buf).ok().and_then(|s| BigUint::from_str_radix(s, radix).ok())
    }

    #[inline]
    fn shl_unit(&self, n_unit: usize) -> BigUint {
        if n_unit == 0 || self.is_zero() {
            return (*self).clone();
        }

        let mut v = repeat(ZERO_BIG_DIGIT).take(n_unit).collect::<Vec<_>>();
        v.extend(self.data.iter().cloned());
        BigUint::new(v)
    }

    #[inline]
    fn shl_bits(&self, n_bits: usize) -> BigUint {
        if n_bits == 0 || self.is_zero() {
            return (*self).clone();
        }

        let mut carry = 0;
        let mut shifted: Vec<BigDigit> = self.data
                                             .iter()
                                             .map(|elem| {
                                                 let (hi, lo) = big_digit::from_doublebigdigit(
                (*elem as DoubleBigDigit) << n_bits | (carry as DoubleBigDigit)
            );
                                                 carry = hi;
                                                 lo
                                             })
                                             .collect();
        if carry != 0 {
            shifted.push(carry);
        }
        return BigUint::new(shifted);
    }

    #[inline]
    fn shr_unit(&self, n_unit: usize) -> BigUint {
        if n_unit == 0 {
            return (*self).clone();
        }
        if self.data.len() < n_unit {
            return Zero::zero();
        }
        BigUint::from_slice(&self.data[n_unit..])
    }

    #[inline]
    fn shr_bits(&self, n_bits: usize) -> BigUint {
        if n_bits == 0 || self.data.is_empty() {
            return (*self).clone();
        }

        let mut borrow = 0;
        let mut shifted_rev = Vec::with_capacity(self.data.len());
        for elem in self.data.iter().rev() {
            shifted_rev.push((*elem >> n_bits) | borrow);
            borrow = *elem << (big_digit::BITS - n_bits);
        }
        let shifted = {
            shifted_rev.reverse();
            shifted_rev
        };
        return BigUint::new(shifted);
    }

    /// Determines the fewest bits necessary to express the `BigUint`.
    pub fn bits(&self) -> usize {
        if self.is_zero() {
            return 0;
        }
        let zeros = self.data.last().unwrap().leading_zeros();
        return self.data.len() * big_digit::BITS - zeros as usize;
    }
}

// `DoubleBigDigit` size dependent
#[inline]
fn get_radix_base(radix: u32) -> (DoubleBigDigit, usize) {
    match radix {
        2 => (4294967296, 32),
        3 => (3486784401, 20),
        4 => (4294967296, 16),
        5 => (1220703125, 13),
        6 => (2176782336, 12),
        7 => (1977326743, 11),
        8 => (1073741824, 10),
        9 => (3486784401, 10),
        10 => (1000000000, 9),
        11 => (2357947691, 9),
        12 => (429981696, 8),
        13 => (815730721, 8),
        14 => (1475789056, 8),
        15 => (2562890625, 8),
        16 => (4294967296, 8),
        _ => panic!("The radix must be within (1, 16]"),
    }
}

/// A Sign is a `BigInt`'s composing element.
#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug)]
pub enum Sign {
    Minus,
    NoSign,
    Plus,
}

impl Neg for Sign {
    type Output = Sign;

    /// Negate Sign value.
    #[inline]
    fn neg(self) -> Sign {
        match self {
            Minus => Plus,
            NoSign => NoSign,
            Plus => Minus,
        }
    }
}

/// A big signed integer type.
#[derive(Clone, Debug)]
pub struct BigInt {
    sign: Sign,
    data: BigUint,
}

impl PartialEq for BigInt {
    #[inline]
    fn eq(&self, other: &BigInt) -> bool {
        self.cmp(other) == Equal
    }
}

impl Eq for BigInt {}

impl PartialOrd for BigInt {
    #[inline]
    fn partial_cmp(&self, other: &BigInt) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigInt {
    #[inline]
    fn cmp(&self, other: &BigInt) -> Ordering {
        let scmp = self.sign.cmp(&other.sign);
        if scmp != Equal {
            return scmp;
        }

        match self.sign {
            NoSign => Equal,
            Plus => self.data.cmp(&other.data),
            Minus => other.data.cmp(&self.data),
        }
    }
}

impl Default for BigInt {
    #[inline]
    fn default() -> BigInt {
        Zero::zero()
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_str_radix_signed(self, 10))
    }
}

impl hash::Hash for BigInt {
    fn hash<H>(&self, state: &mut H)
        where H: hash::Hasher
    {
        (self.sign == Plus).hash(state);
        self.data.hash(state);
    }
}

impl FromStr for BigInt {
    type Err = ParseBigIntError;

    #[inline]
    fn from_str(s: &str) -> Result<BigInt, ParseBigIntError> {
        BigInt::from_str_radix(s, 10)
    }
}

impl Num for BigInt {
    type FromStrRadixErr = ParseBigIntError;

    /// Creates and initializes a BigInt.
    #[inline]
    fn from_str_radix(s: &str, radix: u32) -> Result<BigInt, ParseBigIntError> {
        if s.is_empty() {
            return Err(ParseBigIntError::Other);
        }
        let mut sign = Plus;
        let mut start = 0;
        if s.starts_with("-") {
            sign = Minus;
            start = 1;
        }
        BigUint::from_str_radix(&s[start..], radix).map(|bu| BigInt::from_biguint(sign, bu))
    }
}

impl Shl<usize> for BigInt {
    type Output = BigInt;

    #[inline]
    fn shl(self, rhs: usize) -> BigInt {
        (&self) << rhs
    }
}

impl<'a> Shl<usize> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn shl(self, rhs: usize) -> BigInt {
        BigInt::from_biguint(self.sign, &self.data << rhs)
    }
}

impl Shr<usize> for BigInt {
    type Output = BigInt;

    #[inline]
    fn shr(self, rhs: usize) -> BigInt {
        (&self) >> rhs
    }
}

impl<'a> Shr<usize> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn shr(self, rhs: usize) -> BigInt {
        BigInt::from_biguint(self.sign, &self.data >> rhs)
    }
}

impl Zero for BigInt {
    #[inline]
    fn zero() -> BigInt {
        BigInt::from_biguint(NoSign, Zero::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.sign == NoSign
    }
}

impl One for BigInt {
    #[inline]
    fn one() -> BigInt {
        BigInt::from_biguint(Plus, One::one())
    }
}

impl Signed for BigInt {
    #[inline]
    fn abs(&self) -> BigInt {
        match self.sign {
            Plus | NoSign => self.clone(),
            Minus => BigInt::from_biguint(Plus, self.data.clone()),
        }
    }

    #[inline]
    fn abs_sub(&self, other: &BigInt) -> BigInt {
        if *self <= *other {
            Zero::zero()
        } else {
            self - other
        }
    }

    #[inline]
    fn signum(&self) -> BigInt {
        match self.sign {
            Plus => BigInt::from_biguint(Plus, One::one()),
            Minus => BigInt::from_biguint(Minus, One::one()),
            NoSign => Zero::zero(),
        }
    }

    #[inline]
    fn is_positive(&self) -> bool {
        self.sign == Plus
    }

    #[inline]
    fn is_negative(&self) -> bool {
        self.sign == Minus
    }
}

forward_all_binop!(impl Add for BigInt, add);

impl<'a, 'b> Add<&'b BigInt> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn add(self, other: &BigInt) -> BigInt {
        match (self.sign, other.sign) {
            (NoSign, _) => other.clone(),
            (_, NoSign) => self.clone(),
            (Plus, Plus) => BigInt::from_biguint(Plus, &self.data + &other.data),
            (Plus, Minus) => self - (-other),
            (Minus, Plus) => other - (-self),
            (Minus, Minus) => -((-self) + (-other)),
        }
    }
}

forward_all_binop!(impl Sub for BigInt, sub);

impl<'a, 'b> Sub<&'b BigInt> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn sub(self, other: &BigInt) -> BigInt {
        match (self.sign, other.sign) {
            (NoSign, _) => -other,
            (_, NoSign) => self.clone(),
            (Plus, Plus) => match self.data.cmp(&other.data) {
                Less => BigInt::from_biguint(Minus, &other.data - &self.data),
                Greater => BigInt::from_biguint(Plus, &self.data - &other.data),
                Equal => Zero::zero(),
            },
            (Plus, Minus) => self + (-other),
            (Minus, Plus) => -((-self) + other),
            (Minus, Minus) => (-other) - (-self),
        }
    }
}

forward_all_binop!(impl Mul for BigInt, mul);

impl<'a, 'b> Mul<&'b BigInt> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn mul(self, other: &BigInt) -> BigInt {
        match (self.sign, other.sign) {
            (NoSign, _) | (_, NoSign) => Zero::zero(),
            (Plus, Plus) | (Minus, Minus) => {
                BigInt::from_biguint(Plus, &self.data * &other.data)
            }
            (Plus, Minus) | (Minus, Plus) => {
                BigInt::from_biguint(Minus, &self.data * &other.data)
            }
        }
    }
}

forward_all_binop!(impl Div for BigInt, div);

impl<'a, 'b> Div<&'b BigInt> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn div(self, other: &BigInt) -> BigInt {
        let (q, _) = self.div_rem(other);
        q
    }
}

forward_all_binop!(impl Rem for BigInt, rem);

impl<'a, 'b> Rem<&'b BigInt> for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn rem(self, other: &BigInt) -> BigInt {
        let (_, r) = self.div_rem(other);
        r
    }
}

impl Neg for BigInt {
    type Output = BigInt;

    #[inline]
    fn neg(self) -> BigInt {
        -&self
    }
}

impl<'a> Neg for &'a BigInt {
    type Output = BigInt;

    #[inline]
    fn neg(self) -> BigInt {
        BigInt::from_biguint(self.sign.neg(), self.data.clone())
    }
}

impl CheckedAdd for BigInt {
    #[inline]
    fn checked_add(&self, v: &BigInt) -> Option<BigInt> {
        return Some(self.add(v));
    }
}

impl CheckedSub for BigInt {
    #[inline]
    fn checked_sub(&self, v: &BigInt) -> Option<BigInt> {
        return Some(self.sub(v));
    }
}

impl CheckedMul for BigInt {
    #[inline]
    fn checked_mul(&self, v: &BigInt) -> Option<BigInt> {
        return Some(self.mul(v));
    }
}

impl CheckedDiv for BigInt {
    #[inline]
    fn checked_div(&self, v: &BigInt) -> Option<BigInt> {
        if v.is_zero() {
            return None;
        }
        return Some(self.div(v));
    }
}

impl Integer for BigInt {
    #[inline]
    fn div_rem(&self, other: &BigInt) -> (BigInt, BigInt) {
        // r.sign == self.sign
        let (d_ui, r_ui) = self.data.div_mod_floor(&other.data);
        let d = BigInt::from_biguint(Plus, d_ui);
        let r = BigInt::from_biguint(Plus, r_ui);
        match (self.sign, other.sign) {
            (_, NoSign) => panic!(),
            (Plus, Plus) | (NoSign, Plus) => (d, r),
            (Plus, Minus) | (NoSign, Minus) => (-d, r),
            (Minus, Plus) => (-d, -r),
            (Minus, Minus) => (d, -r),
        }
    }

    #[inline]
    fn div_floor(&self, other: &BigInt) -> BigInt {
        let (d, _) = self.div_mod_floor(other);
        d
    }

    #[inline]
    fn mod_floor(&self, other: &BigInt) -> BigInt {
        let (_, m) = self.div_mod_floor(other);
        m
    }

    fn div_mod_floor(&self, other: &BigInt) -> (BigInt, BigInt) {
        // m.sign == other.sign
        let (d_ui, m_ui) = self.data.div_rem(&other.data);
        let d = BigInt::from_biguint(Plus, d_ui);
        let m = BigInt::from_biguint(Plus, m_ui);
        let one: BigInt = One::one();
        match (self.sign, other.sign) {
            (_, NoSign) => panic!(),
            (Plus, Plus) | (NoSign, Plus) => (d, m),
            (Plus, Minus) | (NoSign, Minus) => {
                if m.is_zero() {
                    (-d, Zero::zero())
                } else {
                    (-d - one, m + other)
                }
            }
            (Minus, Plus) => {
                if m.is_zero() {
                    (-d, Zero::zero())
                } else {
                    (-d - one, other - m)
                }
            }
            (Minus, Minus) => (d, -m),
        }
    }

    /// Calculates the Greatest Common Divisor (GCD) of the number and `other`.
    ///
    /// The result is always positive.
    #[inline]
    fn gcd(&self, other: &BigInt) -> BigInt {
        BigInt::from_biguint(Plus, self.data.gcd(&other.data))
    }

    /// Calculates the Lowest Common Multiple (LCM) of the number and `other`.
    #[inline]
    fn lcm(&self, other: &BigInt) -> BigInt {
        BigInt::from_biguint(Plus, self.data.lcm(&other.data))
    }

    /// Deprecated, use `is_multiple_of` instead.
    #[inline]
    fn divides(&self, other: &BigInt) -> bool {
        return self.is_multiple_of(other);
    }

    /// Returns `true` if the number is a multiple of `other`.
    #[inline]
    fn is_multiple_of(&self, other: &BigInt) -> bool {
        self.data.is_multiple_of(&other.data)
    }

    /// Returns `true` if the number is divisible by `2`.
    #[inline]
    fn is_even(&self) -> bool {
        self.data.is_even()
    }

    /// Returns `true` if the number is not divisible by `2`.
    #[inline]
    fn is_odd(&self) -> bool {
        self.data.is_odd()
    }
}

impl ToPrimitive for BigInt {
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        match self.sign {
            Plus => self.data.to_i64(),
            NoSign => Some(0),
            Minus => {
                self.data.to_u64().and_then(|n| {
                    let m: u64 = 1 << 63;
                    if n < m {
                        Some(-(n as i64))
                    } else if n == m {
                        Some(i64::MIN)
                    } else {
                        None
                    }
                })
            }
        }
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        match self.sign {
            Plus => self.data.to_u64(),
            NoSign => Some(0),
            Minus => None,
        }
    }
}

impl FromPrimitive for BigInt {
    #[inline]
    fn from_i64(n: i64) -> Option<BigInt> {
        if n > 0 {
            FromPrimitive::from_u64(n as u64).and_then(|n| Some(BigInt::from_biguint(Plus, n)))
        } else if n < 0 {
            FromPrimitive::from_u64(u64::MAX - (n as u64) + 1)
                .and_then(|n| Some(BigInt::from_biguint(Minus, n)))
        } else {
            Some(Zero::zero())
        }
    }

    #[inline]
    fn from_u64(n: u64) -> Option<BigInt> {
        if n == 0 {
            Some(Zero::zero())
        } else {
            FromPrimitive::from_u64(n).and_then(|n| Some(BigInt::from_biguint(Plus, n)))
        }
    }
}

/// A generic trait for converting a value to a `BigInt`.
pub trait ToBigInt {
    /// Converts the value of `self` to a `BigInt`.
    fn to_bigint(&self) -> Option<BigInt>;
}

impl ToBigInt for BigInt {
    #[inline]
    fn to_bigint(&self) -> Option<BigInt> {
        Some(self.clone())
    }
}

impl ToBigInt for BigUint {
    #[inline]
    fn to_bigint(&self) -> Option<BigInt> {
        if self.is_zero() {
            Some(Zero::zero())
        } else {
            Some(BigInt {
                sign: Plus,
                data: self.clone(),
            })
        }
    }
}

macro_rules! impl_to_bigint {
    ($T:ty, $from_ty:path) => {
        impl ToBigInt for $T {
            #[inline]
            fn to_bigint(&self) -> Option<BigInt> {
                $from_ty(*self)
            }
        }
    }
}

impl_to_bigint!(isize,  FromPrimitive::from_isize);
impl_to_bigint!(i8,   FromPrimitive::from_i8);
impl_to_bigint!(i16,  FromPrimitive::from_i16);
impl_to_bigint!(i32,  FromPrimitive::from_i32);
impl_to_bigint!(i64,  FromPrimitive::from_i64);
impl_to_bigint!(usize, FromPrimitive::from_usize);
impl_to_bigint!(u8,   FromPrimitive::from_u8);
impl_to_bigint!(u16,  FromPrimitive::from_u16);
impl_to_bigint!(u32,  FromPrimitive::from_u32);
impl_to_bigint!(u64,  FromPrimitive::from_u64);

pub trait RandBigInt {
    /// Generate a random `BigUint` of the given bit size.
    fn gen_biguint(&mut self, bit_size: usize) -> BigUint;

    /// Generate a random BigInt of the given bit size.
    fn gen_bigint(&mut self, bit_size: usize) -> BigInt;

    /// Generate a random `BigUint` less than the given bound. Fails
    /// when the bound is zero.
    fn gen_biguint_below(&mut self, bound: &BigUint) -> BigUint;

    /// Generate a random `BigUint` within the given range. The lower
    /// bound is inclusive; the upper bound is exclusive. Fails when
    /// the upper bound is not greater than the lower bound.
    fn gen_biguint_range(&mut self, lbound: &BigUint, ubound: &BigUint) -> BigUint;

    /// Generate a random `BigInt` within the given range. The lower
    /// bound is inclusive; the upper bound is exclusive. Fails when
    /// the upper bound is not greater than the lower bound.
    fn gen_bigint_range(&mut self, lbound: &BigInt, ubound: &BigInt) -> BigInt;
}

impl<R: Rng> RandBigInt for R {
    fn gen_biguint(&mut self, bit_size: usize) -> BigUint {
        let (digits, rem) = bit_size.div_rem(&big_digit::BITS);
        let mut data = Vec::with_capacity(digits + 1);
        for _ in (0..digits) {
            data.push(self.gen());
        }
        if rem > 0 {
            let final_digit: BigDigit = self.gen();
            data.push(final_digit >> (big_digit::BITS - rem));
        }
        BigUint::new(data)
    }

    fn gen_bigint(&mut self, bit_size: usize) -> BigInt {
        // Generate a random BigUint...
        let biguint = self.gen_biguint(bit_size);
        // ...and then randomly assign it a Sign...
        let sign = if biguint.is_zero() {
            // ...except that if the BigUint is zero, we need to try
            // again with probability 0.5. This is because otherwise,
            // the probability of generating a zero BigInt would be
            // double that of any other number.
            if self.gen() {
                return self.gen_bigint(bit_size);
            } else {
                NoSign
            }
        } else if self.gen() {
            Plus
        } else {
            Minus
        };
        BigInt::from_biguint(sign, biguint)
    }

    fn gen_biguint_below(&mut self, bound: &BigUint) -> BigUint {
        assert!(!bound.is_zero());
        let bits = bound.bits();
        loop {
            let n = self.gen_biguint(bits);
            if n < *bound {
                return n;
            }
        }
    }

    fn gen_biguint_range(&mut self, lbound: &BigUint, ubound: &BigUint) -> BigUint {
        assert!(*lbound < *ubound);
        return lbound + self.gen_biguint_below(&(ubound - lbound));
    }

    fn gen_bigint_range(&mut self, lbound: &BigInt, ubound: &BigInt) -> BigInt {
        assert!(*lbound < *ubound);
        let delta = (ubound - lbound).to_biguint().unwrap();
        return lbound + self.gen_biguint_below(&delta).to_bigint().unwrap();
    }
}

impl BigInt {
    /// Creates and initializes a BigInt.
    ///
    /// The digits are in little-endian base 2^32.
    #[inline]
    pub fn new(sign: Sign, digits: Vec<BigDigit>) -> BigInt {
        BigInt::from_biguint(sign, BigUint::new(digits))
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The digits are in little-endian base 2^32.
    #[inline]
    pub fn from_biguint(sign: Sign, data: BigUint) -> BigInt {
        if sign == NoSign || data.is_zero() {
            return BigInt {
                sign: NoSign,
                data: Zero::zero(),
            };
        }
        BigInt {
            sign: sign,
            data: data,
        }
    }

    /// Creates and initializes a `BigInt`.
    #[inline]
    pub fn from_slice(sign: Sign, slice: &[BigDigit]) -> BigInt {
        BigInt::from_biguint(sign, BigUint::from_slice(slice))
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            BigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            BigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            BigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            BigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> BigInt {
        BigInt::from_biguint(sign, BigUint::from_bytes_be(bytes))
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> BigInt {
        BigInt::from_biguint(sign, BigUint::from_bytes_le(bytes))
    }

    /// Returns the sign and the byte representation of the `BigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::{ToBigInt, Sign};
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        (self.sign, self.data.to_bytes_le())
    }

    /// Returns the sign and the byte representation of the `BigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::{ToBigInt, Sign};
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        (self.sign, self.data.to_bytes_be())
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num::bigint::{BigInt, ToBigInt};
    ///
    /// assert_eq!(BigInt::parse_bytes(b"1234", 10), ToBigInt::to_bigint(&1234));
    /// assert_eq!(BigInt::parse_bytes(b"ABCD", 16), ToBigInt::to_bigint(&0xABCD));
    /// assert_eq!(BigInt::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<BigInt> {
        str::from_utf8(buf).ok().and_then(|s| BigInt::from_str_radix(s, radix).ok())
    }


    /// Converts this `BigInt` into a `BigUint`, if it's not negative.
    #[inline]
    pub fn to_biguint(&self) -> Option<BigUint> {
        match self.sign {
            Plus => Some(self.data.clone()),
            NoSign => Some(Zero::zero()),
            Minus => None,
        }
    }

    #[inline]
    pub fn checked_add(&self, v: &BigInt) -> Option<BigInt> {
        return Some(self.add(v));
    }

    #[inline]
    pub fn checked_sub(&self, v: &BigInt) -> Option<BigInt> {
        return Some(self.sub(v));
    }

    #[inline]
    pub fn checked_mul(&self, v: &BigInt) -> Option<BigInt> {
        return Some(self.mul(v));
    }

    #[inline]
    pub fn checked_div(&self, v: &BigInt) -> Option<BigInt> {
        if v.is_zero() {
            return None;
        }
        return Some(self.div(v));
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseBigIntError {
    ParseInt(ParseIntError),
    Other,
}

impl fmt::Display for ParseBigIntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ParseBigIntError::ParseInt(ref e) => e.fmt(f),
            &ParseBigIntError::Other => "failed to parse provided string".fmt(f),
        }
    }
}

impl Error for ParseBigIntError {
    fn description(&self) -> &str {
        "failed to parse bigint/biguint"
    }
}

impl From<ParseIntError> for ParseBigIntError {
    fn from(err: ParseIntError) -> ParseBigIntError {
        ParseBigIntError::ParseInt(err)
    }
}
