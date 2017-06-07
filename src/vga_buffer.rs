/*
 *  Abstraction over the unsafe usage of the VGA text buffer memory.
 *
 *  This will allow us to print arbitrary text to the console,
 *  as well as support the standard println! and format! macros.
 */

#![allow(dead_code)]

use spin::Mutex;

const COLOR_CODE: ColorCode = ColorCode::new(Color::LightGreen, Color::Black);
const RAW_WRITER: Writer = Writer::new(0, COLOR_CODE, 0xB8000);
pub static WRITER: Mutex<Writer> = Mutex::new(RAW_WRITER);

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga_buffer::print(format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        println!("");
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black      = 0x0,
    Blue       = 0x1,
    Green      = 0x2,
    Cyan       = 0x3,
    Red        = 0x4,
    Magenta    = 0x5,
    Brown      = 0x6,
    LightGray  = 0x7,
    DarkGray   = 0x8,
    LightBlue  = 0x9,
    LightGreen = 0xA,
    LightCyan  = 0xB,
    LightRed   = 0xC,
    PINK       = 0xD,
    YELLOW     = 0xE,
    WHITE      = 0xF
}

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode
}

use volatile::Volatile;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH:  usize = 80;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

use core::ptr::Unique;

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>
}

impl Writer {
    const fn new(column_position: usize, color_code: ColorCode, buffer: usize) -> Writer {
        Writer {
            column_position: column_position,
            color_code: color_code,
            buffer: unsafe { Unique::new(buffer as *mut _) }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.write_new_line(),
            _     => self.write_character(byte)
        }
    }

    fn write_character(&mut self, chr: u8) {
        if self.column_position >= BUFFER_WIDTH {
            self.write_new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        let color_code = self.color_code;
        self.buffer().chars[row][col].write(ScreenChar {
            ascii_char: chr,
            color_code: color_code
        });
        self.column_position += 1;
    }


    fn write_new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            let buf = self.buffer();
            for col in 0..BUFFER_WIDTH {
                let chr = buf.chars[row][col].read();
                buf.chars[row - 1][col].write(chr);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe {
            self.buffer.as_mut()
        }
    }


    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code
        };

        let buf = self.buffer();
        for col in 0..BUFFER_WIDTH {
            buf.chars[row][col].write(blank)
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
