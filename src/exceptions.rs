use crate::memory;
use crate::println;
use crate::processor;
use log::{debug,error};
use rost_macros::exception;
use num_enum::TryFromPrimitive;
use core::convert::TryFrom;

#[rost_macros::exception]
unsafe fn Reset() {
    println!("reset");
    panic!();
}

#[rost_macros::exception]
unsafe fn UndefinedInstruction() {
    println!("undefined instruction handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    println!("undefined instruction at {:#X}", lr - 4);
    panic!();
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum Syscalls {
    CreateThread = 30,
    ExitThread = 31,
    YieldThread = 32
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

    println!("software interrupt handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    //debug!("requested service {:?}", service_id);

    match Syscalls::try_from(service_id) {
        Ok(Syscalls::YieldThread) => {
            debug!("syscall: YieldThread");
            super::threads::reschedule();
        },
        Ok(Syscalls::CreateThread) => {
            debug!("syscall: CreateThread");
        },
        Ok(Syscalls::ExitThread) => {
            debug!("syscall: ExitThread");
        },
        _ => {
            error!("unknown syscall id {}", service_id);
        },
    }
}

#[rost_macros::exception]
unsafe fn PrefetchAbort() {
    println!("prefetch abort handler");
    debug!("processor mode {:?}", processor::get_processor_mode());
    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    println!("prefetch abort at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe fn DataAbort() {
    println!("data abort handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize;
    asm!("mov {}, pc", out(reg) lr);
    println!(
        "data abort at {:#X} for address {:#X}",
        lr - 8,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
    panic!();
}
