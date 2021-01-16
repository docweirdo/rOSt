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

static mut DBGU_BUFFER: Vec<char> = Vec::<char>::new();
static mut PRINT_SYSTEM_TIMER_TASK3: bool = false;

/// Initializes the operating system
pub fn boot() {
    memory::toggle_memory_remap(); // blend sram to 0x0 for IVT

    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    logger::init_logger();
    debug!("processor mode {:?}", processor::get_processor_mode());

    // Initialize needed interrupts

    // set the wanted interval for the system timer
    system_timer::init_system_timer_interrupt(32000);
    system_timer::set_real_time_timer_interval(0x64);
    dbgu::set_dbgu_recv_interrupt(true);
    interrupt_controller::init_system_interrupt(
        || {
            // sys_timer_interrrupt_handler
            // print ! if task3 app is active
            if unsafe { PRINT_SYSTEM_TIMER_TASK3 } {
                unsafe { threads::schedule(); }
                println!("!");
            }
        },
        move || unsafe {
            // dbgu_interrupt_handler,fires when rxready is set
            // push char into variable dbgu_buffer on heap, if app does not fetch -> out-of-memory error in allocator
            DBGU_BUFFER.push(
                dbgu::read_char().expect("there should be char availabe in interrupt") as u8
                    as char,
            )
        },
    );
    // Switch to user code

    processor::set_interrupts_enabled!(true);
    processor::switch_processor_mode!(processor::ProcessorMode::User);

    fn eval_thread() {
      debug!("processor mode {:?}", processor::get_processor_mode());

      fn simple_thread_1() {
        println!("thread: simple_1 running");
        threads::yield_thread();
      }

      let id = threads::create_thread(simple_thread_1);
      println!("create thread {}", id);

      loop {
        println!("thread: eval running");
         if eval_check() {
             break;
         }
         threads::yield_thread();
      }
    }

    //let id = threads::create_thread(eval_thread);
    //println!("create thread {}", id);

    println!("start threading");

    threads::init(eval_thread);

    //println!("the end");
    panic!();
}

const KEY_ENTER: char = 0xD as char;
const KEY_BACKSPACE: char = 0x8 as char;
const KEY_DELETE: char = 0x7F as char;

/// Simple Read–eval–print loop with some basic commands
pub fn eval_check() -> bool {
    // initialize rng for task3
    let mut rng = Pcg64::seed_from_u64(0xDEADBEEF);
    let mut char_buf = alloc::string::String::new();

    println!("waiting for input... (press ENTER to echo)");
    println!("available commands: task3, uptime, swi, undi, dabort, quit");

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
        // task3 app
        "task3" => {
            unsafe {
                PRINT_SYSTEM_TIMER_TASK3 = true;
            }
            loop {
                // check for a new char in the dbgu buffer
                if let Some(last_char) = unsafe { DBGU_BUFFER.pop() } {
                    // quit on q
                    if last_char == 'q' {
                        unsafe {
                            PRINT_SYSTEM_TIMER_TASK3 = false;
                        }
                        break;
                    }
                    /// wait for x realtime clock units
                    fn wait(units: u32) {
                        let last = system_timer::get_current_real_time();
                        loop {
                            if system_timer::get_current_real_time() - last > units {
                                break;
                            }
                        }
                    }
                    // prints a character for a random range between min and max
                    let mut print_character_random = |c: char, min: usize, max: usize| {
                        for _ in 0..rng.gen_range(min, max) {
                            print!("{}", c);
                        }
                    };
                    // print 3 times and wait between
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

/// Rust panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // TODO: print with stack or heap? why does it crash sometimes?
    print_with_stack!("panic handler");
    print_with_stack!("{}", _info);
    loop {}
}
