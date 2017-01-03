use memory::PAGE_SIZE;
use memory::Frame;
use memory::FrameAllocator;

const ENTRY_COUNT: usize = 512;

mod entry;
mod table;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub use self::entry::*;

pub struct Page(usize);

impl Page {
    pub fn for_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xFFFF_8000_0000_0000);
        Page(address / PAGE_SIZE)
    }

    fn start_address(&self) -> usize {
        self.0 * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.0 >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.0 >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.0 >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.0 >> 0) & 0o777
    }
}

use self::table::{Table, Level4};
use core::ptr::Unique;

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new(table::P4)
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.get() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.get_mut() }
    }

    pub fn translate(&self, virtual_addr: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_addr % PAGE_SIZE;
        self.translate_page(Page::for_address(virtual_addr))
            .map(|Frame(num)| num * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {

        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                // is this a 1 GiB page?
                if let Some(Frame(start_frame_num)) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        // address must be 1 GiB aligned
                        assert!(start_frame_num % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        let frame_num = start_frame_num + page.p2_index() * ENTRY_COUNT + page.p1_index();
                        return Some(Frame(frame_num))
                    }
                }
                
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    // 2 MiB page?
                    if let Some(Frame(start_frame_num)) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            // address must be 2 MiB aligned
                            assert!(start_frame_num % ENTRY_COUNT == 0);
                            return Some(Frame(start_frame_num + page.p1_index()))
                        }
                    }
                }

                // should never get here
                None
            })
        };

        let ret_frame = p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].pointed_frame())
        .or_else(huge_page);

        if let Some(ref frame) = ret_frame {
            println!("translate page {}: frame {} (p4: {}, p3: {}, p2: {}, p1: {})",
            page.0, frame.0, page.p4_index(), page.p3_index(), page.p2_index(), page.p1_index());
        }

        ret_frame
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
        let frame = allocator.allocate_frame().expect("No free frames");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
        let page = Page::for_address(frame.start_address());
        self.map_to(page, frame, flags, allocator);
    }

    fn unmap<A>(&mut self, page: Page, allocator: &mut A) where A: FrameAllocator {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("Mapping does not support huge pages");
        
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        unsafe {
            ::x86::shared::tlb::flush(page.start_address());
        }
        //allocator.deallocate_frame(frame);
    }
}


pub fn test_paging<A>(allocator: &mut A) where A: FrameAllocator {
    let mut page_table = unsafe { ActivePageTable::new() };

    test_map(&mut page_table, allocator);
}

fn test_translate(page_table: &ActivePageTable) {
    // address 0 is mapped
    println!("Virtual addr 0 -> physical addr {:?}", page_table.translate(0));
    // second P1 entry
    println!("Virtual addr 4096 (2nd P1 entry) -> physical addr {:?}", page_table.translate(4096));
    // second P2 entry
    println!("Virtual addr 512 * 4096 (2nd P2 entry) -> physical addr {:?}", page_table.translate(512 * 4096));
    // 300th P2 entry
    println!("Virtual addr 300 * 512 * 4096 (300th P2 entry) -> physical addr {:?}", page_table.translate(300 * 512 * 4096));
    // second P3 entry
    println!("Virtual addr 512 * 512 * 4096 (2nd P3 entry) -> physical addr {:?}", page_table.translate(512 * 512 * 4096));
    // last mapped byte
    println!("Virtual addr 512 * 512 * 4096 - 1 (last mapped byte) -> physical addr {:?}", page_table.translate(512 * 512 * 4096 - 1));
}

fn test_map<A: FrameAllocator>(page_table: &mut ActivePageTable, allocator: &mut A) {
    let addr = 42 * 512 * 512 * 4096; // 42nd P3 entry
    let page = Page::for_address(addr);
    let frame = allocator.allocate_frame().expect("No free frames");
    println!("Testing page mapping...");
    println!("Current mapping: 0x{:x} -> {:?}", addr, page_table.translate(addr));

    page_table.map_to(page, frame, EntryFlags::empty(), allocator);

    println!("Current mapping: 0x{:x} -> {:?}", addr, page_table.translate(addr));

    println!("next free frame: {:?}", allocator.allocate_frame());
    access_page(&Page::for_address(addr), page_table);

    println!("Testing page unmapping...");

    println!("Unmapping: page {}", Page::for_address(addr).0);
    page_table.unmap(Page::for_address(addr), allocator);

    println!("Current mapping: 0x{:x} -> {:?}", addr, page_table.translate(addr));
    //access_page(&Page::for_address(addr), page_table);
}

fn access_page(page: &Page, page_table: &ActivePageTable) {
    let Page(ref page_num) = *page;
    println!("access page {}: {:#x}", page_num, unsafe { *(page.start_address() as *const u64) });
}
