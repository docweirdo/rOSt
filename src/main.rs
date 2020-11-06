#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

// print utilities

use arrayvec::ArrayString;
use core::fmt::Write;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) =>  {
        let mut send_buf = ArrayString::<[u8; 64]>::new();
        write!(&mut send_buf, $($arg)*).expect("Can't write");
        send_str(&send_buf);
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        print!($($arg)*);
        print!("\n");
    })
}

pub unsafe fn send_str(chars: &str) {
    for character in chars.chars() {
        send_char(character);
    }
}

pub unsafe fn send_char(character: char) {
    if (read_volatile(DBGU.offset(DBGU_SR / 4)) & DBGU_TXRDY) != 0 {
        write_volatile(DBGU.offset(DBGU_THR / 4), character as u32);
    }
}

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
const PIO_A: *mut u32 = 0xFFFFF400 as *mut u32;
const DBGU: *mut u32 = 0xFFFFF200 as *mut u32;

//Offsets PIO_A
const PIO_PDR: isize = 0x0004;
const PIO_ASR: isize = 0x0070;
const PIO_PUDR: isize = 0x0060;

//Offsets DBGU
const DBGU_CR: isize = 0x0000; // Control Register
const DBGU_SR: isize = 0x0014; // Status Register
const DBGU_RHR: isize = 0x0018; // Receive Holding Register
const DBGU_THR: isize = 0x001C; //Transmit Holding Register
const DBGU_BRGR: isize = 0x0020; // Baud Rate Generator Register

//Bits
const DBGU_RX: u32 = 1 << 30;
const DBGU_TX: u32 = 1 << 31;

// DBGU_SR - Status Register
const DBGU_TXRDY: u32 = 1 << 1; // Transmitter Ready
const DBGU_RXRDY: u32 = 1 << 0; // Receiver Ready

#[no_mangle] // don't mangle the name of this function
pub unsafe extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    // uart_setup();

    println!(
        "{} {}: the start",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("waiting for input...");

    loop {
        eval_check();
    }

    println!("the end");
}

pub unsafe fn uart_setup() {
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
}

const KEY_ENTER: u32 = 0xD;
const KEY_BACKSPACE: u32 = 0x8;
const KEY_DELETE: u32 = 0x7F;

pub unsafe fn eval_check() {
    let mut char_buf = ArrayString::<[u8; 48]>::new();
    loop {
        if (read_volatile(DBGU.offset(DBGU_SR / 4)) & DBGU_RXRDY) != 0 {
            let last_char = read_volatile(DBGU.offset(DBGU_RHR / 4));
            if last_char == KEY_ENTER {
                break;
            }
            if last_char == KEY_DELETE || last_char == KEY_BACKSPACE {
                char_buf.pop();
            } else {
                char_buf.push(core::char::from_u32_unchecked(last_char));
            }
        }
    }

    println!("Received: {}", char_buf);
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
