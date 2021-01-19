use alloc::boxed::Box;
use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum Syscalls {
    SendDBGU = 10,
    ReceiveDBGU = 11,
    Allocate = 20,
    Deallocate = 21,
    CreateThread = 30,
    ExitThread = 31,
    YieldThread = 32,
    GetCurrentRealTime = 40,
    Sleep = 41,
}

pub fn get_current_realtime() -> usize {
    let time: usize;
    unsafe {
        asm!("swi #{}
              mov {}, r0", const Syscalls::GetCurrentRealTime as u32, out(reg) time);
    }
    return time;
}

pub fn sleep(time: usize) -> usize {
    let actual_sleep: usize;
    unsafe {
        asm!("swi #{}", 
        const Syscalls::Sleep as u32, in("r0") time, lateout("r0") actual_sleep)
    }
    return actual_sleep;
}

pub fn allocate(size: usize, align: usize) -> *mut u8 {
    let out_ptr: usize;
    unsafe {
        asm!("
              swi #{}
              mov {}, r0
            ", const Syscalls::Allocate as u32, out(reg) out_ptr, in("r0") size, in("r1") align,);
    }
    return out_ptr as *mut u8;
}

pub fn deallocate(ptr: *mut u8, size: usize, align: usize) {
    unsafe {
        asm!("
              swi #{call_id}
            ", call_id = const Syscalls::Deallocate as u32, in("r0") ptr, in("r1") size, in("r2") align);
    }
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
pub extern "C" fn create_thread<F: FnMut() + 'static>(entry: F) -> usize {
    let id: usize;
    unsafe {
        let entry_raw: (u32, u32) =
            core::mem::transmute(Box::into_raw(Box::new(entry) as Box<dyn FnMut() + 'static>));

        asm!("mov r0, {0}
              mov r1, {1}
              swi #30
              mov {2}, r0", in(reg) entry_raw.0, in(reg) entry_raw.1, out(reg) id);
    }
    return id;
}

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
