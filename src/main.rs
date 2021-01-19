#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(drain_filter)]
#![feature(asm)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::vec::Vec;
use core::panic::PanicInfo;
use log::{debug, error, info};
use rand::prelude::*;
use rand_pcg::Pcg64;

mod allocator;
mod dbgu;
mod exceptions;
mod fmt;
mod helpers;
mod interrupt_controller;
mod logger;
mod memory;
mod processor;
mod system_timer;
mod threads;

/// Sets stack pointers and calls boot function
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() -> ! {
    asm!("
        // initialize stacks by switching to
        // processor mode and setting stack 
        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, {supervisor_mode}
        MSR cpsr_c, r0
    
        ldr sp, ={sp_svc_start}

        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, {fiq_mode}
        MSR cpsr_c, r0

        ldr sp, ={sp_fiq_start}

        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, {irq_mode}
        MSR cpsr_c, r0

        ldr sp, ={sp_irq_start}

        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, {abort_mode}
        MSR cpsr_c, r0

        ldr sp, ={sp_abort_start}

        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, {undefined_mode}
        MSR cpsr_c, r0

        ldr sp, ={sp_undefined_start}

        MRS r0, cpsr
        BIC r0, r0, #0x1F
        ORR r0, r0, {system_mode}
        MSR cpsr_c, r0

        ldr sp, ={sp_system_start}

        // jump to boot

        b {boot}
    ",  supervisor_mode = const processor::ProcessorMode::Supervisor as u32,  
        sp_svc_start = const memory::SP_SVC_START,
        fiq_mode = const processor::ProcessorMode::FIQ as u32,
        sp_fiq_start = const memory::SP_FIQ_START,
        irq_mode = const processor::ProcessorMode::IRQ as u32,
        sp_irq_start = const memory::SP_IRQ_START,
        abort_mode = const processor::ProcessorMode::Abort as u32,
        sp_abort_start = const memory::SP_ABT_START,
        undefined_mode = const processor::ProcessorMode::Undefined as u32,
        sp_undefined_start = const memory::SP_UND_START,
        system_mode = const processor::ProcessorMode::System as u32,
        sp_system_start = const memory::SP_USER_SYSTEM_START,
        boot = sym boot,
        options(noreturn));
}

pub static mut DBGU_BUFFER: Vec<char> = Vec::<char>::new();
static mut RNG: Option<Pcg64> = None;
static mut TASK3_ACTIVE: bool = false;
static mut TASK4_ACTIVE: bool = false;

/// The amount of SysTicks before the scheduler gets called.
static SCHEDULER_INTERVAL: u32 = 5;
static mut SCHEDULER_INTERVAL_COUNTER: u32 = 0;

/// Initializes the operating system.
///
/// TODO: Add detailed description
pub fn boot() {
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    debug_assert!(!processor::interrupts_enabled());
    memory::toggle_memory_remap(); // blend sram to 0x0 for IVT

    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    logger::init_logger(log::LevelFilter::Debug);

    // Initialize needed interrupts

    // set the wanted interval for the system timer
    system_timer::init_system_timer_interrupt(6000);
    system_timer::set_real_time_timer_interval(0x64);
    dbgu::set_dbgu_recv_interrupt(true);
    interrupt_controller::init_system_interrupt(
        || {
            debug_assert!(processor::interrupts_enabled());

            // sys_timer_interrrupt_handler
            // print ! if task3 app is active
            if unsafe { TASK3_ACTIVE } {
                println!("!");
            }
            if unsafe { TASK4_ACTIVE } {
                print!("!");
            }

            interrupt_controller::mark_end_of_interrupt!();

            unsafe {
                SCHEDULER_INTERVAL_COUNTER = if SCHEDULER_INTERVAL_COUNTER == 0 {
                    threads::schedule();
                    SCHEDULER_INTERVAL
                } else {
                    SCHEDULER_INTERVAL_COUNTER - 1
                }
            }
        },
        move || unsafe {
            debug_assert!(processor::interrupts_enabled());

            // dbgu_interrupt_handler,fires when rxready is set
            // push char into variable dbgu_buffer on heap, if app does not fetch -> out-of-memory error in allocator
            let last_char =
                dbgu::read_char().expect("there should be char availabe in interrupt") as u8;

            DBGU_BUFFER.push(last_char as char);
            if TASK4_ACTIVE && last_char != 'q' as u8 {
                rost_api::syscalls::create_thread(move || {
                    task4_print(last_char as char);
                });
            }
            interrupt_controller::mark_end_of_interrupt!();
        },
    );

    processor::set_interrupts_enabled!(true);

    unsafe {
        RNG = Some(Pcg64::seed_from_u64(0xDEADBEEF));
    }

    fn start_thread() {
        debug_assert!(processor::ProcessorMode::User == processor::get_processor_mode());
        debug_assert!(processor::interrupts_enabled());

        rost_api::syscalls::create_thread(eval_thread);
        // syscalls::create_thread(custom_user_code_thread);
    }

    // noreturn
    threads::init_runtime(start_thread);
}

