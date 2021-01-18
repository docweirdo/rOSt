#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

use core::panic::PanicInfo;

#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Syscalls {
    SendDBGU = 10,
    ReceiveDBGU = 11,
    CreateThread = 30,
    ExitThread = 31,
    YieldThread = 32,
}

pub fn send_str_to_dbgu(chars: &str) {
    for character in chars.chars() {
        send_character_to_dbgu(character as u8);
    }
}

pub fn send_character_to_dbgu(character: u8) {
    unsafe {
        asm!("mov r0, {}
              swi #{}
            ", in(reg) character as u8, const Syscalls::SendDBGU as u32);
    }
}

pub fn receive_character_from_dbgu() -> Option<u8> {
    let out_char: u32;
    unsafe {
        asm!("swi #{}
              mov {}, r0", const Syscalls::ReceiveDBGU as u32, out(reg) out_char);
    }
    if out_char == 0xFFFF {
        return None;
    }
    return Some(out_char as u8);
}

/// System call to create a thread via software interrupt.
// pub extern "C" fn create_thread<F: FnMut() + 'static>(entry: F) -> usize {
//     let id: usize;
//     unsafe {
//         let entry_raw: (u32, u32) =
//             core::mem::transmute(Box::into_raw(Box::new(entry) as Box<dyn FnMut() + 'static>));

//         asm!("mov r0, {0}
//               mov r1, {1}
//               swi #30
//               mov {2}, r0", in(reg) entry_raw.0, in(reg) entry_raw.1, out(reg) id);
//     }
//     return id;
// }

/// System call to stop and exit the current thread via software interrupt.
pub extern "C" fn exit_thread() {
    unsafe {
        asm!("swi #31");
    }
}

/// System call to yield the current thread via software interrupt.
pub extern "C" fn yield_thread() {
    unsafe {
        asm!("swi #32");
    }
}

#[no_mangle]
pub fn main() -> () {
    send_str_to_dbgu("test");
    send_str_to_dbgu("test");
}

/// Rust panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    send_str_to_dbgu("panic");
    loop {}
}
