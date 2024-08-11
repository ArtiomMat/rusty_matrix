use nix::{libc, unistd};

use std::vec::Vec;
use std::time::Duration;
use std::io::{stdout, stdin, Write};
use std::thread::{sleep};
use std::cmp::{min, max};

use rand::seq::SliceRandom;
use rand;
use rand::Rng;

enum Color {
    Red,
    DarkRed,
    Green,
    DarkGreen,
    /*Blue,
    DarkBlue,*/
}

const TAIL_LEN : u16 = 5;

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

/// Uses unsafe stuff in it, but it's alright, returns [width,height] slash [cols,rows]
fn get_size() -> [u16; 2] {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ, 
            &mut ws as *mut libc::winsize
        );
    }
    [ws.ws_col, ws.ws_row]
}

/// Prints a chracter with a color
fn put(chr: char, color: Color) {
    match color {
        Color::Red => print!("\x1b[91m"),
        Color::DarkRed => print!("\x1b[31m"),
        Color::Green => print!("\x1b[92m"),
        Color::DarkGreen => print!("\x1b[32m"),
        _ => {}
    }
    print!("{}", chr);
    stdout().flush().expect("Flush");
}

/// Both clear and reset cursor position
fn clear() {
    print!("\x1b[2J");
    stdout().flush().expect("Flush");
}

fn glyph() -> char {
    let mut rng = rand::thread_rng();
    GLYPHS[rng.gen_range(0..GLYPHS.len())]
}

/// Set cursor, relative to top-left is [0,0]
fn set_cur(x: u16, y: u16) {
   print!("\x1b[{};{}H", y+1, x+1);
   stdout().flush().expect("Flush");
}

fn main() {
    let mut size = get_size();
    
    // Stores all the positions and stuff.
    let mut rain: Vec<[u16; 2]> = Vec::new();
    rain.push([2,3]);

    loop {
        // If the size changes we reset the whole ordeal
        let new_size = get_size();
        if new_size != size {
            rain.clear();
            size = new_size; // So nice of them to implement Copy,Clone traits :)
        }
        
        clear();
       
        let mut do_pop = false;

        // Render the drops and their tails & drop them
        for drop in &mut rain {
            if drop[1] as i32 - TAIL_LEN as i32 >= size[1] as i32 {
                // We can pop() because this drop is GUARANTEED to be the lowest drop if it goes
                // out of frame, that is because we generate them so that they are never on the
                // same height!!!!
                // XXX: Maybe just reset y????
                do_pop = true;
                continue;
            }
            
            if drop[1] > 0 {
                let y_top = max(0, (drop[1] as i32) - TAIL_LEN as i32) as u16;
                for y in y_top..=drop[1] {
                    set_cur(drop[0], y);
                    // Head of tail is light green
                    if y == drop[1] { 
                        put(glyph(), Color::Green);
                    } else {
                        put(glyph(), Color::DarkGreen);
                    }
                }
            }

            drop[1] += 1;
        }

        if do_pop {
            return;
            rain.pop();
        }

        // set_cur(1,1);
        // put(GLYPHS[0], Color::Red);
        sleep(Duration::new(0, 1_000_000 * 50));
    }
}

