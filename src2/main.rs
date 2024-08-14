use std::time::*;
use std::thread::sleep;
use std::mem;
use std::env;
use std::io::{stdin, stdout, Write};

use nix::{libc, unistd};

use rand::Rng;

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
    red_fg: bool,
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
            red_fg: false,
        };
        ctx.renew();
        ctx
    }

    pub fn red_fg(&mut self) {
        self.red_fg = true;
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
                glyph(i),
                match self.buf[i] {
                    SYM_FALLOFF..150 => if self.red_fg { Color::DarkRed } else { Color::DarkGreen },
                    150..250 => if self.red_fg { Color::Red } else { Color::Green },
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
            if self.buf[i] == 0 && (0..1).contains(&rand::thread_rng().gen_range(0..24)) {
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

    /// Flushes self.str into STDOUT_FILENO
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
    
    /// Writes a str object
    fn write_str(&mut self, s: &str) {
        self.str.push_str(s);
    }

    /// Writes an ASCII, so ofc u8!
    fn write_ascii(&mut self, c: u8) {
        self.str.push(c as char);
    }
}

/// Fall-off a symbol experiences each frame.
const SYM_FALLOFF: u8 = 40;

const FRAME_TIME: Duration = Duration::new(0, 1_000_000 * 100);

/// Third option of glyphs
/// Must be 26 to match abc
const GLYPH3: [u8; 26] = [b'?', b'!', b'/', b'@', b'#', b'^', b'%', b'&', b'*', b';', b'<', b'>', b'{', b'[', b'}', b']', b'-', b'(', b')', b'~', b'|', b'_', b'\\', b'$', b'+', b'='];

/// Gives a random glyph
fn glyph(i: usize) -> char {
    // Random number based on i and 42.
    let u = 42u32;
    let i = i as u32;
    let i = 36969*(i & 65535) + (i >> 16);
    let u = 18000*(u & 65535) + (u >> 16);
    let mut r = ((i << 16) + (u & 65535)) as usize;
    r %= 26;

    match rand::thread_rng().gen_range(0..=2) {
        0 => ('a' as usize + r) as u8 as char,
        1 => ('A' as usize + r) as u8 as char,
        2 => GLYPH3[r % GLYPH3.len()] as char,
        _ => '.'
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

pub fn rng(u: u32, v: u32) -> u32 {
    let v = 36969*(v & 65535) + (v >> 16);
    let u = 18000*(u & 65535) + (u >> 16);
    (v << 16) + (u & 65535)
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

fn find_arg(args: &Vec<String>, arg: &str) -> Option<usize> {
    for i in 1..args.len() {
        if args[i] == arg {
            return Some(i);
        }
    }
    return None;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if find_arg(&args, "-h").is_some() || find_arg(&args, "--help").is_some() {
        println!("Rusty Matrix by Artiom.");
        println!("-h,--help: Help.");
        println!("-r,--red: Red version because I have a red theme.");
        return;
    }

    let mut ctx = Context::new();

    if find_arg(&args, "-r").is_some() || find_arg(&args, "--red").is_some() {
        ctx.red_fg();
    }

    mem::drop(args);

    loop {
        let now = Instant::now();

        ctx.print();

        if let Some(sleep_time) = FRAME_TIME.checked_sub(now.elapsed()) {
            sleep(sleep_time);
        }
    }
}

