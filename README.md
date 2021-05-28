# indy

`indy` is a versatile MIPS emulator (and eventually a framework) written in
Rust. It was initially written as the final project for the [Computer
Architecture class, given by Prof. Dr. Emilio
Francesquini](http://professor.ufabc.edu.br/~e.francesquini/2021.q1.ac/), in
the first period of 2021 at UFABC.

# Usage

Currently, the emulator can load and run simple user-space-only little-endian
MIPS32 programs, using the MARS simulator syscall convention. The programs can
be loaded in two ways: through ELF files, or through a set of raw binary files
containing the text, data and rodata sections. See the `res/` folder to see an
example of how the files must be arranged for the latter.

To decode a program and show its Assembly code, use the `decode` and
`decodeelf` subcommands. Examples:

```sh
$ cargo run -- decode file # for split files
$ cargo run -- decodeelf file.elf # for ELF files
```

To execute a program, use the `run` and `runelf` subcommands. You can also
specify a memory configuration to emulate:

```sh
$ cargo run --release -- run [--entry ENTRY] [config] file # for split files
$ cargo run --release -- runelf [config] file.elf # for ELF files
```

Note that the `--release` flag is not required, but advised for performance
reasons.

You can check the available memory configurations by running:

```sh
$ cargo run -- run --help
```

# Why indy?

The SGI Indy is a cute little MIPS workstation that was made during the 90s. 

# License

Unless stated otherwise, this code is licensed under the Mozilla Public
License, version 2. You can read it in the `LICENSE` file.

Everything under the `res` folder except the `97.*, 98.*, 99.*` files and the
`extra` folder are written by Emilio Francesquini and licensed under CC-BY-SA
4.0.
