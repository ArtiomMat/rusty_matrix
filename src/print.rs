//! The module that is responsible for printing

use nix::{libc, unistd};
use rand::Rng;
use std::io::{stdin, stdout, Write};

#[derive(Clone, Copy)]
pub enum Color {
    Default, // Default color given by the terminal
    White,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Blue,
    DarkBlue,
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

/// A Sym(symbol) is represented by a single scalar, its intensity. The reasoning for this is the
/// ability to render the whole matrix effect 100% procedurally, using the previous frame and no
/// other data.
/// 0 signifies nothing in that cell.
pub type Sym = u8;

pub struct Context {
    /// Buffers storing all the syms, a flattened array
    /// One is the background, second is the drawn one, the reason is because we need to
    /// read and then directly write the data
    /// buffer and it's really hard and inneficient to modify a single buffer directly.
    bufs: (Vec<Sym>, Vec<Sym>),
    /// Index of background buffer.
    bg_buf_i: usize,
    /// Last recorded terminal size, used to reallocate buf as necessary
    size: [u16; 2],
}

// TODO: Ctrl+C doesn't drop it though. Fuck.
/// When context is dropped we wanna reshow the cursor.
impl Drop for Context {
    fn drop(&mut self) {
        write_str("\x1b[?25h");
    }
}

impl Context {
    pub fn new() -> Context {
        write_str("\x1b[?25l"); // Hide the cursor
        
        flush();
        let mut ctx = Context {
            bufs: (Vec::new(), Vec::new()),
            bg_buf_i: 0,
            size: [0, 0],
        };
        ctx.renew();
        ctx
    }

    pub fn print(&mut self) {
        self.renew();

        write_str("\x1b[H");

        // Render our nice little glyphs out of the symbols
        for s in self.fg_buf() {
            if *s < SYM_FALLOFF {
                write_str(" ");
                continue;
            }

            write_glyph(
                glyph(),
                match *s {
                    SYM_FALLOFF..100 => Color::DarkGreen,
                    100..200 => Color::Green,
                    _ => Color::White,
                },
            );
        }

        flush();
        
        // GODDAMN BORROW CHECKER
        let size_0 = self.size[0] as usize;
        
        let (bg_buf,fg_buf) = if self.bg_buf_i == 0 {
            (&mut self.bufs.0, &self.bufs.1)
        } else {
            (&mut self.bufs.1, &self.bufs.0)
        };

        // STEP 1! determine the new & conjured syms coming from the top.
        // i is used to index the top row of the drawn buffer, to determine the top row on the bg
        // buffer.
        for i in 0..size_0 {
            // Nothing there, so 50/50 we put a new one
            if fg_buf[i] == 0 && rand::random() {
                bg_buf[i] = 255;
            }
            // Right from the previous frame already here
            else if fg_buf[i] == 255 {
            }
            // Is a part of a tail of the head
            else {
                
            }
        }
    }


    /// If the size of the terminal changes then it reallocates the whole thing
    fn renew(&mut self) {
        let new_size = get_size();
        if new_size != self.size {
            self.size = new_size;
            self.bufs.0.clear();
            self.bufs.1.clear();
            self.bufs.0.resize((self.size[0] * self.size[1]) as usize, 0);
            self.bufs.1.resize((self.size[0] * self.size[1]) as usize, 0);
        }
    }
}

/// Fall-off a symbol experiences each frame.
const SYM_FALLOFF: u8 = 5;

fn glyph() -> char {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..=2) {
        0 => rng.gen_range('a'..'z'),
        1 => rng.gen_range('A'..'Z'),
        // TODO: 2, for symbols, but IDK!
        _ => '?' // Never gonna get here, just saying
    }
}

/// Returns [width,height] slash [cols,rows].
pub fn get_size() -> [u16; 2] {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut ws as *mut libc::winsize,
        );
    }
    [ws.ws_col, ws.ws_row]
}

fn write_str(s: &str) {
    unsafe {
        libc::write(
            libc::STDOUT_FILENO,
            s.as_ptr() as *const libc::c_void,
            s.len(),
        );
    }
}

/// Writes an ASCII, so ofc u8!
fn write_ascii(c: u8) {
    unsafe {
        libc::write(
            libc::STDOUT_FILENO,
            std::mem::transmute(&c),
            1,
        );
    }
}

/// Prints out the unicode character
/*
fn write_char(c: char) {
    let mut utf8_buf = [0u8; 4];
    let utf8_b = c.encode_utf8(&mut utf8_buf);
    unsafe {
        libc::write(
            libc::STDOUT_FILENO,
            utf8_b.as_ptr() as *const libc::c_void,
            utf8_b.len(),
        );
    }
}
*/

/// Writes a glyph into STDOUT
fn write_glyph(c: char, fg: Color) {
    match fg {
        Color::White => write_str("\x1b[97m"),

        Color::Red => write_str("\x1b[91m"),
        Color::DarkRed => write_str("\x1b[31m"),

        Color::Green => write_str("\x1b[92m"),
        Color::DarkGreen => write_str("\x1b[32m"),

        _ => {}
    }
    write_ascii(c as u8);
}

/// Flushes STDOUT_FILENO
fn flush() {
    unsafe {
        libc::fsync(libc::STDOUT_FILENO);
    }
}

