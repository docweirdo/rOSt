#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm)]
#![allow(unused_imports)]

#[macro_use]
extern crate num_derive;
extern crate alloc;

use num_traits::{FromPrimitive, ToPrimitive};

use core::panic::PanicInfo;
use core::ptr::write_volatile;
use log::{debug, error, info};

mod dbgu;
mod exceptions;
mod fmt;
mod logger;
mod memory;

// https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
// https://github.com/Amanieu/rfcs/blob/inline-asm/text/0000-inline-asm.md

// init or memory init module? Memory module for stack and remapping? CPU Module for modeswitch and so on?

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
enum ProcessorMode {
    User = 0x10,
    FIQ = 0x11,
    IRQ = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1b,
    System = 0x1F,
}

fn get_mode() -> ProcessorMode {
    let mut cpsr: u32;

    unsafe {
        asm!("MRS {0}, CPSR", out(reg) cpsr);
    }

    ProcessorMode::from_u8((cpsr & 0x1F) as u8).unwrap()
}

#[naked]
#[allow(unused_variables)]
#[inline(always)]
fn switch_processor_mode_naked(new_mode: ProcessorMode) {
    unsafe {
        asm!(
            "
        MOV r2, lr
        MRS r1, cpsr
        BIC r1, r1, #0x1F
        ORR r1, r1, r0
        MSR cpsr_c, r1
        MOV lr, r2
        " // we save lr because lr gets corrupted during mode switch
        );
    }
}

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    memory::init_processor_mode_stacks();

    boot();
    loop {}
}

use log::LevelFilter;

static LOGGER: logger::SimpleLogger = logger::SimpleLogger;

pub fn init_logger() {
    unsafe {
        log::set_logger_racy(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap()
    };
}

pub fn boot() {
    memory::toggle_memory_remap(); // blend sram to 0x0 for IVT
    init_logger();
    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    debug!("processor mode {:?}", get_mode());

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
                ldr r0, =0xFFFFFFFF
                str r0, [r0]"
            );
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
    println!("panic handler");
    loop {}
}
