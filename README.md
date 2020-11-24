# rOSt

This is a project for university containing bits and pieces of what might become an operating system for 32 bit ARM. It is written in Rust for the AT91RM9200, specifically the Portux920T.


## Testing

### Requirements 
- We expect a `qemu-system-arm-portux-fork` in the `PATH` built from https://git.imp.fu-berlin.de/koenigl/qemu-portux (https://mycampus.imp.fu-berlin.de/portal/site/2fde4704-93a2-49e7-8b14-b57da998874a/tool/1e4443b8-8b1f-437b-bea3-331858e0d74f/discussionForum/message/dfViewThread)
- Rust (Nightly version) in the `PATH`

### Steps
1. Run `cargo run` to start qemu with our kernel
