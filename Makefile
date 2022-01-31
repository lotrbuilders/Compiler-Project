IDIR = ./include
BINDIR ?= /usr/local/bin/
LIBDIR ?= /usr/local/lib/utcc/
TMPDIR ?= /tmp/

all: 

install: export UTCC_INCLUDE_DIR = $(LIBDIR)/include/
install: export UTCC_TEMP_DIR = $(TMPDIR)
install: FORCE
	@echo $$INCLUDE_DIR
	cargo build --release
	sudo mkdir -p $(BINDIR)
	sudo mkdir -p $(LIBDIR)
	sudo cp -r $(IDIR) $(LIBDIR)
	sudo cp ./target/release/utcc $(LIBDIR)
	sudo cp ./target/release/utcc $(BINDIR)

clean: FORCE
	cargo clean

FORCE: ; 