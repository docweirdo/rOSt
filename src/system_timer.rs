use crate::helpers::{read_register, read_register_bit, write_register};

struct ST;
#[allow(dead_code)]
impl ST {
    /// System Timer base address
    const BASE_ADDRESS: u32 = 0xFFFFFD00;

    /// ST period interval mode register offset
    const PIMR: u32 = 0x4;

    /// ST Real-time Mode Register
    const RTMR: u32 = 0x000C;

    /// ST Status Register offset
    const SR: u32 = 0x10;

    /// ST interrupt enable register offset
    const IER: u32 = 0x14;

    /// ST intterupt mask register offset
    const IMR: u32 = 0x1C;

    //// ST Current Real-time Register offset
    const CRTR: u32 = 0x24;
}

/// Enables periodic timer interrupt and sets the counter to value.
pub fn init_system_timer_interrupt(value: u16) {
    write_register(ST::BASE_ADDRESS, ST::IER, 0x1);

    //Todo: export in own function
    write_register(ST::BASE_ADDRESS, ST::RTMR, 0x64);
    write_register(ST::BASE_ADDRESS, ST::PIMR, value as u32);
}

pub fn get_current_real_time() -> u32 {
    read_register(ST::BASE_ADDRESS, ST::CRTR)
}

pub fn has_system_timer_elapsed() -> bool {
    read_register_bit(ST::BASE_ADDRESS, ST::SR, 0) != 0
}

pub fn get_periodic_interrupts_enabled() -> bool {
    read_register_bit(ST::BASE_ADDRESS, ST::IMR, 0) != 0
}
