use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_range: (Frame, Frame),
    mb_range: (Frame, Frame)
}

impl AreaFrameAllocator {
    pub fn new(memory_areas: MemoryAreaIter, kernel_range: (usize, usize), mb_range: (usize, usize)) -> AreaFrameAllocator {
        let (kernel_low_addr, kernel_high_addr) = kernel_range;
        let (mb_low_addr, mb_high_addr) = mb_range;
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::for_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_range: (Frame::for_address(kernel_low_addr), Frame::for_address(kernel_high_addr)),
            mb_range: (Frame::for_address(mb_low_addr), Frame::for_address(mb_high_addr))
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        // use area with smallest base address that has free frames (ending
        // frame of the area is at least the next free frame)
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.base_addr + area.length - 1;
            Frame::for_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::for_address(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::for_address(address as usize)
            };

            if self.next_free_frame > current_area_last_frame {
                // choose the next area and retry
                self.choose_next_area();
                return self.allocate_frame();
            }

            let (Frame(kernel_start), Frame(kernel_end)) = self.kernel_range;
            let (Frame(mb_start), Frame(mb_end)) = self.mb_range;
            

            let Frame(next_free) = self.next_free_frame;

            // check if next free frame is in the kernel range
            if kernel_start <= next_free && next_free <= kernel_end {
                self.next_free_frame = Frame(kernel_end + 1);
                self.allocate_frame()
            }
            // check if next free frame is in the multiboot2 range
            else if mb_start <= next_free && next_free <= mb_end {
                self.next_free_frame = Frame(mb_end + 1);
                self.allocate_frame()
            } else {
                // frame is not in any banned areas, allocate it
                self.next_free_frame = Frame(next_free + 1);
                Some(Frame(next_free))
            }
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        unimplemented!()
    }
}