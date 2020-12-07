use crate::helpers::{write_register, read_register, read_register_bit};


struct ST;
#[allow(dead_code)]
impl ST {

    /// System Timer base address
    const BASE_ADDRESS: u32 = 0xFFFFFD00;

    /// ST period interval mode register offset
    const PIMR: u32 = 0x4;

    /// ST Status Register offset
    const SR: u32 = 0x10;

    /// ST interrupt enable register offset
    const IER: u32 = 0x14;

    /// ST intterupt mask register offset
    const IMR: u32 = 0x1C;

}

/// Enables periodic timer interrupt and sets the counter to value.
pub fn init_system_timer_interrupt(value : u16){
    write_register(ST::BASE_ADDRESS, ST::IER, 0x1);

    write_register(ST::BASE_ADDRESS, ST::PIMR, value as u32);
}

pub fn has_system_timer_elapsed() -> bool {
    read_register_bit(ST::BASE_ADDRESS, ST::SR, 0) != 0
}

pub fn get_periodic_interrupts_enabled() -> bool {
    read_register_bit(ST::BASE_ADDRESS, ST::IMR, 0) != 0
}