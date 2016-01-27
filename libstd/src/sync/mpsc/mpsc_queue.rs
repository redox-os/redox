pub use self::PopResult::*;

use alloc::boxed::Box;
use core::ptr;
use core::cell::UnsafeCell;

// TODO: Add docs (what the hell would you expect?)

use sync::atomic::{AtomicPtr, Ordering};

pub enum PopResult<T> {
    Invalid,
    Data(T),
    Empty,
}

struct Node<T> {
    val: Option<T>,
    next: AtomicPtr<Node<T>>,
}

pub struct Queue<T> {
    head: AtomicPtr<Node<T>>,
    tail: UnsafeCell<*mut Node<T>>,
}

unsafe impl<T: Send> Sync for Queue<T> {}

unsafe impl<T: Send> Send for Queue<T> {}

impl<T> Node<T> {
    pub unsafe fn new(v: Option<T>) -> *mut Node<T> {
        Box::into_raw(box Node {
            val: v,
            next: AtomicPtr::new(ptr::null_mut()),
        })
    }
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        let stub = unsafe { Node::new(None) };
        Queue {
            head: AtomicPtr::new(stub),
            tail: UnsafeCell::new(stub),
        }
    }

    pub fn push(&self, t: T) {
        unsafe {
            let n    = Node::new(Some(t));
            let prev = self.head.swap(n, Ordering::AcqRel);

            (*prev).next.store(n, Ordering::Release);
        }
    }

    pub fn pop(&self) -> PopResult<T> {
        unsafe {
            let tail = *self.tail.get();
            let next = (*tail).next.load(Ordering::Acquire);

            if !next.is_null() {
                *self.tail.get() = next;

                // Make sure the queue is coherent
                assert!((*tail).val.is_none());
                assert!((*next).val.is_some());

                let _: Box<Node<T>> = Box::from_raw(tail);
                let ret             = (*next).val.take().unwrap();

                Data(ret)
            } else if self.head.load(Ordering::Acquire) == tail {
                Empty
            } else {
                Invalid
            }
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        unsafe {
            let mut cur = *self.tail.get();
            while !cur.is_null() {
                let _: Box<Node<T>> = Box::from_raw(cur);
                let next            = (*cur).next.load(Ordering::Relaxed); // Use relaxed ordering

                cur = next;
            }
        }
    }
}

