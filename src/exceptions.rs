use crate::memory;
use crate::println;
use crate::processor;
use crate::syscalls::Syscalls;
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
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("undefined instruction at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe extern "C" fn SoftwareInterrupt(arg0: u32, arg1: u32, arg2: u32, service_id: u32) -> usize {
    // let mut lr: usize;
    // asm!("mov {}, r12", out(reg) lr);
    // println!("software interrupt at {:#X}", lr-4);

    trace!(
        "software interrupt handler {} {} {} {}",
        arg0,
        arg1,
        arg2,
        service_id
    );
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    match Syscalls::try_from(service_id) {
        Ok(Syscalls::YieldThread) => {
            trace!("syscall: YieldThread");
            super::threads::schedule();
            return 0;
        }
        Ok(Syscalls::CreateThread) => {
            trace!("syscall: CreateThread");
            let raw_entry: *mut dyn FnMut() = core::mem::transmute((arg0, arg1));
            let entry = Box::<dyn FnMut() + 'static>::from_raw(raw_entry);
            let id = super::threads::create_thread_internal(entry);
            return id;
        }
        Ok(Syscalls::ExitThread) => {
            trace!("syscall: ExitThread");
            super::threads::exit_internal();
            return 0;
        }
        _ => {
            error!("unknown syscall id {}", service_id);
            panic!();
        }
    }
}

#[rost_macros::exception]
unsafe fn PrefetchAbort() {
    error!("prefetch abort handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("prefetch abort at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe fn DataAbort() {
    error!("data abort handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    let mut lr: usize;
    asm!("mov {}, pc", out(reg) lr);
    error!(
        "data abort at {:#X} for address {:#X}",
        lr - 8,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
    panic!();
}
