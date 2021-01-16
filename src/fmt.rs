// print utilities

use crate::dbgu;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) =>  {
            let format_string = alloc::format!($($arg)*);
            crate::fmt::send_str(&format_string);
    }
}

#[macro_export]
macro_rules! println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => {
        crate::print!($($arg)*);
        crate::print!("\n");
    }
}

#[macro_export]
macro_rules! print_with_stack {
    ($size:tt, $($arg:tt)*) =>  {
        {
            use core::fmt::Write;
            let mut send_buf = arrayvec::ArrayString::<[u8; $size]>::new();
            write!(&mut send_buf, $($arg)*).expect("Can't write");
            crate::fmt::send_str(&send_buf);
        }
    }
}

#[macro_export]
macro_rules! println_with_stack {
    () => (crate::print_with_stack!(4, "\n"));
    ($size:tt, $($arg:tt)*) => {
        crate::print_with_stack!($size, $($arg)*);
        crate::print_with_stack!(4, "\n");
    }
}

pub fn send_str(chars: &str) {
    for character in chars.chars() {
        dbgu::write_char(character);
    }
}
