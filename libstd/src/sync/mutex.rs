use cell::UnsafeCell;
use intrinsics::{atomic_cxchg, atomic_xadd, atomic_xchg};
use ops::{Deref, DerefMut, Drop};
use ptr;

use system::syscall::{sys_futex, FUTEX_WAIT, FUTEX_WAKE, FUTEX_REQUEUE};

unsafe fn mutex_lock(m: *mut i32) {
    let mut c = 0;
    //Set to larger value for longer spin test
    for _i in 0..1 {
        c = atomic_cxchg(m, 0, 1).0;
        if c == 0 {
            break;
        }
        //cpu_relax()
    }
    if c == 1 {
        c = atomic_xchg(m, 2);
    }
    while c != 0 {
        let _ = sys_futex(m, FUTEX_WAIT, 2, 0, ptr::null_mut());
        c = atomic_xchg(m, 2);
    }
}

unsafe fn mutex_unlock(m: *mut i32) {
    if *m == 2 {
        *m = 0;
    } else if atomic_xchg(m, 0) == 1 {
        return;
    }
    //Set to larger value for longer spin test
    for _i in 0..1 {
        if *m != 0 {
            if atomic_cxchg(m, 1, 2).0 != 0 {
                return;
            }
        }
        //cpu_relax()
    }
    let _ = sys_futex(m, FUTEX_WAKE, 1, 0, ptr::null_mut());
}

pub struct Condvar {
    lock: UnsafeCell<*mut i32>,
    seq: UnsafeCell<i32>
}

impl Condvar {
    pub fn new() -> Condvar {
        Condvar {
            lock: UnsafeCell::new(ptr::null_mut()),
            seq: UnsafeCell::new(0)
        }
    }

    pub fn notify_one(&self) {
        unsafe {
            let seq = self.seq.get();

            atomic_xadd(seq, 1);

            let _ = sys_futex(seq, FUTEX_WAKE, 1, 0, ptr::null_mut());
        }
    }

    pub fn notify_all(&self) {
        unsafe {
            let lock = self.lock.get();
            let seq = self.seq.get();

            if *lock == ptr::null_mut() {
                return;
            }

            atomic_xadd(seq, 1);

            let _ = sys_futex(seq, FUTEX_REQUEUE, 1, ::usize::MAX, *lock);
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> Result<MutexGuard<'a, T>, ()> {
        unsafe {
            let lock = self.lock.get();
            let seq = self.seq.get();

            if *lock != guard.lock.get() {
                if *lock != ptr::null_mut() {
                    panic!("Condvar used with more than one Mutex");
                }

                atomic_cxchg(lock as *mut usize, 0, guard.lock.get() as usize);
            }

            mutex_unlock(*lock);

            let _ = sys_futex(seq, FUTEX_WAIT, *seq, 0, ptr::null_mut());

            while atomic_xchg(*lock, 2) != 0 {
                let _ = sys_futex(*lock, FUTEX_WAIT, 2, 0, ptr::null_mut());
            }

            mutex_lock(*lock);
        }

        Ok(guard)
    }
}

/// A mutex, i.e. a form of safe shared memory between threads. See rust std's Mutex.
pub struct Mutex<T: ?Sized> {
    lock: UnsafeCell<i32>,
    value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Create a new mutex with value `value`.
    pub fn new(value: T) -> Self {
        Mutex {
            lock: UnsafeCell::new(0),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Lock the mutex
    pub fn lock(&self) -> Result<MutexGuard<T>, ()> {
        unsafe { mutex_lock(self.lock.get()) };
        Ok(MutexGuard::new(&self.lock, &self.value))
    }
}

struct Dummy(UnsafeCell<()>);
unsafe impl Sync for Dummy {}
static DUMMY: Dummy = Dummy(UnsafeCell::new(()));

pub struct StaticMutex {
    lock: UnsafeCell<i32>,
}

impl StaticMutex {
    /// Create a new mutex with value `value`.
    pub const fn new() -> Self {
        StaticMutex {
            lock: UnsafeCell::new(0),
        }
    }

    /// Lock the mutex
    pub fn lock(&'static self) -> Result<MutexGuard<()>, ()> {
        unsafe { mutex_lock(self.lock.get()) };
        Ok(MutexGuard::new(&self.lock, &DUMMY.0)) // TODO catch panics
    }

    pub unsafe fn destroy(&'static self) {
    }
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

/// A mutex guard (returned by .lock())
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a UnsafeCell<i32>,
    data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    fn new(lock: &'mutex UnsafeCell<i32>, data: &'mutex UnsafeCell<T>) -> Self {
        MutexGuard {
            lock: lock,
            data: data,
        }
    }
}

impl<'mutex, T: ?Sized> Deref for MutexGuard<'mutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { mutex_unlock(self.lock.get()) };
    }
}
