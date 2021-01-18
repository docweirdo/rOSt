use crate::memory;
use crate::println;
use crate::processor;
use alloc::boxed::Box;
use core::{convert::TryFrom, mem::size_of};
use log::{debug, error, trace};
use num_enum::TryFromPrimitive;
use processor::ProcessorMode;
use rost_macros::exception;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum Syscalls {
    CreateThread = 30,
    ExitThread = 31,
    YieldThread = 32,
}

/// System call to create a thread via software interrupt.
pub fn create_thread<F: FnMut() + 'static>(entry: F) -> usize {
    let id: usize;
    unsafe {
        let entry_raw: (u32, u32) =
            core::mem::transmute(Box::new(entry) as Box<dyn FnMut() + 'static>);

        asm!("mov r0, {0}
              mov r1, {1}
              swi #30
              mov {2}, r0", in(reg) entry_raw.0, in(reg) entry_raw.1, out(reg) id);

        debug_assert!(id == crate::threads::THREADS.last().unwrap().id);
    }
    return id;
}

/// System call to stop and exit the current thread via software interrupt.
pub fn exit_thread() {
    unsafe {
        asm!("swi #31");
    }
}

/// System call to yield the current thread via software interrupt.
pub fn yield_thread() {
    unsafe {
        asm!("swi #32");
    }
}
