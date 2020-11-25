use crate::println;
use crate::switch_processor_mode_naked;
use crate::ProcessorMode;
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::{self, write_volatile},
};

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

#[naked]
#[inline(always)]
pub fn init_processor_mode_stacks() {
    unsafe {
        asm!("ldr sp, ={}",  const SP_SVC_START);
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
    }
}

// https://rust-embedded.github.io/book/collections/index.html?using-alloc

struct BumpPointerAlloc {
    head: UnsafeCell<usize>,
    end: usize,
}

unsafe impl Sync for BumpPointerAlloc {}

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let head: *mut usize = self.head.get();
        let size: usize = layout.size();
        let align: usize = layout.align();
        let align_mask : usize = !(align - 1);

        // move start up to the next alignment boundary
        let start: usize = (*head + align - 1) & align_mask;

        if start + size > self.end {
            // a null pointer signal an Out Of Memory condition
            ptr::null_mut()
        } else {
            *head = start + size;
            start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // this allocator never deallocates memory
    }
}

const HEAP_START: usize = 0x2300_0000;
const HEAP_END: usize = 0x2400_0000;

#[global_allocator]
static HEAP: BumpPointerAlloc = BumpPointerAlloc {
    head: UnsafeCell::new(HEAP_START),
    end: HEAP_END,
};

pub fn get_current_heap_size() -> usize {
    unsafe { *HEAP.head.get() - HEAP_START }
}

pub fn get_heap_size_left() -> usize {
    unsafe { HEAP_END - *HEAP.head.get() }
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    crate::println_with_stack!("alloc: out of memory");
    loop {}
}
