use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

const USER_THREAD_STACK_SIZE: usize = 1024 * 4;
const KERNEL_THREAD_STACK_SIZE: usize = 1024 * 4;

struct TCB {
    id: usize,
    state: ThreadState,
    user_stack: Vec<u8>,
    kernel_stack: Vec<u8>,
    entry: Box<fn()>,
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

pub fn create_thread(entry: fn()) -> usize {
    unsafe {
        let id = LAST_THREAD_ID;
        LAST_THREAD_ID += 1;
        let mut tcb = TCB {
            id: id,
            state: ThreadState::Stopped,
            user_stack: vec![0; USER_THREAD_STACK_SIZE],
            kernel_stack: vec![0; KERNEL_THREAD_STACK_SIZE],
            entry: Box::new(entry),
        };

        // TODO: prepare thread for switching
        let size = tcb.user_stack.len();
        unsafe {
            let s_ptr = tcb.user_stack.as_mut_ptr().offset(size as isize);
            core::ptr::write(s_ptr.offset(-8) as *mut u64, *tcb.entry as u64);
        }

        THREADS.push(tcb);
        return id;
    }
}

pub fn schedule_threads() {
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
        RUNNING_THREAD_ID = pos;

        // let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
        // let new: *const ThreadContext = &self.threads[pos].ctx;
        switch_thread();
    }
}

pub fn switch_thread() {
    // save registers and context on user stack
    // change stack pointer to new thread
    // copy context from stack into registers
    // jump to pc from stack
}
