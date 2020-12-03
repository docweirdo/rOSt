/// Writes u32 memory at the specified base + offset
pub fn write_register(base: u32, offset: u32, input: u32) {
    unsafe { core::ptr::write_volatile((base + offset) as *mut u32, input) }
}

/// Reads u32 memory at the specified base + offset
pub fn read_register(base: u32, offset: u32) -> u32 {
    unsafe { core::ptr::read_volatile((base + offset) as *const u32) }
}

/// Reads the specified bit from the u32 memory at the specified base + offset
pub fn read_register_bit(base: u32, offset: u32, bit: u32) -> u32 {
    unsafe { core::ptr::read_volatile((base + offset) as *const u32) & 1 << bit  }
}