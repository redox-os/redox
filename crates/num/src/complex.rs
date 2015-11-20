// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


//! Complex numbers.

use redox::fmt;
use redox::ops::{Add, Div, Mul, Neg, Sub};

use {Zero, One, Num, Float};

// FIXME #1284: handle complex NaN & infinity etc. This
// probably doesn't map to C's _Complex correctly.

/// A complex number in Cartesian form.
#[derive(PartialEq, Copy, Clone, Hash, Debug)]
pub struct Complex<T> {
    /// Real portion of the complex number
    pub re: T,
    /// Imaginary portion of the complex number
    pub im: T,
}

pub type Complex32 = Complex<f32>;
pub type Complex64 = Complex<f64>;

impl<T: Clone + Num> Complex<T> {
    /// Create a new Complex
    #[inline]
    pub fn new(re: T, im: T) -> Complex<T> {
        Complex { re: re, im: im }
    }

    /// Returns the square of the norm (since `T` doesn't necessarily
    /// have a sqrt function), i.e. `re^2 + im^2`.
    #[inline]
    pub fn norm_sqr(&self) -> T {
        self.re.clone() * self.re.clone() + self.im.clone() * self.im.clone()
    }

    /// Multiplies `self` by the scalar `t`.
    #[inline]
    pub fn scale(&self, t: T) -> Complex<T> {
        Complex::new(self.re.clone() * t.clone(), self.im.clone() * t)
    }

    /// Divides `self` by the scalar `t`.
    #[inline]
    pub fn unscale(&self, t: T) -> Complex<T> {
        Complex::new(self.re.clone() / t.clone(), self.im.clone() / t)
    }
}

impl<T: Clone + Num + Neg<Output = T>> Complex<T> {
    /// Returns the complex conjugate. i.e. `re - i im`
    #[inline]
    pub fn conj(&self) -> Complex<T> {
        Complex::new(self.re.clone(), -self.im.clone())
    }

    /// Returns `1/self`
    #[inline]
    pub fn inv(&self) -> Complex<T> {
        let norm_sqr = self.norm_sqr();
        Complex::new(self.re.clone() / norm_sqr.clone(),
                     -self.im.clone() / norm_sqr)
    }
}

impl<T: Clone + Float> Complex<T> {
    /// Calculate |self|
    #[inline]
    pub fn norm(&self) -> T {
        self.re.clone().hypot(self.im.clone())
    }
}

impl<T: Clone + Float + Num> Complex<T> {
    /// Calculate the principal Arg of self.
    #[inline]
    pub fn arg(&self) -> T {
        self.im.clone().atan2(self.re.clone())
    }
    /// Convert to polar form (r, theta), such that `self = r * exp(i
    /// * theta)`
    #[inline]
    pub fn to_polar(&self) -> (T, T) {
        (self.norm(), self.arg())
    }
    /// Convert a polar representation into a complex number.
    #[inline]
    pub fn from_polar(r: &T, theta: &T) -> Complex<T> {
        Complex::new(*r * theta.cos(), *r * theta.sin())
    }
}

macro_rules! forward_val_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl<T: Clone + Num> $imp<Complex<T>> for Complex<T> {
            type Output = Complex<T>;

            #[inline]
            fn $method(self, other: Complex<T>) -> Complex<T> {
                (&self).$method(&other)
            }
        }
    }
}

macro_rules! forward_ref_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, T: Clone + Num> $imp<Complex<T>> for &'a Complex<T> {
            type Output = Complex<T>;

            #[inline]
            fn $method(self, other: Complex<T>) -> Complex<T> {
                self.$method(&other)
            }
        }
    }
}

macro_rules! forward_val_ref_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, T: Clone + Num> $imp<&'a Complex<T>> for Complex<T> {
            type Output = Complex<T>;

            #[inline]
            fn $method(self, other: &Complex<T>) -> Complex<T> {
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

// arithmetic
forward_all_binop!(impl Add, add);

// (a + i b) + (c + i d) == (a + c) + i (b + d)
impl<'a, 'b, T: Clone + Num> Add<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn add(self, other: &Complex<T>) -> Complex<T> {
        Complex::new(self.re.clone() + other.re.clone(),
                     self.im.clone() + other.im.clone())
    }
}

forward_all_binop!(impl Sub, sub);

// (a + i b) - (c + i d) == (a - c) + i (b - d)
impl<'a, 'b, T: Clone + Num> Sub<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn sub(self, other: &Complex<T>) -> Complex<T> {
        Complex::new(self.re.clone() - other.re.clone(),
                     self.im.clone() - other.im.clone())
    }
}

forward_all_binop!(impl Mul, mul);

// (a + i b) * (c + i d) == (a*c - b*d) + i (a*d + b*c)
impl<'a, 'b, T: Clone + Num> Mul<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn mul(self, other: &Complex<T>) -> Complex<T> {
        Complex::new(self.re.clone() * other.re.clone() - self.im.clone() * other.im.clone(),
                     self.re.clone() * other.im.clone() + self.im.clone() * other.re.clone())
    }
}

forward_all_binop!(impl Div, div);

// (a + i b) / (c + i d) == [(a + i b) * (c - i d)] / (c*c + d*d)
//   == [(a*c + b*d) / (c*c + d*d)] + i [(b*c - a*d) / (c*c + d*d)]
impl<'a, 'b, T: Clone + Num> Div<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn div(self, other: &Complex<T>) -> Complex<T> {
        let norm_sqr = other.norm_sqr();
        Complex::new((self.re.clone() * other.re.clone() + self.im.clone() * other.im.clone()) /
                     norm_sqr.clone(),
                     (self.im.clone() * other.re.clone() - self.re.clone() * other.im.clone()) /
                     norm_sqr)
    }
}

impl<T: Clone + Num + Neg<Output = T>> Neg for Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn neg(self) -> Complex<T> {
        -&self
    }
}

impl<'a, T: Clone + Num + Neg<Output = T>> Neg for &'a Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn neg(self) -> Complex<T> {
        Complex::new(-self.re.clone(), -self.im.clone())
    }
}

// constants
impl<T: Clone + Num> Zero for Complex<T> {
    #[inline]
    fn zero() -> Complex<T> {
        Complex::new(Zero::zero(), Zero::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }
}

impl<T: Clone + Num> One for Complex<T> {
    #[inline]
    fn one() -> Complex<T> {
        Complex::new(One::one(), Zero::zero())
    }
}

// string conversions
impl<T> fmt::Display for Complex<T> where
    T: fmt::Display + Num + PartialOrd + Clone
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.im < Zero::zero() {
            write!(f, "{}-{}i", self.re, T::zero() - self.im.clone())
        } else {
            write!(f, "{}+{}i", self.re, self.im)
        }
    }
}
