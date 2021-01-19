use super::processor;
use alloc::vec::Vec;
use alloc::{alloc::alloc, alloc::dealloc, boxed::Box};
use core::{alloc::Layout, panic};
use log::debug;

const THREAD_STACK_SIZE: usize = 1024 * 8;
/// The amount of SysTicks before the scheduler gets called.
pub(crate) static SCHEDULER_INTERVAL: u32 = 5;
pub(crate) static mut SCHEDULER_INTERVAL_COUNTER: u32 = 0;

#[repr(C, align(4))]
pub struct TCB {
    pub id: usize,
    state: ThreadState,
    entry: Box<dyn FnMut() + 'static>,
    stack_current: *mut u8,
    stack_start: *mut u8,
}

impl Drop for TCB {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(THREAD_STACK_SIZE, core::mem::align_of::<u64>())
            .expect("Bad layout");
        unsafe {
            dealloc(
                self.stack_start.offset(-(THREAD_STACK_SIZE as isize)),
                layout,
            );
        }
    }
}

pub static mut THREADS: Vec<TCB> = Vec::<TCB>::new();
static mut RUNNING_THREAD_ID: usize = 0;
static mut LAST_THREAD_ID: usize = 0;

#[derive(PartialEq, Eq, Debug)]
enum ThreadState {
    Ready,
    Running,
    //  Waiting,
    Stopped,
}

/// Initializes the first thread to run on the processor after boot.
pub fn init_runtime<F>(entry: F) -> !
where
    F: FnMut() + 'static,
{
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    fn idle_thread() {
        crate::println!("idle thread");
        loop {
            unsafe {
                // wait for interrupt low power mode
                asm!(
                    "mov r0, 0
                     mcr p15, 0, r0, c7, c0, 4"
                );
            }
        }
    }

    let id = create_thread_internal(Box::new(idle_thread));
    debug_assert!(id == 0);

    let id = create_thread_internal(Box::new(entry));
    debug_assert!(id == 1);
    unsafe {
        RUNNING_THREAD_ID = id;
        let thread = THREADS
            .iter_mut()
            .find(|t| t.id == RUNNING_THREAD_ID)
            .unwrap();
        thread.state = ThreadState::Running;
        thread.stack_current = thread.stack_start;

        asm!("mov sp, {stack_address}
              mov pc, {start_address}",
              stack_address = in(reg) thread.stack_current,
              start_address = in(reg) new_thread_entry as u32, options(noreturn));
    }
}

/// prints information about current threads
pub fn print_threads() -> () {
    unsafe {
        crate::println!("threads:");
        for thread in &THREADS {
            crate::println!(
                "  id: {} state: {:?} last_stack_size: {:#X}",
                thread.id,
                thread.state,
                thread.stack_start.offset_from(thread.stack_current) as u32
            );
        }
    }
}

pub fn is_thread_done(id: usize) -> bool {
    unsafe {
        let thread = THREADS.iter().find(|t| t.id == id);
        if let Some(thread) = thread {
            if thread.state == ThreadState::Running || thread.state == ThreadState::Ready {
                return false;
            }
        }
        return true;
    }
}

/// Prepares newly created threads for lifes challenges.   
///
/// Gets executed when the thread is scheduled for the first time  
/// and `switch_thread()` returns to this function. Enables interrupts  
/// and switches the processor to `ProcessorMode::User`. Then calls the  
/// users closure and provides an `exit_thread()` guard beneath.  
unsafe extern "C" fn new_thread_entry() {
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    processor::set_interrupts_enabled!(true);
    // run idle thread in system mode for low power mode
    if RUNNING_THREAD_ID > 0 {
        processor::switch_processor_mode!(processor::ProcessorMode::User);
    }

    (THREADS
        .iter_mut()
        .find(|t| t.id == RUNNING_THREAD_ID)
        .unwrap()
        .entry)();
    rost_api::syscalls::exit_thread();
}

/// Creates TCB and Stack for a new thread.
///
/// Takes the entry function provided by the user and creates  
/// a TCB and stack for the new thread. Builds up a fake stack
/// to be popped when the thread is first switched to by `switch_thread()`.  
/// This fake stack contains a Processor Status in System Mode and
/// the address which gets popped into the Link Register pointing
/// to `new_thread_entry()`.
pub fn create_thread_internal(entry: Box<dyn FnMut() + 'static>) -> usize {
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    unsafe {
        let id = LAST_THREAD_ID;
        LAST_THREAD_ID += 1;

        let layout = Layout::from_size_align(THREAD_STACK_SIZE, core::mem::align_of::<u64>())
            .expect("Bad layout");

        let buffer = alloc(layout);
        if buffer.is_null() {
            panic!("buffer is null");
        }

        let stack_start = buffer.offset(THREAD_STACK_SIZE as isize);

        let mut tcb = TCB {
            id: id,
            state: ThreadState::Ready,
            stack_current: stack_start,
            stack_start: stack_start,
            entry: entry,
        };

        tcb.stack_current = tcb.stack_current.offset(15 * -4);

        core::ptr::write_volatile(
            (tcb.stack_current.offset(0)) as *mut u32,
            processor::ProcessorMode::System as u32,
        );
        core::ptr::write_volatile(
            (tcb.stack_current.offset(4)) as *mut u32,
            new_thread_entry as u32,
        );

        THREADS.push(tcb);
        return id;
    }
}

/// Function called by the Kernel to set the running thread to `ThreadState::Stopped`.
pub fn exit_internal() {
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    unsafe {
        THREADS
            .iter_mut()
            .find(|t| t.id == RUNNING_THREAD_ID)
            .unwrap()
            .state = ThreadState::Stopped;
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
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    unsafe {
        processor::set_interrupts_enabled!(false);

        // remove old threads
        THREADS.retain(|t| t.state != ThreadState::Stopped || t.id == RUNNING_THREAD_ID);

        let running_thread_pos = THREADS
            .iter()
            .position(|t| t.id == RUNNING_THREAD_ID)
            .unwrap();
        let running_thread = &mut THREADS[running_thread_pos];

        let mut next_thread_pos = running_thread_pos;
        while THREADS[next_thread_pos].state != ThreadState::Ready {
            next_thread_pos += 1;
            if next_thread_pos == THREADS.len() {
                next_thread_pos = 1;
                if running_thread.id == 0 {
                    debug_assert!(THREADS.len() == 1);
                    return;
                }
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
        let next_thread = &mut THREADS[next_thread_pos];

        next_thread.state = ThreadState::Running;
        if running_thread.state == ThreadState::Running {
            running_thread.state = ThreadState::Ready;
        }
        RUNNING_THREAD_ID = next_thread.id;

        debug!(
            "t#: {} switch thread from {} sp:{:#X} to {} sp:{:#X}",
            THREADS.len(),
            running_thread.id,
            running_thread
                .stack_start
                .offset_from(running_thread.stack_current) as u32,
            next_thread.id,
            next_thread
                .stack_start
                .offset_from(next_thread.stack_current) as u32
        );

        asm!("mov r0, {old}
          mov r1, {new}",
          old = in(reg) &running_thread.stack_current,
          new = in(reg) &next_thread.stack_current);

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
