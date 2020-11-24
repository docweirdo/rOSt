use crate::switch_processor_mode_naked;
use crate::ProcessorMode;
use core::ptr::write_volatile;


const SRAM_END: usize = 0x0020_4000;
const STACK_SIZE: usize = 1024 * 2;

const SP_USER_SYSTEM_START: usize = SRAM_END - 0 * STACK_SIZE; // end of SRAM
const SP_FIQ_START: usize = SRAM_END - 1 * STACK_SIZE;
const SP_IRQ_START: usize = SRAM_END - 2 * STACK_SIZE;
const SP_SVC_START: usize = SRAM_END - 3 * STACK_SIZE;
const SP_ABT_START: usize = SRAM_END - 4 * STACK_SIZE;
const SP_UND_START: usize = SRAM_END - 5 * STACK_SIZE;

const MC: *mut u32 = 0xFFFFFF00 as *mut u32;
const MC_RCR: isize = 0x0;

pub fn toggle_memory_remap() {
    unsafe { write_volatile(MC.offset(MC_RCR / 4), 1 as u32) }
}

// pub fn init_heap() {
//     let heap_start = 0x2000_0000;
//     let heap_end = 0x2400_0000;
//     let heap_size = heap_end - heap_start;
//     unsafe {
//         ALLOCATOR.lock().init(heap_start, heap_size);
//     }
// }

pub fn init_processor_mode_stacks() {
    unsafe {
        switch_processor_mode_naked(ProcessorMode::FIQ);
        asm!("ldr sp, ={}",  const SP_FIQ_START);
        switch_processor_mode_naked(ProcessorMode::IRQ);
        asm!("ldr sp, ={}",  const SP_IRQ_START);
        switch_processor_mode_naked(ProcessorMode::Abort);
        asm!("ldr sp, ={}",  const SP_ABT_START);
        switch_processor_mode_naked(ProcessorMode::Undefined);
        asm!("ldr sp, ={}",  const SP_UND_START);
        switch_processor_mode_naked(ProcessorMode::System);
        asm!("ldr sp, ={}",  const SP_USER_SYSTEM_START);
        switch_processor_mode_naked(ProcessorMode::Supervisor);
        asm!("ldr sp, ={}",  const SP_SVC_START);
    }
}

// #[alloc_error_handler]
// fn alloc_error(_layout: Layout) -> ! {
//     println!("alloc_error_handler");
//     loop {}
// }
