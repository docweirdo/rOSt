#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm)]
#![allow(unused_imports)]

#[macro_use]
extern crate num_derive;
extern crate alloc;

use alloc::vec::Vec;
use core::panic::PanicInfo;
use log::{debug, error, info};
use rand::prelude::*;
use rand_pcg::Pcg64;

mod dbgu;
mod exceptions;
mod fmt;
mod helpers;
mod interrupt_controller;
mod logger;
mod memory;
mod processor;
mod system_timer;

/// Sets stack pointers and calls boot function
#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    memory::init_processor_mode_stacks!();
    processor::switch_processor_mode!(processor::ProcessorMode::System);

    boot();
    loop {}
}

static mut DBGU_BUFFER: Vec<char> = Vec::<char>::new();
static mut PRINT_SYSTEM_TIMER_TASK3: bool = false;

pub fn boot() {
    memory::toggle_memory_remap(); // blend sram to 0x0 for IVT
    logger::init_logger();
    dbgu::set_dbgu_recv_interrupt(true);
    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    debug!("processor mode {:?}", processor::get_processor_mode());

    system_timer::init_system_timer_interrupt(12000);
    interrupt_controller::init_system_interrupt(
        || {
            //debug!("Interrupt Handler for interrupt line 1");
            //debug!("processor mode {:?}\n", processor::get_processor_mode());
            if unsafe { PRINT_SYSTEM_TIMER_TASK3 } {
                println!("!");
            }
        },
        move || unsafe {
            DBGU_BUFFER.push(
                dbgu::read_char().expect("there should be char availabe in interrupt") as u8
                    as char,
            )
        },
    );

    processor::set_interrupts_enabled!(true);

    loop {
        if eval_check() {
            break;
        }
    }

    println!("the end");
    panic!();
}

const KEY_ENTER: char = 0xD as char;
const KEY_BACKSPACE: char = 0x8 as char;
const KEY_DELETE: char = 0x7F as char;

pub fn eval_check() -> bool {
    let mut rng = Pcg64::seed_from_u64(0xDEADBEEF);
    let mut char_buf = alloc::string::String::new();
    println!("waiting for input... (press ENTER to echo)");
    println!("available commands: task3, st, swi, undi, dabort, quit");
    loop {
        if let Some(last_char) = unsafe { DBGU_BUFFER.pop() } {
            if last_char == KEY_ENTER {
                break;
            }
            if last_char == KEY_DELETE || last_char == KEY_BACKSPACE {
                char_buf.pop();
            } else {
                char_buf.push(last_char);
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
        "task3" => {
            unsafe {
                PRINT_SYSTEM_TIMER_TASK3 = true;
            }
            loop {
                if let Some(last_char) = unsafe { DBGU_BUFFER.pop() } {
                    if last_char == 'q' {
                        unsafe {
                            PRINT_SYSTEM_TIMER_TASK3 = false;
                        }
                        break;
                    }
                    fn wait(units: u32) {
                        let last = system_timer::get_current_real_time();
                        loop {
                            if system_timer::get_current_real_time() - last > units {
                                break;
                            }
                        }
                    }
                    let mut print_character_random = |c: char, min: usize, max: usize| {
                        for _ in 0..rng.gen_range(min, max) {
                            print!("{}", c);
                        }
                    };
                    print_character_random(last_char, 1, 20);
                    wait(500);
                    print_character_random(last_char, 1, 20);
                    wait(500);
                    print_character_random(last_char, 1, 20);
                }
            }
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
    //TODO: Rausfinden warum es manchmal mit stack oder ohne funktioniert bzw nicht funktioniert
    print_with_stack!("panic handler");
    print_with_stack!("{}", _info);
    loop {}
}