fn eval_thread() {
    loop {
        if eval_check() {
            break;
        }
    }
}

fn custom_user_code_thread() {
    const CUSTOM_CODE_ADDRESS: usize = 0x2100_0000;
    // check for custom user code
    if unsafe { core::ptr::read(CUSTOM_CODE_ADDRESS as *const u32) > 0 } {
        unsafe {
            asm!("
            mov lr,  r1
            mov pc, r0", in("r1") rost_api::syscalls::exit_thread, in("r0") CUSTOM_CODE_ADDRESS);
        }
    } else {
        error!(
            "no custom user code loaded into qemu at {:#X}",
            CUSTOM_CODE_ADDRESS
        );
    }
}

const KEY_ENTER: char = 0xD as char;
const KEY_BACKSPACE: char = 0x8 as char;
const KEY_DELETE: char = 0x7F as char;

/// wait for x realtime clock units
fn wait(units: u32) {
    let last = system_timer::get_current_real_time();
    loop {
        if system_timer::get_current_real_time() - last > units {
            break;
        }
    }
}

/// prints a character for a random range between min and max
fn print_character_random<T>(c: T, min: usize, max: usize)
where
    T: core::fmt::Display,
{
    unsafe {
        for _ in 0..RNG.as_mut().unwrap().gen_range(min..max) {
            print!("{}", c);
        }
    }
}

fn task4_print(last_char: char) {
    // print 3 times and wait between
    print_character_random(last_char, 1, 20);
    wait(500);
    print_character_random(last_char, 1, 20);
    wait(500);
    print_character_random(last_char, 1, 20);
}

fn task4() {
    loop {
        // check for a new char in the dbgu buffer
        if let Some(last_char) = unsafe { DBGU_BUFFER.pop() } {
            // quit on q
            if last_char == 'q' {
                break;
            }
        }
    }
}

static mut THREAD_TEST_COUNT: usize = 0;

/// Simple Read–eval–print loop with some basic commands
pub fn eval_check() -> bool {
    let mut char_buf = alloc::string::String::new();

    println!("waiting for input... (press ENTER to echo)");
    println!(
        "available commands: task3, task4, uptime, thread_test, threads, undi, swi, dabort, custom, quit"
    );

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
        allocator::get_current_heap_size(),
        allocator::get_heap_size_left()
    );

    match char_buf.as_str() {
        "swi" => unsafe {
            asm!("swi #99");
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
        "uptime" => {
            println!("{}", system_timer::get_current_real_time());
        }
        "custom" => {
            rost_api::syscalls::create_thread(custom_user_code_thread);
        }
        "task3" => unsafe {
            TASK3_ACTIVE = true;
            let id = rost_api::syscalls::create_thread(custom_user_code_thread);
            loop {
                if threads::is_thread_done(id) {
                    break;
                }
            }
            TASK3_ACTIVE = false;
        },
        "task4" => unsafe {
            TASK4_ACTIVE = true;
            task4();
            TASK4_ACTIVE = false;
        },
        "threads" => {
            threads::print_threads();
        }
        "thread_test" => unsafe {
            for id in 0..20 {
                rost_api::syscalls::create_thread(move || {
                    THREAD_TEST_COUNT += 1;
                    rost_api::syscalls::yield_thread();
                    THREAD_TEST_COUNT += 1;
                    rost_api::syscalls::yield_thread();
                    THREAD_TEST_COUNT += 1;
                    println!("end thread {} {}", id, THREAD_TEST_COUNT);
                });
            }
            rost_api::syscalls::yield_thread();
        },
        "quit" => {
            return true;
        }
        _ => {
            println!("-> Unknown command");
        }
    }

    false
}

/// Rust panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println_with_stack!(256, "panic handler\n{:?}", info);
    loop {}
}
