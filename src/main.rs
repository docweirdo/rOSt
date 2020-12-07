#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm)]
#![allow(unused_imports)]

#[macro_use]
extern crate num_derive;
extern crate alloc;

use core::panic::PanicInfo;
use log::{debug, error, info};

mod dbgu;
mod exceptions;
mod fmt;
mod logger;
mod memory;
mod processor;
mod helpers;
mod system_timer;
mod interrupt_controller;

/// Sets stack pointers and calls boot function
#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    memory::init_processor_mode_stacks!();

    boot();
    loop {}
}

pub fn boot() {
    memory::toggle_memory_remap(); // blend sram to 0x0 for IVT
    logger::init_logger();
    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    processor::switch_processor_mode!(processor::ProcessorMode::System);
    debug!("processor mode {:?}", processor::get_processor_mode());

    system_timer::init_system_timer_interrupt(8000);
    interrupt_controller::init_system_interrupt();
    processor::set_interrupts_enabled!(true);

    println!("waiting for input... (press ENTER to echo)");
    println!("available commands: swi, undi, dabort, quit");

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
    let mut char_buf = alloc::string::String::new();
    loop {
        if dbgu::is_char_available() {
            let last_char = dbgu::read_char();
            if last_char == KEY_ENTER {
                break;
            }
            if last_char == KEY_DELETE || last_char == KEY_BACKSPACE {
                char_buf.pop();
            } else {
                char_buf.push(core::char::from_u32(last_char).expect("fail to convert"));
            }
        }
    }

    println!("Received: {}", char_buf);
    debug!(
        "current heap size: {:#X}, left: {:#X}",
        memory::get_current_heap_size(),
        memory::get_heap_size_left()
    );

    match char_buf.as_str() {
        "swi" => unsafe {
            asm!("swi #0");
        },
        "undi" => unsafe {
            asm!(".word 0xf7f0a000");
        },
        "dabort" => unsafe {
            asm!(
                "
                 ldr r0, =0x90000000
                 str r0, [r0]"
            );

        },
        "st" => {
            println!("{}", system_timer::has_system_timer_elapsed());
        }
        "quit" => {
            return true;
        }
        _ => {
            println!("-> Unknown command");
        }
    }

    false
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("panic handler");
    loop {}
}
