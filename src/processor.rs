use num_traits::{FromPrimitive, ToPrimitive};

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessorMode {
    User = 0x10,
    FIQ = 0x11,
    IRQ = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1b,
    System = 0x1F,
}

/// Returns the current processor mode.
/// Requires the caller to be in a priviliged mode.
pub fn get_processor_mode() -> ProcessorMode {
    let mut cpsr: u32;

    unsafe {
        asm!("MRS {0}, CPSR", out(reg) cpsr);
    }

    ProcessorMode::from_u8((cpsr & 0x1F) as u8).unwrap()
}

/// Switches the processor to the specified mode.
/// Requires the caller to be in a priviliged mode.
#[naked]
#[allow(unused_variables)]
#[inline(always)]
pub fn switch_processor_mode_naked(new_mode: ProcessorMode) {
    unsafe {
        asm!(
            "
        MOV r2, lr
        MRS r1, cpsr
        BIC r1, r1, #0x1F
        ORR r1, r1, r0
        MSR cpsr_c, r1
        MOV lr, r2
        " // we save lr because lr gets corrupted during mode switch
        );
    }
}

pub fn enable_interrupts() {
    unsafe {
        asm!("
            MRS r0, CPSR
            BIC r0, r0, #0x80
            MSR    CPSR_c, r0
        ")
    }
}
