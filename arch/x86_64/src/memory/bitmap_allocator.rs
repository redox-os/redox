use super::AreaFrameAllocator;

const BITMAP_RESERVED: usize = 0;
const BITMAP_FREE: usize = 1;
const BITMAP_USED: usize = 2;

pub struct BitmapAllocator {
    bitmap: &'static mut [u8]
}

impl BitmapAllocator {
    pub fn new(area_frame_allocator: AreaFrameAllocator) -> BitmapAllocator {
        BitmapAllocator {
            bitmap: &mut []
        }
    }
}

impl FrameAllocator for BitmapAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let mut i = 0;
        while i < self.bitmap.len() {
            if self.bitmap[i] == BITMAP_FREE {
                self.bitmap[i] = BITMAP_USED;
                return Some(Frame::containing_address(PhysicalAddress::new(i * 4096)));
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        let i = frame.starting_address().get()/4096;
        if i < self.bitmap.len() && self.bitmap[i] == BITMAP_USED {
            self.bitmap[i] = BITMAP_FREE;
        } else {
            panic!("BitmapAllocator::deallocate_frame: unowned frame {:?}", frame);
        }
    }
}
