use crate::memory;
use crate::println;
use crate::processor;
use alloc::boxed::Box;
use core::convert::TryFrom;
use log::{debug, error, trace};
use num_enum::TryFromPrimitive;
use processor::ProcessorMode;
use rost_macros::exception;

#[rost_macros::exception]
unsafe fn Reset() {
    error!("reset handler");
    panic!();
}

#[rost_macros::exception]
unsafe fn UndefinedInstruction() {
    trace!("undefined instruction handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::Undefined);

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("undefined instruction at {:#X}", lr - 4);
    panic!();
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum Syscalls {
    CreateThread = 30,
    ExitThread = 31,
    YieldThread = 32,
}

#[rost_macros::exception]
unsafe extern "C" fn SoftwareInterrupt(_r0: u32, r1: u32, _r2: u32) {
    let mut service_id: u32;
    asm!("LDR     r12, [r12, #-4] 
          BIC r12,r12,#0xff000000  
          MOV {}, r12", out(reg) service_id);

    // let mut lr: usize;
    // asm!("mov {}, r12", out(reg) lr);
    // println!("software interrupt at {:#X}", lr-4);

    trace!("software interrupt handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    match Syscalls::try_from(service_id) {
        Ok(Syscalls::YieldThread) => {
            trace!("syscall: YieldThread");
            super::threads::schedule();
        }
        Ok(Syscalls::CreateThread) => {
            trace!("syscall: CreateThread {}", r1);
            // super::threads::create_thread();
        }
        Ok(Syscalls::ExitThread) => {
            trace!("syscall: ExitThread");
            super::threads::exit_internal();
        }
        _ => {
            error!("unknown syscall id {}", service_id);
        }
    }
}

#[rost_macros::exception]
unsafe fn PrefetchAbort() {
    error!("prefetch abort handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::Abort);

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("prefetch abort at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe fn DataAbort() {
    error!("data abort handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::Abort);

    let mut lr: usize;
    asm!("mov {}, pc", out(reg) lr);
    error!(
        "data abort at {:#X} for address {:#X}",
        lr - 8,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
    panic!();
}
