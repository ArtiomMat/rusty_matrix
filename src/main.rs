mod print;

use std::time::*;
use std::thread::sleep;

const FRAME_TIME: Duration = Duration::new(0, 1_000_000 * 60);

fn main() {
    let mut ctx = print::Context::new();

    loop {
        let now = Instant::now();

        ctx.print();

        if let Some(sleep_time) = FRAME_TIME.checked_sub(now.elapsed()) {
            sleep(sleep_time);
        }
    }
}

