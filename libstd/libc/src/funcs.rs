use super::{c_char, size_t};

pub unsafe extern fn strlen(ptr: *const c_char) -> size_t {
    let mut i: size_t = 0;
    while *ptr.offset(i as isize) != 0 {
        i += 1;
    }
    i
}

extern crate spin;
use core::cell::Cell;
use self::spin::Mutex;

/// The randomness state.
///
/// This is updated when a new random integer is read.
static STATE: Mutex<[u64; 2]> = Mutex::new([0xBADF00D1, 0xDEADBEEF]);

/// Get a pseudorandom integer.
///
/// Note that this is full-cycle, so apply a modulo when true equidistribution is needed.
pub unsafe extern fn random() -> u64 {
    // Fetch the state.
    let mut state = STATE.lock();

    // Store the first and second part.
    let mut x = state[0];
    let y = state[1];

    // Put the second part into the first slot.
    state[0] = y;
    // Twist the first slot.
    x ^= x << 23;
    // Update the second slot.
    state[1] = x ^ y ^ (x >> 17) ^ (y >> 26);

    // Generate the final integer.
    state[1].wrapping_add(y)
}
