use crate::print;
use crate::println;
use arrayvec::ArrayString;
use core::fmt::Write;
use log::debug;

#[no_mangle]
unsafe extern "C" fn ResetHandler() -> ! {
    println!("reset");
    panic!();
}

#[no_mangle]
unsafe extern "C" fn UndefinedInstructionHandler() -> ! {
    println!("undefined instruction handler");
    debug!("processor mode {:?}", crate::get_mode());
    let pc: usize = 24;
    //asm!("mov {}, pc", out(reg) pc);      // Read from PC, Doku nachlesen!!!
    println!("undefined instruction at {:X}", pc);
    panic!();
}

#[no_mangle]
unsafe extern "C" fn SoftwareInterruptHandler() -> ! {
    println!("software interrupt handler");
    debug!("processor mode {:?}", crate::get_mode());

    let pc: usize = 24;
    //asm!("mov {}, pc", out(reg) pc);      // Read from PC, Doku nachlesen!!!
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
    println!("data abort handler");
    debug!("processor mode {:?}", crate::get_mode());
    let pc: usize = 24;
    //asm!("mov {}, pc", out(reg) pc);      // Read from PC, Doku nachlesen!!!
    println!("data abort at {:X}", pc);
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
