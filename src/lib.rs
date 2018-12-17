#![feature(lang_items, const_fn, ptr_internals, core_intrinsics)]
#![no_std]

#[macro_use]
extern crate bitflags;

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86;

#[macro_use]
mod vga_buffer;
mod memory;

use core::intrinsics;
use core::panic::PanicInfo;

use memory::FrameAllocator;

#[no_mangle]
pub extern fn rust_main(mb_info_addr: usize) {
    vga_buffer::clear_screen();
    println!("Booted{}", "!");

    let boot_info = unsafe {
        multiboot2::load(mb_info_addr)
    };

    //print_memory_areas(boot_info);
    //print_elf_sections(boot_info);

    let mut frame_allocator = get_frame_allocator(mb_info_addr, boot_info);

    //alloc_all_mem(frame_allocator);
    memory::test_paging(&mut frame_allocator);
}

fn get_frame_allocator(mb_info_addr: usize, boot_info: &multiboot2::BootInformation) -> memory::AreaFrameAllocator {
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let kernel_range = get_kernel_range(boot_info);
    let mb_range = get_mb_range(mb_info_addr, boot_info);
    memory::AreaFrameAllocator::new(memory_map_tag.memory_areas(), kernel_range, mb_range)
}

fn alloc_all_mem(frame_allocator: &mut FrameAllocator) {
    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }
}

fn print_memory_areas(boot_info: &multiboot2::BootInformation) {
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
    }
}

fn print_elf_sections(boot_info: &multiboot2::BootInformation) {
    let elf_sections_tag = boot_info.elf_sections_tag().expect("ELF sections tag required");

    println!("kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", section.addr, section.size, section.flags);
    }
}

fn get_kernel_range(boot_info: &multiboot2::BootInformation) -> (usize, usize) {
    let elf_sections_tag = boot_info.elf_sections_tag().expect("ELF sections tag required");

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end   = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}", kernel_start, kernel_end);
    (kernel_start as usize, kernel_end as usize)
}

fn get_mb_range(mb_start: usize, boot_info: &multiboot2::BootInformation) -> (usize, usize) {
    let mb_end = mb_start + (boot_info.total_size as usize);

    println!("mb_start: 0x{:x}, mb_end: 0x{:x}", mb_start, mb_end);
    (mb_start, mb_end)
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        println!("\n\nPANIC in {} at line {}:", location.file(), location.line());
    }

    unsafe { intrinsics::abort() }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
