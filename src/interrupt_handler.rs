use crate::dbgu;
use crate::interrupt_controller;
use crate::processor;
use crate::threads;
use crate::user_tasks;
use crate::{print, println};

pub fn system_timer_period_interval_timer_elapsed() {
    debug_assert!(processor::interrupts_enabled());

    // sys_timer_interrrupt_handler
    // print ! if task3 app is active
    // TODO: do not forget to remove both
    if unsafe { user_tasks::TASK3_ACTIVE } {
        println!("!");
    }
    if unsafe { user_tasks::TASK4_ACTIVE } {
        print!("!");
    }

    interrupt_controller::mark_end_of_interrupt!();

    threads::wakeup_elapsed_threads();

    unsafe {
        if threads::SCHEDULER_INTERVAL_COUNTER == 0 {
            threads::schedule(None);
        } else {
            threads::SCHEDULER_INTERVAL_COUNTER -= 1;
        }
    }
}

pub fn dbgu_character_received() {
    // debug_assert!(processor::interrupts_enabled());

    // dbgu_interrupt_handler,fires when rxready is set
    // push char into variable dbgu_buffer on heap, if app does not fetch -> out-of-memory error in allocator
    let last_char =
        dbgu::read_char().expect("there should be a char available in dbgu interrupt") as u8;

    threads::handle_dbgu_new_character_event(last_char as char);

    // TODO: do not forget to remove
    if unsafe { user_tasks::TASK4_ACTIVE } {
        user_tasks::task4_dbgu(last_char as char);
    }

    interrupt_controller::mark_end_of_interrupt!();
}
