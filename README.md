# rOSt

This is a project for university containing bits and pieces of what might become an operating system for 32 bit ARM. It is written in Rust for the AT91RM9200, specifically the Portux920T.


## Testing

### Requirements to build
- `arm-none-eabi-gcc` in the `PATH`
- Rust via `rustup` in the `PATH`
    - install [rustup](https://rustup.rs/)
    - `$ rustup default nightly`
    - `$ rustup component add rust-src`

#### Steps
1. Run `$ cargo build` to compile (default target path: `target/armv4t-none-eabi/debug/rost`) to binary elf

### Requirements to run
- `qemu-system-arm-portux-fork` in the `PATH` built from https://git.imp.fu-berlin.de/koenigl/qemu-portux

#### Steps
1. Run `$ cargo run` to start qemu with our kernel


## Useful links

* Inline assemly in Rust
    * https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
    * https://github.com/Amanieu/rfcs/blob/inline-asm/text/0000-inline-asm.md

### Todo:
* Test production compilation
* work addresses and offsets into structs for each component
* implement proper allocator
* convert exception_routine macro to nice macro :D
* fix nested exceptions (swi in intq)
* test for proper stack usage and size
* prepare for new naked macro
* implement correct differentiation for system interrupt handler (line 1)
