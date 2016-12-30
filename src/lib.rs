#![feature(lang_items, const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;

#[macro_use]
mod vga_buffer;

#[no_mangle]
pub extern fn rust_main(mb_info_addr: usize) {
    use core::fmt::Write;

    vga_buffer::clear_screen();
    println!("Booted{}", "!");

    let boot_info = unsafe {
        multiboot2::load(mb_info_addr)
    };
    print_memory_areas(boot_info);
    print_elf_sections(boot_info);

    let mb_start = mb_info_addr;
    let mb_end = mb_start + (boot_info.total_size as usize);

    println!("mb_start: 0x{:x}, mb_end: 0x{:x}", mb_start, mb_end);
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

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end   = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}", kernel_start, kernel_end);
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
