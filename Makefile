DESTDIR =
PREFIX = /usr/local
CARGO_FLAGS =

.PHONY: all target/release/xcolor install help

all: target/release/xcolor

target/release/xcolor:
	cargo build --release $(CARGO_FLAGS)

install: target/release/xcolor
	install -s -D -m755 -- target/release/xcolor "$(DESTDIR)$(PREFIX)/bin/xcolor"
	install -D -m644 -- man/xcolor.1 "$(DESTDIR)$(PREFIX)/share/man/man1/xcolor.1"
	install -D -m644 -- extra/xcolor.desktop "$(DESTDIR)$(PREFIX)/share/applications/xcolor.desktop"
	install -D -m644 -- extra/icons/xcolor-16.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/16x16/apps/xcolor.png"
	install -D -m644 -- extra/icons/xcolor-24.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/24x24/apps/xcolor.png"
	install -D -m644 -- extra/icons/xcolor-32.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/32x32/apps/xcolor.png"
	install -D -m644 -- extra/icons/xcolor-48.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/xcolor.png"
	install -D -m644 -- extra/icons/xcolor-256.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/256x256/apps/xcolor.png"
	install -D -m644 -- extra/icons/xcolor-512.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/512x512/apps/xcolor.png"
	install -D -m644 -- extra/icons/xcolor.svg "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/xcolor.png"

help:
	@echo "Available make targets:"
	@echo "  all      - Build xcolor (default)"
	@echo "  install  - Build and install xcolor"
	@echo "  help     - Print this help"
