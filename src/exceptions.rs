use crate::memory;
use crate::println;
use crate::processor;
use core::convert::TryFrom;
use log::{debug, error};
use num_enum::TryFromPrimitive;
use rost_macros::exception;

#[rost_macros::exception]
unsafe fn Reset() {
    println!("reset");
    panic!();
}

#[rost_macros::exception]
unsafe fn UndefinedInstruction() {
    debug!("undefined instruction handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

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
unsafe fn SoftwareInterrupt() {
    let mut service_id: u32;
    asm!("LDR     r12, [r12, #-4] 
          BIC r12,r12,#0xff000000  
          MOV {}, r12", out(reg) service_id);

    // let mut lr: usize;
    // asm!("mov {}, r12", out(reg) lr);
    // println!("software interrupt at {:#X}", lr-4);

    debug!("software interrupt handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    //debug!("requested service {:?}", service_id);

    match Syscalls::try_from(service_id) {
        Ok(Syscalls::YieldThread) => {
            debug!("syscall: YieldThread");
            super::threads::schedule();
        }
        Ok(Syscalls::CreateThread) => {
            debug!("syscall: CreateThread");
        }
        Ok(Syscalls::ExitThread) => {
            debug!("syscall: ExitThread");
            super::threads::exit();
        }
        _ => {
            error!("unknown syscall id {}", service_id);
        }
    }
}

#[rost_macros::exception]
unsafe fn PrefetchAbort() {
    error!("prefetch abort handler");
    debug!("processor mode {:?}", processor::get_processor_mode());
    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("prefetch abort at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe fn DataAbort() {
    error!("data abort handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize;
    asm!("mov {}, pc", out(reg) lr);
    error!(
        "data abort at {:#X} for address {:#X}",
        lr - 8,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
    panic!();
}
