.PHONY: build
build:
	cargo build --release

.PHONY: show-help
show-help: build
	./target/release/e2e -h

.PHONY: test
test:
	cargo test $(MOD)
