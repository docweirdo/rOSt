#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm)]

extern crate alloc;

use core::panic::PanicInfo;
use log::error;

mod allocator;
mod dbgu;
mod exception_handlers;
mod fmt;
mod helpers;
mod interrupt_controller;
mod interrupt_handlers;
mod logger;
mod memory;
mod processor;
mod syscall_handlers;
mod system_timer;
mod threads;
mod user_tasks;

/// Initial OS entry point: Sets stack pointers and calls boot function
/// # Safety
///
/// This function should not be called before the horsemen are ready.
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

/// Initializes the operating system.
///
/// TODO: Add detailed description
pub fn boot() {
    memory::toggle_memory_remap(); // blend sram to 0x0 for IVT
    allocator::init_allocator(); // init allocator before print allocations

    assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    assert!(!processor::interrupts_enabled());

    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    logger::init_logger(log::LevelFilter::Debug);

    // Initialize needed interrupts

    // set the wanted interval for the system timer
    system_timer::init_system_timer_interrupt(core::time::Duration::from_millis(10));
    system_timer::set_real_time_timer_interval(core::time::Duration::from_millis(10));
    dbgu::set_dbgu_recv_interrupt(true);
    interrupt_controller::init_system_interrupt();

    processor::set_interrupts_enabled!(true);

    fn start_thread() {
        assert!(processor::ProcessorMode::User == processor::get_processor_mode());
        assert!(processor::interrupts_enabled());

        rost_api::syscalls::create_thread(user_tasks::read_eval_print_loop);
        // syscalls::create_thread(custom_user_code_thread);
    }

    // noreturn
    threads::init_runtime(start_thread);
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

/// Rust panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println_with_stack!(256, "panic handler\n{:?}", info);
    loop {}
}
