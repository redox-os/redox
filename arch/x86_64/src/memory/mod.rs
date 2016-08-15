//! # Memory management
//! Some code was borrowed from [Phil Opp's Blog](http://os.phil-opp.com/allocating-frames.html)

pub use paging::{PAGE_SIZE, PhysicalAddress};

use self::area_frame_alloc::AreaFrameAllocator;

pub mod area_frame_alloc;

/// The current memory map. It's size is maxed out to 512 entries, due to it being
/// from 0x500 to 0x5000 (800 is the absolute total)
static mut MEMORY_MAP: [MemoryArea; 512] = [MemoryArea { base_addr: 0, length: 0, _type: 0, acpi: 0 }; 512];

/// Memory does not exist
const MEMORY_AREA_NULL: u32 = 0;

/// Memory is free to use
const MEMORY_AREA_FREE: u32 = 1;

/// Memory is reserved
const MEMORY_AREA_RESERVED: u32 = 2;

/// Memory is used by ACPI, and can be reclaimed
const MEMORY_AREA_ACPI: u32 = 3;

#[derive(Clone)]
pub struct MemoryAreaIter {
    _type: u32,
    i: usize
}

impl MemoryAreaIter {
    fn new(_type: u32) -> Self {
        MemoryAreaIter {
            _type: _type,
            i: 0
        }
    }
}

impl Iterator for MemoryAreaIter {
    type Item = &'static MemoryArea;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < unsafe { MEMORY_MAP.len() } {
            let entry = unsafe { &MEMORY_MAP[self.i] };
            self.i += 1;
            if entry._type == self._type {
                return Some(entry);
            }
        }
        None
    }
}

/// Init memory module
/// Must be called once, and only once,
pub unsafe fn init(kernel_start: usize, kernel_end: usize) -> AreaFrameAllocator {
    // Copy memory map from bootloader location
    for (i, mut entry) in MEMORY_MAP.iter_mut().enumerate() {
        *entry = *(0x500 as *const MemoryArea).offset(i as isize);
        if entry.length > 0 {
            println!("{:?}", entry);
        }
    }

    AreaFrameAllocator::new(kernel_start, kernel_end, MemoryAreaIter::new(MEMORY_AREA_FREE))
}

/// A memory map area
#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct MemoryArea {
    pub base_addr: u64,
    pub length: u64,
    pub _type: u32,
    pub acpi: u32
}

/// A frame, allocated by the frame allocator.
/// Do not add more derives, or make anything `pub`!
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize
}

impl Frame {
    /// Get the address of this frame
    pub fn start_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.number * PAGE_SIZE)
    }

    //TODO: Set private
    pub fn clone(&self) -> Frame {
        Frame {
            number: self.number
        }
    }

    /// Create a frame containing `address`
    pub fn containing_address(address: PhysicalAddress) -> Frame {
        Frame {
            number: address.get() / PAGE_SIZE
        }
    }

    //TODO: Set private
    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
 }

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}
