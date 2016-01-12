
// Compatibility macros/typedefs needed for Solaris -> Linux port
pub fn p2_align(x: u64, align: u64) -> u64 {
    x & -(align as i64) as u64
}

fn p2_cross(x: u64, y: u64, align: u64) -> bool {
    x ^ y > align - 1
}

fn p2_round_up(x: u64, align: u64) -> u64 {
    ((x - 1) | (align - 1)) + 1
}

fn p2_boundary(off: u64, len: u64, align: u64) -> bool {
    (off ^ (off + len - 1)) > (align - 1)
}

fn p2_phase(x: u64, align: u64) -> u64 {
    x & (align - 1)
}

fn p2_nphase(x: u64, align: u64) -> u64 {
    -(x as i64) as u64 & (align - 1)
}

fn p2_nphase_typed(x: u64, align: u64) -> u64 {
    -(x as i64) as u64 & (align - 1)
}

fn is_p2(x: u64) -> bool {
    x & (x - 1) == 0
}

fn is_p2_aligned(v: u64, a: u64) -> bool {
    v & (a - 1) == 0
}

pub fn highbit64(u: u64) -> u32 {
    63 - u.leading_zeros()
}

// Typed version of the P2* macros.  These macros should be used to ensure
// that the result is correctly calculated based on the data type of (x),
// which is passed in as the last argument, regardless of the data
// type of the alignment.  For example, if (x) is of type uint64_t,
// and we want to round it up to a page boundary using "PAGESIZE" as
// the alignment, we can do either
//      P2ROUNDUP(x, (uint64_t)PAGESIZE)
// or
//      P2ROUNDUP_TYPED(x, PAGESIZE, uint64_t)
//
// #define P2ALIGN_TYPED(x, align, type)       \
// ((type)(x) & -(type)(align))
// #define P2PHASE_TYPED(x, align, type)       \
// ((type)(x) & ((type)(align) - 1))
// #define P2NPHASE_TYPED(x, align, type)      \
// (-(type)(x) & ((type)(align) - 1))
// #define P2ROUNDUP_TYPED(x, align, type)     \
// ((((type)(x) - 1) | ((type)(align) - 1)) + 1)
// #define P2END_TYPED(x, align, type)     \
// (-(~(type)(x) & -(type)(align)))
// #define P2PHASEUP_TYPED(x, align, phase, type)  \
// ((type)(phase) - (((type)(phase) - (type)(x)) & -(type)(align)))
// #define P2CROSS_TYPED(x, y, align, type)    \
// (((type)(x) ^ (type)(y)) > (type)(align) - 1)
// #define P2SAMEHIGHBIT_TYPED(x, y, type)     \
// (((type)(x) ^ (type)(y)) < ((type)(x) & (type)(y)))
//
//
// avoid any possibility of clashing with <stddef.h> version
// #if defined(_KERNEL) && !defined(_KMEMUSER) && !defined(offsetof)
// #define offsetof(s, m)  ((size_t)(&(((s *)0)->m)))
// #endif
