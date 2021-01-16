use super::processor;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use log::debug;
use log::trace;

const THREAD_STACK_SIZE: usize = 1024 * 8;

#[repr(C, align(4))]
struct TCB {
    id: usize,
    state: ThreadState,
    entry: Box<dyn FnMut()>,
    stack_pos: u32,
    stack: Vec<u8>,
}

static mut THREADS: Vec<TCB> = Vec::<TCB>::new();
static mut RUNNING_THREAD_ID: usize = 0;
static mut LAST_THREAD_ID: usize = 0;

#[derive(PartialEq, Eq, Debug)]
enum ThreadState {
    Ready,
    Running,
    //  Waiting,
    Stopped,
}

use core::mem;

#[repr(C, align(8))]
struct AlignToFour([u8; 8]);

unsafe fn aligned_vec(n_bytes: usize) -> Vec<u8> {
    // Lazy math to ensure we always have enough.
    let n_units = (n_bytes / mem::size_of::<AlignToFour>()) + 1;

    let mut aligned: Vec<AlignToFour> = Vec::with_capacity(n_units);

    let ptr = aligned.as_mut_ptr();
    let len_units = aligned.len();
    let cap_units = aligned.capacity();

    mem::forget(aligned);

    Vec::from_raw_parts(
        ptr as *mut u8,
        len_units * mem::size_of::<AlignToFour>(),
        cap_units * mem::size_of::<AlignToFour>(),
    )
}

pub fn init<F>(entry: F) -> !
where
    F: FnMut() + 'static,
{
    let id = create_thread(entry);
    unsafe {
        RUNNING_THREAD_ID = id;
        THREADS[id].state = ThreadState::Running;
        THREADS[id].stack_pos = THREADS[id]
            .stack
            .as_mut_ptr()
            .offset(THREADS[id].stack.capacity() as isize) as u32;
        for v in &mut THREADS[id].stack {
            *v = 0;
        }

        asm!("mov sp, {stack_address}
              mov pc, {start_address}",
              stack_address = in(reg) THREADS[id].stack_pos,
              start_address = in(reg) new_thread_entry as u32, options(noreturn));
    }
}

unsafe extern "C" fn new_thread_entry() {
    processor::switch_processor_mode!(processor::ProcessorMode::User);

    (THREADS[RUNNING_THREAD_ID].entry)();
    yield_thread();
}

pub fn create_thread<F>(entry: F) -> usize
where
    F: FnMut() + 'static,
{
    unsafe {
        let id = LAST_THREAD_ID;
        LAST_THREAD_ID += 1;
        let mut tcb = TCB {
            id: id,
            state: ThreadState::Ready,
            stack_pos: 0,
            stack: aligned_vec(THREAD_STACK_SIZE),
            entry: Box::new(entry),
        };

        let capacity = tcb.stack.capacity();
        let s_ptr: *mut u8 = tcb.stack.as_mut_ptr().offset(capacity as isize);
        tcb.stack_pos = s_ptr.offset(15 * -4) as u32;

        core::ptr::write_volatile(
            (tcb.stack_pos) as *mut u32,
            processor::ProcessorMode::User as u32,
        );
        core::ptr::write_volatile((tcb.stack_pos + 4) as *mut u32, new_thread_entry as u32);

        THREADS.push(tcb);
        return id;
    }
}

pub fn yield_thread() {
    unsafe {
        asm!("swi #32");
    }
}

pub fn schedule() {
    unsafe {
        let mut pos = RUNNING_THREAD_ID;
        while THREADS[pos].state != ThreadState::Ready {
            pos += 1;
            if pos == THREADS.len() {
                pos = 0;
            }
            if pos == RUNNING_THREAD_ID {
                return;
            }
        }

        THREADS[pos].state = ThreadState::Running;
        let old_pos = RUNNING_THREAD_ID;
        THREADS[old_pos].state = ThreadState::Ready;
        RUNNING_THREAD_ID = pos;

        for thread in &THREADS {
            trace!("thread: {} {:?}", thread.id, thread.state);
        }

        trace!(
            "switch thread from {} sp:{:#X} to {} sp:{:#X}",
            old_pos,
            THREADS[old_pos].stack_pos,
            pos,
            THREADS[pos].stack_pos
        );

        asm!("mov r0, {old}
          mov r1, {new}",
          old = in(reg) &THREADS[old_pos].stack_pos,
          new = in(reg) &THREADS[pos].stack_pos);

        switch_thread();
    }
}

#[naked]
unsafe extern "C" fn switch_thread() {
    asm!(
        "push {{r0-r12}}
         push {{r14}}
         mrs r2, CPSR
         push {{r2}}

         stm r0, {{sp}}
         ldm r1, {{sp}}

         pop {{r2}}
         msr CPSR, r2
         pop {{r14}}
         pop {{r0-r12}}
         
         mov pc, lr",
        options(noreturn)
    );
}
