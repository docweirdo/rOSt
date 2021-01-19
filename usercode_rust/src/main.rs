#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;
use rand::prelude::*;
use rand_pcg::Pcg64;

extern crate alloc;

static mut RNG: Option<Pcg64> = None;

macro_rules! print {
    ($($arg:tt)*) =>  {
            let format_string = alloc::format!($($arg)*);
            rost_api::syscalls::send_str_to_dbgu(&format_string);
    }
}

macro_rules! println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\n");
    }
}

/// wait for x realtime clock units
fn wait(units: usize) {
    let last = rost_api::syscalls::get_current_realtime();
    loop {
        if rost_api::syscalls::get_current_realtime() - last > units {
            break;
        }
    }
}

/// prints a character for a random range between min and max
fn print_character_random<T>(c: T, min: usize, max: usize)
where
    T: core::fmt::Display,
{
    unsafe {
        for _ in 0..RNG.as_mut().unwrap().gen_range(min..max) {
            print!("{}", c);
        }
    }
}

fn task3() {
    loop {
        // check for a new char in the dbgu buffer
        if let Some(last_char) = rost_api::syscalls::receive_character_from_dbgu() {
            let last_char: char = last_char as char;
            // quit on q
            if last_char as char == 'q' {
                break;
            }
            // print 3 times and wait between
            print_character_random(last_char, 1, 20);
            rost_api::syscalls::yield_thread();
            wait(500);
            print_character_random(last_char, 1, 20);
            rost_api::syscalls::yield_thread();
            wait(500);
            print_character_random(last_char, 1, 20);
        }
    }
}

#[no_mangle]
pub fn main() -> () {
    unsafe {
        RNG = Some(Pcg64::seed_from_u64(0xDEADBEEF));
    }
    task3();
    println!("end task3");
}

/// Rust panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("panic in usercode");
    loop {}
}

use core::alloc::{GlobalAlloc, Layout};

struct SystemAlloc {}

unsafe impl Sync for SystemAlloc {}

unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        rost_api::syscalls::allocate(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        rost_api::syscalls::deallocate(ptr, layout.size(), layout.align())
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: SystemAlloc = SystemAlloc {};

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    panic!("out of memory");
}
