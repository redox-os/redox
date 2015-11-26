#![feature(test)]

extern crate test;
extern crate num;

use std::mem::replace;
use test::Bencher;
use num::{BigUint, Zero, One, FromPrimitive};

fn factorial(n: usize) -> BigUint {
    let mut f: BigUint = One::one();
    for i in 1..(n + 1) {
        let bu: BigUint = FromPrimitive::from_usize(i).unwrap();
        f = f * bu;
    }
    f
}

fn fib(n: usize) -> BigUint {
    let mut f0: BigUint = Zero::zero();
    let mut f1: BigUint = One::one();
    for _ in 0..n {
        let f2 = f0 + &f1;
        f0 = replace(&mut f1, f2);
    }
    f0
}

#[bench]
fn factorial_100(b: &mut Bencher) {
    b.iter(|| {
        factorial(100);
    });
}

#[bench]
fn fib_100(b: &mut Bencher) {
    b.iter(|| {
        fib(100);
    });
}

#[bench]
fn to_string(b: &mut Bencher) {
    let fac = factorial(100);
    let fib = fib(100);
    b.iter(|| {
        fac.to_string();
    });
    b.iter(|| {
        fib.to_string();
    });
}

#[bench]
fn shr(b: &mut Bencher) {
    let n = {
        let one: BigUint = One::one();
        one << 1000
    };
    b.iter(|| {
        let mut m = n.clone();
        for _ in 0..10 {
            m = m >> 1;
        }
    })
}
