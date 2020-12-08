use core::debug_assert_ne;

use crate::helpers::{read_register, write_register};
use crate::processor;
use crate::system_timer;
use log::debug;

pub struct AIC;
#[allow(dead_code)]
impl AIC {
    /// Advanced Interrupt Controller base address
    pub const BASE_ADDRESS: u32 = 0xFFFFF000;

    /// AIC Source Vector register 1 aka. system interrupt offset
    const SVR1: u32 = 0x84;

    /// AIC Interrupt Mask register offset
    const IMR: u32 = 0x110;

    /// AIC Interrupt Enable Command register offset
    const IECR: u32 = 0x120;

    /// End of Interrupt Command Register
    pub const EOICR: u32 = 0x130;
}

static mut SYS_TIMER_INTERRUPT_HANDLER: Option<alloc::boxed::Box<dyn FnMut()>> = None;
static mut DBGU_INTERRUPT_HANDLER: Option<alloc::boxed::Box<dyn FnMut()>> = None;

/// Sets system interrupt vector and enables the interrupt.
pub fn init_system_interrupt<F, G>(sys_timer_interrrupt_handler: F, dbgu_interrupt_handler: G)
where
    F: FnMut() + 'static,
    G: FnMut() + 'static,
{
    unsafe {
        SYS_TIMER_INTERRUPT_HANDLER = Some(alloc::boxed::Box::new(sys_timer_interrrupt_handler));
        DBGU_INTERRUPT_HANDLER = Some(alloc::boxed::Box::new(dbgu_interrupt_handler));
    };

    write_register(AIC::BASE_ADDRESS, AIC::SVR1, trampoline as *mut () as u32);

    let mut status: u32 = read_register(AIC::BASE_ADDRESS, AIC::IMR);

    status = status | 0x2;

    write_register(AIC::BASE_ADDRESS, AIC::IECR, status);
}

#[naked]
#[no_mangle]
extern "C" fn trampoline() {
    fn handler() {
        unsafe {
            if system_timer::get_periodic_interrupts_enabled()
                && system_timer::has_system_timer_elapsed()
            {
                if SYS_TIMER_INTERRUPT_HANDLER.is_some() {
                    SYS_TIMER_INTERRUPT_HANDLER.as_mut().unwrap()();
                }
            } else {
                if DBGU_INTERRUPT_HANDLER.is_some() {
                    DBGU_INTERRUPT_HANDLER.as_mut().unwrap()();
                }
            }
        }
    }
    processor::exception_routine!(handler, 4, true, true);
}

macro_rules! _mark_end_of_interrupt{
    () => {
        unsafe { asm!("
            ldr r0, ={}
            str r0, [r0, #{}]
        ", const $crate::interrupt_controller::AIC::BASE_ADDRESS, const $crate::interrupt_controller::AIC::EOICR) };
    }
}

pub(crate) use _mark_end_of_interrupt as mark_end_of_interrupt;
