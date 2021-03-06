use core::marker::PhantomData;

use memory::paging::entry::*;
use memory::paging::ENTRY_COUNT;
use memory::FrameAllocator;

pub const P4: *mut Table<Level4> = 0xFFFFFFFF_FFFFF000 as *mut _;

pub trait TableLevel {
    const LEVEL: u8;
}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {
    const LEVEL: u8 = 4;
}
impl TableLevel for Level3 {
    const LEVEL: u8 = 3;
}
impl TableLevel for Level2 {
    const LEVEL: u8 = 2;
} 
impl TableLevel for Level1 {
    const LEVEL: u8 = 1;
}

pub trait HeirarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HeirarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HeirarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HeirarchicalLevel for Level2 {
    type NextLevel = Level1;
}

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>
}

use core::ops::{Index, IndexMut};

impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

impl<L> Table<L> where L: TableLevel {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L> Table<L> where L: HeirarchicalLevel {
    fn next_table_addr(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
            let table_addr = self as *const _ as usize;
            Some((table_addr << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A) -> &mut Table<L::NextLevel> where A: FrameAllocator {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(HUGE_PAGE), "mapping does not support huge pages");
            let frame = allocator.allocate_frame().expect("no frames available");
            println!("Creating new level {} page table in frame {}", L::LEVEL - 1, frame.0);
            self.entries[index].set(frame, PRESENT | WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }

        self.next_table_mut(index).unwrap()
    }
}
