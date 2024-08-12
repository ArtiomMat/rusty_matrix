use nix::{libc, unistd};
use std::io::{stdin, stdout, Write};
use rand::Rng;

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
    /// Buffer storing all the syms, a flattened array
    buf: Vec<Sym>,
    /// Last recorded terminal size, used to reallocate buf as necessary
    size: [u16; 2],
}

impl Context {
    pub fn new() -> Context {
        let mut ctx = Context {
            buf: Vec::new(),
            size: [0,0],
        };
        ctx.renew();
        ctx
    }
   
    pub fn print(&mut self) {
        self.renew();

        set_cur(0,0);
        
        // Render our nice little glyphs out of the symbols
        for s in &self.buf {
            write_glyph(glyph(), match s {
                0..100 => Color::DarkGreen,
                100..200 => Color::Green,
                _ => Color::White,
            });
        }
        
        flush();

        // Finally update the actual state
        let mut rng = rand::thread_rng();
        self.buf.insert(rng.gen_range(0..(self.size[0]*self.size[1])) as usize, 250);
    }

    /// If the size of the terminal changes then it reallocates the whole thing
    fn renew(&mut self) {
        let new_size = get_size();
        if new_size != self.size {
            // Commented because it will throw a runtime error anyway, rust is safe :)
            /*if new_size[0] < 2 || new_size[1] < 2 {
                panic!("The size of the terminal is invalid.");
            }*/
            self.size = new_size;
            self.buf.resize((self.size[0] * self.size[1]) as usize, 0);
        }
    }
}

const GLYPHS: [char; 8] = [
   'ぁ',
   'け',
   'だ',
   'め',
   'ぐ',
   'ゐ',
   'も',
   'ぶ',
];


/// Fall-off a symbol experiences each frame.
const SYM_FALLOFF : u8 = 5;

/// Set cursor, relative to top-left is [0,0]
fn set_cur(x: u16, y: u16) {
    write_str("\x1b[");
    write_str(&(y+1).to_string());
    write_str(";");
    write_str(&(y+1).to_string());
    write_str("H");
}

fn glyph() -> char {
    let mut rng = rand::thread_rng();
    GLYPHS[rng.gen_range(0..GLYPHS.len())]
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

// Prints out the unicode character
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
    write_char(c);
}

/// Flushes STDOUT_FILENO
fn flush() {
    unsafe {
        libc::fsync(libc::STDOUT_FILENO);
    }
}

