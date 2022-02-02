# Compiler-Project
This is an optimizing and retargetable compiler for a significant subset of C, written in RUST using an SSA quadruples style IR; A retargetable DAG covering burg style instruction selector and a chaitin-briggs allocator. 

## Backend
The backend uses [rburg](https://github.com/lotrbuilders/rburg) and aims to create an easily retargetable backend for any register-based processor. The code needed to create a new backend is minimized as much as possible by making heavy use of both procedural macros and `macro_rules!` macros. Minimizing this is an ongoing effort.

## Limitations
The compiler does not currently handle xor, shifts, modulus, modify-assign and increment/decrement operations. Enums, typedefs, unions, floats and unsigned numbers are also not supported and no declaration specifiers are currently implemented. Both switch statements and goto are currently missing. Lastly K&R style function declarations and compound assignments are currently not supported.


## Installing
Using cargo the compiler can be built directly from the main directory without any necessary configuration. It defaults to debug mode, which is only interesting for compiler development. It is recomended to use `cargo run --release` for normal testing. 

There is a makefile to install the compiler for system wide use, using `make install`. By default it installs the compiler at `/usr/local/lib/utcc/` and `/usr/local/bin/` and temp files are stored at /tmp/. These settings can all be changed by setting the environment variables **LIBDIR**, **BINDIR** and **TMPDIR** respectively. The makefile requires super user rights to install these files. Users are encouraged to check the makefile themselves to ensure that it safe.

### Submodule
This repository contains submodules. To properly initialize these execute `git submodule init` followed by `git submodule update` after cloning the repository.

### Requirements
- A unix compatible system or subsystem. On windows [WSL 2.0](https://docs.microsoft.com/en-us/windows/wsl/install) is recommended
- [Rust](https://www.rust-lang.org/tools/install)
- A C preprocessor installed
- [nasm](https://www.nasm.us/) (x86-64 backend)
- An x86-64 compiler with C standard library for linking (x86-64 backend)

## Test Suite
The test suite consists of a large selection of correctness tests and a small selection of performance tests.
The former can be ran by executing `cargo test --test=fullscale`. Specific tests can be run using `cargo test --test=fullscale -- full_scale_{NAME_OF_TEST_FOLDER} --exact`
The latter can be ran using `cargo test --test=performance`

## Licensing
The test-suite(found under `/tests/src/`) is licensed under MIT. All other files are licensed under MPL-2.0
