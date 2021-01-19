use crate::system_timer;
use crate::{alloc::borrow::ToOwned, print};
use crate::{allocator, println, threads};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use log::trace;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;

pub static mut RNG: Option<Pcg64> = None;
pub static mut TASK3_ACTIVE: bool = false;
pub static mut TASK4_ACTIVE: bool = false;

/// prints a character for a random range between min and max
fn print_character_random<T>(c: T, min: usize, max: usize)
where
    T: core::fmt::Display,
{
    unsafe {
        for _ in 0..RNG.as_mut().unwrap().gen_range(min..max) {
            crate::print!("{}", c);
        }
    }
}

pub fn task4_dbgu(last_char: char) {
    if unsafe { TASK4_ACTIVE } && last_char != 'q' {
        rost_api::syscalls::create_thread(move || {
            // print 3 times and wait between
            print_character_random(last_char, 5, 30);
            rost_api::syscalls::sleep(1000);
            print_character_random(last_char, 5, 30);
            rost_api::syscalls::sleep(1000);
            print_character_random(last_char, 5, 30);
        });
    }
}

const KEY_ENTER: char = 0xD as char;
const KEY_BACKSPACE: char = 0x8 as char;
const KEY_DELETE: char = 0x7F as char;

static mut THREAD_TEST_COUNT: usize = 0;

struct Command {
    name: String,
    handler: Box<dyn FnMut() + 'static>,
}

static mut COMMANDS: Vec<Command> = Vec::new();

impl Command {
    fn new<F: FnMut() + 'static>(name: &str, handler: F) -> Self {
        return Command {
            name: name.to_owned(),
            handler: Box::new(handler),
        };
    }
}

fn add_command(name: &str, handler: impl FnMut() + 'static) {
    unsafe {
        COMMANDS.push(Command::new(name, handler));
    }
}

/// Simple Read–eval–print loop with some basic commands
pub fn read_eval_print_loop() {
    add_command("task3", || unsafe {
        TASK3_ACTIVE = true;
        let id = rost_api::syscalls::create_thread(crate::custom_user_code_thread);
        loop {
            if threads::is_thread_done(id) {
                break;
            }
        }
        TASK3_ACTIVE = false;
    });
    add_command("task4", || unsafe {
        TASK4_ACTIVE = true;
        loop {
            // check for a new char in the dbgu buffer
            if let Some(last_char) = rost_api::syscalls::receive_character_from_dbgu() {
                let last_char: char = last_char as char;
                // quit on q
                if last_char == 'q' {
                    break;
                }
            }
        }
        TASK4_ACTIVE = false;
    });
    add_command("uptime", || {
        println!("{}", system_timer::get_current_real_time());
    });
    add_command("custom", || {
        rost_api::syscalls::create_thread(crate::custom_user_code_thread);
    });
    add_command("swi", || unsafe {
        asm!("swi #99");
    });
    add_command("undi", || unsafe {
        asm!(".word 0xf7f0a000");
    });
    add_command("undi", || unsafe {
        asm!(
            "
         ldr r0, =0x90000000
         str r0, [r0]"
        );
    });
    add_command("threads", || {
        threads::print_threads();
    });
    add_command("thread_test", || unsafe {
        for id in 0..10 {
            rost_api::syscalls::create_thread(move || {
                THREAD_TEST_COUNT += 1;
                println!("thread {} slept {}", id, rost_api::syscalls::sleep(50 * id));
                THREAD_TEST_COUNT += 1;
                println!("thread {} slept {}", id, rost_api::syscalls::sleep(50 * id));
                THREAD_TEST_COUNT += 1;
                println!("thread end {} {}", id, THREAD_TEST_COUNT);
            });
        }
        rost_api::syscalls::yield_thread();
    });

    unsafe {
        RNG = Some(Pcg64::seed_from_u64(0xDEADBEEF));
    }

    loop {
        let mut char_buf = alloc::string::String::new();

        println!("\nwaiting for input... (press ENTER to echo)");
        print!("available commands:\n  ");
        unsafe {
            for cmd in &COMMANDS {
                print!("{} ", cmd.name);
            }
        }
        print!("\n$ ");

        loop {
            if let Some(last_char) = rost_api::syscalls::receive_character_from_dbgu() {
                let last_char: char = last_char as char;
                if last_char == KEY_ENTER {
                    println!();
                    break;
                }
                if last_char == KEY_DELETE || last_char == KEY_BACKSPACE {
                    char_buf.pop();
                    print!("\x08 \x08");
                } else {
                    char_buf.push(last_char);
                    print!("{}", last_char);
                }
            }
        }

        trace!(
            "current heap size: {:#X}, left: {:#X}",
            allocator::get_current_heap_size(),
            allocator::get_heap_size_left()
        );

        unsafe {
            if let Some(cmd) = COMMANDS.iter_mut().find(|c| c.name == char_buf.as_str()) {
                //println!("Executing command: {}", cmd.name);
                (cmd.handler)();
            } else {
                // builtin commands
                match char_buf.as_str() {
                    "quit" => {
                        break;
                    }
                    _ => {
                        println!("-> Unknown command: {}", char_buf);
                    }
                }
            }
        }
    }
}
