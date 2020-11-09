// print utilities

use crate::dbgu;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) =>  {
        let mut send_buf = ArrayString::<[u8; 64]>::new();
        write!(&mut send_buf, $($arg)*).expect("Can't write");
        crate::fmt::send_str(&send_buf);
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\n");
    }
}

pub fn send_str(chars: &str) {
    for character in chars.chars() {
        dbgu::write_char(character);
    }
}
