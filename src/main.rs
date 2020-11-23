#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm)]

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

const SP_USER_SYSTEM_START: usize = 0x2FFFFF; // end of SRAM
const SP_FIQ_START: usize = 0x2DFFFF; // 190KB + 1,99.. KB for each stack
const SP_IRQ_START: usize = 0x2BFFFF;
const SP_SVC_START: usize = 0x29FFFF;
const SP_ABT_START: usize = 0x27FFFF;
const SP_UND_START: usize = 0x25FFFF;

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
enum ProcessorMode {
    User = 0b10000,
    FIQ = 0b10001,
    IRQ = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

fn get_mode() -> ProcessorMode {
    let mut cpsr: u32 = 0;

    unsafe {
        asm!("MRS {0}, CPSR", out(reg) cpsr);
    }

    ProcessorMode::from_u8((cpsr & 0x1F) as u8).unwrap()
}

fn switch_mode(new_mode: ProcessorMode) {
    // info!("Switching to processor mode {:?}", new_mode);

    // if get_mode() == ProcessorMode::User {
    //     error!("switch_mode: You can't escape from user mode");
    // }

    unsafe {
        asm!(
            "
        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, r1
        MSR cpsr_c, r0
        ",
               in("r1") new_mode as u8
        );
    }
}

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("ldr sp, ={}",  const SP_SVC_START);
    }
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
        // switch_mode(ProcessorMode::FIQ);
        // asm!("ldr sp, ={}",  const SP_FIQ_START);
        switch_mode(ProcessorMode::IRQ);
        asm!("ldr sp, ={}",  const SP_IRQ_START);
        switch_mode(ProcessorMode::Abort);
        asm!("ldr sp, ={}",  const SP_ABT_START);
        switch_mode(ProcessorMode::Undefined);
        asm!("ldr sp, ={}",  const SP_UND_START);
        switch_mode(ProcessorMode::System);
        //asm!("ldr sp, ={}",  const SP_USER_SYSTEM_START);
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

// The reset handler
#[no_mangle]
unsafe extern "C" fn ResetHandler() -> ! {
    println!("swi");
    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn UndefinedInstruction() -> ! {
    println!("swi");
    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn SoftwareInterrupt() -> ! {
    asm!("nop");
    asm!("nop");

    println!("swi");

    loop {}
}

// The reset handler
#[no_mangle]
unsafe extern "C" fn PrefetchAbort() -> ! {
    println!("swi");

    loop {}
}
