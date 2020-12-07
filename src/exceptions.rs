use crate::memory;
use crate::println;
use crate::processor;
use log::debug;

#[no_mangle]
#[naked]
unsafe extern "C" fn ResetHandler() -> ! {
    println!("reset");
    panic!();
}

#[no_mangle]
#[naked]
unsafe extern "C" fn UndefinedInstructionHandler() -> ! {
    fn handler() {
        println!("undefined instruction handler");
        debug!("processor mode {:?}", processor::get_processor_mode());

        let mut lr: usize;
        unsafe {
            asm!("mov {}, r14", out(reg) lr);
        }
        println!("undefined instruction at {:#X}", lr - 4);
    }
    processor::exception_routine!(handler, 4, false, false);
}

#[no_mangle]
#[naked]
unsafe extern "C" fn SoftwareInterruptHandler() -> ! {
    fn handler() {
        println!("software interrupt handler");
        debug!("processor mode {:?}", processor::get_processor_mode());
        let mut lr: usize;
        unsafe {
            asm!("mov {}, r14", out(reg) lr);
        }
        println!("software interrupt at {:#X}", lr - 4);
    };
    processor::exception_routine!(handler, 0, false, false);
}

#[no_mangle]
#[naked]
unsafe extern "C" fn PrefetchAbortHandler() -> ! {
    fn handler() {
        println!("prefetch abort handler");
        debug!("processor mode {:?}", processor::get_processor_mode());
        let mut lr: usize;
        unsafe {
            asm!("mov {}, r14", out(reg) lr);
        }
        println!("prefetch abort at {:#X}", lr - 4);
    };
    processor::exception_routine!(handler, 4, false, false);
}

#[no_mangle]
#[naked]
unsafe extern "C" fn DataAbortHandler() -> ! {
    fn handler() {
        println!("data abort handler");
        debug!("processor mode {:?}", processor::get_processor_mode());

        let mut lr: usize;
        unsafe {
            asm!("mov {}, pc", out(reg) lr);
        }
        println!(
            "data abort at {:#X} for address {:#X}",
            lr - 8,
            memory::mc_get_abort_address() // doesn't work in the emulator
        );
    };
    processor::exception_routine!(handler, 4, false, false);
}
