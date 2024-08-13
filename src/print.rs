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
    str: String,
    /// Buffer storing all the syms, a flattened array
    /// Once was double buffering but found a way to keep it a single buffer, 200 IQ idea.
    buf: Vec<Sym>,
    /// Last recorded terminal size, used to reallocate buf as necessary
    size: [u16; 2],
}

// TODO: Ctrl+C doesn't drop it though. Fuck.
/// When context is dropped we wanna reshow the cursor.
impl Drop for Context {
    fn drop(&mut self) {
        self.write_str("\x1b[?25h");
    }
}

impl Context {
    pub fn new() -> Context {
        println!("\x1b[?25l"); // Hide the cursor

        let mut ctx = Context {
            str: String::new(),
            buf: Vec::new(),
            size: [0, 0],
        };
        ctx.renew();
        ctx
    }

    pub fn print(&mut self) {
        self.renew();

        let total_s = self.size[0] * self.size[1];

        self.write_str("\x1b[H");

        // Render our nice little glyphs out of the symbols
        for i in 0..self.buf.len() {
            if self.buf[i] < SYM_FALLOFF {
                self.write_str(" ");
                continue;
            }

            self.write_glyph(
                glyph(),
                match self.buf[i] {
                    SYM_FALLOFF..130 => Color::DarkGreen,
                    130..250 => Color::Green,
                    _ => Color::White,
                },
            );
        }

        self.flush();
        
        // First shift the mf.
        for i in (self.size[0]..total_s).rev() {
            self.buf[i as usize] = self.buf[(i - self.size[0]) as usize];
        }

        // Determine the new & conjured syms coming from the top.
        // Since we already shifted the 2nd row and above, we can just override the top row with
        // the top row!
        for i in 0..(self.size[0] as usize) {
            // Nothing there, so 50/50 we put a new one
            if self.buf[i] == 0 && (0..1).contains(&rand::thread_rng().gen_range(0..18)) {
                self.buf[i] = 255;
            }
            // Right from the previous frame already here
            else if self.buf[i] == 255 {
                self.buf[i] = 255 - SYM_FALLOFF;
            }
            // Is a part of a tail of the head
            else if (0..3).contains(&rand::thread_rng().gen_range(0..5)) && self.buf[i] >= SYM_FALLOFF {
                self.buf[i] = self.buf[i] - SYM_FALLOFF;
            } else {
                self.buf[i] = 0;
            }
        }
    }

    /// If the size of the terminal changes then it reallocates the whole thing
    fn renew(&mut self) {
        let new_size = get_size();
        if new_size != self.size {
            let total = new_size[0] as usize * new_size[1] as usize;
            self.size = new_size;
            self.buf.clear();
            self.str.reserve(total);
            self.buf.resize(total, 0);
        }
    }

    /// Writes a glyph into STDOUT
    fn write_glyph(&mut self, c: char, fg: Color) {
        match fg {
            Color::White => self.write_str("\x1b[97m"),

            Color::Red => self.write_str("\x1b[91m"),
            Color::DarkRed => self.write_str("\x1b[31m"),

            Color::Green => self.write_str("\x1b[92m"),
            Color::DarkGreen => self.write_str("\x1b[32m"),

            _ => {}
        }
        self.write_ascii(c as u8);
    }

    /// Flushes STDOUT_FILENO
    fn flush(&mut self) {
        unsafe {
            libc::write(
                libc::STDOUT_FILENO,
                self.str.as_ptr() as *const libc::c_void,
                self.str.len(),
            );
            libc::fsync(libc::STDOUT_FILENO);
        }
        // Keep the capacity the same though
        self.str.truncate(0);
    }
    
    fn write_str(&mut self, s: &str) {
        /*unsafe {
            libc::write(
                libc::STDOUT_FILENO,
                s.as_ptr() as *const libc::c_void,
                s.len(),
            );
        }*/
        self.str.push_str(s);
    }

    /// Writes an ASCII, so ofc u8!
    fn write_ascii(&mut self, c: u8) {
        /*unsafe {
            libc::write(
                libc::STDOUT_FILENO,
                std::mem::transmute(&c),
                1,
            );
        }*/
        self.str.push(c as char);
    }
}

/// Fall-off a symbol experiences each frame.
const SYM_FALLOFF: u8 = 20;

/// Third option of glyphs
const GLYPH3: [u8; 26] = [b'?', b'!', b'/', b'@', b'#', b'^', b'%', b'&', b'*', b';', b'<', b'>', b'{', b'[', b'}', b']', b'-', b'(', b')', b'~', b'|', b'_', b'\\', b'$', b'+', b'='];

fn glyph() -> char {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..=2) {
        0 => rng.gen_range('a'..'z'),
        1 => rng.gen_range('A'..'Z'),
        // TODO: 2, for symbols, but IDK!
        _ => GLYPH3[rng.gen_range(0..GLYPH3.len())] as char // Never gonna get here, just saying
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



// /// Prints out the unicode character
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


