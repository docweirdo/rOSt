#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm)]

use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;

mod dbgu;
mod fmt;

#[no_mangle]
pub extern "C" fn _irq_handler() -> ! {
    println!("interrupt handler");
    loop {}
}

const SP_START: usize = 0x2FFFFF; // end of SDRAM

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("ldr sp, ={}",  const SP_START);
    }
    boot();
    loop {}
}

pub fn boot() {
    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // // printf statement
    // println!(
    //     "{} {:#X} {} PIO_A: {:p}",
    //     "Hello", 0x8BADF00D as u32, 'c', 0xFFFFF400 as *mut u32
    // );

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

    println!("Received: {}", char_buf);

    match char_buf.as_str() {
        "swi" => unsafe {
            asm!("swi #0");
        },
        "undi" => unsafe {
            asm!(".word 0xf7f0a000");
        },
        "dabort" => unsafe {
            write_volatile(0xFFFFFFFF as *mut u32, 0x1);
        },
        "quit" => {
            return true;
        }
        _ => {
            println!("  Unknown command");
        }
    }

    false
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Kernel Panic!!! Jump ship!");

    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn ResetHandler() -> ! {
    let _x = 42;

    // can't return so we go into an infinite loop here
    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn UndefinedInstruction() -> ! {
    let _x = 43;

    // can't return so we go into an infinite loop here
    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn SoftwareInterrupt() -> ! {
    let _x = 43;

    // can't return so we go into an infinite loop here
    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn PrefetchAbort() -> ! {
    let _x = 44;

    // can't return so we go into an infinite loop here
    loop {}
}
