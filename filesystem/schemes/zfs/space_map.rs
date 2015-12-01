const SPACE_MAP_HISTOGRAM_SIZE: usize = 32;

pub struct SpaceMapPhys {
    object: u64, // on-disk space map object
    objsize: u64, // size of the object
    alloc: u64, // space allocated from the map
    pad: [u64; 5], // reserved

    //
    // The smp_histogram maintains a histogram of free regions. Each
    // bucket, smp_histogram[i], contains the number of free regions
    // whose size is:
    // 2^(i+sm_shift) <= size of free region in bytes < 2^(i+sm_shift+1)
    //
    histogram: [u64; SPACE_MAP_HISTOGRAM_SIZE],
}
