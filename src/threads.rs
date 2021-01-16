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

/// Creates a vector which is aligned to four bytes in memory.
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

/// Initializes the first thread to run on the processor after boot.
pub fn init<F>(entry: F) -> !
where
    F: FnMut() + 'static,
{
    fn idle_thread() {
        crate::println!("idle thread");
        loop {}
    }
    create_thread(idle_thread);

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

/// Prepares newly created threads for lifes challenges.   
///
/// Gets executed when the thread is scheduled for the first time  
/// and `switch_thread()` returns to this function. Enables interrupts  
/// and switches the processor to `ProcessorMode::User`. Then calls the  
/// users closure and provides an `exit_thread()` guard beneath.  
unsafe extern "C" fn new_thread_entry() {
    processor::set_interrupts_enabled!(true);
    processor::switch_processor_mode!(processor::ProcessorMode::User);

    (THREADS[RUNNING_THREAD_ID].entry)();
    exit_thread();
}

/// Creates TCB and Stack for a new thread.
///
/// Takes the entry function provided by the user and creates  
/// a TCB and stack for the new thread. Builds up a fake stack
/// to be popped when the thread is first switched to by `switch_thread()`.  
/// This fake stack contains a Processor Status in System Mode and
/// the address which gets popped into the Link Register pointing
/// to `new_thread_entry()`.
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
            processor::ProcessorMode::System as u32,
        );
        core::ptr::write_volatile((tcb.stack_pos + 4) as *mut u32, new_thread_entry as u32);

        THREADS.push(tcb);
        return id;
    }
}

/// System call to stop and exit the current thread via software interrupt.
pub fn exit_thread() {
    unsafe {
        asm!("swi #31");
    }
}

/// System call to yield the current thread via software interrupt.
pub fn yield_thread() {
    unsafe {
        asm!("swi #32");
    }
}

/// Function called by the Kernel to set the running thread to `ThreadState::Stopped`.
pub fn exit() {
    unsafe {
        THREADS
            .iter_mut()
            .find(|t| t.id == RUNNING_THREAD_ID)
            .unwrap()
            .state = ThreadState::Stopped;
        // TODO: remove old threads
        // THREADS.retain(|t| t.state != ThreadState::Stopped || t.id == RUNNING_THREAD_ID);
    }
    schedule();
}

/// Schedules and switches to a new thread to run on the processor.    
///
/// This function needs to be called in a privileged mode and cycles  
/// through threads until it finds the next one that is ready. It then  
/// calls `switch_thread` to switch to the selected thread.  
/// TCBs and Stacks of threads with `ThreadState::Stopped` are removed.
pub fn schedule() {
    unsafe {
        processor::set_interrupts_enabled!(false);
        let running_thread_pos = THREADS
            .iter()
            .position(|t| t.id == RUNNING_THREAD_ID)
            .unwrap();
        let mut running_thread = &mut THREADS[running_thread_pos];

        let mut next_thread_pos = running_thread_pos;
        while THREADS[next_thread_pos].state != ThreadState::Ready {
            next_thread_pos += 1;
            if next_thread_pos == THREADS.len() {
                next_thread_pos = 1;
            }
            if next_thread_pos == running_thread_pos {
                if running_thread.state == ThreadState::Stopped {
                    next_thread_pos = 0;
                    break;
                } else {
                    return;
                }
            }
        }
        let mut next_thread = &mut THREADS[next_thread_pos];

        next_thread.state = ThreadState::Running;
        if running_thread.state == ThreadState::Running {
            running_thread.state = ThreadState::Ready;
        }
        RUNNING_THREAD_ID = next_thread.id;

        for thread in &THREADS {
            trace!("thread: {} {:?}", thread.id, thread.state);
        }

        trace!(
            "switch thread from {} sp:{:#X} to {} sp:{:#X}",
            running_thread.id,
            running_thread.stack_pos,
            next_thread.id,
            next_thread.stack_pos
        );

        crate::print!("\n");

        asm!("mov r0, {old}
          mov r1, {new}",
          old = in(reg) &running_thread.stack_pos,
          new = in(reg) &next_thread.stack_pos);

        switch_thread();
    }
}

/// Switches from the current thread to the thread whose stack  
/// pointer is passed in r1. The function saves the current context
/// to the stack of the current thread, saves the stack pointer in the
/// TCB and switches the stack pointer with the stack pointer of the
/// new thread. It then pops the context off the new threads stack
/// and returns to the now different instruction pointed to by the
/// Link Register.
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
