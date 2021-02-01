use crate::memory;
use crate::processor;
use log::{error, trace};
use processor::ProcessorMode;

#[rost_macros::exception]
unsafe extern "C" fn Reset() {
    panic!("reset handler");
}

#[rost_macros::exception]
unsafe extern "C" fn UndefinedInstruction(lr: usize) {
    trace!("undefined instruction handler");
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    panic!("undefined instruction at {:#X}", lr - 4);
}

#[rost_macros::exception]
unsafe extern "C" fn SoftwareInterrupt(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    service_id: usize,
) -> usize {
    trace!(
        "software interrupt handler {} {} {} {}",
        arg0,
        arg1,
        arg2,
        service_id
    );
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    crate::syscall_handlers::syscall_handler(arg0, arg1, arg2, service_id)
}

#[rost_macros::exception]
unsafe extern "C" fn PrefetchAbort(lr: usize) {
    error!("prefetch abort handler");
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    panic!("prefetch abort at {:#X}", lr - 4);
}

#[rost_macros::exception]
unsafe extern "C" fn DataAbort(lr: usize) {
    error!("data abort handler");
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    panic!(
        "data abort at {:#X} for address {:#X}",
        lr - 4,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
}
