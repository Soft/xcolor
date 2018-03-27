DESTDIR =
PREFIX = /usr/local
CARGO_FLAGS =

.PHONY: all target/release/xcolor install help

all: target/release/xcolor

target/release/xcolor:
	cargo build --release $(CARGO_FLAGS)

install: target/release/xcolor
	install -m755 -- target/release/xcolor "$(DESTDIR)$(PREFIX)/bin/"
	install -m644 -- man/xcolor.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

help:
	@echo "Available make targets:"
	@echo "  all      - Build xcolor (default)"
	@echo "  install  - Build and install xcolor and manual pages"
	@echo "  help     - Print this help"
