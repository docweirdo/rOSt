use alloc::boxed::Box;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

/// syscall calling convention
/// syscall id via swi assembly and r0-r2 are used for possible arguments 

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
    JoinThread = 33,
    Subscribe = 34,
    Unsubscribe = 35,
    GetCurrentRealTime = 40,
    Sleep = 41,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive, Ord, PartialOrd)]
#[repr(u32)]
pub enum ThreadServices {
    DBGU = 10,
}

#[inline(never)]
pub extern "C" fn subscribe(service: ThreadServices) {
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::Subscribe as u32, in("r0") service as u32);
    }
}

#[inline(never)]
pub extern "C" fn unsubscribe(service: ThreadServices) {
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::Unsubscribe as u32, in("r0") service as u32);
    }
}

#[inline(never)]
pub extern "C" fn get_current_realtime() -> usize {
    let time: usize;
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::GetCurrentRealTime as u32, lateout("r0") time);
    }
    time
}

#[inline(never)]
pub extern "C" fn sleep_ms(time_ms: usize) -> usize {
    let actual_sleep: usize;
    unsafe {
        asm!("swi #{call_id}", 
        call_id = const Syscalls::Sleep as u32, in("r0") time_ms, lateout("r0") actual_sleep)
    }
    actual_sleep
}

#[inline(never)]
pub fn join_thread(thread_id: usize, timeout: Option<usize>) -> usize {
    let child_thread_result: usize;
    unsafe {
        asm!("swi #{call_id}", 
        call_id = const Syscalls::JoinThread as u32, in("r0") thread_id, in("r1") timeout.unwrap_or_default(), lateout("r0") child_thread_result)
    }
    child_thread_result
}

#[inline(never)]
pub extern "C" fn allocate(size: usize, align: usize) -> *mut u8 {
    let out_ptr: usize;
    unsafe {
        asm!("
              swi #{call_id}
            ", call_id = const Syscalls::Allocate as u32, lateout("r0") out_ptr, in("r0") size, in("r1") align,);
    }
    out_ptr as *mut u8
}

#[inline(never)]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize, align: usize) {
    unsafe {
        asm!("
              swi #{call_id}
            ", call_id = const Syscalls::Deallocate as u32, in("r0") ptr, in("r1") size, in("r2") align);
    }
}

#[inline(never)]
pub fn send_str_to_dbgu(chars: &str) {
    for character in chars.chars() {
        send_character_to_dbgu(character as u8);
    }
}

#[inline(never)]
pub extern "C" fn send_character_to_dbgu(character: u8) {
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::SendDBGU as u32, in("r0") character as u8);
    }
}

#[inline(never)]
pub extern "C" fn receive_character_from_dbgu() -> u8 {
    let out_char: u32;
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::ReceiveDBGU as u32, lateout("r0") out_char, in("r0") 1);
    }
    out_char as u8
}

#[inline(never)]
pub fn receive_character_from_dbgu_noblock() -> Option<u8> {
    let out_char: u32;
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::ReceiveDBGU as u32, lateout("r0") out_char, in("r0") 0);
    }
    if out_char == 0xFFFF {
        return None;
    }
    Some(out_char as u8)
}

/// System call to create a thread via software interrupt.
#[inline(never)]
pub fn create_thread<F: FnMut() + 'static>(entry: F) -> usize {
    let id: usize;
    unsafe {
        let entry_raw: (u32, u32) =
            core::mem::transmute(Box::into_raw(Box::new(entry) as Box<dyn FnMut() + 'static>));

        asm!("swi #{call_id}", call_id = const Syscalls::CreateThread as u32, in("r0") entry_raw.0, in("r1") entry_raw.1, lateout("r0") id);
    }
    id
}

/// System call to stop and exit the current thread via software interrupt.
#[inline(never)]
pub extern "C" fn exit_thread() {
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::ExitThread as u32);
    }
}

/// System call to yield the current thread via software interrupt.
#[inline(never)]
pub extern "C" fn yield_thread() {
    unsafe {
        asm!("swi #{call_id}", call_id = const Syscalls::YieldThread as u32);
    }
}
