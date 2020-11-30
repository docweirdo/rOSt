# rOSt

This is a project for university containing bits and pieces of what might become an operating system for 32 bit ARM. It is written in Rust for the AT91RM9200, specifically the Portux920T.


## Testing

### Requirements 
- `qemu-system-arm-portux-fork` in the `PATH` built from https://git.imp.fu-berlin.de/koenigl/qemu-portux (https://mycampus.imp.fu-berlin.de/portal/site/2fde4704-93a2-49e7-8b14-b57da998874a/tool/1e4443b8-8b1f-437b-bea3-331858e0d74f/discussionForum/message/dfViewThread)
- `arm-none-eabi-gcc` in the `PATH`
- Rust (Nightly version) in the `PATH`

### Steps
1. Run `cargo run` to start qemu with our kernel


## Useful links

* Inline assemly in Rust
    * https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
    * https://github.com/Amanieu/rfcs/blob/inline-asm/text/0000-inline-asm.md
