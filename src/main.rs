#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

//
//
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
const PIO_A: *mut u32 =  0xFFFFF400 as *mut u32;
const DBGU: *mut u32 =   0xFFFFF200 as *mut u32;

//Offsets PIO_A
const PIO_PDR: isize =    0x0004;  
const PIO_ASR: isize =    0x0070;
const PIO_PUDR: isize =   0x0060;

//Offsets DBGU
const DBGU_CR: isize =    0x0000;
const DBGU_THR: isize =   0x001C;
const DBGU_SR: isize =    0x0014;
const DBGU_BRGR: isize =  0x0020;

//Bits
const DBGU_RX: u32 =    1<<30;
const DBGU_TX: u32 =    1<<31;
const DBGU_TXRDY: u32 = 1<<1;


#[no_mangle] // don't mangle the name of this function
pub unsafe extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    let a = 1;
    let b = 2;
    let c = 3;

    uart_setup();

    loop{
        print_char();
    }
}

pub unsafe fn uart_setup() {

    //Disable PIO Controll
    write_volatile(PIO_A.offset(PIO_PDR), DBGU_TX);
    write_volatile(PIO_A.offset(PIO_PDR), DBGU_RX);

    //Set Periphal A
    write_volatile(PIO_A.offset(PIO_ASR), DBGU_TX);
    write_volatile(PIO_A.offset(PIO_ASR), DBGU_RX);

    //Disable Pull up
    write_volatile(PIO_A.offset(PIO_PUDR), DBGU_TX);
    write_volatile(PIO_A.offset(PIO_PUDR), DBGU_RX);

    //Enable DBGU
    write_volatile(DBGU.offset(DBGU_CR), DBGU_TX);

    //Set Baudrate
    write_volatile(DBGU.offset(DBGU_BRGR), 0x01);

}

pub unsafe fn print_char() {

    if (read_volatile(DBGU.offset(DBGU_SR)) ^ DBGU_TXRDY) != 0 {
        write_volatile(DBGU.offset(DBGU_THR), 67);
    }

}


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
