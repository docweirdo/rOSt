#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm)]
#![allow(unused_imports)]

#[macro_use]
extern crate num_derive;

use num_traits::{FromPrimitive, ToPrimitive};

use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use log::{error, info};

mod dbgu;
mod fmt;
mod logger;

// https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
// https://github.com/Amanieu/rfcs/blob/inline-asm/text/0000-inline-asm.md


const SRAM_END : usize = 0x0020_4000;
const STACK_SIZE : usize =  (1024*2);

const SP_USER_SYSTEM_START: usize = SRAM_END - 0 * STACK_SIZE; // end of SRAM
const SP_FIQ_START: usize = SRAM_END - 1 * STACK_SIZE;
const SP_IRQ_START: usize = SRAM_END - 2 * STACK_SIZE;
const SP_SVC_START: usize = SRAM_END - 3 * STACK_SIZE;
const SP_ABT_START: usize = SRAM_END - 4 * STACK_SIZE;
const SP_UND_START: usize = SRAM_END - 5 * STACK_SIZE;

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

#[no_mangle]
#[naked]
fn switch_mode(new_mode: ProcessorMode) {
    // info!("Switching to processor mode {:?}", new_mode);

    // if get_mode() == ProcessorMode::User {
    //     error!("switch_mode: You can't escape from user mode");
    // }

    unsafe {
        asm!(
            "
        MOV r2, lr
        MRS r1, cpsr
        BIC r1, r1, #0x1F
        ORR r1, r1, r0
        MSR cpsr_c, r1
        MOV lr, r2
        "  // we save lr because lr gets corrupted during mode switch
        );
    }
}

const MC: *mut u32 = 0xFFFFFF00 as *mut u32;
const MC_RCR: isize = 0x0;

fn toggle_memory_remap() {
    unsafe{
        write_volatile(MC.offset(MC_RCR / 4), 1 as u32)
    }   
}

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("ldr sp, ={}",  const SP_SVC_START);
    }
    toggle_memory_remap();
    boot();
    loop {}
}

use log::LevelFilter;

static LOGGER: logger::SimpleLogger = logger::SimpleLogger;

pub fn init_logger() {
    unsafe {
        log::set_logger_racy(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Info))
            .unwrap()
    };
}

pub fn boot() {
    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    //init_logger();

    unsafe {
        switch_mode(ProcessorMode::FIQ);
        asm!("ldr sp, ={}",  const SP_FIQ_START);
        switch_mode(ProcessorMode::IRQ);
        asm!("ldr sp, ={}",  const SP_IRQ_START);
        switch_mode(ProcessorMode::Abort);
        asm!("ldr sp, ={}",  const SP_ABT_START);
        switch_mode(ProcessorMode::Undefined);
        asm!("ldr sp, ={}",  const SP_UND_START);
        switch_mode(ProcessorMode::System);
        asm!("ldr sp, ={}",  const SP_USER_SYSTEM_START);
        switch_mode(ProcessorMode::Supervisor);
    }

    println!("mode {:?}", get_mode());
    // switch_mode(ProcessorMode::System);
    // info!("mode {:?}", get_mode());
    // switch_mode(ProcessorMode::User);
    // info!("mode {:?}", get_mode());
    // switch_mode(ProcessorMode::System);
    // info!("mode {:?}", get_mode());

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


#[no_mangle]
unsafe extern "C" fn ResetHandler() -> ! {
    println!("swi");
    loop {}
}


#[no_mangle]
unsafe extern "C" fn UndefinedInstruction() -> ! {
    println!("undefined instruction");
    loop {}
}


#[no_mangle]
unsafe extern "C" fn SoftwareInterrupt() -> ! {
    println!("software interrupt");

    loop {}
}

#[no_mangle]
unsafe extern "C" fn PrefetchAbort() -> ! {
    println!("prefetch abort");

    loop {}
}


#[no_mangle]
unsafe extern "C" fn DataAbort() -> ! {
    println!("data abort");

    loop {}
}


#[no_mangle]
unsafe extern "C" fn HardwareInterrupt() -> ! {
    println!("hardware interrupt");

    loop {}
}

#[no_mangle]
unsafe extern "C" fn FastInterrupt() -> ! {
    println!("fast interrupt");

    loop {}
}