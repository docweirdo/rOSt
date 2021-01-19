use crate::helpers;

const SRAM_END: usize = 0x2300_0000;
const STACK_SIZE: usize = 1024 * 4;

pub const SP_USER_SYSTEM_START: usize = SRAM_END - 0 * STACK_SIZE; // end of SRAM
pub const SP_FIQ_START: usize = SRAM_END - 1 * STACK_SIZE;
pub const SP_IRQ_START: usize = SRAM_END - 2 * STACK_SIZE;
pub const SP_SVC_START: usize = SRAM_END - 3 * STACK_SIZE;
pub const SP_ABT_START: usize = SRAM_END - 4 * STACK_SIZE;
pub const SP_UND_START: usize = SRAM_END - 5 * STACK_SIZE;

struct MemoryController;
#[allow(dead_code)]
impl MemoryController {
    const BASE_ADDRESS: u32 = 0xFFFFFF00;
    /// MC Remap Control Registe
    const RCR: u32 = 0x0;
    /// MC Abort Status Register
    const MC_ASR: u32 = 0x4;
    /// MC Abort Address Status Register
    const AASR: u32 = 0x8;
}

pub fn toggle_memory_remap() {
    helpers::write_register(MemoryController::BASE_ADDRESS, MemoryController::RCR, 1);
}

pub fn mc_get_abort_address() -> u32 {
    helpers::read_register(MemoryController::BASE_ADDRESS, MemoryController::AASR)
}
