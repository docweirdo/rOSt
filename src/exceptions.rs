use crate::print;
use crate::println;
use arrayvec::ArrayString;
use core::fmt::Write;

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

    //let mut pc: usize = 24;
    //asm!("mov {}, pc", out(reg) pc);      // Read from PC, Doku nachlesen!!!
    //println!("software interrupt at {:X}", pc);
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