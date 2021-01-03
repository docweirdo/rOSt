use crate::memory;
use crate::println;
use crate::processor;
use log::debug;
use rost_macros::exception;

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
}

#[rost_macros::exception]
unsafe fn SoftwareInterrupt() {
    println!("software interrupt handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    println!("software interrupt at {:#X}", lr - 4);
}

#[rost_macros::exception]
unsafe fn PrefetchAbort() {
    println!("prefetch abort handler");
    debug!("processor mode {:?}", processor::get_processor_mode());
    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    println!("prefetch abort at {:#X}", lr - 4);
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
}
