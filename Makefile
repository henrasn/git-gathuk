PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin

.PHONY: all build install install-system uninstall test clean

all: build

build:
	cargo build --release

install: build
	install -Dm755 target/release/git-gathuk $(DESTDIR)$(BINDIR)/git-gathuk
	ln -sf git-gathuk $(DESTDIR)$(BINDIR)/gathuk
	@echo "==> Installed to $(DESTDIR)$(BINDIR)/git-gathuk (alias: gathuk)"

install-system: build
	sudo install -Dm755 target/release/git-gathuk /usr/local/bin/git-gathuk
	sudo ln -sf git-gathuk /usr/local/bin/gathuk
	@echo "==> Installed system-wide to /usr/local/bin/git-gathuk (alias: gathuk)"

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/git-gathuk $(DESTDIR)$(BINDIR)/gathuk
	@echo "==> Uninstalled from $(DESTDIR)$(BINDIR)/git-gathuk"

test:
	cargo test

clean:
	cargo clean
