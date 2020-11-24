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
use log::{debug, error, info};

mod dbgu;
mod fmt;
mod logger;

// use linked_list_allocator::LockedHeap;

// #[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();

// https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
// https://github.com/Amanieu/rfcs/blob/inline-asm/text/0000-inline-asm.md

const SRAM_END: usize = 0x0020_4000;
const STACK_SIZE: usize = 1024 * 2;

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

#[naked]
#[allow(unused_variables)]
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

const MC: *mut u32 = 0xFFFFFF00 as *mut u32;
const MC_RCR: isize = 0x0;

fn toggle_memory_remap() {
    unsafe { write_volatile(MC.offset(MC_RCR / 4), 1 as u32) }
}

// pub fn init_heap() {
//     let heap_start = 0x2000_0000;
//     let heap_end = 0x2400_0000;
//     let heap_size = heap_end - heap_start;
//     unsafe {
//         ALLOCATOR.lock().init(heap_start, heap_size);
//     }
// }

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    init_processor_mode_stacks();
    init_logger();
    toggle_memory_remap(); // blend sram to 0x0 for IVT

    // init_heap();
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

pub fn init_processor_mode_stacks() {
    unsafe {
        switch_processor_mode_naked(ProcessorMode::FIQ);
        asm!("ldr sp, ={}",  const SP_FIQ_START);
        switch_processor_mode_naked(ProcessorMode::IRQ);
        asm!("ldr sp, ={}",  const SP_IRQ_START);
        switch_processor_mode_naked(ProcessorMode::Abort);
        asm!("ldr sp, ={}",  const SP_ABT_START);
        switch_processor_mode_naked(ProcessorMode::Undefined);
        asm!("ldr sp, ={}",  const SP_UND_START);
        switch_processor_mode_naked(ProcessorMode::System);
        asm!("ldr sp, ={}",  const SP_USER_SYSTEM_START);
        switch_processor_mode_naked(ProcessorMode::Supervisor);
        asm!("ldr sp, ={}",  const SP_SVC_START);
    }
}

pub fn boot() {
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

#[no_mangle]
unsafe extern "C" fn ResetHandler() -> ! {
    println!("reset");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn UndefinedInstructionHandler() -> ! {
    println!("undefined instruction");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn SoftwareInterruptHandler() -> ! {
    //println!("processor mode {:?}", get_mode());

    let mut pc: usize = 24;
    //asm!("mov {}, pc", out(reg) pc);
    println!("software interrupt at {:X}", pc);
    panic!();
}

#[no_mangle]
unsafe extern "C" fn PrefetchAbortHandler() -> ! {
    println!("prefetch abort");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn DataAbortHandler() -> ! {
    println!("data abort");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn HardwareInterruptHandler() -> ! {
    println!("hardware interrupt");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn FastInterruptHandler() -> ! {
    println!("fast interrupt");
    panic!();
}

// #[alloc_error_handler]
// fn alloc_error(_layout: Layout) -> ! {
//     println!("alloc_error_handler");
//     loop {}
// }
