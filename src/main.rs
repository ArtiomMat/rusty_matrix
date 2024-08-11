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
    let mut rng = rand::thread_rng();

    let mut size: [u16; 2] = [0,0];
    
    // Stores all the positions and stuff.
    // There are 3 values, x,y,t, t being the size of the tail.
    let mut rain: Vec<[u16; 3]> = Vec::new();
    

    loop {
        // If the size changes we reset the whole ordeal
        let new_size = get_size();
        if new_size != size {
            rain.clear();
            size = new_size; // So nice of them to implement Copy,Clone traits :)
            for _ in 0..200 {
                rain.push([rng.gen_range(0..size[0]), rng.gen_range(0..size[1]), rng.gen_range(4..16)]);
            }
        }
        
        clear();
       
        // Render the drops and their tails & drop them
        for drop in &mut rain {
            if drop[1] as i32 - drop[2] as i32 >= size[1] as i32 {
                // Place it back up
                drop[1] = 0;
                drop[0] = rng.gen_range(0..size[0]);
                continue;
            }
            
            if drop[1] > 0 {
                let y_top = max(0, (drop[1] as i32) - drop[2] as i32) as u16;
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

        // set_cur(1,1);
        // put(GLYPHS[0], Color::Red);
        sleep(Duration::new(0, 1_000_000 * 150));
    }
}

