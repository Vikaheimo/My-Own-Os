# My-Own-Os

This is a hobby x86-64 kernel / operating system project written in rust. Maybe someday I will give this project a better name.

## Setup required

1. Rust (obliviously) with the nightly toolchain.
2. QEMU for running the os.

The project is currently split using workspaces. The root workspace is just a convenient wrapper for running the kernel. The kernel workspace contains all of the actual code (for now).

## Project goals

I'd like to create a working network driver, but thats a far away goal. Closer goals are (not in order):

1. Working filesystem "persisting" data to disk
2. A working user space.
3. Some sort of shell to interact with the os.
4. A scheduler so we get concurrency working.
5. Deprecate PIT timer in favour of some more modern solution.

## Special thanks

Philipp Oppermann and his great blog [Writing an OS in Rust](https://os.phil-opp.com/). Without his examples, this project wouldn't have even started.
