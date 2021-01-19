use crate::print_with_stack;
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
};

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
            if size >= 256 && log::log_enabled!(log::Level::Trace) {
                print_with_stack!(128, "alloc {:#X} size: {}\n", start, size);
            }
            start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, layout: Layout) {
        if layout.size() >= 256 && log::log_enabled!(log::Level::Trace) {
            print_with_stack!(128, "dealloc {:#X} size: {}\n", _ptr as u32, layout.size());
        }

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
    crate::println_with_stack!(32, "alloc: out of memory");
    loop {}
}
