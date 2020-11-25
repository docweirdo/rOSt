use crate::memory;
use crate::println;
use crate::processor;
use log::debug;

#[no_mangle]
unsafe extern "C" fn ResetHandler() -> ! {
    println!("reset");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn UndefinedInstructionHandler() -> ! {
    println!("undefined instruction handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize = 0xDEADBEEF;
    asm!("mov {}, r14", out(reg) lr);
    println!("undefined instruction at {:#X}", lr - 4);
    panic!();
}

#[no_mangle]
#[naked]
unsafe extern "C" fn SoftwareInterruptHandler() -> ! {
    println!("software interrupt handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize = 0xDEADBEEF;
    asm!("mov {}, r14", out(reg) lr);
    println!("software interrupt at {:#X}", lr - 4);
    panic!();
}

#[no_mangle]
unsafe extern "C" fn PrefetchAbortHandler() -> ! {
    println!("prefetch abort");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn DataAbortHandler() -> ! {
    println!("data abort handler");
    debug!("processor mode {:?}", processor::get_processor_mode());

    let mut lr: usize = 0xDEADBEEF;
    asm!("mov {}, pc", out(reg) lr);
    println!(
        "data abort at {:#X} for address {:#X}",
        lr - 8,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
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
