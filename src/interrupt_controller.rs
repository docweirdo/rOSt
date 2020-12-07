use core::debug_assert_ne;

use crate::helpers::{read_register, write_register};
use crate::processor;
use log::debug;

/// Advanced Interrupt Controller base address
pub const AIC: u32 = 0xFFFFF000;

/// AIC Source Vector register 1 aka. system interrupt offset
const AIC_SVR1: u32 = 0x84;

/// AIC Interrupt Mask register offset
const AIC_IMR: u32 = 0x110;

/// AIC Interrupt Enable Command register offset
const AIC_IECR: u32 = 0x120;

/// End of Interrupt Command Register
pub const AIC_EOICR: u32 = 0x130;

static mut INTERRUPT_HANDLER: Option<alloc::boxed::Box<dyn Fn()>> = None;

/// Sets system interrupt vector and enables the interrupt.
pub fn init_system_interrupt(new_interrrupt_handler: fn()) {
    unsafe {
        INTERRUPT_HANDLER = Some(alloc::boxed::Box::new(new_interrrupt_handler));
    };
    write_register(AIC, AIC_SVR1, trampoline as *mut () as u32);

    let mut status: u32 = read_register(AIC, AIC_IMR);

    status = status | 0x2;

    write_register(AIC, AIC_IECR, status);
}

#[naked]
#[no_mangle]
extern "C" fn trampoline() {
    fn handler() {
        unsafe {
            let mut sp: usize;
            unsafe {
                asm!("mov {}, sp", out(reg) sp);
            }
            debug!("sp {:X}", sp);
            if INTERRUPT_HANDLER.is_some() {
                INTERRUPT_HANDLER.as_ref().unwrap()();
            }

            // debug!("spsadssp");
        }
    }
    processor::exception_routine!(handler, 4);
}

macro_rules! _mark_end_of_interrupt{
    () => {
        unsafe { asm!("
            ldr r0, ={}
            str r0, [r0, #{}]
        ", const $crate::interrupt_controller::AIC, const $crate::interrupt_controller::AIC_EOICR) };
    }
}

pub(crate) use _mark_end_of_interrupt as mark_end_of_interrupt;
