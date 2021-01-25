use crate::system_timer;

use super::processor;
use crate::alloc::borrow::ToOwned;
use alloc::{alloc::alloc, alloc::dealloc, boxed::Box};
use alloc::{
    collections::btree_map::BTreeMap, collections::btree_set::BTreeSet,
    collections::vec_deque::VecDeque, vec::Vec,
};
use core::{alloc::Layout, debug_assert, panic};
use log::trace;

const THREAD_STACK_SIZE: usize = 1024 * 8;
const IDLE_THREAD_ID: ThreadId = 0;
/// The amount of SysTicks before the scheduler gets called.
pub(crate) static SCHEDULER_INTERVAL: u32 = 5;
pub(crate) static mut SCHEDULER_INTERVAL_COUNTER: u32 = 0;

#[repr(C, align(4))]
pub struct TCB {
    pub id: ThreadId,
    pub(crate) state: ThreadState,
    entry: Box<dyn FnMut() + 'static>,
    stack_current: *mut u8,
    stack_start: *mut u8,
    pub(crate) parent_thread_id: ThreadId,
    pub(crate) subscribed_services:
        BTreeMap<rost_api::syscalls::ThreadServices, VecDeque<ThreadMessage>>,
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
static mut RUNNING_THREAD_ID: ThreadId = 0;
static mut LAST_THREAD_ID: ThreadId = 0;

type TimeoutValue = usize;
type ThreadId = usize;

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ThreadMessage {
    DBGU(char),
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum WaitingReason {
    DBGU,
    Sleep(TimeoutValue),
    Join(BTreeSet<ThreadId>, Option<TimeoutValue>),
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ThreadState {
    Ready,
    Running,
    Waiting(WaitingReason),
    Stopped,
}

/// Initializes the first thread to run on the processor after boot.
pub fn init_runtime<F>(entry: F) -> !
where
    F: FnMut() + 'static,
{
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());

    unsafe {
        THREADS.reserve(24);
    }

    fn idle_thread() {
        trace!("executing idle thread");
        loop {
            unsafe {
                // wait for interrupt low power mode
                asm!(
                    "mov {tmp}, 0
                     mcr p15, 0, {tmp}, c7, c0, 4"
                , tmp = out(reg) _);
            }
        }
    }

    let id = create_thread_internal(Box::new(idle_thread));
    debug_assert!(id == IDLE_THREAD_ID);
    debug_assert!(get_thread_by_id(id).unwrap().parent_thread_id == IDLE_THREAD_ID);

    let id = create_thread_internal(Box::new(entry));
    debug_assert!(id == 1);
    debug_assert!(get_thread_by_id(id).unwrap().parent_thread_id == IDLE_THREAD_ID);
    unsafe {
        RUNNING_THREAD_ID = id;
        let thread = get_current_thread();
        thread.state = ThreadState::Running;
        thread.stack_current = thread.stack_start;

        asm!("mov sp, {stack_address}
              mov pc, {start_address}",
              stack_address = in(reg) thread.stack_current,
              start_address = in(reg) new_thread_entry, options(noreturn));
    }
}

/// prints information about current threads
pub fn print_threads() {
    unsafe {
        crate::println!("threads:");
        for thread in &THREADS {
            crate::println!(
                "  id: {} state: {:?} last_stack_size: {:#X}",
                thread.id,
                thread.state,
                thread.stack_start.offset_from(thread.stack_current) as u32,
            );
        }
    }
}

/// returns running thread
pub fn get_current_thread<'a>() -> &'a mut TCB {
    unsafe {
        THREADS
            .iter_mut()
            .find(|t| t.id == RUNNING_THREAD_ID)
            .unwrap()
    }
}

/// returns thread for given id or None if not found
pub fn get_thread_by_id<'a>(thread_id: usize) -> Option<&'a mut TCB> {
    unsafe { THREADS.iter_mut().find(|t| t.id == thread_id) }
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

    (get_current_thread().entry)();
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

        let stack_start = buffer.add(THREAD_STACK_SIZE);

        let mut tcb = TCB {
            id,
            parent_thread_id: RUNNING_THREAD_ID,
            state: ThreadState::Ready,
            stack_current: stack_start,
            stack_start,
            entry,
            subscribed_services: BTreeMap::new(),
        };

        tcb.stack_current = tcb.stack_current.offset(15 * -4);

        core::ptr::write_volatile(
            (tcb.stack_current.offset(0)) as *mut usize,
            processor::ProcessorMode::System as usize,
        );
        core::ptr::write_volatile(
            (tcb.stack_current.offset(4)) as *mut usize,
            new_thread_entry as usize,
        );

        THREADS.push(tcb);
        id
    }
}

/// Function called by the Kernel to set the running thread to `ThreadState::Stopped`.
pub fn exit_internal() {
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    unsafe {
        let current_thread = get_current_thread();
        current_thread.state = ThreadState::Stopped;

        // remove id from joined parent thread if available
        if let Some(parent_thread) = get_thread_by_id(current_thread.parent_thread_id) {
            if let ThreadState::Waiting(WaitingReason::Join(joined_thread_ids, _)) =
                &mut parent_thread.state
            {
                let present = joined_thread_ids.remove(&RUNNING_THREAD_ID);
                if present && joined_thread_ids.is_empty() {
                    parent_thread.state = ThreadState::Ready;
                }
            }
        }
    }
    schedule(None);
}

pub fn wakeup_elapsed_threads() {
    unsafe {
        let current_timestamp = system_timer::get_current_real_time() as usize;
        // find waiting thread with elapsed timestamp
        let thread = THREADS.iter_mut().find(|t| {
            if let ThreadState::Waiting(WaitingReason::Sleep(wakeup_timestamp)) = t.state {
                if wakeup_timestamp <= current_timestamp {
                    return true;
                }
            }
            false
        });

        // found waiting thread with elapsed timestamp -> schedule
        if let Some(thread) = thread {
            thread.state = ThreadState::Ready;
            schedule(Some(thread.id));
        }
    }
}

pub fn handle_dbgu_new_character_event(character: char) {
    unsafe {
        for thread in &mut THREADS {
            if let Some(messages) = thread
                .subscribed_services
                .get_mut(&rost_api::syscalls::ThreadServices::DBGU)
            {
                messages.push_back(ThreadMessage::DBGU(character));
            }
            if let ThreadState::Waiting(WaitingReason::DBGU) = thread.state {
                debug_assert!(thread
                    .subscribed_services
                    .contains_key(&rost_api::syscalls::ThreadServices::DBGU));
                thread.state = ThreadState::Ready;
            }
        }
    }
}

/// Schedules and switches to a new thread to run on the processor.    
///
/// This function needs to be called in a privileged mode and cycles  
/// through threads until it finds the next one that is ready. It then  
/// calls `switch_thread` to switch to the selected thread.  
/// TCBs and Stacks of threads with `ThreadState::Stopped` are removed.
pub fn schedule(next_thread_id: Option<usize>) {
    debug_assert!(processor::ProcessorMode::System == processor::get_processor_mode());
    unsafe {
        if THREADS.is_empty() {
            log::error!("scheduler called before thread initialization");
            return;
        }
        processor::set_interrupts_enabled!(false);

        // remove stopped threads but not the current one if stopped
        THREADS.retain(|t| t.state != ThreadState::Stopped || t.id == RUNNING_THREAD_ID);

        let running_thread_pos = THREADS
            .iter()
            .position(|t| t.id == RUNNING_THREAD_ID)
            .unwrap();
        let running_thread = &mut THREADS[running_thread_pos];

        let mut next_thread_pos: usize;

        // first check for optional argument: specific next_thread_id to schedule
        if let Some(next_thread_id) = next_thread_id {
            if let Some(pos) = THREADS.iter().position(|t| t.id == next_thread_id) {
                next_thread_pos = pos;
            } else {
                panic!("scheduler: invalid thread_id given");
            }
        } else {
            next_thread_pos = running_thread_pos;

            // simple round-robin-scheduler
            // find the next thread which is ready from the current position
            while THREADS[next_thread_pos].state != ThreadState::Ready {
                next_thread_pos += 1;

                // cycle from the beginning if last pos in threads array
                if next_thread_pos == THREADS.len() {
                    // start at first real thread after idle
                    next_thread_pos = 1;
                    // but if we are already in the idle thread nothing else to do
                    if running_thread.id == 0 {
                        debug_assert!(!THREADS
                            .iter()
                            .any(|t| t.id != IDLE_THREAD_ID && t.state == ThreadState::Ready));
                        return;
                    }
                }
                // back at the thread we started our journey
                if next_thread_pos == running_thread_pos {
                    // no other thread ready and this thread is stopped or waiting
                    // then switch to the idle thread
                    match running_thread.state {
                        ThreadState::Waiting(_) | ThreadState::Stopped => {
                            next_thread_pos = 0;
                            break;
                        }
                        _ => {
                            // else stay in the same thread
                            return;
                        }
                    }
                }
            }
        }
        let next_thread = &mut THREADS[next_thread_pos];
        debug_assert!(next_thread.state == ThreadState::Ready);

        next_thread.state = ThreadState::Running;
        // only switch back old thread to ready if not waiting or stopped
        if running_thread.state == ThreadState::Running {
            running_thread.state = ThreadState::Ready;
        }
        RUNNING_THREAD_ID = next_thread.id;

        if crate::user_tasks::TASK4_ACTIVE {
            let convert = |id| match id {
                0 => "idle".to_owned(),
                2 => "repl".to_owned(),
                _ => alloc::format!("{}", id),
            };
            crate::println!(
                " {} -> {}",
                convert(running_thread.id),
                convert(next_thread.id)
            );
        }

        log::trace!(
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

        switch_thread(&running_thread.stack_current, &next_thread.stack_current);

        // always reset scheduler interval counter to prevent immediate rescheduling
        // maybe called from wakeup or other places
        SCHEDULER_INTERVAL_COUNTER = SCHEDULER_INTERVAL;
        processor::set_interrupts_enabled!(true);
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
#[inline(never)]
unsafe extern "C" fn switch_thread(_running_thread: &*mut u8, _current_thread: &*mut u8) {
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
