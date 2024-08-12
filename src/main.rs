mod bless;

use std::time::*;
use std::thread::sleep;

const FRAME_TIME: Duration = Duration::new(0, 1_000_000 * 250);

const TAIL_LEN : u16 = 5;

/// Both clear and reset cursor position
fn clear() {
    print!("\x1b[2J");
}


fn main() {
    let mut ctx = bless::Context::new();

    loop {
        let now = Instant::now();

        ctx.print();

        if let Some(sleep_time) = FRAME_TIME.checked_sub(now.elapsed()) {
            sleep(sleep_time);
        }
    }
}

