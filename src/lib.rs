#![feature(lang_items, const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;

mod vga_buffer;

#[no_mangle]
pub extern fn rust_main() {
    let hello = b"Hello Rust!!";
    let color_byte = 0x1f; // white foreground, blue background

    let mut hello_colored = [color_byte; 24];

    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }

    // write 'Hello World!' to the center of the VGA text buffer
    let buffer_ptr = (0xB8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

    print_stuff();
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] extern fn panic_fmt() -> ! {loop{}}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}

fn print_stuff() {
    use vga_buffer::{ColorCode, Color, Writer};

    let color_code = ColorCode::new(Color::LightGreen, Color::Black);
    let mut writer = Writer::new(0, color_code, 0xB8000);
    writer.write_str("rust is cool");
}
