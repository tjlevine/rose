pub use self::area_frame_allocator::AreaFrameAllocator;

// temporary testing function
pub use self::paging::test_paging;

use self::paging::PhysicalAddress;

mod area_frame_allocator;
mod paging;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame(usize);

pub const PAGE_SIZE: usize = 4096;

impl Frame {
    fn for_address(address: usize) -> Frame {
        Frame(address / PAGE_SIZE)
    }

    fn start_address(&self) -> PhysicalAddress {
        self.0 * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}
