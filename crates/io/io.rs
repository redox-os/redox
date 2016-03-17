use core::cmp::PartialEq;
use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, Not};

pub trait Io<T> {
    fn read(&self) -> T;
    fn write(&mut self, value: T);

    fn readf(&self, flags: T) -> bool where T: BitAnd<Output = T> + PartialEq<T> + Copy {
        (self.read() & flags) as T == flags
    }

    fn writef(&mut self, flags: T, value: bool) where T: BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T> {
        let tmp: T = match value {
            true => self.read() | flags,
            false => self.read() & !flags,
        };
        self.write(tmp);
    }
}

pub struct ReadOnly<T, I: Io<T>> {
    inner: I,
    value: PhantomData<T>,
}

impl<T, I: Io<T>> ReadOnly<T, I> {
    pub fn new(inner: I) -> ReadOnly<T, I> {
        ReadOnly {
            inner: inner,
            value: PhantomData,
        }
    }

    pub fn read(&self) -> T {
        self.inner.read()
    }

    pub fn readf(&self, flags: T) -> bool where T: BitAnd<Output = T> + PartialEq<T> + Copy {
        self.inner.readf(flags)
    }
}

pub struct WriteOnly<T, I: Io<T>> {
    inner: I,
    value: PhantomData<T>,
}

impl<T, I: Io<T>> WriteOnly<T, I> {
    pub fn new(inner: I) -> WriteOnly<T, I> {
        WriteOnly {
            inner: inner,
            value: PhantomData,
        }
    }

    pub fn write(&mut self, value: T) {
        self.inner.write(value)
    }

    pub fn writef(&mut self, flags: T, value: bool) where T: BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T> {
        self.inner.writef(flags, value)
    }
}
