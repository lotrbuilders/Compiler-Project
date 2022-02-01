# Compiler-Project


## Backend


## Limitations
The compiler does not currently handle xor, shifts, modulus, modify-assign and increment/decrement operations. Enums, typedefs, unions, floats and unsigned numbers are also not supported and no declaration specifiers are currently implemented. Both switch statements and goto are currently missing. Lastly K&R style function declarations and compound assignments are currently not supported.

## Installing
Using cargo the compiler can be built directly from the main directory without any necessary configuration. It defaults to debug mode, which is only interesting for compiler development. It is recomended to use `cargo run --release` for normal testing. 

There is a makefile to install the compiler for system wide use, using `make install`. By default it installs the compiler at /usr/local/lib/utcc/ and /usr/local/bin/ and temp files are stored at /tmp/. These settings can all be changed by setting the environment variables LIBDIR, BINDIR and TMPDIR respectively. The makefile requires super user rights to install these files. Users are encouraged to check the makefile themselves to ensure that it safe.

### Requirements
- Linux or WSL
- Cargo
- A C preprocessor installed
- [nasm](https://www.nasm.us/) (x86-64 backend)
- An x86-64 compiler with C standard library for linking (x86-64 backend)

## Licensing
The test-suite(found under /tests/src/) is licensed under MIT. The rest of the compiler is licensed under TODO
