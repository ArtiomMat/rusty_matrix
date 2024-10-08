use std::time::*;
use std::thread::sleep;
use std::mem;
use std::env;

use libc;

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
    jap_glyph: bool,
}

impl Context {
    pub fn new() -> Context {
        println!("\x1b[?25l"); // Hide the cursor

        let mut ctx = Context {
            str: String::new(),
            buf: Vec::new(),
            size: [0, 0],
            red_fg: false,
            jap_glyph: false,
        };
        ctx.renew();
        ctx
    }

    pub fn red_fg(&mut self) {
        self.red_fg = true;
    }

    pub fn jap_glyph(&mut self) {
        self.jap_glyph = true;
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
                self.glyph(),
                match self.buf[i] {
                    SYM_FALLOFF..180 => if self.red_fg { Color::DarkRed } else { Color::DarkGreen },
                    180..252 => if self.red_fg { Color::Red } else { Color::Green },
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
            if self.buf[i] == 0 && (0..1).contains(&rand::thread_rng().gen_range(0..40)) {
                self.buf[i] = 255;
            }
            // Right from the previous frame already here
            else if self.buf[i] == 255 {
                self.buf[i] = 255 - SYM_FALLOFF;
            }
            // Is a part of a tail of the head
            else if (0..9).contains(&rand::thread_rng().gen_range(0..10)) && self.buf[i] >= SYM_FALLOFF {
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
        self.write_char(c);
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
    fn write_char(&mut self, c: char) {
        self.str.push(c);
    }
    
    /// Gives a random glyph
    fn glyph(&self) -> char {
        let mut rng = rand::thread_rng();

        if self.jap_glyph {
            return JAP_GLYPHS[rng.gen_range(0..JAP_GLYPHS.len())];
        }
        
        match rng.gen_range(0..=2) {
            0 => rng.gen_range('a'..'z'),
            1 => rng.gen_range('A'..'Z'),
            _ => GLYPH3[rng.gen_range(0..GLYPH3.len())] as char // Never gonna get here, just saying
        }
    }
}

/// Fall-off a symbol experiences each frame.
const SYM_FALLOFF: u8 = 5;

const FRAME_TIME: Duration = Duration::new(0, 1_000_000 * 60);

const JAP_GLYPHS: [char; 57] = [
    '｡', 'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 
    'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 
    'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
];

/// Third option of glyphs
const GLYPH3: [u8; 26] = [b'?', b'!', b'/', b'@', b'#', b'^', b'%', b'&', b'*', b';', b'<', b'>', b'{', b'[', b'}', b']', b'-', b'(', b')', b'~', b'|', b'_', b'\\', b'$', b'+', b'='];


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

fn find_arg(args: &Vec<String>, arg: &str) -> Option<usize> {
    for i in 1..args.len() {
        if args[i] == arg {
            return Some(i);
        }
    }
    return None;
}

extern "C" fn handle_sigint(_: i32) {
    unsafe {
        println!("\x1b[?25h"); // Show cursor again
        libc::exit(0);
    }
}

fn main() {
    // SIGINT handler
    unsafe {
        let sa = libc::sigaction {
            sa_sigaction: handle_sigint as libc::sighandler_t,
            sa_mask: mem::zeroed(),
            sa_flags: 0,
            sa_restorer: None,
        };
        
        _ = libc::sigaction(libc::SIGINT, &sa, std::ptr::null_mut());
    }

    let args: Vec<String> = env::args().collect();

    if find_arg(&args, "-h").is_some() || find_arg(&args, "--help").is_some() {
        println!("Rusty Matrix by Artiom.");
        println!("-h,--help: Help.");
        println!("-r,--red: Red version because I have a red theme.");
        println!("-j,--japanese: Write half-width katana characters.");
        return;
    }

    let mut ctx = Context::new();

    if find_arg(&args, "-r").is_some() || find_arg(&args, "--red").is_some() {
        ctx.red_fg();
    }

    if find_arg(&args, "-j").is_some() || find_arg(&args, "--japanese").is_some() {
        ctx.jap_glyph();
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

