.PHONY: dev
dev:
	nix develop

.PHONY: test
test:
	cargo test --verbose --jobs 1 -- --nocapture --test-threads=1

.PHONY: build
build:
	cross build --release --target x86_64-unknown-linux-gnu
	cross build --release --target aarch64-unknown-linux-gnu

	cargo build --release --target x86_64-apple-darwin
