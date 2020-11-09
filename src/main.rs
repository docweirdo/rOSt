#![no_std]
#![no_main]

use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;

mod dbgu;
mod fmt;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // printf statement
    println!(
        "{} {:#X} {} PIO_A: {:p}",
        "Hello", 0x8BADF00D as u32, 'c', 0xFFFFF400 as *mut u32
    );

    println!("waiting for input... (press ENTER to echo)");

    loop {
        if eval_check() {
            break;
        }
    }

    println!("the end");
    panic!();
}

const KEY_ENTER: u32 = 0xD;
const KEY_BACKSPACE: u32 = 0x8;
const KEY_DELETE: u32 = 0x7F;

pub fn eval_check() -> bool {
    let mut char_buf = ArrayString::<[u8; 48]>::new();
    loop {
        if dbgu::is_char_available() {
            let last_char = dbgu::read_char();
            if last_char == KEY_ENTER {
                break;
            }
            if last_char == KEY_DELETE || last_char == KEY_BACKSPACE {
                char_buf.pop();
            } else {
                if char_buf.is_full() {
                    println!("Over capacity! echo buffered string");
                    break;
                }
                char_buf.push(core::char::from_u32(last_char).expect("fail to convert"));
            }
        }
    }

    if char_buf.eq("quit") {
        return true;
    }

    println!("Received: {}", char_buf);

    false
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Kernel Panic!!! Jump ship!");

    loop {}
}
