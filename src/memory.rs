use crate::helpers;
use crate::println;
use crate::processor;
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
};

const SRAM_END: usize = 0x2300_0000;
const STACK_SIZE: usize = 1024 * 4;

pub const SP_USER_SYSTEM_START: usize = SRAM_END - 0 * STACK_SIZE; // end of SRAM
pub const SP_FIQ_START: usize = SRAM_END - 1 * STACK_SIZE;
pub const SP_IRQ_START: usize = SRAM_END - 2 * STACK_SIZE;
pub const SP_SVC_START: usize = SRAM_END - 3 * STACK_SIZE;
pub const SP_ABT_START: usize = SRAM_END - 4 * STACK_SIZE;
pub const SP_UND_START: usize = SRAM_END - 5 * STACK_SIZE;

const MC: u32 = 0xFFFFFF00;
const MC_RCR: u32 = 0x0;
// const MC_ASR: u32 = 0x4;
const MC_AASR: u32 = 0x8;

pub fn toggle_memory_remap() {
    helpers::write_register(MC, MC_RCR, 1);
}

pub fn mc_get_abort_address() -> u32 {
    helpers::read_register(MC, MC_AASR)
}

macro_rules! _init_processor_mode_stacks {
    () => {
        #[allow(unused_unsafe)]
        unsafe {
            use crate::processor;
            asm!("ldr sp, ={}",  const memory::SP_SVC_START);
            processor::switch_processor_mode!(processor::ProcessorMode::FIQ);
            asm!("ldr sp, ={}",  const memory::SP_FIQ_START);
            processor::switch_processor_mode!(processor::ProcessorMode::IRQ);
            asm!("ldr sp, ={}",  const memory::SP_IRQ_START);
            processor::switch_processor_mode!(processor::ProcessorMode::Abort);
            asm!("ldr sp, ={}",  const memory::SP_ABT_START);
            processor::switch_processor_mode!(processor::ProcessorMode::Undefined);
            asm!("ldr sp, ={}",  const memory::SP_UND_START);
            processor::switch_processor_mode!(processor::ProcessorMode::System);
            asm!("ldr sp, ={}",  const memory::SP_USER_SYSTEM_START);
            processor::switch_processor_mode!(processor::ProcessorMode::Supervisor);
        }
    };

}

pub(crate) use _init_processor_mode_stacks as init_processor_mode_stacks;

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
        let align_mask: usize = !(align - 1);

        // crate::println_with_stack!("\nsize: {} align: {}", size, align);

        // move start up to the next alignment boundary
        let start: usize = (*head + align - 1) & align_mask;

        if start + size > self.end {
            // a null pointer signal an Out Of Memory condition
            core::ptr::null_mut()
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
static GLOBAL_ALLOCATOR: BumpPointerAlloc = BumpPointerAlloc {
    head: UnsafeCell::new(HEAP_START),
    end: HEAP_END,
};

pub fn get_current_heap_size() -> usize {
    unsafe { *GLOBAL_ALLOCATOR.head.get() - HEAP_START }
}

pub fn get_heap_size_left() -> usize {
    unsafe { HEAP_END - *GLOBAL_ALLOCATOR.head.get() }
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    crate::println_with_stack!("alloc: out of memory");
    loop {}
}
