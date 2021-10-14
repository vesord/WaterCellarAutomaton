CARGO := $(shell which cargo)
INSTALL_RUST = install_rust.sh

.PHONY: all
all:
ifndef CARGO
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > $(INSTALL_RUST)
	chmod +x $(INSTALL_RUST)
	./$(INSTALL_RUST) -y
	source '$$HOME/.cargo/env'
endif
	cargo build --release
	cp target/release/mod1 .

.PHONY: run
run:
	./mod1
