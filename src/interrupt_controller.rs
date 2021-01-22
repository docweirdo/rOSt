use crate::helpers::{read_register, write_register};
use crate::interrupt_handler;
use crate::processor;
use crate::system_timer;

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

/// Sets system interrupt vector and enables the interrupt.
pub fn init_system_interrupt() {
    write_register(
        AIC::BASE_ADDRESS,
        AIC::SVR1,
        system_interrupt_trampoline as *mut () as u32,
    );

    let mut status: u32 = read_register(AIC::BASE_ADDRESS, AIC::IMR);

    status |= 0x2;

    // enable system interrupt
    write_register(AIC::BASE_ADDRESS, AIC::IECR, status);
}

/// This function wraps the exception handler for simple pass over as a function.  
/// The handler function evaluates wether a specific handler is set before returning    
/// the address of the handler. The exception macro wraps the handler for correct exception handling.
#[rost_macros::interrupt]
unsafe fn system_interrupt() {
    if system_timer::get_periodic_interrupts_enabled() && system_timer::has_system_timer_elapsed() {
        interrupt_handler::system_timer_period_interval_timer_elapsed();
    }
    if super::dbgu::is_char_available() {
        interrupt_handler::dbgu_character_received();
    }
}

macro_rules! _mark_end_of_interrupt{
    () => {
        #[allow(unused_unsafe)]
        unsafe { asm!("
            ldr r0, ={}
            str r0, [r0, #{}]
        ", const $crate::interrupt_controller::AIC::BASE_ADDRESS, const $crate::interrupt_controller::AIC::EOICR) };
    }
}

pub(crate) use _mark_end_of_interrupt as mark_end_of_interrupt;
