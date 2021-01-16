use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use super::processor;

const THREAD_STACK_SIZE: usize = 1024 * 4;

#[repr(C, align(4))]
struct TCB {
    stack: Vec<u8>,
    stack_pos: u32,
    id: usize,
    state: ThreadState,
    entry: Box<fn()>,
}

static mut THREADS: Vec<TCB> = Vec::<TCB>::new();
static mut RUNNING_THREAD_ID: usize = 0;
static mut LAST_THREAD_ID: usize = 0;
static mut RESCHEDULE: bool = false;

#[derive(PartialEq, Eq, Debug)]
enum ThreadState {
    Ready,
    Running,
    //  Waiting,
    Stopped,
}

use core::mem;

#[repr(C, align(4))]
struct AlignToFour([u8; 4]);

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

pub fn init(entry: fn()) -> ! {
    let id = create_thread(entry);
    unsafe { 
        RUNNING_THREAD_ID = id;
        THREADS[id].state = ThreadState::Running;
        THREADS[id].stack_pos = 0;
        for v in &mut THREADS[id].stack {
            *v = 0;
        }
        asm!("mov sp, {stack_address}
              mov lr, {yield_address}
              mov pc, {start_address}",
              stack_address = in(reg) &THREADS[id].stack,
              yield_address = in(reg) yield_thread as u32,
              start_address = in(reg) entry as u32, options(noreturn));
    }
}

// kompletten context sichern & stack switch => fn (thread_function) ==> yield 
// kompletten context sichern & switch_to to stack - #64 (edit lr)
pub fn create_thread(entry: fn()) -> usize {
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

        let size = tcb.stack.len();
        tcb.stack_pos = tcb.stack.as_mut_ptr().offset(size as isize) as u32 - (31*4);
        unsafe {
            let s_ptr : *mut u8 = tcb.stack.as_mut_ptr().offset(size as isize);

             // r14_irq, r2-r12, spsr, __ , cpsr, __ , lr_user, __

            core::ptr::write(s_ptr.offset(1*-4) as *mut u32, *tcb.entry as u32); // lr_irq
            // for i in 2..13 {
            //     core::ptr::write(s_ptr.offset(i*-4) as *mut u32, *tcb.entry as u32); // r2-r12              
            // }
            core::ptr::write(s_ptr.offset(14*-4) as *mut u32, processor::ProcessorMode::User as u32); // spsr               
            //core::ptr::write(s_ptr.offset(15*-4) as *mut u32, processor::ProcessorMode::User as u32); // r0
            core::ptr::write(s_ptr.offset(16*-4) as *mut u32, processor::ProcessorMode::IRQ as u32); // cpsr
            //core::ptr::write(s_ptr.offset(16*-4) as *mut u32, yield_thread as u32); // user_lr
            core::ptr::write(s_ptr.offset(18*-4) as *mut u32, yield_thread as u32); // user_lr

        }
        THREADS.push(tcb);
        return id;
    }
}

pub fn reschedule() {
    unsafe {
        RESCHEDULE = true;
    }
}

pub fn yield_thread() {
    unsafe { asm!("swi #32"); }
}

use log::debug;

#[no_mangle]
pub unsafe extern fn schedule() {
        RESCHEDULE = false;

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

        //debug!("switch thread to {}", pos);

        THREADS[pos].state = ThreadState::Running;
        let old_pos = RUNNING_THREAD_ID;
        THREADS[old_pos].state = ThreadState::Ready;
        RUNNING_THREAD_ID = pos;

    
        asm!("mov r0, {old}
              mov r1, {new}",
              old = in(reg) THREADS[old_pos].stack_pos,
              new = in(reg) THREADS[pos].stack_pos);

              asm!("
              push {{r0-r12}}
              stm r0, {{sp}}
              mov sp, r1
              pop {{r0-r12}}
              mov pc, lr", sym schedule_internal);
}

#[naked]
#[no_mangle]
unsafe extern fn switch_thread() {
    asm!("push {{r0-r12}}
          stm r0, {{sp}}
          mov sp, r1
          pop {{r0-r12}}
          mov pc, lr", options(noreturn));
}
