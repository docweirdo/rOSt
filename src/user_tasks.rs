use core::time::Duration;

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
            rost_api::syscalls::sleep_ms(1000);
            if unsafe { TASK4_ACTIVE } {
                print_character_random(last_char, 5, 30);
            }
            rost_api::syscalls::sleep_ms(1000);
            if unsafe { TASK4_ACTIVE } {
                print_character_random(last_char, 5, 30);
            }
        });
    }
}

const KEY_ENTER: char = 0xD as char;
const KEY_BACKSPACE: char = 0x8 as char;
const KEY_DELETE: char = 0x7F as char;
const KEY_TAB: char = 0x9 as char;

const KEY_ESCAPE: u8 = 0x1B;
const KEY_LEFT_SQUARE_BRACKET: u8 = 0x5B;

static mut THREAD_TEST_COUNT: usize = 0;

struct Command {
    name: String,
    handler: Box<dyn FnMut() + 'static>,
}

static mut COMMANDS: Vec<Command> = Vec::new();

impl Command {
    fn new<F: FnMut() + 'static>(name: &str, handler: F) -> Self {
        Command {
            name: name.to_owned(),
            handler: Box::new(handler),
        }
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
        rost_api::syscalls::join_thread(id, None);
        TASK3_ACTIVE = false;
    });
    add_command("task4", || unsafe {
        TASK4_ACTIVE = true;
        rost_api::syscalls::subscribe(rost_api::syscalls::ThreadServices::DBGU);
        loop {
            let last_char = rost_api::syscalls::receive_character_from_dbgu() as char;
            if last_char == 'q' {
                break;
            }
        }
        TASK4_ACTIVE = false;
    });
    add_command("task5", || {
        /// wait for x realtime clock units
        fn busy_wait_ms(mut units: usize) {
            units = units / system_timer::get_real_time_unit_interval().as_millis() as usize;
            let last = rost_api::syscalls::get_current_realtime();
            loop {
                if rost_api::syscalls::get_current_realtime() - last >= units {
                    break;
                }
            }
        }
        fn run_thread(last_char: char) {
            rost_api::syscalls::create_thread(move || {
                if last_char.is_uppercase() {
                    for _ in 0..11 {
                        print!("{}", last_char);
                        busy_wait_ms(500);
                    }
                } else {
                    for _ in 0..11 {
                        print!("{}", last_char);
                        rost_api::syscalls::sleep_ms(500);
                    }
                }
            });
        }

        run_thread('A');
        run_thread('B');
        run_thread('c');

        rost_api::syscalls::subscribe(rost_api::syscalls::ThreadServices::DBGU);
        loop {
            let last_char = rost_api::syscalls::receive_character_from_dbgu() as char;
            if last_char == 'q' {
                break;
            }

            run_thread(last_char);
        }
    });
    add_command("uptime", || {
        println!(
            "uptime: {:?}",
            system_timer::get_current_real_time_as_duration()
        );
    });
    add_command("custom_code", || {
        let id = rost_api::syscalls::create_thread(crate::custom_user_code_thread);
        rost_api::syscalls::join_thread(id, None);
    });
    add_command("software_interrupt", || unsafe {
        asm!("swi #99");
    });
    add_command("undefined_instruction", || unsafe {
        asm!(".word 0xf7f0a000");
    });
    add_command("data_abort", || unsafe {
        asm!(
            "
         ldr {tmp}, =0x90000000
         str {tmp}, [{tmp}]", tmp = out(reg) _
        );
    });
    add_command("heap_size", || {
        println!(
            "current heap size: {:#X}, left: {:#X}",
            allocator::get_current_heap_size(),
            allocator::get_heap_size_left()
        );
    });
    add_command("threads", || {
        threads::print_threads();
    });
    add_command("sleep_test", || {
        println!(
            "sleep with duration 5s - start_at: {:?}",
            system_timer::get_current_real_time_as_duration()
        );
        println!(
            "reported_duration: {:?}",
            core::time::Duration::from_millis(rost_api::syscalls::sleep_ms(5000) as u64)
        );
        println!(
            "stop_at: {:?}",
            system_timer::get_current_real_time_as_duration()
        );
    });
    add_command("thread_test", || unsafe {
        THREAD_TEST_COUNT = 0;
        let mut thread_ids: Vec<usize> = Vec::new();

        fn sleep_ms_thread(id: usize, time_ms: usize) {
            let slept_duration = core::time::Duration::from_millis(rost_api::syscalls::sleep_ms(
                id * time_ms,
            ) as u64);
            let expected_duration = core::time::Duration::from_millis((id * time_ms) as u64);
            assert!(
                slept_duration
                    .checked_sub(expected_duration)
                    .unwrap_or_default()
                    < Duration::from_millis(50)
            );
            println!(
                "thread {} slept {:?} expected: {:?}",
                id, slept_duration, expected_duration
            );
        }

        for id in 0..=250 {
            thread_ids.push(rost_api::syscalls::create_thread(move || {
                THREAD_TEST_COUNT += 1;
                sleep_ms_thread(id, 50);
                THREAD_TEST_COUNT += 1;
                if THREAD_TEST_COUNT == 500 {
                    threads::print_threads();
                    println!(
                        "current heap size: {:#X}, left: {:#X}",
                        allocator::get_current_heap_size(),
                        allocator::get_heap_size_left()
                    );
                }
                sleep_ms_thread(id, 75);
                THREAD_TEST_COUNT += 1;
            }));
        }

        for id in thread_ids {
            rost_api::syscalls::join_thread(id, None);
        }

        assert_eq!(THREAD_TEST_COUNT, 753);
        println!("thread_test end {}", THREAD_TEST_COUNT);
    });

    add_command("dbgu_test", || {
        println!("dbgu_test: start");
        let mut thread_ids: Vec<usize> = Vec::new();

        for id in 0..3 {
            thread_ids.push(rost_api::syscalls::create_thread(move || {
                rost_api::syscalls::subscribe(rost_api::syscalls::ThreadServices::DBGU);
                println!(
                    "dbgu_test: thread {} got {}",
                    id,
                    rost_api::syscalls::receive_character_from_dbgu() as char
                );
                rost_api::syscalls::sleep_ms(50);
                println!(
                    "dbgu_test: thread {} got {}",
                    id,
                    rost_api::syscalls::receive_character_from_dbgu() as char
                );
            }));
        }

        for id in thread_ids {
            rost_api::syscalls::join_thread(id, None);
        }

        println!("dbgu_test: the end");
    });

    unsafe {
        RNG = Some(Pcg64::seed_from_u64(0xDEADBEEF));
    }

    let mut history: Vec<String> = Vec::new();

    loop {
        rost_api::syscalls::subscribe(rost_api::syscalls::ThreadServices::DBGU);

        let mut char_buf = alloc::string::String::new();

        println!("\nwaiting for input... (press ENTER to echo)");
        print!("available commands (autocomplete enabled):\n  ");
        unsafe {
            COMMANDS.sort_by(|a, b| a.name.cmp(&b.name));
            for cmd in &COMMANDS {
                print!("{} ", cmd.name);
            }
        }
        print!("\n$ ");

        let mut found_autocomplete_commands: Vec<&str> = Vec::new();

        fn replace_displayed_text(char_buf: &mut String, w: &str) {
            for _ in char_buf.chars() {
                print!("{0} {0}", KEY_BACKSPACE);
            }
            char_buf.clear();
            char_buf.push_str(w);
            print!("{}", char_buf);
        }

        loop {
            let last_char: char = rost_api::syscalls::receive_character_from_dbgu() as char;

            if last_char == KEY_ENTER {
                println!();
                rost_api::syscalls::unsubscribe(rost_api::syscalls::ThreadServices::DBGU);

                if !char_buf.is_empty() {
                    if let Some(pos) = history.iter().position(|s| **s == char_buf) {
                        let old_pos_element = history.remove(pos);
                        history.push(old_pos_element);
                    } else {
                        history.push(char_buf.clone());
                    }
                }
                break;
            }
            if last_char == KEY_DELETE || last_char == KEY_BACKSPACE {
                if char_buf.pop().is_some() {
                    print!("{0} {0}", KEY_BACKSPACE);
                }
                found_autocomplete_commands.clear();
            } else if last_char == KEY_TAB {
                unsafe {
                    if found_autocomplete_commands.is_empty() {
                        found_autocomplete_commands = COMMANDS
                            .iter()
                            .filter_map(|c| {
                                if c.name.starts_with(&char_buf) {
                                    Some(c.name.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                    if !found_autocomplete_commands.is_empty() {
                        let pos = {
                            if let Some(pos) = found_autocomplete_commands
                                .iter()
                                .position(|name| name == &char_buf)
                            {
                                if found_autocomplete_commands.len() == pos + 1 {
                                    0
                                } else {
                                    pos + 1
                                }
                            } else {
                                0
                            }
                        };
                        replace_displayed_text(&mut char_buf, &found_autocomplete_commands[pos]);
                    }
                }
            } else {
                found_autocomplete_commands.clear();
                char_buf.push(last_char);

                match char_buf.as_bytes() {
                    // check for ascii escape sequence
                    [.., KEY_ESCAPE, KEY_LEFT_SQUARE_BRACKET, command] => {
                        let command = *command;
                        // remove last three characters because we found a escape sequence
                        for _ in 1..=3 {
                            char_buf.pop();
                        }
                        match command {
                            // CURSOR DOWN: upwards in history
                            b'A' => {
                                if !history.is_empty() {
                                    if !char_buf.is_empty() {
                                        if let Some(find_pos) =
                                            history.iter().position(|s| *s == char_buf)
                                        {
                                            let pos;
                                            if find_pos > 0 {
                                                pos = find_pos - 1;
                                            } else {
                                                pos = 0;
                                            }
                                            replace_displayed_text(&mut char_buf, &history[pos]);
                                        }
                                    } else {
                                        replace_displayed_text(
                                            &mut char_buf,
                                            &history.last().unwrap(),
                                        );
                                    }
                                }
                            }
                            // CURSOR DOWN: downwards in history
                            b'B' => {
                                if !history.is_empty() && !char_buf.is_empty() {
                                    if let Some(find_pos) =
                                        history.iter().position(|s| *s == char_buf)
                                    {
                                        if find_pos + 1 < history.len() {
                                            replace_displayed_text(
                                                &mut char_buf,
                                                &history[find_pos + 1],
                                            );
                                        } else {
                                            replace_displayed_text(&mut char_buf, "");
                                        }
                                    }
                                }
                            }
                            _ => {
                                print!("unknown escape sequence: {}\n$ ", command as char);
                            }
                        }
                    }
                    _ => {
                        if last_char.is_alphanumeric() || matches!(last_char, ' ' | '_') {
                            print!("{}", last_char);
                        }
                    }
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
                let id = rost_api::syscalls::create_thread(move || (cmd.handler)());
                rost_api::syscalls::join_thread(id, None);
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
