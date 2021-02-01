use crate::helpers;

/*

Offsets:
PIO_PDR 0x04    Disable Register
PIO ASR 0x0070  Peripheral Select A Register
PIO_PUDR 0x0060   Pull-up Disable (64 enable)

^Page 354^

0xFFFF F200 DBGU Address
DBGU: PIO Controller A Periphal A i/o line PA30/31

#define PIO_A 0xFF FFF 400

Master CLock divided 16 times value(DBGU_BRGR), min val 1, max val 65536
DBGU_CR 0x0000  Control Register => TXEN = bit 6
DBGU_MR 0x0004  Mode Register
DBGU_THR 0x001C Transmit Holding Register => 0-7
DBGU_BRGR 0x0020 Baud Rate Generator

DBGU_SR 0x0014  Status Register => TXRDY = bit 1

*/

//Base Addresses
//const PIO_A: *mut u32 = 0xFFFFF400 as *mut u32;
struct DBGU;
#[allow(dead_code)]
impl DBGU {
    /// DBGU Base Address
    const BASE_ADDRESS: u32 = 0xFFFFF200;

    /// Control Register Offset
    const CR: u32 = 0x0000;

    /// Interrupt Enable Register Offset
    const IER: u32 = 0x0008;

    /// Interrupt Disable Register Offset
    const IDR: u32 = 0x000C;

    /// Interrupt Mask Register Offset
    const IMR: u32 = 0x0010;

    /// Status Register Offset
    const SR: u32 = 0x0014;
    /// Receive Holding Register Offset
    const RHR: u32 = 0x0018;
    /// Transmit Holding Register Offset
    const THR: u32 = 0x001C;

    /// Baud Rate Generator Register Offset
    const BRGR: u32 = 0x0020;

    /// DBGU_SR - Status Register Bits

    /// Receiver Ready Bit
    const RXRDY: u32 = 0;
    /// Transmitter Ready Bit
    const TXRDY: u32 = 1;
}

/*
pub unsafe fn dbgu_setup() {
    //Disable PIO Controll
    //  write_volatile(PIO_A.offset(PIO_PDR / 4), DBGU_TX | DBGU_RX);

    //Set Periphal A
    //  write_volatile(PIO_A.offset(PIO_ASR / 4), DBGU_TX | DBGU_RX);

    //Disable Pull up
    // write_volatile(PIO_A.offset(PIO_PUDR / 4), DBGU_TX | DBGU_RX);

    //Enable DBGU
    // write_volatile(DBGU.offset(DBGU_CR / 4), DBGU_TX | DBGU_RX);

    //Set Baudrate
    // write_volatile(DBGU.offset(DBGU_BRGR / 4), 65536);
//}
*/

/// Enable or disable DBGU Receive Interrupt
pub fn set_dbgu_recv_interrupt(value: bool) {
    if value {
        helpers::write_register(DBGU::BASE_ADDRESS, DBGU::IER, 0x1);
    } else {
        helpers::write_register(DBGU::BASE_ADDRESS, DBGU::IDR, 0x1);
    }
}

/// Checks if the DBGU receive holding register holds a character
pub fn is_char_available() -> bool {
    helpers::read_register_bit(DBGU::BASE_ADDRESS, DBGU::SR, DBGU::RXRDY) != 0
}

/// Returns a character from the DBGU Receive Holding Register or None if not available
pub fn read_char() -> Option<u32> {
    if is_char_available() {
        Some(helpers::read_register(DBGU::BASE_ADDRESS, DBGU::RHR))
    } else {
        None
    }
}

/// Writes a character to the DBGU Transmit Holding Register when the DBGU is ready
/// TODO: Loop or return error if not ready?
pub fn write_char(character: char) {
    if helpers::read_register_bit(DBGU::BASE_ADDRESS, DBGU::SR, DBGU::TXRDY) != 0 {
        helpers::write_register(DBGU::BASE_ADDRESS, DBGU::THR, character as u32);
    }
}
