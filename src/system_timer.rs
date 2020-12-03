use crate::helpers::{write_register, read_register, read_register_bit};

/// System Timer base address
const ST: u32 = 0xFFFFFD00;

/// ST period interval mode register offset
const ST_PIMR: u32 = 0x4;

/// ST period Status Register offset
const ST_SR: u32 = 0x10;

/// ST period interrupt enable register offset
const ST_IER: u32 = 0x14;

/// Enables periodic timer interrupt and sets the counter to value.
pub fn init_system_timer_interrupt(value : u16){
    write_register(ST, ST_IER, 0x1);

    write_register(ST, ST_PIMR, value as u32);
}

pub fn has_system_timer_elapsed() -> bool {
    read_register_bit(ST, ST_SR, 0) != 0
}